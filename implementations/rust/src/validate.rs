//! Conformant validation of SOM 1.0 messages.
//!
//! The rules applied here, each from `spec/conformance.md`:
//!
//! - the payload schema is selected from the envelope's `message_type` and
//!   from nothing else (§2);
//! - `som_version` must be a 1.x version; `0.3.2` and every other pre-1.0 or
//!   post-1.x value is rejected, and the version never selects a schema (§3);
//! - `format` is asserted, so `uuid` and `date-time` are checked rather than
//!   treated as annotations (§4);
//! - an unrecognised `message_type` is ignored, not rejected (§5), and
//!   unrecognised `com.{vendor}.` extensions are accepted, because the schemas
//!   admit any value under a namespaced key.
//!
//! Every `$ref` in the schemas resolves inside its own document. Compilation
//! runs with a retriever that refuses and records every request, so a schema
//! that ever tried to fetch a remote document would fail to compile rather
//! than reach the network.

use std::fmt;
use std::sync::{Arc, Mutex, OnceLock};

use serde_json::Value;

use crate::schemas::{self, Family};

/// What a valid message means for a consumer.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Outcome {
    /// The envelope and its payload are valid for the family `message_type`
    /// names.
    Accepted {
        /// The family the payload was validated against.
        family: Family,
        /// The envelope's `message_type`.
        message_type: String,
    },
    /// The envelope is valid but its `message_type` names no SOM 1.0 family.
    /// A conformant consumer ignores the message (conformance §5).
    Ignored {
        /// The unrecognised `message_type`.
        message_type: String,
    },
}

/// Which document a violation was found in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Part {
    /// The envelope, validated against `envelope.schema.json` and §3.
    Envelope,
    /// The payload, validated against the family's schema.
    Payload(Family),
}

/// Why a document was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ViolationKind {
    /// A JSON Schema keyword failed. `keyword` is the keyword's name, such as
    /// `format`, `required`, `additionalProperties` or `falseSchema`.
    Schema {
        /// The failing keyword.
        keyword: String,
    },
    /// `som_version` is not a SOM 1.x version (conformance §3). The envelope
    /// schema accepts any `major.minor.patch`, so this is not a schema error.
    WireVersion {
        /// The version found on the wire.
        found: String,
    },
}

/// One reason a document was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Violation {
    /// The document it was found in.
    pub part: Part,
    /// What failed.
    pub kind: ViolationKind,
    /// JSON Pointer to the failing value, relative to `part`; empty for the
    /// document itself.
    pub instance_path: String,
    /// Human-readable description.
    pub message: String,
}

impl Violation {
    /// The failing schema keyword, if this is a schema violation.
    #[must_use]
    pub fn keyword(&self) -> Option<&str> {
        match &self.kind {
            ViolationKind::Schema { keyword } => Some(keyword),
            ViolationKind::WireVersion { .. } => None,
        }
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.part {
            Part::Envelope => f.write_str("envelope")?,
            Part::Payload(family) => write!(f, "{family} payload")?,
        }
        let path = if self.instance_path.is_empty() {
            "/"
        } else {
            &self.instance_path
        };
        write!(f, " {path}: {}", self.message)
    }
}

/// A refused document: at least one violation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Rejection {
    /// Every violation found, envelope first.
    pub violations: Vec<Violation>,
}

impl fmt::Display for Rejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rejected with {} violation(s)", self.violations.len())?;
        for v in &self.violations {
            write!(f, "; {v}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Rejection {}

/// A schema failed to compile. Only possible if the embedded schemas are
/// malformed or reference a document outside themselves, which the crate's
/// own tests rule out for the schemas it ships.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SchemaError {
    /// The schema file that failed.
    pub schema: &'static str,
    /// The compiler's message.
    pub message: String,
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "schema {} failed to compile: {}",
            self.schema, self.message
        )
    }
}

impl std::error::Error for SchemaError {}

/// Whether `version` is a SOM 1.x wire version: exactly `1.<minor>.<patch>`.
///
/// This is a gate, not a branch: a consumer accepts every 1.x release
/// (compatibility policy) and never selects behaviour from the minor or patch.
#[must_use]
pub fn is_supported_wire_version(version: &str) -> bool {
    let mut parts = version.split('.');
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some("1"), Some(minor), Some(patch), None) if digits(minor) && digits(patch)
    )
}

#[derive(Default)]
struct RecordingRefusal {
    requested: Arc<Mutex<Vec<String>>>,
}

