//! Access to the repository's own examples and schemas.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use serde_json::Value;
use som::Family;

/// The repository root: this crate lives at `implementations/rust/`.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn examples() -> PathBuf {
    repo_root().join("examples")
}

pub fn load(path: &Path) -> Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// `*.json` files in `dir`, sorted by name.
pub fn json_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    files
}

/// `examples/<dir>/` to the family a bare payload in it belongs to, and the
/// minimum file count, as `tools/validate.py` declares them. A bare payload
/// carries no envelope, so the directory is the only thing that can name its
/// family; a full envelope is dispatched on its own `message_type`.
pub const EXAMPLE_DIRS: [(&str, Family, usize); 7] = [
    ("hurricane-run", Family::StoryContext, 7),
    ("story-context", Family::StoryContext, 3),
    ("telling", Family::Telling, 2),
    ("delivery", Family::DeliveryMediaAvailable, 2),
    ("link", Family::Link, 1),
    ("system-audit", Family::SystemAudit, 1),
    ("skill-warning", Family::SkillWarning, 1),
];

/// The `family` member of a negative case, as `tools/validate_negative.py`
/// names them.
pub fn negative_family(name: &str) -> Family {
    match name {
        "story-context" => Family::StoryContext,
        "telling" => Family::Telling,
        "delivery" => Family::DeliveryMediaAvailable,
        "link" => Family::Link,
        "system-audit" => Family::SystemAudit,
        "skill-warning" => Family::SkillWarning,
        other => panic!("unknown negative family {other:?}"),
    }
}

pub fn is_envelope(document: &Value) -> bool {
    document.get("payload").is_some()
}

/// The embedded schema, parsed.
pub fn schema(family: Family) -> Value {
    serde_json::from_str(family.schema_source()).unwrap()
}

pub fn envelope_schema() -> Value {
    serde_json::from_str(som::schemas::ENVELOPE).unwrap()
}

/// JSON Pointers to every object in `value`, depth first, with the pointer to
/// each object preceding the pointers into it.
pub fn object_pointers(value: &Value, at: &str, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            out.push(at.to_owned());
            for (k, v) in map {
                let escaped = k.replace('~', "~0").replace('/', "~1");
                object_pointers(v, &format!("{at}/{escaped}"), out);
            }
        }
        Value::Array(items) => {
            for (i, v) in items.iter().enumerate() {
                object_pointers(v, &format!("{at}/{i}"), out);
            }
        }
        _ => {}
    }
}
