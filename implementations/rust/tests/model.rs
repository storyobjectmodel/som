//! The typed model on its own, without the validator: every example
//! round-trips, every typed enum is pinned to the schema's enum array, and
//! the model's small API behaves. These run at every feature set.

mod common;

use std::str::FromStr;

use common::{EXAMPLE_DIRS, envelope_schema, examples, is_envelope, json_files, load, schema};
use serde::Serialize;
use serde_json::{Value, json};
use som::Family;
use som::model::audit::{AuditAction, AuditTargetKind, SystemAuditType};
use som::model::delivery::MediaAvailableType;
use som::model::envelope::{Envelope, OriginatingSystem, SystemType};
use som::model::link::{ComplianceGateStatus, LinkEventType};
use som::model::skill_warning::{Severity, SkillWarningType};
use som::model::story_context::{
    AcquisitionState, AssertionTargetKind, AssertionType, AssetStatus, AssetType, Author,
    ClearanceStatus, ComplianceSeverity, ComplianceStatus, Credibility, DisclosureLevel,
    EvidentialPosition, GateBlockKind, GateStatus, GovernmentApprovalStatus, LifecyclePhase,
    MigrationPolicy, PremiseConfidence, PriorityLevel, Review, SkillPriority, SkillType,
    SourceType, StoryType, UsageState,
};
use som::model::telling::{TellingEventType, TransformType};
use som::model::{
    DecodeError, Extensions, MessagePayload, Payload, UnknownRegistryValue, decode_message,
};

#[test]
fn every_example_directory_is_covered() {
    let mut on_disk: Vec<String> = std::fs::read_dir(examples())
        .unwrap()
        .map(|e| e.unwrap())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().into_string().unwrap())
        .filter(|name| name != "negative")
        .collect();
    on_disk.sort();
    let mut declared: Vec<String> = EXAMPLE_DIRS
        .iter()
        .map(|(d, _, _)| (*d).to_owned())
        .collect();
    declared.sort();
    assert_eq!(on_disk, declared, "an example directory has no family");
}

#[test]
fn every_example_round_trips_through_the_typed_model() {
    let mut checked = 0;
    for (dir, family, _) in EXAMPLE_DIRS {
        for path in json_files(&examples().join(dir)) {
            let document = load(&path);
            let encoded: Value = if is_envelope(&document) {
                let typed = decode_message(document.clone())
                    .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
                    .expect("a SOM 1.0 message_type");
                assert!(matches!(
                    (&typed.payload, family),
                    (Payload::StoryContext(_), Family::StoryContext)
                ));
                assert_eq!(typed.payload.family(), Some(family));
                serde_json::to_value(&typed).unwrap()
            } else {
                let typed = Payload::decode(family, document.clone())
                    .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
                assert_eq!(typed.family(), Some(family));
                serde_json::to_value(&typed).unwrap()
            };
            assert_eq!(encoded, document, "{} did not round-trip", path.display());
            checked += 1;
        }
    }
    assert_eq!(checked, 18, "the corpus README counts 18 worked messages");
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
    assert_eq!(Family::from_message_type("STORY.CONTEXT"), None);
}

/// The embedded schema's `$id` is the documented stable URL, for each family
/// and for the envelope.
#[test]
fn schema_ids_are_the_published_urls() {
    for &family in Family::ALL {
        let id = schema(family)["$id"].as_str().unwrap().to_owned();
        assert_eq!(id, family.schema_id(), "{family}");
        assert!(id.ends_with(family.schema_file()), "{family}");
    }
    assert_eq!(
        envelope_schema()["$id"].as_str().unwrap(),
        som::schemas::ENVELOPE_SCHEMA_ID
    );
}

/// The variants of a closed enum, with an exhaustive `match` beside the list:
/// a variant added to the enum does not compile here until it is added to the
/// list, and the list is then checked against the schema.
macro_rules! variants {
    ($ty:ident { $($v:ident),+ $(,)? }) => {{
        fn exhaustive(v: $ty) {
            match v {
                $($ty::$v => {}),+
            }
        }
        let _ = exhaustive;
        vec![$($ty::$v),+]
    }};
}