impl jsonschema::Retrieve for RecordingRefusal {
    fn retrieve(
        &self,
        uri: &jsonschema::Uri<String>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let uri = uri.as_str().to_owned();
        self.requested
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(uri.clone());
        Err(format!("remote schema retrieval is disabled: {uri}").into())
    }
}

/// Compiles `schema` as Draft 2020-12 with format assertion on and remote
/// retrieval refused. Returns the URIs the compiler asked for, if any.
fn compile(
    name: &'static str,
    schema: &Value,
) -> (Result<jsonschema::Validator, SchemaError>, Vec<String>) {
    let retriever = RecordingRefusal::default();
    let requested = Arc::clone(&retriever.requested);
    let built = jsonschema::draft202012::options()
        .should_validate_formats(true)
        .with_retriever(retriever)
        .build(schema)
        .map_err(|e| SchemaError {
            schema: name,
            message: e.to_string(),
        });
    let requested = requested
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    (built, requested)
}

fn parse_and_compile(
    name: &'static str,
    source: &str,
    requested: &mut Vec<String>,
) -> Result<jsonschema::Validator, SchemaError> {
    let schema: Value = serde_json::from_str(source).map_err(|e| SchemaError {
        schema: name,
        message: e.to_string(),
    })?;
    let (built, asked) = compile(name, &schema);
    requested.extend(asked);
    built
}

/// The SOM 1.0 validator: the envelope schema and the six payload schemas,
/// compiled once and reusable from any thread.
///
/// Compilation costs milliseconds; validation costs microseconds. Build one
/// with [`Validator::new`] and keep it, or use [`Validator::shared`] for a
/// process-wide instance.
pub struct Validator {
    envelope: jsonschema::Validator,
    story_context: jsonschema::Validator,
    telling: jsonschema::Validator,
    link: jsonschema::Validator,
    delivery_media_available: jsonschema::Validator,
    system_audit: jsonschema::Validator,
    skill_warning: jsonschema::Validator,
    retrieval_requests: Vec<String>,
}

impl fmt::Debug for Validator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Validator")
            .field("families", &Family::ALL)
            .field("retrieval_requests", &self.retrieval_requests)
            .finish_non_exhaustive()
    }
}

static SHARED: OnceLock<Result<Validator, SchemaError>> = OnceLock::new();

impl Validator {
    /// Compiles the embedded schemas.
    ///
    /// # Errors
    ///
    /// [`SchemaError`] if an embedded schema does not parse or compile. The
    /// shipped schemas do, and a test pins that; the error exists so the
    /// library has no panic path.
    pub fn new() -> Result<Self, SchemaError> {
        let mut requested = Vec::new();
        let mut compiled = |family: Family| {
            parse_and_compile(family.schema_file(), family.schema_source(), &mut requested)
        };
        let story_context = compiled(Family::StoryContext)?;
        let telling = compiled(Family::Telling)?;
        let link = compiled(Family::Link)?;
        let delivery_media_available = compiled(Family::DeliveryMediaAvailable)?;
        let system_audit = compiled(Family::SystemAudit)?;
        let skill_warning = compiled(Family::SkillWarning)?;
        let envelope =
            parse_and_compile("envelope.schema.json", schemas::ENVELOPE, &mut requested)?;
        Ok(Validator {
            envelope,
            story_context,
            telling,
            link,
            delivery_media_available,
            system_audit,
            skill_warning,
            retrieval_requests: requested,
        })
    }

