//! `system-audit.schema.json`: `system.audit`.

use serde::{Deserialize, Serialize};

use super::common::Actor;

/// A governance audit record: a clearance, suppression, withholding or
/// override.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemAudit {
    /// Optional in the payload; when present, `system.audit`. The envelope's
    /// `message_type` is what selects the schema.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_type: Option<SystemAuditType>,
    /// UUID of the audit record.
    pub audit_id: String,
    /// What was done.
    pub action: AuditAction,
    /// What it was done to.
    pub target: AuditTarget,
    /// Who did it.
    pub actor: Actor,
    /// Why.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// RFC 3339.
    pub recorded_at: String,
}

/// The `system.audit` constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SystemAuditType {
    /// `system.audit`.
    #[default]
    #[serde(rename = "system.audit")]
    SystemAudit,
}

/// `action`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditAction {
    /// `CLEARED`.
    Cleared,
    /// `SUPPRESSED`.
    Suppressed,
    /// `WITHHELD`.
    Withheld,
    /// `OVERRIDDEN`.
    Overridden,
}

/// `target`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditTarget {
    /// What kind of thing.
    pub kind: AuditTargetKind,
    /// Its identifier.
    pub id: String,
}

/// `target.kind`. There is no `STORY`: the assertion target set is the wider
/// of the two until the audit set converges on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditTargetKind {
    /// `LINK`.
    Link,
    /// `ASSET`.
    Asset,
    /// `TELLING`.
    Telling,
}
