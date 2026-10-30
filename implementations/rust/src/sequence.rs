//! Is a story over time coherent? The checks of `tools/validate_sequence.py`
//! across a run of `story.context` snapshots of one story.
//!
//! No single message can answer this. The rules, as that tool applies them:
//!
//! Within each snapshot:
//! - it validates against the `story.context` schema;
//! - every `content_refs[].source_id` resolves to an `editorial_source`
//!   (error), and the reference names the provider of the source it is keyed
//!   to rather than another one (heuristic, warning);
//! - no source was received after the snapshot's `updated_at` (error).
//!
//! Across the run:
//! - `story_id` never changes (error);
//! - `sequence_number` strictly increases (error); a gap is a warning;
//! - `updated_at` never goes backwards (error);
//! - no top-level member and no asset disappears (error);
//! - an asset keeps its `asset_type` for life (error);
//! - a story never leaves `KILLED`, `SPIKED` or `ARCHIVED` (error);
//! - a settled assertion review re-opening to `PENDING` is a warning;
//! - with envelopes: a changed snapshot from the same `system_id` under the
//!   same `message_id` is a stale republish (error).
//!
//! Two deliberate differences from the Python tool. Snapshots are validated
//! with format assertion on, as conformance §4 requires. And `date-time`
//! values are compared by the instant they name, falling back to string order
//! only when one does not parse, so `16:41:00.5Z` is later than `16:41:00Z`
//! and offsets compare correctly.
//!
//! The checks read untyped JSON rather than the typed model, so that a
//! snapshot the typed model refuses is still reported against the rules,
//! as the Python tool reports it.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io;
use std::path::Path;

use serde_json::{Map, Value};

use crate::model::story_context::StoryType;
use crate::schemas::Family;
use crate::time::parse_rfc3339;
use crate::validate::{Rejection, Validator};

/// One snapshot in a run: a `story.context` payload, with its envelope when
/// the input was a wire message.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Snapshot {
    /// Where it came from, such as a file name.
    pub name: String,
    /// The `story.context` payload.
    pub payload: Value,
    /// The wire envelope, if the input was one.
    pub envelope: Option<Value>,
}

impl Snapshot {
    /// A bare payload.
    pub fn payload(name: impl Into<String>, payload: Value) -> Self {
        Snapshot {
            name: name.into(),
            payload,
            envelope: None,
        }
    }

    /// A document that is either a wire message or a bare payload. A document
    /// with both `payload` and `message_id` is taken as a wire message.
    pub fn from_document(name: impl Into<String>, document: Value) -> Self {
        let is_envelope = document.get("payload").is_some() && document.get("message_id").is_some();
        if is_envelope {
            let payload = document
                .get("payload")
                .cloned()
                .unwrap_or(Value::Object(Map::new()));
            Snapshot {
                name: name.into(),
                payload,
                envelope: Some(document),
            }
        } else {
            Snapshot::payload(name, document)
        }
    }

    /// Every `*.json` file in `dir` whose name does not begin with `_`, in
    /// file-name order, as the Python tool selects them.
    ///
    /// # Errors
    ///
    /// An I/O error reading the directory or a file, or
    /// [`io::ErrorKind::InvalidData`] naming the file that is not JSON.
    pub fn load_dir(dir: impl AsRef<Path>) -> io::Result<Vec<Snapshot>> {
        let mut paths = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            // The Python tool globs `*.json`, which is case-sensitive, so a
            // `.JSON` file is not a snapshot there either.
            #[allow(clippy::case_sensitive_file_extension_comparisons)]
            let selected = name.ends_with(".json") && !name.starts_with('_');
            if selected && path.is_file() {
                paths.push((name.to_owned(), path));
            }
        }
        paths.sort();
        paths
            .into_iter()
            .map(|(name, path)| {
                let text = std::fs::read_to_string(&path)?;
                let document: Value = serde_json::from_str(&text).map_err(|e| {
                    io::Error::new(io::ErrorKind::InvalidData, format!("{name}: {e}"))
                })?;
                Ok(Snapshot::from_document(name, document))
            })
            .collect()
    }
}

