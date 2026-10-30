//! The hurricane run is coherent, and each sequence rule catches a run
//! mutated to break it.

#![cfg(feature = "validate")]

mod common;

use common::examples;
use serde_json::{Value, json};
use som::sequence::{Rule, SequenceReport, Snapshot, check_run};
use som::validate::Validator;

fn hurricane() -> Vec<Snapshot> {
    let run = Snapshot::load_dir(examples().join("hurricane-run")).unwrap();
    assert_eq!(run.len(), 7);
    run
}

fn check(run: &[Snapshot]) -> SequenceReport {
    check_run(&Validator::new().unwrap(), run)
}

/// Asserts the report holds exactly the given error and no other finding.
fn only_error(report: &SequenceReport, rule: Rule) {
    assert!(report.has_error(rule), "expected {rule:?}: {report:#?}");
    assert!(
        report.errors.iter().all(|f| f.rule == rule) && report.warnings.is_empty(),
        "expected only {rule:?}: {report:#?}"
    );
    assert!(!report.is_coherent());
}

fn only_warning(report: &SequenceReport, rule: Rule) {
    assert!(report.has_warning(rule), "expected {rule:?}: {report:#?}");
    assert!(
        report.warnings.iter().all(|f| f.rule == rule) && report.errors.is_empty(),
        "expected only {rule:?}: {report:#?}"
    );
    assert!(report.is_coherent());
    assert!(!report.is_coherent_strict());
}

#[test]
fn hurricane_run_is_coherent() {
    let report = check(&hurricane());
    assert!(report.is_coherent_strict(), "{report:#?}");
}

#[test]
fn hurricane_run_is_ordered_by_file_name() {
    let names: Vec<String> = hurricane().into_iter().map(|s| s.name).collect();
    let expected: Vec<String> = (1..=7).map(|i| format!("hurricane-0{i}.json")).collect();
    assert_eq!(names, expected);
}

#[test]
fn story_id_changing_is_an_error() {
    let mut run = hurricane();
    run[3].payload["story_id"] = Value::from("hurricane-2026-0911-rev");
    only_error(&check(&run), Rule::StoryIdChanged);
}

#[test]
fn sequence_number_repeating_is_an_error() {
    let mut run = hurricane();
    run[4].payload["sequence_number"] = run[3].payload["sequence_number"].clone();
    let report = check(&run);
    assert!(report.has_error(Rule::SequenceNotIncreasing), "{report:#?}");
    assert!(
        report
            .errors
            .iter()
            .all(|f| f.rule == Rule::SequenceNotIncreasing)
    );
}

#[test]
fn sequence_number_going_backwards_is_an_error() {
    let mut run = hurricane();
    run.swap(2, 3);
    run[2].payload["updated_at"] = run[3].payload["updated_at"].clone();
    let report = check(&run);
    assert!(report.has_error(Rule::SequenceNotIncreasing), "{report:#?}");
}

#[test]
fn sequence_number_gap_is_a_warning() {
    let mut run = hurricane();
    run.remove(3);
    let report = check(&run);
    only_warning(&report, Rule::SequenceGap);
}

#[test]
fn updated_at_going_backwards_is_an_error() {
    let mut run = hurricane();
    run[5].payload["updated_at"] = Value::from("2026-09-11T00:00:00Z");
    for source in run[5].payload["editorial_source"].as_array_mut().unwrap() {
        source["received_at"] = Value::from("2026-09-10T00:00:00Z");
    }
    only_error(&check(&run), Rule::UpdatedAtBackwards);
}

/// The instant, not the spelling: `16:41:00.5Z` sorts before `16:41:00Z` as a
/// string but is half a second later.
#[test]
fn updated_at_is_compared_as_an_instant() {
    let mut run = hurricane();
    let at = run[5].payload["updated_at"].as_str().unwrap().to_owned();
    let later = at.replacen('Z', ".5Z", 1);
    assert!(later < at, "the spelling sorts the later instant first");
    run[6].payload["updated_at"] = Value::from(later);
    run[5].payload["updated_at"] = Value::from(at);
    let report = check(&run);
    assert!(!report.has_error(Rule::UpdatedAtBackwards), "{report:#?}");
}

#[test]
fn a_member_disappearing_is_an_error() {
    let mut run = hurricane();
    run[4].payload.as_object_mut().unwrap().remove("priority");
    only_error(&check(&run), Rule::MemberDisappeared);
}

#[test]
fn an_asset_dropped_is_an_error() {
    let mut run = hurricane();
    let assets = run[5].payload["assets"].as_array_mut().unwrap();
    assert!(assets.len() > 1);
    assets.remove(0);
    only_error(&check(&run), Rule::AssetDropped);
}

#[test]
fn an_asset_changing_type_is_an_error() {
    let mut run = hurricane();
    let asset = &mut run[6].payload["assets"][0];
    let new_type = if asset["asset_type"] == "STILL" {
        "AUDIO"
    } else {
        "STILL"
    };
    asset["asset_type"] = Value::from(new_type);
    only_error(&check(&run), Rule::AssetTypeChanged);
}

