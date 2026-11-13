//! `envelope.schema.json`.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::common::Extensions;

/// The envelope every SOM message carries. `P` is the payload: a typed family
/// payload, or an untyped JSON object by default.
///
/// The withdrawn `source` and `signature` members are refused on decode, as
/// is any other member the schema does not list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope<P = Map<String, Value>> {
    /// The schema pack version the payload conforms to. Informative only.
    pub som_version: String,
    /// A UUID; `UUIDv7` recommended.
    pub message_id: String,
    /// A UUID linking every message about one story lifecycle.
    pub correlation_id: String,
    /// The UUID of the message that directly caused this one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<String>,
    /// The only parsing discriminator.
    pub message_type: String,
    /// RFC 3339 date-time: the authoritative time of the event.
    pub timestamp: String,
    /// Who published this message.
    pub originating_system: OriginatingSystem,
    /// Begins `som.`.
    pub topic: String,
    /// On snapshot-style messages such as `story.context`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modification_header: Option<ModificationHeader>,
    /// Short-key actor lookups; the schema's `_actors`.
    #[serde(rename = "_actors", default, skip_serializing_if = "Option::is_none")]
    pub actors: Option<Map<String, Value>>,
    /// The payload.
    pub payload: P,
    /// Vendor fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
    /// JSON-LD context; deferred, permitted for forward compatibility. The
    /// schema's `@context`.
    #[serde(rename = "@context", default, skip_serializing_if = "Option::is_none")]
    pub context: Option<JsonLdContext>,
}

impl<P> Envelope<P> {
    /// Replaces the payload and keeps every other member.
    pub fn map_payload<Q>(self, f: impl FnOnce(P) -> Q) -> Envelope<Q> {
        match self.try_map_payload(|p| Ok::<Q, std::convert::Infallible>(f(p))) {
            Ok(envelope) => envelope,
        }
    }

    /// Replaces the payload with a fallible conversion, such as decoding an
    /// untyped object into a family payload.
    ///
    /// # Errors
    ///
    /// Whatever `f` returns; the envelope is consumed either way.
    pub fn try_map_payload<Q, E>(
        self,
        f: impl FnOnce(P) -> Result<Q, E>,
    ) -> Result<Envelope<Q>, E> {
        let Envelope {
            som_version,
            message_id,
            correlation_id,
            causation_id,
            message_type,
            timestamp,
            originating_system,
            topic,
            modification_header,
            actors,
            payload,
            extensions,
            context,
        } = self;
        Ok(Envelope {
            som_version,
            message_id,
            correlation_id,
            causation_id,
            message_type,
            timestamp,
            originating_system,
            topic,
            modification_header,
            actors,
            payload: f(payload)?,
            extensions,
            context,
        })
    }
}

/// The publisher of a message, re-stamped by whoever publishes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OriginatingSystem {
    /// The publishing system's identifier.
    pub system_id: String,
    /// The kind of system; a closed enum.
    pub system_type: SystemType,
    /// Display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_name: Option<String>,
    /// Vendor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    /// Version of the publishing system.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl OriginatingSystem {
    /// A publisher with the two required members and nothing else.
    pub fn new(system_id: impl Into<String>, system_type: SystemType) -> Self {
        OriginatingSystem {
            system_id: system_id.into(),
            system_type,
            system_name: None,
            vendor: None,
            version: None,
        }
    }
}

/// `originating_system.system_type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemType {
    /// `ncs`: a newsroom computer system.
    Ncs,
    /// `mos_device`.
    MosDevice,
    /// `graphics`.
    Graphics,
    /// `automation`.
    Automation,
    /// `wire_service`.
    WireService,
    /// `ai_agent`.
    AiAgent,
    /// `compliance_engine`.
    ComplianceEngine,
    /// `editorial_dashboard`.
    EditorialDashboard,
    /// `archive`.
    Archive,
    /// `prompter`.
    Prompter,
    /// `camera`.
    Camera,
    /// `audio`.
    Audio,
    /// `skill_worker`.
    SkillWorker,
    /// `custom`.
    Custom,
}

/// `modification_header`. The schema does not close this object, so members
/// it does not name are kept in `other`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModificationHeader {
    /// Story version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub story_version: Option<i64>,
    /// RFC 3339 date-time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
    /// Who modified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified_by: Option<String>,
    /// What changed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_summary: Option<String>,
    /// History entries; their shape is not specified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<Value>>,
    /// Members the schema does not name.
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

/// `@context`: a string or an object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonLdContext {
    /// An IRI.
    Iri(String),
    /// An inline context.
    Object(Map<String, Value>),
}