    /// A process-wide validator, compiled on first use.
    ///
    /// # Errors
    ///
    /// As [`Validator::new`]; a failure is remembered and returned to every
    /// caller.
    pub fn shared() -> Result<&'static Validator, SchemaError> {
        SHARED
            .get_or_init(Validator::new)
            .as_ref()
            .map_err(Clone::clone)
    }

    /// Every external document the schema compiler asked for while compiling.
    /// Each request was refused; for the SOM 1.0 schemas this is empty.
    #[must_use]
    pub fn retrieval_requests(&self) -> &[String] {
        &self.retrieval_requests
    }

    fn family_validator(&self, family: Family) -> &jsonschema::Validator {
        match family {
            Family::StoryContext => &self.story_context,
            Family::Telling => &self.telling,
            Family::Link => &self.link,
            Family::DeliveryMediaAvailable => &self.delivery_media_available,
            Family::SystemAudit => &self.system_audit,
            Family::SkillWarning => &self.skill_warning,
        }
    }

    /// Validates a full wire message: envelope, wire version, and the payload
    /// against the schema its `message_type` selects.
    ///
    /// # Errors
    ///
    /// [`Rejection`] with every violation found, envelope violations first.
    /// A `message_type` SOM 1.0 does not define is not an error: the envelope
    /// is still validated and the outcome is [`Outcome::Ignored`].
    pub fn validate_message(&self, message: &Value) -> Result<Outcome, Rejection> {
        let mut violations = schema_violations(&self.envelope, message, Part::Envelope);

        if let Some(Value::String(version)) = message.get("som_version")
            && !is_supported_wire_version(version)
        {
            violations.push(Violation {
                part: Part::Envelope,
                kind: ViolationKind::WireVersion {
                    found: version.clone(),
                },
                instance_path: "/som_version".to_owned(),
                message: format!(
                    "som_version {version:?} is not SOM 1.x; pre-1.0 traffic is not \
                     SOM 1.0 (conformance section 3)"
                ),
            });
        }

        let message_type = match message.get("message_type") {
            Some(Value::String(t)) => Some(t.as_str()),
            _ => None,
        };
        let family = message_type.and_then(Family::from_message_type);
        if let (Some(family), Some(payload)) = (family, message.get("payload")) {
            violations.extend(schema_violations(
                self.family_validator(family),
                payload,
                Part::Payload(family),
            ));
        }

        // The envelope schema requires a string message_type, so a message
        // with none has a violation above. The fallback keeps this function
        // free of any panic path should that ever change.
        let Some(message_type) = message_type else {
            violations.push(Violation {
                part: Part::Envelope,
                kind: ViolationKind::Schema {
                    keyword: "required".to_owned(),
                },
                instance_path: String::new(),
                message: "\"message_type\" is a required property".to_owned(),
            });
            return Err(Rejection { violations });
        };
        if !violations.is_empty() {
            return Err(Rejection { violations });
        }
        let message_type = message_type.to_owned();
        Ok(match family {
            Some(family) => Outcome::Accepted {
                family,
                message_type,
            },
            None => Outcome::Ignored { message_type },
        })
    }

    /// Validates a bare payload against one family's schema. For callers that
    /// already know the family from a validated envelope, and for fixtures that
    /// hold a payload without its envelope.
    ///
    /// # Errors
    ///
    /// [`Rejection`] with every schema violation found.
    pub fn validate_payload(&self, family: Family, payload: &Value) -> Result<(), Rejection> {
        let violations = schema_violations(
            self.family_validator(family),
            payload,
            Part::Payload(family),
        );
        if violations.is_empty() {
            Ok(())
        } else {
            Err(Rejection { violations })
        }
    }
}

fn schema_violations(
    validator: &jsonschema::Validator,
    instance: &Value,
    part: Part,
) -> Vec<Violation> {
    validator
        .iter_errors(instance)
        .map(|e| Violation {
            part,
            kind: ViolationKind::Schema {
                keyword: e.kind().keyword().to_owned(),
            },
            instance_path: e.instance_path().as_str().to_owned(),
            message: e.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn an_external_ref_is_requested_refused_and_recorded() {
        let schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$ref": "https://example.com/elsewhere.schema.json"
        });
        let (built, requested) = compile("probe", &schema);
        assert!(
            built.is_err(),
            "an unresolvable external ref must not compile"
        );
        assert_eq!(requested, vec!["https://example.com/elsewhere.schema.json"]);
    }

    #[test]
    fn wire_versions() {
        for ok in ["1.0.0", "1.1.0", "1.12.3"] {
            assert!(is_supported_wire_version(ok), "{ok}");
        }
        for bad in [
            "0.3.2", "0.9.9", "2.0.0", "1.0", "1.0.0.0", "01.0.0", "1.x.0", "", "1..0",
        ] {
            assert!(!is_supported_wire_version(bad), "{bad}");
        }
    }

    #[test]
    fn the_shared_validator_is_one_instance() {
        let a = Validator::shared().unwrap();
        let b = Validator::shared().unwrap();
        assert!(std::ptr::eq(a, b));
    }

    #[test]
    fn a_message_without_a_message_type_is_rejected_not_panicked_on() {
        let rejection = Validator::new()
            .unwrap()
            .validate_message(&json!({"payload": {}}))
            .unwrap_err();
        assert!(
            rejection
                .violations
                .iter()
                .all(|v| v.part == Part::Envelope)
        );
        assert!(
            rejection
                .violations
                .iter()
                .any(|v| v.message.contains("message_type"))
        );
    }
}
