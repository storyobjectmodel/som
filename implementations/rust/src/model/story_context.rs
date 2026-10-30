//! `story-context.schema.json`: `story.context`.
//!
//! A `story.context` is a whole snapshot, never a delta (conformance §6), so
//! every optional member is an `Option` and an absent member stays absent on
//! re-encode: absent and empty are different statements.
//!
//! The withdrawn members `ai_enrichments`, `instances`, `broadcaster_id`,
//! `sources` and `content_ref` (and `voice_count` on an asset) are refused on
//! decode, as is any member the schema does not list. Cross-member rules (an
//! `ACTIVE` story carries a `lifecycle` and no other story does; a `HUMAN`
//! author names no model; the claim members an `assertion_type` requires) and
//! string patterns are enforced by the validator.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};

use super::common::{Extensions, Locator, TimeRanges, registry};

/// The complete state of a story at one moment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoryContext {
    /// Immutable across every snapshot of the story.
    pub story_id: String,
    /// Slug.
    pub slug: String,
    /// Headline.
    pub headline: String,
    /// Story type.
    pub story_type: StoryType,
    /// Subject-matter tags; order is significant and the first is primary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<StoryTag>>,
    /// Present exactly when `story_type` is `ACTIVE`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<Lifecycle>,
    /// Strictly increasing across snapshots.
    pub sequence_number: u64,
    /// RFC 3339; moves forward across snapshots.
    pub updated_at: String,
    /// Newsroom; derive from `skills_config.newsroom` if absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub newsroom_id: Option<String>,
    /// System or entity with authoritative control over the story.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub story_owner: Option<StoryOwner>,
    /// Priority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,
    /// Premise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub premise: Option<Premise>,
    /// Compliance flags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compliance: Option<Vec<ComplianceFlag>>,
    /// Editorial gates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub editorial_gates: Option<Vec<EditorialGate>>,
    /// 5W1H kernel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub story_meaning: Option<StoryMeaning>,
    /// Publication gate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub government_approval: Option<GovernmentApproval>,
    /// Editorial sources.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub editorial_source: Option<Vec<EditorialSource>>,
    /// Assets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assets: Option<Vec<Asset>>,
    /// Skills configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skills_config: Option<SkillsConfig>,
    /// Content references.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_refs: Option<Vec<ContentRef>>,
    /// Relations to other stories.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relations: Option<Vec<Relation>>,
    /// Collaboration state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collaboration: Option<Collaboration>,
    /// Vendor fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
    /// Claims about content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assertions: Option<Vec<Assertion>>,
}

/// `story_type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StoryType {
    /// `PLANNED`.
    Planned,
    /// `ACTIVE`: the only type that carries a `lifecycle`.
    Active,
    /// `KILLED`; terminal.
    Killed,
    /// `SPIKED`; terminal.
    Spiked,
    /// `ARCHIVED`; terminal.
    Archived,
    /// `ORPHAN`: a shell story holding a clip that arrived with no story,
    /// which always retires to `ARCHIVED`.
    Orphan,
}

impl StoryType {
    /// Whether a story never leaves this type: `KILLED`, `SPIKED` or
    /// `ARCHIVED`.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            StoryType::Killed | StoryType::Spiked | StoryType::Archived
        )
    }
}

/// `tags[]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoryTag {
    /// `newsroom`, `iptc-mediatopic` or `com.{vendor}.{name}`.
    pub scheme: String,
    /// Non-empty value.
    pub value: String,
    /// Display label; never resolve a trigger policy on it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// `lifecycle`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lifecycle {
    /// Current phase. There is no `LIVE`: on-air is derived from the telling.
    pub phase: LifecyclePhase,
    /// RFC 3339.
    pub phase_entered_at: String,
    /// Previous phase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_phase: Option<LifecyclePhase>,
    /// Owner of the phase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase_owner: Option<PhaseOwner>,
}

/// `lifecycle.phase`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LifecyclePhase {
    /// `DEVELOPING`.
    Developing,
    /// `READY_TO_AIR`.
    ReadyToAir,
    /// `BREAKING`.
    Breaking,
    /// `PUBLISHED`.
    Published,
}

/// `lifecycle.phase_owner`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhaseOwner {
    /// User identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

/// `story_owner`: a string or an object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StoryOwner {
    /// An identifier.
    Id(String),
    /// A structured owner; its shape is not specified.
    Object(Map<String, Value>),
}

