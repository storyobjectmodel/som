//! The envelope builder: `UUIDv7` ordering, the topic rule, the stamped
//! version, deterministic stamping, and stable name-based ids.

mod common;

use std::time::{Duration, UNIX_EPOCH};

use common::{examples, load};
use som::builder::{BuildError, EnvelopeBuilder, RECORD_ID_NAMESPACE, name_based_id};
use som::model::envelope::{OriginatingSystem, SystemType};
use som::model::{Payload, StoryContext, TellingEvent};
use som::{Family, Uuid};

fn system() -> OriginatingSystem {
    OriginatingSystem {
        vendor: Some("example".to_owned()),
        ..OriginatingSystem::new("planning-01", SystemType::Ncs)
    }
}

fn story() -> StoryContext {
    serde_json::from_value(load(&examples().join("story-context/hurricane-beat6.json"))).unwrap()
}

fn builder() -> EnvelopeBuilder {
    EnvelopeBuilder::new(system())
        .correlation_id(name_based_id("planning-01", "hurricane-2026-0911"))
        .topic("som.story.context.hurricane-2026-0911")
}

#[test]
fn built_envelope_is_stamped() {
    let envelope = builder().build(story()).unwrap();
    assert_eq!(envelope.som_version, som::SOM_VERSION);
    assert_eq!(envelope.message_type, "story.context");
    assert_eq!(envelope.originating_system, system());
    assert_eq!(
        envelope.correlation_id,
        name_based_id("planning-01", "hurricane-2026-0911").to_string()
    );
    let id = Uuid::parse_str(&envelope.message_id).unwrap();
    assert_eq!(id.get_version_num(), 7);
    assert_eq!(envelope.message_id, id.hyphenated().to_string());
}

#[cfg(feature = "validate")]
#[test]
fn built_envelope_is_valid() {
    use som::validate::{Outcome, Validator};
    let envelope = builder().build(story()).unwrap();
    let value = serde_json::to_value(&envelope).unwrap();
    assert_eq!(
        Validator::new().unwrap().validate_message(&value),
        Ok(Outcome::Accepted {
            family: Family::StoryContext,
            message_type: "story.context".to_owned()
        })
    );
    let telling: TellingEvent =
        serde_json::from_value(load(&examples().join("telling/started.json"))).unwrap();
    let envelope = builder()
        .topic("som.telling.started")
        .causation_id(Uuid::now_v7())
        .build(Payload::Telling(telling))
        .unwrap();
    let value = serde_json::to_value(&envelope).unwrap();
    assert!(Validator::new().unwrap().validate_message(&value).is_ok());
}

#[test]
fn message_type_comes_from_the_payload() {
    let telling: TellingEvent =
        serde_json::from_value(load(&examples().join("telling/started.json"))).unwrap();
    let envelope = builder()
        .topic("som.telling.started")
        .build(Payload::Telling(telling))
        .unwrap();
    assert_eq!(envelope.message_type, "telling.started");
    assert_eq!(
        Family::from_message_type(&envelope.message_type),
        Some(Family::Telling)
    );
}

#[test]
fn message_ids_are_v7_and_strictly_increasing() {
    let b = builder();
    let ids: Vec<Uuid> = (0..2000)
        .map(|_| Uuid::parse_str(&b.build(story()).unwrap().message_id).unwrap())
        .collect();
    for pair in ids.windows(2) {
        assert_eq!(pair[1].get_version_num(), 7);
        assert!(pair[0] < pair[1], "{} then {}", pair[0], pair[1]);
        assert!(
            pair[0].hyphenated().to_string() < pair[1].hyphenated().to_string(),
            "the text form must sort the same way"
        );
    }
}

/// Identical input, identical output: a caller-minted id and a fixed
/// timestamp make the envelope a pure function of its inputs.
#[test]
fn a_caller_minted_message_id_makes_the_envelope_deterministic() {
    let id = Uuid::parse_str("0199a1c4-7a2e-7b31-8c55-4d2f9e6a1b00").unwrap();
    let at = UNIX_EPOCH + Duration::new(1_789_144_860, 123_456_789);
    let make = || {
        builder()
            .timestamp_at(at)
            .build_with_message_id(id, story())
            .unwrap()
    };
    let (a, b) = (make(), make());
    assert_eq!(a, b);
    assert_eq!(a.message_id, "0199a1c4-7a2e-7b31-8c55-4d2f9e6a1b00");
    assert_eq!(a.timestamp, "2026-09-11T16:41:00.123456Z");
    assert_eq!(
        serde_json::to_value(&a).unwrap(),
        serde_json::to_value(&b).unwrap()
    );
}

#[test]
fn topic_must_begin_with_som() {
    for bad in [
        "story.context",
        "SOM.story.context",
        "som",
        "somx.story",
        "",
    ] {
        assert_eq!(
            builder().topic(bad).build(story()).unwrap_err(),
            BuildError::TopicNotSom(bad.to_owned()),
            "{bad:?}"
        );
    }
    assert!(builder().topic("som.").build(story()).is_ok());
}

#[test]
fn topic_and_correlation_id_are_required() {
    assert_eq!(
        EnvelopeBuilder::new(system())
            .topic("som.story.context")
            .build(story())
            .unwrap_err(),
        BuildError::MissingCorrelationId
    );
    assert_eq!(
        EnvelopeBuilder::new(system())
            .correlation_id(Uuid::now_v7())
            .build(story())
            .unwrap_err(),
        BuildError::MissingTopic
    );
    assert_eq!(
        BuildError::TopicNotSom("x".to_owned()).to_string(),
        "topic \"x\" does not begin with som."
    );
}

#[test]
fn timestamp_defaults_to_now_as_a_valid_date_time() {
    let before = std::time::SystemTime::now();
    let envelope = builder().build(story()).unwrap();
    let stamped = som::time::parse_rfc3339(&envelope.timestamp).unwrap();
    let floor = som::time::parse_rfc3339(&som::time::format_rfc3339_micros(before)).unwrap();
    assert!(
        stamped >= floor,
        "{} is before {}",
        envelope.timestamp,
        floor.unix_seconds
    );
    let fixed = builder()
        .timestamp("2026-09-11T12:41:00-04:00")
        .build(story())
        .unwrap();
    assert_eq!(
        fixed.timestamp, "2026-09-11T12:41:00-04:00",
        "the spelling is kept"
    );
}

#[test]
fn name_based_ids_are_stable_and_scoped() {
    let a = name_based_id("ncs-nyc-01", "story-42");
    assert_eq!(a, name_based_id("ncs-nyc-01", "story-42"));
    assert_eq!(a.get_version_num(), 5);
    assert_ne!(a, name_based_id("ncs-nyc-02", "story-42"));
    assert_ne!(a, name_based_id("ncs-nyc-01", "story-43"));
    assert_ne!(name_based_id("ab", "c"), name_based_id("a", "bc"));
}

/// Fixed vectors, computed independently with Python's `uuid.uuid5`, so any
/// implementation can reproduce the derivation.
#[test]
fn name_based_ids_match_the_documented_derivation() {
    assert_eq!(
        RECORD_ID_NAMESPACE,
        Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            b"https://github.com/storyobjectmodel/som/tree/main/implementations/rust#record-ids"
        )
    );
    assert_eq!(
        RECORD_ID_NAMESPACE.to_string(),
        "28330528-38ae-5396-a8cb-fa4e4ee65de9"
    );
    assert_eq!(
        name_based_id("ncs-nyc-01", "story-42").to_string(),
        "1ad4ea94-4c6e-5474-b375-d0d7e9898c9b"
    );
}
