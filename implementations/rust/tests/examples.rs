//! Every example outside `examples/negative/` validates, and the typed model
//! agrees with the schemas on which objects are closed and which members are
//! required.

#![cfg(feature = "validate")]

mod common;

use common::{EXAMPLE_DIRS, examples, is_envelope, json_files, load, object_pointers};
use serde_json::Value;
use som::model::{Payload, decode_message};
use som::validate::{Outcome, Validator};

#[test]
fn every_example_validates() {
    let validator = Validator::new().unwrap();
    let mut checked = 0;
    for (dir, family, minimum) in EXAMPLE_DIRS {
        let files = json_files(&examples().join(dir));
        assert!(
            files.len() >= minimum,
            "examples/{dir}/ holds {} file(s), expected at least {minimum}",
            files.len()
        );
        for path in files {
            let document = load(&path);
            if is_envelope(&document) {
                let outcome = validator
                    .validate_message(&document)
                    .unwrap_or_else(|r| panic!("{}: {r}", path.display()));
                assert_eq!(
                    outcome,
                    Outcome::Accepted {
                        family,
                        message_type: document["message_type"].as_str().unwrap().to_owned()
                    },
                    "{}",
                    path.display()
                );
            } else {
                validator
                    .validate_payload(family, &document)
                    .unwrap_or_else(|r| panic!("{}: {r}", path.display()));
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 18, "the corpus README counts 18 worked messages");
}

/// Schema acceptance and typed acceptance of one document, with the typed
/// re-encoding when it decoded.
fn both(
    validator: &Validator,
    family: som::Family,
    document: &Value,
) -> (bool, Result<Value, String>) {
    let schema_accepts = if is_envelope(document) {
        validator.validate_message(document).is_ok()
    } else {
        validator.validate_payload(family, document).is_ok()
    };
    let typed = if is_envelope(document) {
        decode_message(document.clone())
            .map(|t| serde_json::to_value(t.unwrap()).unwrap())
            .map_err(|e| e.to_string())
    } else {
        Payload::decode(family, document.clone())
            .map(|t| serde_json::to_value(t).unwrap())
            .map_err(|e| e.to_string())
    };
    (schema_accepts, typed)
}

/// For every object in every example: add a member no schema names. Where the
/// schema closes that object, the typed model must refuse it too; where the
/// schema leaves it open, the typed model must keep it and round-trip.
#[test]
fn typed_model_closes_exactly_the_objects_the_schema_closes() {
    const UNKNOWN: &str = "x_member_no_schema_names";
    let validator = Validator::new().unwrap();
    let (mut closed, mut open) = (0, 0);
    for (dir, family, _) in EXAMPLE_DIRS {
        for path in json_files(&examples().join(dir)) {
            let document = load(&path);
            let mut pointers = Vec::new();
            object_pointers(&document, "", &mut pointers);
            for pointer in pointers {
                let mut mutated = document.clone();
                mutated
                    .pointer_mut(&pointer)
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
                    .insert(UNKNOWN.to_owned(), Value::from(1));
                let (schema_accepts, typed) = both(&validator, family, &mutated);
                let at = format!("{}#{pointer}", path.display());
                if schema_accepts {
                    open += 1;
                    assert_eq!(
                        typed.as_ref(),
                        Ok(&mutated),
                        "{at}: the schema leaves this object open"
                    );
                } else {
                    closed += 1;
                    assert!(typed.is_err(), "{at}: the schema closes this object");
                }
            }
        }
    }
    println!(
        "unknown member added: {closed} closed objects refused it, {open} open objects kept it"
    );
    assert!(closed > 100 && open > 0, "closed {closed}, open {open}");
}

/// The members the schemas require only under a condition the typed model
/// cannot express: an `if`/`then`, an `anyOf` of `required` lists, or the
/// shape of an open `claim` object. Removing one of these fails the schema
/// and passes the typed model, and nowhere else may that happen.
fn conditionally_required(pointer: &str, key: &str) -> bool {
    let last = pointer.rsplit('/').next().unwrap_or("");
    let parent = pointer
        .rsplit_once('/')
        .map_or("", |(p, _)| p.rsplit('/').next().unwrap_or(""));
    let is_payload_root = pointer.is_empty() || pointer == "/payload";
    match key {
        // telling-event: exposure members by message_type; link-event:
        // committed_* on link.committed and withdrawn on link.withdrawn;
        // story.context: lifecycle present exactly when ACTIVE.
        "exposure_start" | "exposure_end" | "committed_by" | "committed_at" | "withdrawn"
        | "lifecycle"
            if is_payload_root =>
        {
            true
        }
        // delivery.media_available and media_refs[]: one of source or locator.
        "source" | "locator" if is_payload_root || parent == "media_refs" => true,
        // compliance[].media_range: time_range with source or locator.
        "source" | "locator" | "time_range" if last == "media_range" => true,
        // assertions[].claim members by assertion_type.
        "metric" | "value" | "story_id" | "detection_class" | "verdict" if last == "claim" => true,
        _ => false,
    }
}

/// For every member of every object in every example: remove it. Where the
/// schema still accepts, the typed model must still accept and round-trip.
/// Where the schema refuses, the typed model refuses too, except for the
/// conditional rules listed above, which only the validator can see. The
/// typed model is therefore never stricter than the schema about presence.
#[test]
fn typed_model_requires_no_more_and_no_less_than_the_schema() {
    let validator = Validator::new().unwrap();
    let (mut optional, mut required, mut conditional) = (0, 0, 0);
    for (dir, family, _) in EXAMPLE_DIRS {
        for path in json_files(&examples().join(dir)) {
            let document = load(&path);
            let mut pointers = Vec::new();
            object_pointers(&document, "", &mut pointers);
            for pointer in pointers {
                let keys: Vec<String> = document
                    .pointer(&pointer)
                    .unwrap()
                    .as_object()
                    .unwrap()
                    .keys()
                    .cloned()
                    .collect();
                for key in keys {
                    let mut mutated = document.clone();
                    mutated
                        .pointer_mut(&pointer)
                        .unwrap()
                        .as_object_mut()
                        .unwrap()
                        .remove(&key);
                    let (schema_accepts, typed) = both(&validator, family, &mutated);
                    let at = format!("{}#{pointer}/{key}", path.display());
                    if schema_accepts {
                        optional += 1;
                        assert_eq!(
                            typed.as_ref(),
                            Ok(&mutated),
                            "{at}: the schema does not require this member"
                        );
                    } else if conditionally_required(&pointer, &key) {
                        conditional += 1;
                        assert!(typed.is_ok(), "{at}: only the validator sees this rule");
                    } else {
                        required += 1;
                        assert!(typed.is_err(), "{at}: the schema requires this member");
                    }
                }
            }
        }
    }
    println!(
        "member removed: {optional} optional, {required} required, {conditional} conditionally required"
    );
    assert!(
        optional > 200 && required > 100 && conditional > 10,
        "optional {optional}, required {required}, conditional {conditional}"
    );
}