/// `priority`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Priority {
    /// Level.
    pub level: PriorityLevel,
    /// Why.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Who escalated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub escalated_by: Option<String>,
    /// RFC 3339.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub escalated_at: Option<String>,
}

/// `priority.level`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PriorityLevel {
    /// `ROUTINE`.
    Routine,
    /// `STANDARD`.
    Standard,
    /// `HIGH`.
    High,
    /// `URGENT`.
    Urgent,
    /// `FLASH`.
    Flash,
}

/// `premise`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Premise {
    /// What the story expects to find.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_outcome: Option<String>,
    /// Confidence in the premise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<PremiseConfidence>,
    /// Whether the premise has changed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub premise_changed: Option<bool>,
    /// The outcome before the change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_outcome: Option<String>,
    /// What actually happened.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual_outcome: Option<String>,
    /// Asset ids affected by the change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub affected_assets: Option<Vec<String>>,
    /// RFC 3339.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_detected_at: Option<String>,
    /// Who or what detected the change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_detected_by: Option<String>,
}

/// `premise.confidence`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PremiseConfidence {
    /// `LOW`.
    Low,
    /// `MEDIUM`.
    Medium,
    /// `HIGH`.
    High,
}

/// `compliance[]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComplianceFlag {
    /// Flag identifier.
    pub flag_id: String,
    /// The schema's `type`, such as `LEGAL_REVIEW` or `MINOR_INVOLVED`; an
    /// open string.
    #[serde(rename = "type")]
    pub flag_type: String,
    /// Severity.
    pub severity: ComplianceSeverity,
    /// Detail.
    pub detail: String,
    /// Jurisdiction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jurisdiction: Option<String>,
    /// Regulation reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regulation_ref: Option<String>,
    /// Who raised it.
    pub raised_by: String,
    /// RFC 3339.
    pub raised_at: String,
    /// Status.
    pub status: ComplianceStatus,
    /// A hold on a media range; evaluated against the Source timeline.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_range: Option<MediaRange>,
}

/// `compliance[].severity`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComplianceSeverity {
    /// `LOW`.
    Low,
    /// `MEDIUM`.
    Medium,
    /// `HIGH`.
    High,
    /// `CRITICAL`.
    Critical,
}

/// `compliance[].status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComplianceStatus {
    /// `ACTIVE`.
    Active,
    /// `RESOLVED`.
    Resolved,
    /// `WAIVED`.
    Waived,
}

/// `compliance[].media_range`: a source or a locator, with a time range. The
/// schema requires `time_range` with either `source` or `locator`; the
/// validator enforces it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MediaRange {
    /// Fully-qualified TAMS Source URI, `tams://store/id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Non-TAMS store reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locator: Option<Locator>,
    /// The restricted stretch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_range: Option<TimeRanges>,
}

/// `editorial_gates[]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditorialGate {
    /// Gate identifier.
    pub gate_id: String,
    /// Gate type; an open string.
    pub gate_type: String,
    /// Status.
    pub status: GateStatus,
    /// The skill that requires this gate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_by_skill: Option<String>,
    /// The rule that raised it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate_rule_id: Option<String>,
    /// The event type that resolves it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution_event_type: Option<String>,
    /// Who it is assigned to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assigned_to: Option<String>,
    /// What the gate holds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocks: Option<Vec<GateBlock>>,
}

/// `editorial_gates[].status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GateStatus {
    /// `PENDING`.
    Pending,
    /// `APPROVED`.
    Approved,
    /// `REJECTED`.
    Rejected,
}

/// `editorial_gates[].blocks[]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GateBlock {
    /// `{kind, ref}`.
    Target(GateBlockTarget),
    /// A bare string; deprecated, accepted for migration only.
    Bare(String),
}

/// `editorial_gates[].blocks[]` in its structured form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateBlockTarget {
    /// `ASSET` or `PHASE`.
    pub kind: GateBlockKind,
    /// The blocked asset id or lifecycle-phase token; the schema's `ref`.
    #[serde(rename = "ref")]
    pub reference: String,
}

/// `editorial_gates[].blocks[].kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GateBlockKind {
    /// `ASSET`: `ref` is an asset id.
    Asset,
    /// `PHASE`: `ref` is a lifecycle-phase token.
    Phase,
}

/// `story_meaning`: the 5W1H kernel. Each member is capped at 280
/// characters, `summary` at 600; the validator enforces the caps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoryMeaning {
    /// Who.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub who: Option<String>,
    /// What.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub what: Option<String>,
    /// When.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
    /// Where; the schema's `where`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#where: Option<String>,
    /// Why.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,
    /// How.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub how: Option<String>,
    /// Summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// 0 to 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Number>,
    /// References to the provenance of the kernel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance_refs: Option<Vec<String>>,
}

