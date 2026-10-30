//! Every case in `examples/negative/` is rejected, for the reason the case
//! states, and only for that reason.
//!
//! Each case has its own test, so a regression names the rule it broke. Each
//! expectation is the violation the case's `must_fail_because` describes:
//! which document, which keyword (or the §3 wire-version rule), where, and a
//! word of the message that ties it to the stated reason.
//!
//! The typed model is checked too: it refuses the cases that are structural
//! (unknown or withdrawn members, closed enums, missing required members) and
//! accepts the ones only a validator can see (formats, patterns, `anyOf`, the
//! wire version), which is why decoding is never a substitute for
//! validation.

#![cfg(feature = "validate")]

mod common;

use common::{examples, json_files, load, negative_family};
use serde_json::Value;
use som::Family;
use som::model::{Payload, decode_message};
use som::validate::{Part, Validator, ViolationKind};

/// What a case must be rejected for.
struct Expect {
    part: Part,
    /// A schema keyword, or `None` for the §3 wire-version rule.
    keyword: Option<&'static str>,
    instance_path: &'static str,
    /// A fragment of the violation message tying it to the stated reason.
    mentions: &'static str,
}

const fn schema(
    part: Part,
    keyword: &'static str,
    instance_path: &'static str,
    mentions: &'static str,
) -> Expect {
    Expect {
        part,
        keyword: Some(keyword),
        instance_path,
        mentions,
    }
}

const ENV: Part = Part::Envelope;
const STORY: Part = Part::Payload(Family::StoryContext);

/// Whether the typed model refuses the case on decode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Typed {
    Refuses,
    Accepts,
}

fn run_case(file: &str, reason: &str, expected: &[Expect], typed: Typed) {
    let path = examples().join("negative").join(file);
    let case = load(&path);
    assert_eq!(
        case["must_fail_because"].as_str(),
        Some(reason),
        "{file} states a different reason than this test expects"
    );
    let family = negative_family(case["family"].as_str().unwrap());
    let message = case["message"].clone();
    let validator = Validator::new().unwrap();

    let rejection = if message.get("payload").is_some() {
        assert_eq!(
            Family::from_message_type(message["message_type"].as_str().unwrap()),
            Some(family),
            "{file}: message_type selects a different family than the case declares"
        );
        validator.validate_message(&message)
    } else {
        validator.validate_payload(family, &message).map(|()| {
            panic!("{file} was accepted: {reason}");
        })
    }
    .expect_err(&format!("{file} was accepted: {reason}"));

    for e in expected {
        let found = rejection.violations.iter().any(|v| {
            let kind_matches = match (&v.kind, e.keyword) {
                (ViolationKind::Schema { keyword }, Some(k)) => keyword == k,
                (ViolationKind::WireVersion { .. }, None) => true,
                _ => false,
            };
            v.part == e.part
                && kind_matches
                && v.instance_path == e.instance_path
                && v.message.contains(e.mentions)
        });
        assert!(
            found,
            "{file}: expected {:?} {:?} at {:?} mentioning {:?}; got {rejection}",
            e.part, e.keyword, e.instance_path, e.mentions
        );
    }
    assert_eq!(
        rejection.violations.len(),
        expected.len(),
        "{file}: rejected for more than the stated reason: {rejection}"
    );

    let decoded = if message.get("payload").is_some() {
        decode_message(message)
            .map(|_| ())
            .map_err(|e| e.to_string())
    } else {
        Payload::decode(family, message)
            .map(|_| ())
            .map_err(|e| e.to_string())
    };
    let outcome = if decoded.is_ok() {
        Typed::Accepts
    } else {
        Typed::Refuses
    };
    assert_eq!(outcome, typed, "{file}: typed decode gave {decoded:?}");
}

macro_rules! negative_case {
    ($test:ident, $file:literal, $reason:literal, [$($expect:expr),+ $(,)?], $typed:expr) => {
        #[test]
        fn $test() {
            run_case($file, $reason, &[$($expect),+], $typed);
        }
    };
}

