//! The conformance rules that the example corpus does not exercise on its
//! own: dispatch on `message_type` only (§2), the wire-version rule (§3),
//! format assertion (§4), and ignoring what is not recognised (§5). Every
//! message here is built from a real example in the repository.

#![cfg(feature = "validate")]

mod common;

use common::{examples, load};
use serde_json::{Value, json};
use som::Family;
use som::model::decode_message;
use som::validate::{Outcome, Part, Validator, ViolationKind};

fn story_envelope() -> Value {
    load(&examples().join("story-context/hurricane-beat6-envelope.json"))
}

fn wrap(message_type: &str, topic: &str, payload: Value) -> Value {
    let mut envelope = story_envelope();
    envelope["message_type"] = Value::from(message_type);
    envelope["topic"] = Value::from(topic);
    envelope["payload"] = payload;
    envelope
}

#[test]
fn schemas_compile_without_fetching_anything() {
    let validator = Validator::shared().unwrap();
    assert_eq!(
        validator.retrieval_requests(),
        &[] as &[String],
        "a schema asked for a remote document"
    );
}

#[test]
fn every_message_type_selects_its_family() {
    for &family in Family::ALL {
        for message_type in family.message_types() {
            assert_eq!(Family::from_message_type(message_type), Some(family));
        }
    }
    assert_eq!(Family::from_message_type("som.story.context"), None);
    assert_eq!(Family::from_message_type("story"), None);
}

/// §2: the topic says one family, `message_type` another; `message_type` wins
/// in both directions.
#[test]
fn payload_schema_is_selected_by_message_type_not_topic() {
    let validator = Validator::new().unwrap();
    let telling = load(&examples().join("telling/started.json"));
    let story = story_envelope()["payload"].clone();

    let telling_under_story_topic = wrap("telling.started", "som.story.context.x", telling.clone());
    assert_eq!(
        validator.validate_message(&telling_under_story_topic),
        Ok(Outcome::Accepted {
            family: Family::Telling,
            message_type: "telling.started".to_owned()
        })
    );

    let story_under_telling_topic = wrap("story.context", "som.telling.started", story.clone());
    assert_eq!(
        validator.validate_message(&story_under_telling_topic),
        Ok(Outcome::Accepted {
            family: Family::StoryContext,
            message_type: "story.context".to_owned()
        })
    );

    let rejection = validator
        .validate_message(&wrap("story.context", "som.telling.started", telling))
        .unwrap_err();
    assert!(
        rejection
            .violations
            .iter()
            .all(|v| v.part == Part::Payload(Family::StoryContext))
    );
}

/// §5: an unrecognised `message_type` is ignored, not rejected, once the
/// envelope itself is valid.
#[test]
fn unknown_message_type_is_ignored() {
    let validator = Validator::new().unwrap();
    let message = wrap(
        "skill.suggestion.created",
        "som.skill.suggestion.created",
        json!({"anything": ["at", "all"]}),
    );
    assert_eq!(
        validator.validate_message(&message),
        Ok(Outcome::Ignored {
            message_type: "skill.suggestion.created".to_owned()
        })
    );
    assert!(decode_message(message).unwrap().is_none());
}

#[test]
fn unknown_message_type_with_an_invalid_envelope_is_rejected() {
    let validator = Validator::new().unwrap();
    let mut message = wrap("skill.run.completed", "som.skill.run.completed", json!({}));
    message["message_id"] = Value::from("not-a-uuid");
    let rejection = validator.validate_message(&message).unwrap_err();
    assert_eq!(rejection.violations.len(), 1);
    assert_eq!(rejection.violations[0].part, Part::Envelope);
}

/// §5: unrecognised `com.{vendor}.` extensions are ignored, on the envelope
/// and on the payload, and survive the typed model.
#[test]
fn unknown_vendor_extensions_are_accepted_and_kept() {
    let validator = Validator::new().unwrap();
    let mut message = story_envelope();
    message["extensions"] = json!({"com.example.unheard-of": {"nested": [1, 2.5, null]}});
    message["payload"]["extensions"]["com.other-vendor.flag"] = Value::from(true);
    assert!(matches!(
        validator.validate_message(&message),
        Ok(Outcome::Accepted { .. })
    ));
    let typed = decode_message(message.clone()).unwrap().unwrap();
    assert_eq!(serde_json::to_value(&typed).unwrap(), message);
}