/// `government_approval`: a publication gate. An executor must not publish
/// while the status blocks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GovernmentApproval {
    /// Status.
    pub status: GovernmentApprovalStatus,
    /// Jurisdiction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jurisdiction: Option<String>,
}

/// `government_approval.status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GovernmentApprovalStatus {
    /// `NOT_REQUIRED`.
    NotRequired,
    /// `REQUESTED`.
    Requested,
    /// `UNDER_REVIEW`.
    UnderReview,
    /// `APPROVED`.
    Approved,
    /// `RETURNED_FOR_REVISION`.
    ReturnedForRevision,
    /// `REJECTED`.
    Rejected,
}

/// `editorial_source[]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditorialSource {
    /// Source identifier; what `content_refs[].source_id` resolves to.
    pub source_id: String,
    /// Source type.
    pub source_type: SourceType,
    /// Provider.
    pub provider: String,
    /// Credibility.
    pub credibility: Credibility,
    /// RFC 3339; not after the snapshot's `updated_at`.
    pub received_at: String,
    /// Hash of the content as received.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
}

registry! {
    /// `editorial_source[].source_type`.
    SourceType {
        /// `WIRE`.
        Wire => "WIRE",
        /// `OFFICIAL`.
        Official => "OFFICIAL",
        /// `FIELD_CREW`.
        FieldCrew => "FIELD_CREW",
        /// `POOL_FEED`.
        PoolFeed => "POOL_FEED",
        /// `GOVERNMENT`.
        Government => "GOVERNMENT",
        /// `INTERNAL`.
        Internal => "INTERNAL",
        /// `PRESS_RELEASE`.
        PressRelease => "PRESS_RELEASE",
        /// `SCANNER`.
        Scanner => "SCANNER",
        /// `USER_GENERATED`.
        UserGenerated => "USER_GENERATED",
        /// `SOCIAL`.
        Social => "SOCIAL",
        /// `ARCHIVE`.
        Archive => "ARCHIVE",
    }
}

/// `editorial_source[].credibility`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Credibility {
    /// `TRUSTED`.
    Trusted,
    /// `VERIFIED`.
    Verified,
    /// `ENDORSED`.
    Endorsed,
    /// `UNVERIFIED`.
    Unverified,
}

/// `assets[]`. The withdrawn `voice_count` is refused.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Asset {
    /// Minted once; means one thing for its whole life.
    pub asset_id: String,
    /// Asset type.
    pub asset_type: AssetType,
    /// Editorial state only; on-air is derived from the telling.
    pub status: AssetStatus,
    /// Evidential position.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidential_position: Option<EvidentialPosition>,
    /// Primary assets only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acquisition_state: Option<AcquisitionState>,
    /// Editorial lineage: the asset ids this one derives from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derived_from: Option<Vec<String>>,
    /// Who or what produced it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub produced_by: Option<String>,
    /// Maintained from link events; committed links only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Vec<AssetUsage>>,
    /// What invalidated it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invalidated_by: Option<String>,
    /// Standards clearance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standards_clearance: Option<StandardsClearance>,
    /// Delivery specification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivery_spec: Option<DeliverySpec>,
    /// Media references.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_refs: Option<Vec<MediaRef>>,
    /// Media-authenticity credential (C2PA).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authenticity_credential: Option<AuthenticityCredential>,
    /// Governed per-newsroom registry key; `UPPER_SNAKE` or `x-` vendor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framing_treatment: Option<String>,
    /// Authorship provenance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<AuthorshipProvenance>,
}

registry! {
    /// `assets[].asset_type`.
    AssetType {
        /// `VIDEO`.
        Video => "VIDEO",
        /// `AUDIO`.
        Audio => "AUDIO",
        /// `GRAPHIC`.
        Graphic => "GRAPHIC",
        /// `STILL`.
        Still => "STILL",
        /// `CG`.
        Cg => "CG",
        /// `SCRIPT`.
        Script => "SCRIPT",
        /// `PROMPTER`.
        Prompter => "PROMPTER",
        /// `LOWER_THIRD`.
        LowerThird => "LOWER_THIRD",
        /// `FULL_FRAME`.
        FullFrame => "FULL_FRAME",
        /// `MAP`.
        Map => "MAP",
        /// `SOCIAL_CARD`.
        SocialCard => "SOCIAL_CARD",
        /// `TRANSCRIPT`.
        Transcript => "TRANSCRIPT",
        /// `SUMMARY`.
        Summary => "SUMMARY",
        /// `SOCIAL_POST`.
        SocialPost => "SOCIAL_POST",
        /// `ARTICLE`.
        Article => "ARTICLE",
        /// `ANALYSIS`.
        Analysis => "ANALYSIS",
        /// `CUSTOM`.
        Custom => "CUSTOM",
    }
}