fn wire_values<T: Serialize>(variants: &[T]) -> Vec<Value> {
    variants
        .iter()
        .map(|v| serde_json::to_value(v).unwrap())
        .collect()
}

fn schema_values(schema: &Value, pointer: &str) -> Vec<Value> {
    match schema.pointer(pointer) {
        Some(Value::Array(items)) => items.clone(),
        Some(constant) => vec![constant.clone()],
        None => panic!("no enum or const at {pointer}"),
    }
}

fn pin<T: Serialize>(schema: &Value, pointer: &str, variants: &[T]) {
    let mut expected = schema_values(schema, pointer);
    let mut actual = wire_values(variants);
    expected.sort_by_key(ToString::to_string);
    actual.sort_by_key(ToString::to_string);
    assert_eq!(actual, expected, "{pointer}");
}

/// Every closed enum in the typed model carries exactly the members the
/// schema lists, no more and no fewer; every registry's seed values match
/// the schema's seed list. A schema change that adds a member fails here,
/// which is the point: the typed model must follow.
// One schema, one table: splitting the pins across functions would hide
// which members are covered.
#[allow(clippy::too_many_lines)]
#[test]
fn story_context_enums_carry_exactly_the_schema_members() {
    let story = schema(Family::StoryContext);
    pin(
        &story,
        "/properties/story_type/enum",
        &variants!(StoryType {
            Planned,
            Active,
            Killed,
            Spiked,
            Archived,
            Orphan
        }),
    );
    let phases = variants!(LifecyclePhase {
        Developing,
        ReadyToAir,
        Breaking,
        Published
    });
    pin(&story, "/$defs/lifecycle/properties/phase/enum", &phases);
    pin(
        &story,
        "/$defs/lifecycle/properties/previous_phase/enum",
        &phases,
    );
    pin(
        &story,
        "/properties/priority/properties/level/enum",
        &variants!(PriorityLevel {
            Routine,
            Standard,
            High,
            Urgent,
            Flash
        }),
    );
    pin(
        &story,
        "/properties/premise/properties/confidence/enum",
        &variants!(PremiseConfidence { Low, Medium, High }),
    );
    pin(
        &story,
        "/$defs/compliance_flag/properties/severity/enum",
        &variants!(ComplianceSeverity {
            Low,
            Medium,
            High,
            Critical
        }),
    );
    pin(
        &story,
        "/$defs/compliance_flag/properties/status/enum",
        &variants!(ComplianceStatus {
            Active,
            Resolved,
            Waived
        }),
    );
    pin(
        &story,
        "/$defs/editorial_gate/properties/status/enum",
        &variants!(GateStatus {
            Pending,
            Approved,
            Rejected
        }),
    );
    pin(
        &story,
        "/$defs/editorial_gate/properties/blocks/items/anyOf/0/properties/kind/enum",
        &[GateBlockKind::Asset, GateBlockKind::Phase],
    );
    pin(
        &story,
        "/properties/government_approval/properties/status/enum",
        &variants!(GovernmentApprovalStatus {
            NotRequired,
            Requested,
            UnderReview,
            Approved,
            ReturnedForRevision,
            Rejected
        }),
    );
    pin(
        &story,
        "/$defs/editorial_source_entry/properties/source_type/anyOf/0/enum",
        SourceType::SEED,
    );
    assert_eq!(SourceType::SEED.len(), 11);
    pin(
        &story,
        "/$defs/editorial_source_entry/properties/credibility/enum",
        &variants!(Credibility {
            Trusted,
            Verified,
            Endorsed,
            Unverified
        }),
    );
    pin(
        &story,
        "/$defs/asset/properties/asset_type/anyOf/0/enum",
        AssetType::SEED,
    );
    assert_eq!(AssetType::SEED.len(), 17);
    pin(
        &story,
        "/$defs/asset/properties/status/enum",
        &variants!(AssetStatus {
            Ready,
            InProduction,
            Prepared,
            Invalidated
        }),
    );
    pin(
        &story,
        "/$defs/asset/properties/evidential_position/enum",
        &variants!(EvidentialPosition {
            Primary,
            Secondary,
            Tertiary
        }),
    );
    pin(
        &story,
        "/$defs/asset/properties/acquisition_state/enum",
        &[AcquisitionState::Capturing, AcquisitionState::Captured],
    );
    pin(
        &story,
        "/$defs/asset/properties/usage/items/properties/state/const",
        &variants!(UsageState { Committed }),
    );
    pin(
        &story,
        "/$defs/asset/properties/standards_clearance/properties/status/enum",
        &variants!(ClearanceStatus {
            Cleared,
            Blocked,
            Pending
        }),
    );
    let skills = "/$defs/skills_config/properties/active_skills/items/properties";
    pin(
        &story,
        &format!("{skills}/skill_type/enum"),
        &[SkillType::Newsroom, SkillType::Vendor, SkillType::Reference],
    );
    pin(
        &story,
        &format!("{skills}/disclosure_level/enum"),
        &variants!(DisclosureLevel { L1, L2, L3 }),
    );
    pin(
        &story,
        &format!("{skills}/migration_policy/enum"),
        &variants!(MigrationPolicy { Hot, Cold, Gated }),
    );
    pin(
        &story,
        &format!("{skills}/skill_priority/enum"),
        &variants!(SkillPriority {
            Critical,
            Normal,
            Low
        }),
    );
    pin(
        &story,
        "/$defs/review_state/properties/state/enum",
        &[Review::Pending, Review::Confirmed, Review::Rejected],
    );
    pin(
        &story,
        "/$defs/authorship_provenance/properties/author/enum",
        &[Author::Human, Author::Model],
    );
    pin(
        &story,
        "/$defs/assertion/properties/assertion_type/enum",
        &variants!(AssertionType {
            FactCheck,
            Detection,
            Match
        }),
    );
    pin(
        &story,
        "/$defs/assertion/properties/target/properties/kind/enum",
        &variants!(AssertionTargetKind {
            Asset,
            Link,
            Telling,
            Story
        }),
    );

    pin(
        &envelope_schema(),
        "/$defs/originating_system/properties/system_type/enum",
        &variants!(SystemType {
            Ncs,
            MosDevice,
            Graphics,
            Automation,
            WireService,
            AiAgent,
            ComplianceEngine,
            EditorialDashboard,
            Archive,
            Prompter,
            Camera,
            Audio,
            SkillWorker,
            Custom
        }),
    );
}