#[test]
fn leaving_a_terminal_story_type_is_an_error() {
    let mut run = hurricane();
    run[5].payload["story_type"] = Value::from("KILLED");
    run[5].payload.as_object_mut().unwrap().remove("lifecycle");
    run[6].payload["story_type"] = Value::from("ACTIVE");
    let report = check(&run);
    assert!(report.has_error(Rule::LeftTerminalStoryType), "{report:#?}");
}

#[test]
fn a_settled_review_reopening_is_a_warning() {
    let mut run = hurricane();
    assert_eq!(
        run[5].payload["assertions"][0]["review"]["state"],
        "PENDING"
    );
    assert_eq!(
        run[6].payload["assertions"][0]["review"]["state"],
        "CONFIRMED"
    );
    let confirmed = run[6].payload["assertions"][0]["review"].clone();
    run[5].payload["assertions"][0]["review"] = confirmed;
    run[6].payload["assertions"][0]["review"] = json!({"state": "PENDING"});
    only_warning(&check(&run), Rule::ReviewReopened);
}

#[test]
fn a_content_ref_to_an_undeclared_source_is_an_error() {
    let mut run = hurricane();
    run[2].payload["content_refs"][0]["source_id"] = Value::from("src-nowhere");
    only_error(&check(&run), Rule::ContentRefUnresolved);
}

#[test]
fn a_content_ref_keyed_to_the_wrong_source_is_a_warning() {
    let mut run = hurricane();
    let p = &mut run[6].payload;
    let sources: Vec<String> = p["editorial_source"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["source_id"].as_str().unwrap().to_owned())
        .collect();
    let current = p["content_refs"][0]["source_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let other = sources.iter().find(|s| **s != current).unwrap().clone();
    p["content_refs"][0]["source_id"] = Value::from(other);
    only_warning(&check(&run), Rule::ContentRefMayPointAtWrongSource);
}

#[test]
fn a_content_ref_that_names_no_provider_is_a_warning() {
    let mut run = hurricane();
    let reference = &mut run[6].payload["content_refs"][0];
    reference["uri"] = Value::from("https://example.com/item/1");
    reference.as_object_mut().unwrap().remove("content_format");
    only_warning(&check(&run), Rule::ContentRefDoesNotMentionSource);
}

#[test]
fn a_source_received_after_the_snapshot_is_an_error() {
    let mut run = hurricane();
    run[6].payload["editorial_source"][0]["received_at"] = Value::from("2099-01-01T00:00:00Z");
    only_error(&check(&run), Rule::SourceReceivedAfterUpdate);
}

#[test]
fn an_invalid_snapshot_is_reported() {
    let mut run = hurricane();
    run[3].payload["updated_at"] = Value::from("not a date");
    let report = check(&run);
    assert_eq!(report.invalid.len(), 1);
    assert_eq!(report.invalid[0].0, "hurricane-04.json");
    assert!(!report.is_coherent());
}

#[test]
fn a_single_snapshot_is_a_warning() {
    let run = vec![hurricane().remove(0)];
    only_warning(&check(&run), Rule::SingleSnapshot);
}

#[test]
fn an_empty_run_is_an_error() {
    only_error(&check(&[]), Rule::EmptyRun);
}

fn as_envelopes(run: &[Snapshot]) -> Vec<Snapshot> {
    let template = common::load(&examples().join("story-context/hurricane-beat6-envelope.json"));
    run.iter()
        .enumerate()
        .map(|(i, s)| {
            let mut envelope = template.clone();
            envelope["message_id"] =
                Value::from(format!("0199a1c4-7a2e-7b31-8c55-4d2f9e6a1b{i:02}"));
            envelope["payload"] = s.payload.clone();
            Snapshot::from_document(s.name.clone(), envelope)
        })
        .collect()
}

#[test]
fn hurricane_run_as_envelopes_is_coherent() {
    let run = as_envelopes(&hurricane());
    assert!(run.iter().all(|s| s.envelope.is_some()));
    assert!(check(&run).is_coherent_strict());
}

#[test]
fn a_republish_under_a_stale_message_id_is_an_error() {
    let mut run = as_envelopes(&hurricane());
    let stale = run[2].envelope.as_ref().unwrap()["message_id"].clone();
    run[3].envelope.as_mut().unwrap()["message_id"] = stale;
    only_error(&check(&run), Rule::StaleMessageId);
}

#[test]
fn a_stale_message_id_from_another_system_is_not_this_rule() {
    let mut run = as_envelopes(&hurricane());
    let stale = run[2].envelope.as_ref().unwrap()["message_id"].clone();
    let envelope = run[3].envelope.as_mut().unwrap();
    envelope["message_id"] = stale;
    envelope["originating_system"]["system_id"] = Value::from("another-system");
    assert!(check(&run).is_coherent_strict());
}