/// `assets[].status`. There is no `FINALIZING`: it was withdrawn at 1.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssetStatus {
    /// `READY`.
    Ready,
    /// `IN_PRODUCTION`.
    InProduction,
    /// `PREPARED`.
    Prepared,
    /// `INVALIDATED`.
    Invalidated,
}

/// `assets[].evidential_position`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvidentialPosition {
    /// `PRIMARY`.
    Primary,
    /// `SECONDARY`.
    Secondary,
    /// `TERTIARY`.
    Tertiary,
}

/// `assets[].acquisition_state`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AcquisitionState {
    /// `CAPTURING`.
    Capturing,
    /// `CAPTURED`.
    Captured,
}

/// `assets[].usage[]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetUsage {
    /// The link.
    pub link_id: String,
    /// The destination.
    pub destination_id: String,
    /// Always `committed`.
    pub state: UsageState,
}

/// `assets[].usage[].state`: the constant `committed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum UsageState {
    /// `committed`.
    #[default]
    #[serde(rename = "committed")]
    Committed,
}

/// `assets[].standards_clearance`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StandardsClearance {
    /// Status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<ClearanceStatus>,
    /// What blocks clearance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked_by: Option<Vec<String>>,
    /// RFC 3339.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_reviewed: Option<String>,
}

/// `assets[].standards_clearance.status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClearanceStatus {
    /// `CLEARED`.
    Cleared,
    /// `BLOCKED`.
    Blocked,
    /// `PENDING`.
    Pending,
}

/// `assets[].delivery_spec`. The schema leaves this object open, so members
/// it does not name are kept in `other`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeliverySpec {
    /// Format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    /// Resolution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    /// Frame rate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_rate: Option<Number>,
    /// Members the schema does not name.
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

/// `assets[].media_refs[]`: a TAMS source or a locator. The schema requires
/// one of `source` or `locator`; the validator enforces it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MediaRef {
    /// Fully-qualified TAMS Source URI, `tams://store/id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Non-TAMS store reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locator: Option<Locator>,
    /// Label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Time range or ranges onto the Source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_range: Option<TimeRanges>,
}

/// `assets[].authenticity_credential`: the C2PA axis, distinct from
/// credibility. `present: false` is recorded, not hidden.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthenticityCredential {
    /// Whether a credential is present.
    pub present: bool,
    /// The standard, such as C2PA.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standard: Option<String>,
    /// The claim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim: Option<String>,
    /// Who validated it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated_by: Option<String>,
}

/// `skills_config`. The withdrawn `broadcaster` is refused.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillsConfig {
    /// Newsroom; renamed from `broadcaster`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub newsroom: Option<String>,
    /// Skills specification version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skills_spec_version: Option<String>,
    /// The executor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_executor_id: Option<String>,
    /// Scope context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_context: Option<String>,
    /// Active skills.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_skills: Option<Vec<ActiveSkill>>,
}

/// `skills_config.active_skills[]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActiveSkill {
    /// Skill identifier.
    pub skill_id: String,
    /// Skill version.
    pub skill_version: String,
    /// Skill type.
    pub skill_type: SkillType,
    /// Disclosure level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disclosure_level: Option<DisclosureLevel>,
    /// Migration policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub migration_policy: Option<MigrationPolicy>,
    /// Skill priority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_priority: Option<SkillPriority>,
}

/// `active_skills[].skill_type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SkillType {
    /// `NEWSROOM`.
    Newsroom,
    /// `VENDOR`.
    Vendor,
    /// `REFERENCE`.
    Reference,
}

/// `active_skills[].disclosure_level`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DisclosureLevel {
    /// `L1`.
    L1,
    /// `L2`.
    L2,
    /// `L3`.
    L3,
}

/// `active_skills[].migration_policy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MigrationPolicy {
    /// `HOT`.
    Hot,
    /// `COLD`.
    Cold,
    /// `GATED`.
    Gated,
}