#[test]
fn event_enums_carry_exactly_the_schema_members() {
    let telling = schema(Family::Telling);
    pin(
        &telling,
        "/properties/message_type/enum",
        &variants!(TellingEventType {
            Started,
            Ended,
            Exposed
        }),
    );
    pin(
        &telling,
        "/properties/transforms/items/properties/transform_type/anyOf/0/enum",
        TransformType::SEED,
    );

    let link = schema(Family::Link);
    pin(
        &link,
        "/properties/message_type/enum",
        &variants!(LinkEventType {
            Committed,
            GateChanged,
            Withdrawn
        }),
    );
    pin(
        &link,
        "/properties/compliance_gate_status/enum",
        &variants!(ComplianceGateStatus {
            Pending,
            Cleared,
            Blocked
        }),
    );

    pin(
        &schema(Family::DeliveryMediaAvailable),
        "/properties/message_type/const",
        &variants!(MediaAvailableType { MediaAvailable }),
    );
}

#[test]
fn governance_enums_carry_exactly_the_schema_members() {
    let audit = schema(Family::SystemAudit);
    pin(
        &audit,
        "/properties/message_type/const",
        &variants!(SystemAuditType { SystemAudit }),
    );
    pin(
        &audit,
        "/properties/action/enum",
        &variants!(AuditAction {
            Cleared,
            Suppressed,
            Withheld,
            Overridden
        }),
    );
    pin(
        &audit,
        "/properties/target/properties/kind/enum",
        &variants!(AuditTargetKind {
            Link,
            Asset,
            Telling
        }),
    );

    let warning = schema(Family::SkillWarning);
    pin(
        &warning,
        "/properties/severity/enum",
        &[Severity::Hold, Severity::Flag, Severity::Inform],
    );
    pin(
        &warning,
        "/properties/message_type/const",
        &variants!(SkillWarningType { Raised }),
    );
}

