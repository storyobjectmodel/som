//! Story Object Model (SOM) 1.0.
//!
//! SOM is a small set of JSON message families published on a bus so that the
//! systems of a newsroom can say what they know about a story without
//! point-to-point integration. This crate produces and consumes every SOM 1.0
//! family and validates against the repository's own normative schemas, which
//! it embeds at compile time.
//!
//! - [`model`]: typed messages for the envelope and the six payload families,
//!   closed where the schemas close them, with JSON-equal round-trips.
//! - `validate` (feature `validate`, on by default): conformant validation
//!   with formats asserted, the payload schema selected from `message_type`
//!   alone, and the wire-version rule of `spec/conformance.md` §3.
//! - [`builder`]: an envelope builder that mints `UUIDv7` message ids, and a
//!   name-based id helper for idempotent replays.
//! - `sequence` (feature `validate`): coherence checks across a run of
//!   `story.context` snapshots, the rules of `tools/validate_sequence.py`.
//! - [`time`]: the RFC 3339 handling SOM needs, and no more.
//!
//! # Example
//!
//! A compliance engine records a clearance, publishes it, and a consumer
//! validates and decodes it.
//!
//! ```
//! # #[cfg(feature = "validate")] {
//! use som::builder::{EnvelopeBuilder, name_based_id};
//! use som::model::audit::{AuditAction, AuditTarget, AuditTargetKind, SystemAudit};
//! use som::model::envelope::{OriginatingSystem, SystemType};
//! use som::model::{Actor, Payload, decode_message};
//! use som::validate::{Outcome, Validator};
//!
//! let audit = SystemAudit {
//!     message_type: None,
//!     audit_id: name_based_id("compliance-01", "audit-77").to_string(),
//!     action: AuditAction::Cleared,
//!     target: AuditTarget {
//!         kind: AuditTargetKind::Link,
//!         id: "link-93".to_owned(),
//!     },
//!     actor: Actor {
//!         actor_id: "j.doe".to_owned(),
//!         actor_type: "user".to_owned(),
//!     },
//!     reason: Some("legal review complete".to_owned()),
//!     recorded_at: "2026-09-11T16:41:00Z".to_owned(),
//! };
//!
//! let envelope = EnvelopeBuilder::new(OriginatingSystem::new(
//!     "compliance-01",
//!     SystemType::ComplianceEngine,
//! ))
//! .correlation_id(name_based_id("ncs-01", "hurricane-2026-0911"))
//! .topic("som.system.audit")
//! .build(audit)?;
//! let wire = serde_json::to_value(&envelope)?;
//!
//! let validator = Validator::shared()?;
//! assert!(matches!(
//!     validator.validate_message(&wire)?,
//!     Outcome::Accepted { family: som::Family::SystemAudit, .. }
//! ));
//! let typed = decode_message(wire)?.expect("system.audit is a SOM 1.0 family");
//! assert!(matches!(typed.payload, Payload::SystemAudit(_)));
//! # }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! The typed model does not replace validation: string formats and patterns,
//! numeric ranges and the schemas' conditional rules are enforced by the
//! validator. Validate, then decode.

pub mod builder;
pub mod model;
pub mod schemas;
#[cfg(feature = "validate")]
pub mod sequence;
pub mod time;
#[cfg(feature = "validate")]
pub mod validate;

pub use schemas::Family;
pub use uuid::Uuid;

/// The wire version this crate's producers stamp (conformance §3).
pub const SOM_VERSION: &str = "1.0.0";
