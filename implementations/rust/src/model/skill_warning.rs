//! `skill-warning.schema.json`: `skill.warning.raised`.

use serde::{Deserialize, Serialize};

use super::common::Extensions;

/// A warning raised by a skills executor. Its timestamp lives on the
/// envelope; the retired `instance_ref` and a payload `timestamp` are refused.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillWarning {
    /// UUID of the warning; `UUIDv7`.
    pub warning_id: String,
    /// The skill that raised it, such as `nbcu/editorial-standards`.
    pub skill_id: String,
    /// `major.minor.patch`.
    pub skill_version: String,
    /// The story it concerns.
    pub story_id: String,
    /// The firing level, `{level}:{id}`: `link:{link_id}`, `asset:{asset_id}`
    /// or `story:{story_id}`.
    pub scope: String,
    /// Lower-case by design.
    pub severity: Severity,
    /// The rule that fired.
    pub rule_id: String,
    /// Whether the warning may be overridden.
    pub non_overridable: bool,
    /// Fields affected.
    pub affected_fields: Vec<String>,
    /// Human-readable detail.
    pub detail: String,
    /// What the warning blocks.
    pub blocks: Vec<String>,
    /// Reference to the warning definition.
    pub skill_warning_ref: String,
    /// Governed render key, such as `sync.ai.banner`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// Optional in the payload; when present, `skill.warning.raised`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_type: Option<SkillWarningType>,
    /// Vendor fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// `severity`: the permanent lower-case carve-out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// `hold`: the executor must withhold output on the affected fields.
    Hold,
    /// `flag`: mark for review.
    Flag,
    /// `inform`: advisory.
    Inform,
}

/// The `skill.warning.raised` constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SkillWarningType {
    /// `skill.warning.raised`.
    #[default]
    #[serde(rename = "skill.warning.raised")]
    Raised,
}
