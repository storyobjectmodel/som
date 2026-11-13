//! `link-event.schema.json`: `link.committed`, `link.gate_changed`,
//! `link.withdrawn`.

use serde::{Deserialize, Serialize};

use super::common::Extensions;

/// A link event payload: the asset-to-destination connection. Which members
/// each `message_type` requires (`link.committed`: `committed_by` and
/// `committed_at`; `link.withdrawn`: `withdrawn`, which no other event may
/// carry) is enforced by the validator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkEvent {
    /// Which link event this is.
    pub message_type: LinkEventType,
    /// UUID of the link.
    pub link_id: String,
    /// The linked asset.
    pub asset_id: String,
    /// The destination.
    pub destination_id: String,
    /// Required on `link.committed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committed_by: Option<String>,
    /// RFC 3339; required on `link.committed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committed_at: Option<String>,
    /// The compliance gate.
    pub compliance_gate_status: ComplianceGateStatus,
    /// Present on `link.withdrawn` and only there.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub withdrawn: Option<Withdrawal>,
    /// Vendor fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// `message_type` of a link event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinkEventType {
    /// `link.committed`.
    #[serde(rename = "link.committed")]
    Committed,
    /// `link.gate_changed`.
    #[serde(rename = "link.gate_changed")]
    GateChanged,
    /// `link.withdrawn`.
    #[serde(rename = "link.withdrawn")]
    Withdrawn,
}

impl LinkEventType {
    /// The wire value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            LinkEventType::Committed => "link.committed",
            LinkEventType::GateChanged => "link.gate_changed",
            LinkEventType::Withdrawn => "link.withdrawn",
        }
    }
}

/// `compliance_gate_status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComplianceGateStatus {
    /// `PENDING`.
    Pending,
    /// `CLEARED`.
    Cleared,
    /// `BLOCKED`.
    Blocked,
}

/// `withdrawn`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Withdrawal {
    /// Who withdrew the link.
    pub withdrawn_by: String,
    /// RFC 3339.
    pub withdrawn_at: String,
    /// Why.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