const CASES: [&str; 20] = [
    "audit-flat-actor.json",
    "delivery-neither-source-nor-locator.json",
    "envelope-extensions-not-namespaced.json",
    "envelope-message-id-not-uuid.json",
    "envelope-missing-correlation-id.json",
    "envelope-signature-not-allowed.json",
    "envelope-som-version-0-3-2.json",
    "envelope-source-not-allowed.json",
    "envelope-system-type-unknown.json",
    "envelope-timestamp-not-date-time.json",
    "envelope-topic-not-som.json",
    "envelope-unknown-property.json",
    "link-gate-status-unknown.json",
    "story-context-ai-enrichments.json",
    "story-context-asset-status-finalizing.json",
    "story-context-asset-voice-count.json",
    "story-context-lifecycle-phase-live.json",
    "story-context-missing-story-id.json",
    "telling-transform-id.json",
    "telling-transform-missing-type.json",
];

#[test]
fn every_negative_file_has_a_named_case() {
    let on_disk: Vec<String> = json_files(&examples().join("negative"))
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap().to_owned())
        .collect();
    assert_eq!(on_disk, CASES, "a negative case was added or removed");
}

negative_case!(
    negative_audit_flat_actor,
    "audit-flat-actor.json",
    "audit carries actor{} and recorded_at, not flat actor_id",
    [
        schema(
            Part::Payload(Family::SystemAudit),
            "required",
            "",
            "\"actor\""
        ),
        schema(
            Part::Payload(Family::SystemAudit),
            "additionalProperties",
            "",
            "'actor_id'"
        ),
    ],
    Typed::Refuses
);

negative_case!(
    negative_delivery_neither_source_nor_locator,
    "delivery-neither-source-nor-locator.json",
    "delivery.media_available must carry a source or a locator",
    [schema(
        Part::Payload(Family::DeliveryMediaAvailable),
        "anyOf",
        "",
        "anyOf"
    )],
    Typed::Accepts
);

negative_case!(
    negative_envelope_extensions_not_namespaced,
    "envelope-extensions-not-namespaced.json",
    "extensions keys must be reverse-domain namespaced com.{vendor}.*",
    [schema(
        ENV,
        "additionalProperties",
        "/extensions",
        "'vendor_field'"
    )],
    Typed::Refuses
);

negative_case!(
    negative_envelope_message_id_not_uuid,
    "envelope-message-id-not-uuid.json",
    "message_id must be a UUID; format is asserted at 1.0",
    [schema(ENV, "format", "/message_id", "\"uuid\"")],
    Typed::Accepts
);

negative_case!(
    negative_envelope_missing_correlation_id,
    "envelope-missing-correlation-id.json",
    "correlation_id is required on every message (decision #18)",
    [schema(ENV, "required", "", "\"correlation_id\"")],
    Typed::Refuses
);

negative_case!(
    negative_envelope_signature_not_allowed,
    "envelope-signature-not-allowed.json",
    "signature is not part of the 1.0 envelope",
    [schema(ENV, "falseSchema", "/signature", "False schema")],
    Typed::Refuses
);

negative_case!(
    negative_envelope_som_version_0_3_2,
    "envelope-som-version-0-3-2.json",
    "som_version 0.3.2 is not SOM 1.0; 1.0.0 is the only wire version",
    [Expect {
        part: ENV,
        keyword: None,
        instance_path: "/som_version",
        mentions: "\"0.3.2\"",
    }],
    Typed::Accepts
);

negative_case!(
    negative_envelope_source_not_allowed,
    "envelope-source-not-allowed.json",
    "source was renamed originating_system in v0.3 and is rejected",
    [schema(ENV, "falseSchema", "/source", "False schema")],
    Typed::Refuses
);

negative_case!(
    negative_envelope_system_type_unknown,
    "envelope-system-type-unknown.json",
    "originating_system.system_type is a closed enum",
    [schema(
        ENV,
        "enum",
        "/originating_system/system_type",
        "\"newsroom\""
    )],
    Typed::Refuses
);

