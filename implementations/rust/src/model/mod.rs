//! Typed models of the envelope and the six payload families.
//!
//! The types mirror the schemas' structure: closed objects refuse unknown
//! members, closed enums carry exactly the schema's members, registries admit
//! `x-` vendor values, and optional members stay optional. A value decoded
//! and re-encoded is equal, as JSON, to the value it came from.
//!
//! Identifiers and date-times stay `String`s on purpose. A `Uuid` re-encodes
//! as lowercase hyphenated text and a parsed date-time loses its spelling, so
//! either would break the round-trip the model promises; parse them at the
//! point of use with [`crate::Uuid`] and [`crate::time::parse_rfc3339`].
//!
//! The types do not replace validation. String formats and patterns, numeric
//! ranges and the schemas' conditional rules are checked by the validator
//! (`som::validate`, feature `validate`), which is the authority on what is
//! valid. One known divergence: JSON Schema's `integer` admits `3.0`, and the
//! typed model's integer members do not.

pub mod audit;
pub mod common;
pub mod delivery;
pub mod envelope;
pub mod link;
pub mod skill_warning;
pub mod story_context;
pub mod telling;

use std::fmt;

use serde::{Serialize, Serializer};
use serde_json::Value;

use crate::schemas::Family;

pub use audit::SystemAudit;
pub use common::{
    Actor, Extensions, InvalidExtensionKey, Locator, TimeRanges, UnknownRegistryValue,
};
pub use delivery::MediaAvailable;
pub use envelope::{Envelope, OriginatingSystem, SystemType};
pub use link::LinkEvent;
pub use skill_warning::SkillWarning;
pub use story_context::StoryContext;
pub use telling::TellingEvent;

/// A typed payload that knows the `message_type` its envelope must carry.
///
/// Implemented by the six family payloads and by [`Payload`]. A producer may
/// implement it for a payload outside the 1.0 families, such as
/// `skill.suggestion.created`, which the envelope schema names as an example
/// and a conformant consumer ignores (conformance §5).
pub trait MessagePayload: Serialize {
    /// The envelope `message_type` for this payload.
    fn message_type(&self) -> &str;

    /// The SOM 1.0 family this payload belongs to, or `None` for a
    /// `message_type` outside the six.
    fn family(&self) -> Option<Family> {
        Family::from_message_type(self.message_type())
    }
}

impl MessagePayload for StoryContext {
    fn message_type(&self) -> &'static str {
        "story.context"
    }
}

impl MessagePayload for TellingEvent {
    fn message_type(&self) -> &str {
        self.message_type.as_str()
    }
}

impl MessagePayload for LinkEvent {
    fn message_type(&self) -> &str {
        self.message_type.as_str()
    }
}

impl MessagePayload for MediaAvailable {
    fn message_type(&self) -> &'static str {
        "delivery.media_available"
    }
}

impl MessagePayload for SystemAudit {
    fn message_type(&self) -> &'static str {
        "system.audit"
    }
}

impl MessagePayload for SkillWarning {
    fn message_type(&self) -> &'static str {
        "skill.warning.raised"
    }
}

/// Any SOM 1.0 payload. Non-exhaustive for the same reason as [`Family`].
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Payload {
    /// `story.context`.
    StoryContext(Box<StoryContext>),
    /// `telling.*`.
    Telling(TellingEvent),
    /// `link.*`.
    Link(LinkEvent),
    /// `delivery.media_available`.
    MediaAvailable(MediaAvailable),
    /// `system.audit`.
    SystemAudit(SystemAudit),
    /// `skill.warning.raised`.
    SkillWarning(SkillWarning),
}

impl Payload {
    /// Decodes `payload` as the given family.
    ///
    /// # Errors
    ///
    /// The `serde_json` error if the value does not have the family's shape:
    /// a missing required member, an unknown member of a closed object, or a
    /// value outside a closed enum.
    pub fn decode(family: Family, payload: Value) -> Result<Payload, serde_json::Error> {
        Ok(match family {
            Family::StoryContext => Payload::StoryContext(serde_json::from_value(payload)?),
            Family::Telling => Payload::Telling(serde_json::from_value(payload)?),
            Family::Link => Payload::Link(serde_json::from_value(payload)?),
            Family::DeliveryMediaAvailable => {
                Payload::MediaAvailable(serde_json::from_value(payload)?)
            }
            Family::SystemAudit => Payload::SystemAudit(serde_json::from_value(payload)?),
            Family::SkillWarning => Payload::SkillWarning(serde_json::from_value(payload)?),
        })
    }
}

impl Serialize for Payload {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Payload::StoryContext(p) => p.serialize(serializer),
            Payload::Telling(p) => p.serialize(serializer),
            Payload::Link(p) => p.serialize(serializer),
            Payload::MediaAvailable(p) => p.serialize(serializer),
            Payload::SystemAudit(p) => p.serialize(serializer),
            Payload::SkillWarning(p) => p.serialize(serializer),
        }
    }
}

impl MessagePayload for Payload {
    fn message_type(&self) -> &str {
        match self {
            Payload::StoryContext(p) => p.message_type(),
            Payload::Telling(p) => p.message_type(),
            Payload::Link(p) => p.message_type(),
            Payload::MediaAvailable(p) => p.message_type(),
            Payload::SystemAudit(p) => p.message_type(),
            Payload::SkillWarning(p) => p.message_type(),
        }
    }
}

/// Why a message could not be decoded into typed form.
#[derive(Debug)]
#[non_exhaustive]
pub enum DecodeError {
    /// The envelope did not decode.
    Envelope(serde_json::Error),
    /// The payload did not decode as the family `message_type` names.
    Payload(Family, serde_json::Error),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::Envelope(e) => write!(f, "envelope: {e}"),
            DecodeError::Payload(family, e) => write!(f, "{family} payload: {e}"),
        }
    }
}

impl std::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DecodeError::Envelope(e) | DecodeError::Payload(_, e) => Some(e),
        }
    }
}

/// Decodes a wire message into a typed envelope, selecting the payload type
/// from `message_type` alone. Returns `Ok(None)` for a `message_type` SOM 1.0
/// does not define, which a consumer ignores.
///
/// Decoding is not validation: run the validator first.
///
/// # Errors
///
/// [`DecodeError`] if the envelope or the payload does not have its typed
/// shape.
pub fn decode_message(message: Value) -> Result<Option<Envelope<Payload>>, DecodeError> {
    let envelope: Envelope = serde_json::from_value(message).map_err(DecodeError::Envelope)?;
    let Some(family) = Family::from_message_type(&envelope.message_type) else {
        return Ok(None);
    };
    envelope
        .try_map_payload(|payload| Payload::decode(family, Value::Object(payload)))
        .map(Some)
        .map_err(|e| DecodeError::Payload(family, e))
}