/// `active_skills[].skill_priority`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SkillPriority {
    /// `CRITICAL`.
    Critical,
    /// `NORMAL`.
    Normal,
    /// `LOW`.
    Low,
}

/// `content_refs[]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentRef {
    /// URI; `format: uri`.
    pub uri: String,
    /// MIME type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Content format, such as `ap-som-body-json-v1`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_format: Option<String>,
    /// Placement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,
    /// Provider.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Role.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Must resolve to an `editorial_source[].source_id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
}

/// `relations[]`. The schema leaves this object open, so members it does not
/// name are kept in `other`. The underscored members are the schema's
/// `_version`, `_created_at`, `_created_by`, `_modified_at`, `_modified_by`
/// and `_history`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relation {
    /// Relation type.
    pub relation_type: String,
    /// The related story.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_story_id: Option<String>,
    /// `_version`.
    #[serde(rename = "_version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
    /// `_created_at`, RFC 3339.
    #[serde(
        rename = "_created_at",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub created_at: Option<String>,
    /// `_created_by`.
    #[serde(
        rename = "_created_by",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub created_by: Option<String>,
    /// `_modified_at`, RFC 3339.
    #[serde(
        rename = "_modified_at",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub modified_at: Option<String>,
    /// `_modified_by`.
    #[serde(
        rename = "_modified_by",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub modified_by: Option<String>,
    /// `_history`; the entries' shape is not specified.
    #[serde(rename = "_history", default, skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<Value>>,
    /// Members the schema does not name.
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

/// `collaboration`. The schema leaves this object open, so members it does
/// not name are kept in `other`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Collaboration {
    /// Editing version; renamed from `version`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub editing_version: Option<i64>,
    /// Active editors; the entries' shape is not specified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_editors: Option<Vec<Value>>,
    /// Lock status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lock_status: Option<String>,
    /// Members the schema does not name.
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

/// A review state on provenance or an assertion. `REJECTED` entries are
/// marked, not deleted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewState {
    /// State.
    pub state: Review,
    /// Reviewer, in the audit `actor_id` shape.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewer: Option<String>,
    /// RFC 3339.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<String>,
}

/// `review.state`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Review {
    /// `PENDING`.
    Pending,
    /// `CONFIRMED`.
    Confirmed,
    /// `REJECTED`.
    Rejected,
}

/// Authorship provenance. A `HUMAN` author carries no `model` or
/// `model_version`; the validator enforces it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorshipProvenance {
    /// Who or what authored the output.
    pub author: Author,
    /// Model name; `MODEL` authors only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Model version; `MODEL` authors only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    /// 0 to 1; only where a calibrated confidence exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Number>,
    /// RFC 3339.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<String>,
    /// Inputs the output was generated from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_sources: Option<Vec<String>>,
    /// Hash of the prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_hash: Option<String>,
    /// Human review.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review: Option<ReviewState>,
}

/// `provenance.author`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Author {
    /// `HUMAN`.
    Human,
    /// `MODEL`.
    Model,
}

/// `assertions[]`: a claim about content. A `MATCH` is a proposal until its
/// review is `CONFIRMED`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assertion {
    /// Assertion identifier.
    pub assertion_id: String,
    /// Assertion type.
    pub assertion_type: AssertionType,
    /// What the claim is about.
    pub target: AssertionTarget,
    /// The typed claim. `FACT_CHECK` requires `metric` and `value`,
    /// `DETECTION` requires `detection_class` and `verdict`, `MATCH` requires
    /// `story_id`; the validator enforces it.
    pub claim: Map<String, Value>,
    /// Who made the underlying claim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributed_to: Option<String>,
    /// RFC 3339.
    pub asserted_at: String,
    /// 0 to 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Number>,
    /// The source transcript or asset the claim came from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub originated_from: Option<String>,
    /// Human review.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review: Option<ReviewState>,
    /// Who or what made the assertion.
    pub provenance: AuthorshipProvenance,
}

/// `assertions[].assertion_type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssertionType {
    /// `FACT_CHECK`.
    FactCheck,
    /// `DETECTION`.
    Detection,
    /// `MATCH`.
    Match,
}

/// `assertions[].target`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssertionTarget {
    /// What kind of thing.
    pub kind: AssertionTargetKind,
    /// Its identifier.
    pub id: String,
}

/// `assertions[].target.kind`: the audit target set plus `STORY`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssertionTargetKind {
    /// `ASSET`.
    Asset,
    /// `LINK`.
    Link,
    /// `TELLING`.
    Telling,
    /// `STORY`.
    Story,
}
