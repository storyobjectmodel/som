//! The normative schemas, embedded from the repository's own `schema/`
//! directory so the crate can never validate against a stale copy.

use std::fmt;

/// `schema/envelope.schema.json`.
pub const ENVELOPE: &str = include_str!("../../../schema/envelope.schema.json");
/// `schema/story-context.schema.json`.
pub const STORY_CONTEXT: &str = include_str!("../../../schema/story-context.schema.json");
/// `schema/telling-event.schema.json`.
pub const TELLING_EVENT: &str = include_str!("../../../schema/telling-event.schema.json");
/// `schema/link-event.schema.json`.
pub const LINK_EVENT: &str = include_str!("../../../schema/link-event.schema.json");
/// `schema/delivery-media-available.schema.json`.
pub const DELIVERY_MEDIA_AVAILABLE: &str =
    include_str!("../../../schema/delivery-media-available.schema.json");
/// `schema/system-audit.schema.json`.
pub const SYSTEM_AUDIT: &str = include_str!("../../../schema/system-audit.schema.json");
/// `schema/skill-warning.schema.json`.
pub const SKILL_WARNING: &str = include_str!("../../../schema/skill-warning.schema.json");

/// The `$id` of the envelope schema.
pub const ENVELOPE_SCHEMA_ID: &str = "https://storyobjectmodel.com/schema/1.0/envelope.schema.json";

/// A payload family: one payload schema, selected by the envelope's
/// `message_type` and by nothing else (conformance §2).
///
/// Non-exhaustive because a 1.x release may add a family without a major
/// version (compatibility policy); a consumer matches on the families it
/// handles and ignores the rest (conformance §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Family {
    /// `story.context`.
    StoryContext,
    /// `telling.started`, `telling.ended`, `telling.exposed`.
    Telling,
    /// `link.committed`, `link.gate_changed`, `link.withdrawn`.
    Link,
    /// `delivery.media_available`.
    DeliveryMediaAvailable,
    /// `system.audit`.
    SystemAudit,
    /// `skill.warning.raised`.
    SkillWarning,
}

impl Family {
    /// Every payload family SOM 1.0 defines, in schema-file order.
    pub const ALL: &'static [Family] = &[
        Family::StoryContext,
        Family::Telling,
        Family::Link,
        Family::DeliveryMediaAvailable,
        Family::SystemAudit,
        Family::SkillWarning,
    ];

    /// The `message_type` values this family's payload schema admits.
    #[must_use]
    pub const fn message_types(self) -> &'static [&'static str] {
        match self {
            Family::StoryContext => &["story.context"],
            Family::Telling => &["telling.started", "telling.ended", "telling.exposed"],
            Family::Link => &["link.committed", "link.gate_changed", "link.withdrawn"],
            Family::DeliveryMediaAvailable => &["delivery.media_available"],
            Family::SystemAudit => &["system.audit"],
            Family::SkillWarning => &["skill.warning.raised"],
        }
    }

    /// The family a `message_type` names, or `None` for one SOM 1.0 does not
    /// define, which a consumer must ignore rather than reject (conformance §5).
    #[must_use]
    pub fn from_message_type(message_type: &str) -> Option<Family> {
        Family::ALL
            .iter()
            .copied()
            .find(|f| f.message_types().contains(&message_type))
    }

    /// The schema file name under `schema/`.
    #[must_use]
    pub const fn schema_file(self) -> &'static str {
        match self {
            Family::StoryContext => "story-context.schema.json",
            Family::Telling => "telling-event.schema.json",
            Family::Link => "link-event.schema.json",
            Family::DeliveryMediaAvailable => "delivery-media-available.schema.json",
            Family::SystemAudit => "system-audit.schema.json",
            Family::SkillWarning => "skill-warning.schema.json",
        }
    }

    /// The schema's `$id`: the stable URL that identifies and serves it for
    /// the life of 1.x.
    #[must_use]
    pub const fn schema_id(self) -> &'static str {
        match self {
            Family::StoryContext => {
                "https://storyobjectmodel.com/schema/1.0/story-context.schema.json"
            }
            Family::Telling => "https://storyobjectmodel.com/schema/1.0/telling-event.schema.json",
            Family::Link => "https://storyobjectmodel.com/schema/1.0/link-event.schema.json",
            Family::DeliveryMediaAvailable => {
                "https://storyobjectmodel.com/schema/1.0/delivery-media-available.schema.json"
            }
            Family::SystemAudit => {
                "https://storyobjectmodel.com/schema/1.0/system-audit.schema.json"
            }
            Family::SkillWarning => {
                "https://storyobjectmodel.com/schema/1.0/skill-warning.schema.json"
            }
        }
    }

    /// The embedded schema text.
    #[must_use]
    pub const fn schema_source(self) -> &'static str {
        match self {
            Family::StoryContext => STORY_CONTEXT,
            Family::Telling => TELLING_EVENT,
            Family::Link => LINK_EVENT,
            Family::DeliveryMediaAvailable => DELIVERY_MEDIA_AVAILABLE,
            Family::SystemAudit => SYSTEM_AUDIT,
            Family::SkillWarning => SKILL_WARNING,
        }
    }
}

impl fmt::Display for Family {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Family::StoryContext => "story.context",
            Family::Telling => "telling",
            Family::Link => "link",
            Family::DeliveryMediaAvailable => "delivery.media_available",
            Family::SystemAudit => "system.audit",
            Family::SkillWarning => "skill.warning.raised",
        };
        f.write_str(name)
    }
}