negative_case!(
    negative_envelope_timestamp_not_date_time,
    "envelope-timestamp-not-date-time.json",
    "timestamp must be RFC 3339 date-time",
    [schema(ENV, "format", "/timestamp", "\"date-time\"")],
    Typed::Accepts
);

negative_case!(
    negative_envelope_topic_not_som,
    "envelope-topic-not-som.json",
    "topic must begin with som.",
    [schema(ENV, "pattern", "/topic", "^som")],
    Typed::Accepts
);

negative_case!(
    negative_envelope_unknown_property,
    "envelope-unknown-property.json",
    "the envelope permits no unknown properties",
    [schema(ENV, "additionalProperties", "", "'unexpected'")],
    Typed::Refuses
);

negative_case!(
    negative_link_gate_status_unknown,
    "link-gate-status-unknown.json",
    "compliance_gate_status is PENDING, CLEARED or BLOCKED",
    [schema(
        Part::Payload(Family::Link),
        "enum",
        "/compliance_gate_status",
        "\"APPROVED\""
    )],
    Typed::Refuses
);

negative_case!(
    negative_story_context_ai_enrichments,
    "story-context-ai-enrichments.json",
    "ai_enrichments[] was removed in v0.3.2 and is rejected",
    [schema(
        STORY,
        "falseSchema",
        "/ai_enrichments",
        "False schema"
    )],
    Typed::Refuses
);

negative_case!(
    negative_story_context_asset_status_finalizing,
    "story-context-asset-status-finalizing.json",
    "FINALIZING was withdrawn at 1.0 (see spec/open-register.md)",
    [schema(STORY, "enum", "/assets/0/status", "\"FINALIZING\"")],
    Typed::Refuses
);

negative_case!(
    negative_story_context_asset_voice_count,
    "story-context-asset-voice-count.json",
    "voice_count was withdrawn at 1.0; assets permit no unknown properties",
    [schema(
        STORY,
        "additionalProperties",
        "/assets/0",
        "'voice_count'"
    )],
    Typed::Refuses
);

negative_case!(
    negative_story_context_lifecycle_phase_live,
    "story-context-lifecycle-phase-live.json",
    "LIVE is derived from the Telling, never stored on the story (decision #16)",
    [schema(STORY, "enum", "/lifecycle/phase", "\"LIVE\"")],
    Typed::Refuses
);

negative_case!(
    negative_story_context_missing_story_id,
    "story-context-missing-story-id.json",
    "story_id is required",
    [schema(STORY, "required", "", "\"story_id\"")],
    Typed::Refuses
);

negative_case!(
    negative_telling_transform_id,
    "telling-transform-id.json",
    "transform_id was withdrawn at 1.0",
    [schema(
        Part::Payload(Family::Telling),
        "additionalProperties",
        "/transforms/0",
        "'transform_id'"
    )],
    Typed::Refuses
);

negative_case!(
    negative_telling_transform_missing_type,
    "telling-transform-missing-type.json",
    "every transform must declare transform_type",
    [schema(
        Part::Payload(Family::Telling),
        "required",
        "/transforms/0",
        "\"transform_type\""
    )],
    Typed::Refuses
);

/// Conformance §7: the schemas alone accept the 0.3.2 case. This crate
/// rejects it under §3, and that rejection is its only violation.
#[test]
fn the_0_3_2_case_passes_the_envelope_schema_alone() {
    let case = load(&examples().join("negative/envelope-som-version-0-3-2.json"));
    let rejection = Validator::new()
        .unwrap()
        .validate_message(&case["message"])
        .unwrap_err();
    assert!(
        rejection
            .violations
            .iter()
            .all(|v| matches!(v.kind, ViolationKind::WireVersion { .. }))
    );
    let mut accepted = case["message"].clone();
    accepted["som_version"] = Value::from("1.0.0");
    assert!(
        Validator::new()
            .unwrap()
            .validate_message(&accepted)
            .is_ok()
    );
}