/// Which rule a finding came from. Non-exhaustive: a later release may add
/// a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Rule {
    /// The run holds no snapshots. Error.
    EmptyRun,
    /// The run holds one snapshot, so nothing cross-snapshot can be checked.
    /// Warning.
    SingleSnapshot,
    /// `content_refs[].source_id` names no `editorial_source`. Error.
    ContentRefUnresolved,
    /// A content reference names another source's provider and not its own.
    /// Warning.
    ContentRefMayPointAtWrongSource,
    /// A content reference names no provider at all. Warning.
    ContentRefDoesNotMentionSource,
    /// A source's `received_at` is after the snapshot's `updated_at`. Error.
    SourceReceivedAfterUpdate,
    /// `story_id` differs from the first snapshot's. Error.
    StoryIdChanged,
    /// `sequence_number` did not increase. Error.
    SequenceNotIncreasing,
    /// `sequence_number` skipped values. Warning.
    SequenceGap,
    /// `updated_at` went backwards. Error.
    UpdatedAtBackwards,
    /// A top-level member present before is absent now. Error.
    MemberDisappeared,
    /// An asset present before is absent now. Error.
    AssetDropped,
    /// An asset's `asset_type` changed. Error.
    AssetTypeChanged,
    /// The story left a terminal `story_type`. Error.
    LeftTerminalStoryType,
    /// A `CONFIRMED` or `REJECTED` assertion review went back to `PENDING`.
    /// Warning.
    ReviewReopened,
    /// A changed snapshot was republished by the same system under the same
    /// `message_id`. Error.
    StaleMessageId,
}

/// One finding about a run.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Finding {
    /// The snapshot it concerns, or `None` for the run as a whole.
    pub snapshot: Option<String>,
    /// The rule.
    pub rule: Rule,
    /// What was found.
    pub detail: String,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let at = self.snapshot.as_deref().unwrap_or("<run>");
        write!(f, "{at}: {:?}: {}", self.rule, self.detail)
    }
}

/// The result of checking a run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct SequenceReport {
    /// Snapshots that failed the `story.context` schema, by name.
    pub invalid: Vec<(String, Rejection)>,
    /// Rule violations.
    pub errors: Vec<Finding>,
    /// Things worth knowing that are legal.
    pub warnings: Vec<Finding>,
}

impl SequenceReport {
    /// No invalid snapshot and no error.
    #[must_use]
    pub fn is_coherent(&self) -> bool {
        self.invalid.is_empty() && self.errors.is_empty()
    }

    /// Coherent with no warnings either; the Python tool's `--strict`.
    #[must_use]
    pub fn is_coherent_strict(&self) -> bool {
        self.is_coherent() && self.warnings.is_empty()
    }

    /// Whether any error came from `rule`.
    #[must_use]
    pub fn has_error(&self, rule: Rule) -> bool {
        self.errors.iter().any(|f| f.rule == rule)
    }

    /// Whether any warning came from `rule`.
    #[must_use]
    pub fn has_warning(&self, rule: Rule) -> bool {
        self.warnings.iter().any(|f| f.rule == rule)
    }

    fn error(&mut self, snapshot: Option<&str>, rule: Rule, detail: String) {
        self.errors.push(Finding {
            snapshot: snapshot.map(str::to_owned),
            rule,
            detail,
        });
    }

    fn warn(&mut self, snapshot: Option<&str>, rule: Rule, detail: String) {
        self.warnings.push(Finding {
            snapshot: snapshot.map(str::to_owned),
            rule,
            detail,
        });
    }
}

/// Checks a run of snapshots of one story, in the order given.
#[must_use]
pub fn check_run(validator: &Validator, run: &[Snapshot]) -> SequenceReport {
    let mut report = SequenceReport::default();
    if run.is_empty() {
        report.error(None, Rule::EmptyRun, "no snapshots".to_owned());
        return report;
    }
    for snapshot in run {
        if let Err(rejection) = validator.validate_payload(Family::StoryContext, &snapshot.payload)
        {
            report.invalid.push((snapshot.name.clone(), rejection));
        }
        check_snapshot(&mut report, &snapshot.name, &snapshot.payload);
    }
    check_sequence(&mut report, run);
    report
}

fn str_of<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(Value::as_str)
}

fn array_of<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn compare_date_time(a: &str, b: &str) -> Ordering {
    match (parse_rfc3339(a), parse_rfc3339(b)) {
        (Some(x), Some(y)) => x.cmp(&y),
        _ => a.cmp(b),
    }
}

