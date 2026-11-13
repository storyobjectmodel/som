//! Building envelopes for publication, and deriving stable ids.

use std::fmt;
use std::time::SystemTime;

use serde_json::{Map, Value};
use uuid::Uuid;

use crate::SOM_VERSION;
use crate::model::envelope::{Envelope, JsonLdContext, ModificationHeader, OriginatingSystem};
use crate::model::{Extensions, MessagePayload};
use crate::time::format_rfc3339_micros;

/// The namespace under which [`name_based_id`] derives `UUIDv5` ids:
/// `28330528-38ae-5396-a8cb-fa4e4ee65de9`, which is itself the `UUIDv5` of the
/// URL `https://github.com/storyobjectmodel/som/tree/main/implementations/rust#record-ids`
/// in the RFC 9562 URL namespace.
pub const RECORD_ID_NAMESPACE: Uuid = Uuid::from_bytes([
    0x28, 0x33, 0x05, 0x28, 0x38, 0xae, 0x53, 0x96, 0xa8, 0xcb, 0xfa, 0x4e, 0x4e, 0xe6, 0x5d, 0xe9,
]);

/// Derives a stable UUID for a producer's own record, so the same record
/// always maps to the same SOM id and a replay is idempotent.
///
/// `scope` names the producer (its `system_id` is a good choice) so two
/// producers with the same record id do not collide. The id is the `UUIDv5`,
/// under [`RECORD_ID_NAMESPACE`], of the UTF-8 name
/// `{byte length of scope}:{scope}{record_id}`; the length prefix keeps
/// `("ab", "c")` and `("a", "bc")` apart. Any language can reproduce it.
///
/// Use it for ids that identify a record (`correlation_id`, `telling_id`,
/// `link_id`, `delivery_id`, `audit_id`), never for `message_id`, which is
/// unique per publication.
#[must_use]
pub fn name_based_id(scope: &str, record_id: &str) -> Uuid {
    let name = format!("{}:{scope}{record_id}", scope.len());
    Uuid::new_v5(&RECORD_ID_NAMESPACE, name.as_bytes())
}

/// Why an envelope could not be built.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BuildError {
    /// No `correlation_id` was given; every message requires one.
    MissingCorrelationId,
    /// No `topic` was given.
    MissingTopic,
    /// The topic does not begin with `som.`.
    TopicNotSom(String),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildError::MissingCorrelationId => f.write_str("correlation_id is required"),
            BuildError::MissingTopic => f.write_str("topic is required"),
            BuildError::TopicNotSom(topic) => write!(f, "topic {topic:?} does not begin with som."),
        }
    }
}

impl std::error::Error for BuildError {}

/// Builds an [`Envelope`] around a typed payload.
///
/// The builder stamps `som_version` `1.0.0`, mints a fresh `UUIDv7`
/// `message_id` for every envelope, takes `message_type` from the payload,
/// and stamps `timestamp` with the current time unless one is given. It
/// refuses an envelope with no `correlation_id`, no topic, or a topic outside
/// `som.`. Validate the result (`som::validate::Validator`, feature
/// `validate`) before publishing: the builder guarantees the envelope's
/// shape, not the payload's.
#[derive(Debug, Clone)]
#[must_use = "a builder does nothing until `build` is called"]
pub struct EnvelopeBuilder {
    originating_system: OriginatingSystem,
    correlation_id: Option<Uuid>,
    causation_id: Option<Uuid>,
    topic: Option<String>,
    timestamp: Option<String>,
    modification_header: Option<ModificationHeader>,
    actors: Option<Map<String, Value>>,
    extensions: Option<Extensions>,
    context: Option<JsonLdContext>,
}

impl EnvelopeBuilder {
    /// Starts an envelope published by `originating_system`.
    pub fn new(originating_system: OriginatingSystem) -> Self {
        EnvelopeBuilder {
            originating_system,
            correlation_id: None,
            causation_id: None,
            topic: None,
            timestamp: None,
            modification_header: None,
            actors: None,
            extensions: None,
            context: None,
        }
    }

    /// The story lifecycle this message belongs to. Required.
    pub fn correlation_id(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// The message that directly caused this one.
    pub fn causation_id(mut self, id: Uuid) -> Self {
        self.causation_id = Some(id);
        self
    }

    /// The topic. Required; must begin with `som.`.
    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// The event time, as an RFC 3339 `date-time` already spelled out.
    /// Defaults to the time of each `build` call. The spelling is kept as
    /// given; the validator checks it.
    pub fn timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    /// The event time, formatted as UTC with microsecond precision.
    pub fn timestamp_at(self, time: SystemTime) -> Self {
        self.timestamp(format_rfc3339_micros(time))
    }

    /// The modification header, for snapshot-style messages.
    pub fn modification_header(mut self, header: ModificationHeader) -> Self {
        self.modification_header = Some(header);
        self
    }

    /// Short-key actor lookups.
    pub fn actors(mut self, actors: Map<String, Value>) -> Self {
        self.actors = Some(actors);
        self
    }

    /// Vendor fields.
    pub fn extensions(mut self, extensions: Extensions) -> Self {
        self.extensions = Some(extensions);
        self
    }

    /// JSON-LD context.
    pub fn context(mut self, context: JsonLdContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Builds the envelope. Each call mints a new `UUIDv7` `message_id`, so one
    /// builder can stamp a series of messages that share a correlation and a
    /// topic.
    ///
    /// # Errors
    ///
    /// [`BuildError`] if no `correlation_id` or topic was given, or the topic
    /// does not begin with `som.`.
    pub fn build<P: MessagePayload>(&self, payload: P) -> Result<Envelope<P>, BuildError> {
        self.build_with_message_id(Uuid::now_v7(), payload)
    }

    /// Builds the envelope with a `message_id` the caller minted. For a
    /// producer that keeps its own time-ordered generator, and for tests that
    /// need identical output from identical input. A `message_id` is unique
    /// per publication: reusing one for a changed snapshot is the stale
    /// republish the sequence checker reports.
    ///
    /// # Errors
    ///
    /// As [`EnvelopeBuilder::build`].
    pub fn build_with_message_id<P: MessagePayload>(
        &self,
        message_id: Uuid,
        payload: P,
    ) -> Result<Envelope<P>, BuildError> {
        let correlation_id = self
            .correlation_id
            .ok_or(BuildError::MissingCorrelationId)?;
        let topic = self.topic.clone().ok_or(BuildError::MissingTopic)?;
        if !topic.starts_with("som.") {
            return Err(BuildError::TopicNotSom(topic));
        }
        Ok(Envelope {
            som_version: SOM_VERSION.to_owned(),
            message_id: message_id.hyphenated().to_string(),
            correlation_id: correlation_id.hyphenated().to_string(),
            causation_id: self.causation_id.map(|id| id.hyphenated().to_string()),
            message_type: payload.message_type().to_owned(),
            timestamp: self
                .timestamp
                .clone()
                .unwrap_or_else(|| format_rfc3339_micros(SystemTime::now())),
            originating_system: self.originating_system.clone(),
            topic,
            modification_header: self.modification_header.clone(),
            actors: self.actors.clone(),
            payload,
            extensions: self.extensions.clone(),
            context: self.context.clone(),
        })
    }
}