/// The family's `message_type` values are the schema's: the enum or const
/// of the payload's own `message_type`, where the payload carries one.
#[test]
fn family_message_types_match_the_payload_schemas() {
    for &family in Family::ALL {
        let payload_schema = schema(family);
        let Some(declared) = payload_schema.pointer("/properties/message_type") else {
            assert_eq!(
                family,
                Family::StoryContext,
                "{family} has no payload message_type"
            );
            continue;
        };
        let values = schema_values(
            declared,
            if declared.get("enum").is_some() {
                "/enum"
            } else {
                "/const"
            },
        );
        let expected: Vec<Value> = family
            .message_types()
            .iter()
            .map(|&t| Value::from(t))
            .collect();
        assert_eq!(values, expected, "{family}");
    }
}

/// The patterns the typed model hand-codes are the schemas' patterns.
#[test]
fn hand_coded_patterns_match_the_schemas() {
    const EXTENSIONS_KEY: &str = "^com\\.[a-z0-9-]+\\.";
    const REGISTRY_EXTENSION: &str = "^x-[a-z0-9_]+$";

    let mut extension_objects = 0;
    let mut visit = |value: &Value| {
        let mut stack = vec![value];
        while let Some(v) = stack.pop() {
            if let Value::Object(map) = v {
                if let Some(Value::Object(patterns)) = map.get("patternProperties") {
                    let keys: Vec<&String> = patterns.keys().collect();
                    assert_eq!(keys, [EXTENSIONS_KEY]);
                    assert_eq!(map.get("additionalProperties"), Some(&Value::Bool(false)));
                    extension_objects += 1;
                }
                stack.extend(map.values());
            } else if let Value::Array(items) = v {
                stack.extend(items);
            }
        }
    };
    visit(&envelope_schema());
    for &family in Family::ALL {
        visit(&schema(family));
    }
    assert_eq!(
        extension_objects, 6,
        "envelope and five payload families carry extensions"
    );

    let story = schema(Family::StoryContext);
    for pointer in [
        "/$defs/asset/properties/asset_type/anyOf/1/pattern",
        "/$defs/editorial_source_entry/properties/source_type/anyOf/1/pattern",
    ] {
        assert_eq!(
            story.pointer(pointer).unwrap(),
            REGISTRY_EXTENSION,
            "{pointer}"
        );
    }
    assert_eq!(
        schema(Family::Telling)
            .pointer("/properties/transforms/items/properties/transform_type/anyOf/1/pattern")
            .unwrap(),
        REGISTRY_EXTENSION
    );
}

#[test]
fn extension_keys() {
    use som::model::common::{is_extension_key, is_registry_extension};
    for ok in ["com.example.x", "com.a-1.y.z", "com.v."] {
        assert!(is_extension_key(ok), "{ok}");
    }
    for bad in [
        "com.",
        "com..x",
        "com.Example.x",
        "org.example.x",
        "com.ex_ample.x",
        "comx.a.b",
    ] {
        assert!(!is_extension_key(bad), "{bad}");
    }
    for ok in ["x-a", "x-drone_feed", "x-2"] {
        assert!(is_registry_extension(ok), "{ok}");
    }
    for bad in ["x-", "x-A", "X-a", "x-a-b", "xa"] {
        assert!(!is_registry_extension(bad), "{bad}");
    }
}

#[test]
fn extensions_refuse_keys_outside_the_namespace() {
    let mut extensions = Extensions::new();
    assert_eq!(extensions.insert("com.example.flag", json!(true)), Ok(None));
    assert_eq!(
        extensions.insert("com.example.flag", json!(false)),
        Ok(Some(json!(true)))
    );
    let refused = extensions.insert("org.example.flag", json!(1)).unwrap_err();
    assert_eq!(refused.key(), "org.example.flag");
    assert_eq!(
        refused.to_string(),
        "extension key \"org.example.flag\" is not namespaced com.{vendor}."
    );
    assert_eq!(extensions.len(), 1);
    assert_eq!(extensions.get("com.example.flag"), Some(&json!(false)));
    assert_eq!((&extensions).into_iter().count(), 1);

    let decoded: Result<Extensions, _> = serde_json::from_value(json!({"vendor_field": 1}));
    assert!(decoded.unwrap_err().to_string().contains("vendor_field"));
}