#[test]
fn extension_keys_outside_com_are_refused_by_the_typed_model() {
    let mut message = story_envelope();
    message["payload"]["extensions"] = json!({"org.example.field": 1});
    assert!(decode_message(message.clone()).is_err());
    let rejection = Validator::new()
        .unwrap()
        .validate_message(&message)
        .unwrap_err();
    assert_eq!(rejection.violations.len(), 1);
    assert_eq!(
        rejection.violations[0].part,
        Part::Payload(Family::StoryContext)
    );
    assert_eq!(rejection.violations[0].instance_path, "/extensions");
}

/// §3 and the compatibility policy: every 1.x is accepted, and the version
/// selects nothing.
#[test]
fn any_1_x_wire_version_is_accepted() {
    let validator = Validator::new().unwrap();
    for version in ["1.0.0", "1.1.0", "1.9.42"] {
        let mut message = story_envelope();
        message["som_version"] = Value::from(version);
        assert_eq!(
            validator.validate_message(&message),
            Ok(Outcome::Accepted {
                family: Family::StoryContext,
                message_type: "story.context".to_owned()
            }),
            "{version}"
        );
    }
}

#[test]
fn pre_1_0_and_post_1_x_wire_versions_are_rejected() {
    let validator = Validator::new().unwrap();
    for version in ["0.3.2", "0.3.1", "0.9.0", "2.0.0"] {
        let mut message = story_envelope();
        message["som_version"] = Value::from(version);
        let rejection = validator.validate_message(&message).unwrap_err();
        assert_eq!(
            rejection.violations[0].kind,
            ViolationKind::WireVersion {
                found: version.to_owned()
            }
        );
    }
}

/// §4: formats are asserted in payloads too, not only on the envelope.
#[test]
fn payload_formats_are_asserted() {
    let validator = Validator::new().unwrap();
    let mut telling = load(&examples().join("telling/started.json"));
    telling["telling_id"] = Value::from("NOT-A-UUID");
    telling["exposure_start"] = Value::from("yesterday");
    let rejection = validator
        .validate_payload(Family::Telling, &telling)
        .unwrap_err();
    let mut paths: Vec<&str> = rejection
        .violations
        .iter()
        .filter(|v| v.keyword() == Some("format"))
        .map(|v| v.instance_path.as_str())
        .collect();
    paths.sort_unstable();
    assert_eq!(paths, ["/exposure_start", "/telling_id"]);
}

/// The governed-but-extensible registries admit their seed values and
/// `x-[a-z0-9_]+`, and the typed model agrees with the schema on each.
#[test]
fn registry_values_agree_with_the_schema() {
    let validator = Validator::new().unwrap();
    let story = load(&examples().join("story-context/hurricane-beat6.json"));
    let telling = load(&examples().join("telling/started-with-transforms.json"));
    let cases: [(&str, bool); 9] = [
        ("VIDEO", true),
        ("x-drone_feed", true),
        ("x-2", true),
        ("DRONE", false),
        ("x-", false),
        ("x-Drone", false),
        ("X-drone", false),
        ("x-drone-feed", false),
        ("video", false),
    ];
    for (value, valid) in cases {
        let mut s = story.clone();
        s["assets"][0]["asset_type"] = Value::from(value);
        s["editorial_source"][0]["source_type"] = Value::from(value.replace("VIDEO", "WIRE"));
        let schema_ok = validator.validate_payload(Family::StoryContext, &s).is_ok();
        let typed_ok = som::model::Payload::decode(Family::StoryContext, s).is_ok();
        assert_eq!(
            (schema_ok, typed_ok),
            (valid, valid),
            "story registries: {value:?}"
        );

        let mut t = telling.clone();
        t["transforms"][0]["transform_type"] = Value::from(value.replace("VIDEO", "CROP"));
        let schema_ok = validator.validate_payload(Family::Telling, &t).is_ok();
        let typed_ok = som::model::Payload::decode(Family::Telling, t).is_ok();
        assert_eq!(
            (schema_ok, typed_ok),
            (valid, valid),
            "transform_type: {value:?}"
        );
    }
}