fn tokens(text: &str) -> BTreeSet<String> {
    text.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

fn text_of(v: Option<&Value>) -> String {
    match v {
        None => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
    }
}

fn check_snapshot(report: &mut SequenceReport, name: &str, p: &Value) {
    let at = Some(name);
    let sources: Vec<&Map<String, Value>> = array_of(p, "editorial_source")
        .iter()
        .filter_map(Value::as_object)
        .collect();
    let declared: BTreeSet<Option<&str>> = sources
        .iter()
        .map(|s| s.get("source_id").and_then(Value::as_str))
        .collect();

    for (i, reference) in array_of(p, "content_refs").iter().enumerate() {
        let Some(reference) = reference.as_object() else {
            continue;
        };
        let Some(sid) = reference
            .get("source_id")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
        else {
            continue;
        };
        if !declared.contains(&Some(sid)) {
            let mut names: Vec<&str> = declared.iter().flatten().copied().collect();
            names.sort_unstable();
            report.error(
                at,
                Rule::ContentRefUnresolved,
                format!("content_refs[{i}].source_id {sid:?} is not in editorial_source {names:?}"),
            );
            continue;
        }

        let mut words = tokens(&text_of(reference.get("uri")));
        words.extend(tokens(&text_of(reference.get("content_format"))));
        let hits = |source: &Map<String, Value>| -> BTreeSet<String> {
            tokens(&text_of(source.get("provider")))
                .into_iter()
                .filter(|t| words.contains(t))
                .collect()
        };
        let named = sources
            .iter()
            .find(|s| s.get("source_id").and_then(Value::as_str) == Some(sid));
        if named.is_some_and(|s| !hits(s).is_empty()) {
            continue;
        }
        let other = sources.iter().find_map(|s| {
            let id = s.get("source_id").and_then(Value::as_str);
            let found = hits(s);
            (id != Some(sid) && !found.is_empty()).then_some((id, found))
        });
        let provider = named
            .and_then(|s| s.get("provider"))
            .and_then(Value::as_str)
            .unwrap_or("?");
        if let Some((other_id, found)) = other {
            report.warn(
                at,
                Rule::ContentRefMayPointAtWrongSource,
                format!(
                    "content_refs[{i}] is keyed to {sid} ({provider}) but names {found:?}, \
                     which matches {}",
                    other_id.unwrap_or("?")
                ),
            );
        } else if named.is_some_and(|s| !s.is_empty()) {
            report.warn(
                at,
                Rule::ContentRefDoesNotMentionSource,
                format!(
                    "content_refs[{i}] is keyed to {sid} ({provider}) but neither the uri nor \
                     the content_format names it"
                ),
            );
        }
    }

    if let Some(updated) = str_of(p, "updated_at").filter(|s| !s.is_empty()) {
        for source in &sources {
            let Some(received) = source
                .get("received_at")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            else {
                continue;
            };
            if compare_date_time(received, updated) == Ordering::Greater {
                report.error(
                    at,
                    Rule::SourceReceivedAfterUpdate,
                    format!(
                        "{} received_at {received} is after the snapshot's updated_at {updated}",
                        text_of(source.get("source_id"))
                    ),
                );
            }
        }
    }
}

fn integer(v: Option<&Value>) -> Option<i128> {
    let v = v?;
    v.as_i64()
        .map(i128::from)
        .or_else(|| v.as_u64().map(i128::from))
}

fn assets_by_id(p: &Value) -> BTreeMap<&str, &Value> {
    array_of(p, "assets")
        .iter()
        .filter_map(|a| Some((a.get("asset_id")?.as_str()?, a)))
        .collect()
}

fn reviews(p: &Value) -> BTreeMap<Option<&str>, &str> {
    array_of(p, "assertions")
        .iter()
        .filter_map(|x| {
            let state = x.get("review")?.get("state")?.as_str()?;
            Some((x.get("assertion_id").and_then(Value::as_str), state))
        })
        .collect()
}

fn without_ordering_members(p: &Value) -> Value {
    match p {
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(k, _)| k.as_str() != "sequence_number" && k.as_str() != "updated_at")
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        ),
        other => other.clone(),
    }
}

fn is_terminal(p: &Value) -> bool {
    p.get("story_type")
        .cloned()
        .and_then(|v| serde_json::from_value::<StoryType>(v).ok())
        .is_some_and(StoryType::is_terminal)
}

/// The `asset_type` first seen for each `asset_id`, across the run so far.
type SeenTypes = BTreeMap<String, Option<Value>>;

fn check_sequence(report: &mut SequenceReport, run: &[Snapshot]) {
    if run.len() < 2 {
        report.warn(
            None,
            Rule::SingleSnapshot,
            "only one snapshot; nothing cross-snapshot can be checked".to_owned(),
        );
    }

    let first_id = run[0].payload.get("story_id");
    let mut seen_types = SeenTypes::new();

    for (i, snapshot) in run.iter().enumerate() {
        let name = snapshot.name.as_str();
        let p = &snapshot.payload;

        if p.get("story_id") != first_id {
            report.error(
                Some(name),
                Rule::StoryIdChanged,
                format!(
                    "{} -> {}: an update is a new snapshot of the same story, never a new story",
                    text_of(first_id),
                    text_of(p.get("story_id"))
                ),
            );
        }

        if i == 0 {
            for (id, asset) in assets_by_id(p) {
                seen_types.insert(id.to_owned(), asset.get("asset_type").cloned());
            }
            continue;
        }
        let previous = &run[i - 1];
        check_ordering(report, name, &previous.payload, p);
        check_nothing_dropped(report, name, &previous.payload, p);
        check_asset_identity(report, name, &mut seen_types, p);
        check_lifecycle(report, name, &previous.payload, p);
        check_reviews(report, name, &previous.payload, p);
        if let (Some(env), Some(previous_env)) = (&snapshot.envelope, &previous.envelope) {
            check_attribution(report, name, (&previous.payload, previous_env), (p, env));
        }
    }
}

