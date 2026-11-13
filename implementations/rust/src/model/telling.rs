//! `telling-event.schema.json`: `telling.started`, `telling.ended`,
//! `telling.exposed`.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::common::{Actor, Extensions, registry};

/// A telling event payload. Which exposure members each `message_type`
/// requires (`telling.started`: `exposure_start` and no `exposure_end`;
/// `telling.ended`: `exposure_end`; `telling.exposed`: both) is enforced by
/// the validator, not by this type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TellingEvent {
    /// Which telling event this is.
    pub message_type: TellingEventType,
    /// UUID of the telling.
    pub telling_id: String,
    /// UUID of the link this telling followed from.
    pub link_id: String,
    /// RFC 3339; immutable once stamped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exposure_start: Option<String>,
    /// RFC 3339; immutable once stamped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exposure_end: Option<String>,
    /// RFC 3339; mutable intended air, never used to derive on-air state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheduled_start: Option<String>,
    /// Vendor fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
    /// Application-ordered, append-only edge reshapes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transforms: Option<Vec<Transform>>,
}

/// `message_type` of a telling event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TellingEventType {
    /// `telling.started`.
    #[serde(rename = "telling.started")]
    Started,
    /// `telling.ended`.
    #[serde(rename = "telling.ended")]
    Ended,
    /// `telling.exposed`.
    #[serde(rename = "telling.exposed")]
    Exposed,
}

impl TellingEventType {
    /// The wire value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            TellingEventType::Started => "telling.started",
            TellingEventType::Ended => "telling.ended",
            TellingEventType::Exposed => "telling.exposed",
        }
    }
}

/// One edge reshape recorded against a telling. The withdrawn
/// `transform_id` is refused.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transform {
    /// What reshape.
    pub transform_type: TransformType,
    /// Per-type parameters; the schema leaves their shape open.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Map<String, Value>>,
    /// Who applied it.
    pub applied_by: Actor,
    /// RFC 3339.
    pub applied_at: String,
}

registry! {
    /// `transforms[].transform_type`.
    TransformType {
        /// `CROP`.
        Crop => "CROP",
        /// `TRIM`.
        Trim => "TRIM",
        /// `CAPTION_BURN`.
        CaptionBurn => "CAPTION_BURN",
    }
}