#[test]
fn registries_parse_seed_and_vendor_values_and_refuse_the_rest() {
    assert_eq!(AssetType::from_str("VIDEO"), Ok(AssetType::Video));
    assert_eq!(
        "x-drone_feed".parse::<AssetType>(),
        Ok(AssetType::Extension("x-drone_feed".to_owned()))
    );
    assert_eq!(AssetType::Video.to_string(), "VIDEO");
    assert_eq!(AssetType::Video.as_str(), "VIDEO");
    let refused: UnknownRegistryValue = "video".parse::<AssetType>().unwrap_err();
    assert_eq!(refused.registry, "AssetType");
    assert_eq!(refused.value, "video");
    assert_eq!(
        refused.to_string(),
        "\"video\" is neither a AssetType seed value nor an x- vendor value"
    );
    assert!("x-Drone".parse::<SourceType>().is_err());
    assert_eq!(
        "x-a".parse::<TransformType>(),
        Ok(TransformType::Extension("x-a".to_owned()))
    );
    let json = serde_json::to_value(TransformType::CaptionBurn).unwrap();
    assert_eq!(json, "CAPTION_BURN");
}

#[test]
fn story_type_terminal_set() {
    for t in [StoryType::Killed, StoryType::Spiked, StoryType::Archived] {
        assert!(t.is_terminal(), "{t:?}");
    }
    for t in [StoryType::Planned, StoryType::Active, StoryType::Orphan] {
        assert!(!t.is_terminal(), "{t:?}");
    }
}

#[test]
fn envelope_map_payload_keeps_every_other_member() {
    let message = load(&examples().join("story-context/hurricane-beat6-envelope.json"));
    let envelope: Envelope = serde_json::from_value(message.clone()).unwrap();
    let mapped = envelope.clone().map_payload(|p| p.len());
    assert_eq!(
        mapped.payload,
        message["payload"].as_object().unwrap().len()
    );
    assert_eq!(mapped.message_id, envelope.message_id);
    let failed: Result<Envelope<()>, &str> = envelope.try_map_payload(|_| Err("no"));
    assert_eq!(failed.unwrap_err(), "no");
}

#[test]
fn decode_errors_name_the_part_and_carry_their_source() {
    use std::error::Error;
    let mut message = load(&examples().join("story-context/hurricane-beat6-envelope.json"));
    message["payload"]["story_type"] = Value::from("LIVE");
    let err = decode_message(message.clone()).unwrap_err();
    assert!(matches!(err, DecodeError::Payload(Family::StoryContext, _)));
    assert!(err.to_string().starts_with("story.context payload:"));
    assert!(err.source().is_some());

    message["payload"]["story_type"] = Value::from("ACTIVE");
    message["signature"] = Value::from("x");
    let err = decode_message(message).unwrap_err();
    assert!(matches!(err, DecodeError::Envelope(_)));
    assert!(err.to_string().contains("signature"));
}

/// A producer may envelope a payload outside the six families; the
/// `message_type` is its own and the family is `None`.
#[test]
fn a_custom_payload_has_no_family() {
    #[derive(Serialize)]
    struct Suggestion {
        text: String,
    }
    impl MessagePayload for Suggestion {
        fn message_type(&self) -> &'static str {
            "skill.suggestion.created"
        }
    }
    let suggestion = Suggestion {
        text: "lead with the evacuation order".to_owned(),
    };
    assert_eq!(suggestion.family(), None);
    let envelope = som::builder::EnvelopeBuilder::new(OriginatingSystem::new(
        "skills-01",
        SystemType::SkillWorker,
    ))
    .correlation_id(som::Uuid::nil())
    .topic("som.skill.suggestion.created")
    .build(suggestion)
    .unwrap();
    assert_eq!(envelope.message_type, "skill.suggestion.created");
    let wire = serde_json::to_value(&envelope).unwrap();
    assert!(decode_message(wire).unwrap().is_none());
}

/// Documented divergence: JSON Schema's `integer` admits `3.0`; the typed
/// model's integer members do not.
#[test]
fn integral_floats_are_refused_by_the_typed_model() {
    let mut story = load(&examples().join("story-context/hurricane-beat6.json"));
    story["sequence_number"] = json!(6.0);
    assert!(Payload::decode(Family::StoryContext, story).is_err());
}