fn check_ordering(report: &mut SequenceReport, name: &str, q: &Value, p: &Value) {
    if let (Some(now), Some(before)) = (
        integer(p.get("sequence_number")),
        integer(q.get("sequence_number")),
    ) {
        if now <= before {
            report.error(
                Some(name),
                Rule::SequenceNotIncreasing,
                format!("{before} -> {now}: owner-only and strictly monotonic"),
            );
        } else if now > before + 1 {
            report.warn(Some(name), Rule::SequenceGap, format!("{before} -> {now}"));
        }
    }

    let now_at = str_of(p, "updated_at").unwrap_or("");
    let before_at = str_of(q, "updated_at").unwrap_or("");
    if compare_date_time(now_at, before_at) == Ordering::Less {
        report.error(
            Some(name),
            Rule::UpdatedAtBackwards,
            format!("{before_at} -> {now_at}"),
        );
    }
}

fn check_nothing_dropped(report: &mut SequenceReport, name: &str, q: &Value, p: &Value) {
    let empty = Map::new();
    let now_keys = p.as_object().unwrap_or(&empty);
    let gone: Vec<&String> = q
        .as_object()
        .unwrap_or(&empty)
        .keys()
        .filter(|k| !now_keys.contains_key(*k))
        .collect();
    if !gone.is_empty() {
        report.error(
            Some(name),
            Rule::MemberDisappeared,
            format!(
                "{gone:?}: every message is a whole snapshot; omitting what you did not \
                 change erases it for every consumer"
            ),
        );
    }

    let before_assets = assets_by_id(q);
    let now_assets = assets_by_id(p);
    let dropped: Vec<&&str> = before_assets
        .keys()
        .filter(|id| !now_assets.contains_key(*id))
        .collect();
    if !dropped.is_empty() {
        report.error(Some(name), Rule::AssetDropped, format!("{dropped:?}"));
    }
}

fn check_asset_identity(report: &mut SequenceReport, name: &str, seen: &mut SeenTypes, p: &Value) {
    for (id, asset) in assets_by_id(p) {
        let now_type = asset.get("asset_type").cloned();
        match seen.get(id) {
            Some(Some(was)) if now_type.as_ref() != Some(was) => {
                report.error(
                    Some(name),
                    Rule::AssetTypeChanged,
                    format!(
                        "asset {id}: {} -> {}; an asset_id means one thing for its whole life",
                        text_of(Some(was)),
                        text_of(now_type.as_ref())
                    ),
                );
            }
            Some(_) => {}
            None => {
                seen.insert(id.to_owned(), now_type);
            }
        }
    }
}

fn check_lifecycle(report: &mut SequenceReport, name: &str, q: &Value, p: &Value) {
    if is_terminal(q) && !is_terminal(p) {
        report.error(
            Some(name),
            Rule::LeftTerminalStoryType,
            format!(
                "{} -> {}",
                text_of(q.get("story_type")),
                text_of(p.get("story_type"))
            ),
        );
    }
}

fn check_reviews(report: &mut SequenceReport, name: &str, q: &Value, p: &Value) {
    let before = reviews(q);
    for (id, state) in reviews(p) {
        let was = before.get(&id).copied();
        if matches!(was, Some("CONFIRMED" | "REJECTED")) && state == "PENDING" {
            report.warn(
                Some(name),
                Rule::ReviewReopened,
                format!(
                    "assertion {}: {} -> PENDING; legal, but worth an audit record",
                    id.unwrap_or("?"),
                    was.unwrap_or("?")
                ),
            );
        }
    }
}

fn check_attribution(
    report: &mut SequenceReport,
    name: &str,
    (q, previous_env): (&Value, &Value),
    (p, env): (&Value, &Value),
) {
    let changed = without_ordering_members(q) != without_ordering_members(p);
    let system = |e: &Value| {
        e.get("originating_system")
            .and_then(|o| o.get("system_id"))
            .cloned()
    };
    let same_system = system(env) == system(previous_env);
    let same_id = env.get("message_id") == previous_env.get("message_id");
    if changed && same_system && same_id {
        report.error(
            Some(name),
            Rule::StaleMessageId,
            "a new snapshot is a new message and needs its own message_id".to_owned(),
        );
    }
}
