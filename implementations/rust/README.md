# som: a Rust implementation of SOM 1.0

A library for producing and consuming Story Object Model 1.0 messages. It
embeds this repository's own `schema/` directory at compile time, so there is
no copy of the schemas to drift, and it is tested against the repository's
own examples and negative corpus.

## Using it

The crate is not on crates.io. Depend on it from this repository:

```toml
[dependencies]
som = { git = "https://github.com/storyobjectmodel/som" }
```

or, from a checkout, `som = { path = "../som/implementations/rust" }`. Either
way Cargo has the whole repository, which the crate needs: `src/schemas.rs`
reads `../../schema/*.json` with `include_str!`. Vendoring the crate on its
own does not build; vendor the repository.

Minimum supported Rust version 1.89, edition 2024.

```rust
use som::builder::{EnvelopeBuilder, name_based_id};
use som::model::audit::{AuditAction, AuditTarget, AuditTargetKind, SystemAudit};
use som::model::envelope::{OriginatingSystem, SystemType};
use som::model::{Actor, decode_message};
use som::validate::{Outcome, Validator};

let audit = SystemAudit {
    message_type: None,
    audit_id: name_based_id("compliance-01", "audit-77").to_string(),
    action: AuditAction::Cleared,
    target: AuditTarget { kind: AuditTargetKind::Link, id: "link-93".to_owned() },
    actor: Actor { actor_id: "j.doe".to_owned(), actor_type: "user".to_owned() },
    reason: Some("legal review complete".to_owned()),
    recorded_at: "2026-09-11T16:41:00Z".to_owned(),
};
let envelope = EnvelopeBuilder::new(OriginatingSystem::new("compliance-01", SystemType::ComplianceEngine))
    .correlation_id(name_based_id("ncs-01", "hurricane-2026-0911"))
    .topic("som.system.audit")
    .build(audit)?;
let wire = serde_json::to_value(&envelope)?;

let validator = Validator::shared()?;
assert!(matches!(validator.validate_message(&wire)?, Outcome::Accepted { .. }));
let typed = decode_message(wire)?.expect("system.audit is a SOM 1.0 family");
```

The same example is the crate's documentation example and runs as a doctest.

## Features

| Feature | Default | Brings |
|---|---|---|
| `validate` | on | `som::validate` and `som::sequence`, and the `jsonschema` dependency |

Without `validate` the crate is the typed model, the envelope builder and the
RFC 3339 helpers, and its dependencies are `serde`, `serde_json` and `uuid`.
A consumer that validates elsewhere, or a producer that only needs the types,
can turn the feature off. A conformant publisher still has to publish valid
payloads (conformance §1), and the typed model alone does not guarantee that.

## Conformance

Per `spec/conformance.md` §1, the families this implementation produces and
consumes:

| Family | Produces | Consumes |
|---|---|---|
| Envelope | yes | yes |
| `story.context` | yes | yes |
| `telling.started`, `telling.ended`, `telling.exposed` | yes | yes |
| `link.committed`, `link.gate_changed`, `link.withdrawn` | yes | yes |
| `delivery.media_available` | yes | yes |
| `system.audit` | yes | yes |
| `skill.warning.raised` | yes | yes |

Any other `message_type` is ignored, not rejected (§5): `Validator` returns
`Outcome::Ignored` and `decode_message` returns `Ok(None)`. A producer can
still envelope such a payload by implementing `MessagePayload` for it.

- The payload schema is selected from `message_type` and nothing else (§2).
- `som_version` must be 1.x, so `0.3.2` is rejected even though the envelope
  schema accepts it, and the version never selects a schema (§3).
- Formats are asserted (§4), on the payload as well as the envelope. Format
  assertion covers every format the validator knows, not only `uuid` and
  `date-time`. The schemas use one more, `uri` on `content_refs[].uri`, which
  this crate asserts; `tools/validate.py` asserts it only when Python's
  `rfc3987` or `rfc3986-validator` package is installed, which the install
  line does not include. No example in the corpus exercises the difference.
- Unknown `com.{vendor}.` extensions pass and survive the typed model (§5).

Every `$ref` in the schemas is local. Compilation refuses and records any
remote fetch, and the crate is built without the validator's HTTP features.

## What is in it

- **Typed models** (`som::model`) for the envelope and the six payload
  families. Objects the schema closes refuse unknown members, so the
  withdrawn fields (`source`, `signature`, `ai_enrichments`, `voice_count`,
  `transform_id`, `instance_ref` and the rest) are refused on decode.
  Extensions accept only `com.{vendor}.` keys. Enums carry exactly the
  schema's members, including the lower-case skill severities; the
  governed-but-extensible registries (`asset_type`, `source_type`,
  `transform_type`) carry their seed values plus `Extension(x-...)`. Optional
  members stay optional, and a decoded value re-encodes to equal JSON.
  Identifiers and date-times stay strings for the same reason: a `Uuid`
  re-encodes lowercase and a parsed time loses its spelling.
- **Validation** (`som::validate::Validator`), Draft 2020-12 with format
  assertion on, compiled once; `Validator::shared()` is a process-wide
  instance. A rejection lists every violation with the document it was found
  in, the failing keyword and a JSON Pointer to the value.
- **An envelope builder** (`som::builder::EnvelopeBuilder`) that mints a
  `UUIDv7` `message_id` per message, stamps `som_version` `1.0.0`, requires a
  `correlation_id`, and refuses a topic that does not begin with `som.`.
  `build_with_message_id` takes a caller-minted id, so a producer with its
  own generator, or a test, gets identical output from identical input.
- **Name-based ids** (`som::builder::name_based_id`): the `UUIDv5`, under the
  namespace `28330528-38ae-5396-a8cb-fa4e4ee65de9`, of
  `{byte length of scope}:{scope}{record_id}`. A producer derives
  `correlation_id`, `telling_id`, `link_id` and similar ids from its own
  record ids, so a replay publishes the same ids. The namespace is the
  `UUIDv5` of
  `https://github.com/storyobjectmodel/som/tree/main/implementations/rust#record-ids`
  in the URL namespace; a test holds vectors computed independently in Python.
- **A sequence checker** (`som::sequence`) with the rules of
  `tools/validate_sequence.py`: `story_id` immutable, `sequence_number`
  increasing, `updated_at` moving forward, nothing dropped between
  snapshots, asset types stable, terminal story types final, and a changed
  snapshot never republished under the same `message_id` by the same system.
  It differs from the Python tool in two ways: snapshots are validated with
  formats asserted, and date-times are compared as instants, not as strings.
- **RFC 3339** (`som::time`): parsing to an instant and formatting with
  microsecond precision, which is all SOM needs; it is not a date library.

The typed model does not replace validation: string formats, patterns,
numeric ranges and the schemas' conditional rules (`if`/`then`, `anyOf` of
`required` lists, the claim members an `assertion_type` requires) are
enforced by the validator. Validate, then decode. One known divergence in
the other direction: JSON Schema's `integer` admits `3.0`, and the typed
model's integer members do not.

## Stability

This is a 0.x crate. Within 0.x a minor version may change the API; a patch
version does not. The crate's own enums that may grow with the standard or
with the crate (`Family`, `Payload`, `Outcome`, the error and finding types)
are `#[non_exhaustive]`, so a 1.x family can be added without breaking a
consumer that matches on them. The enums that mirror a schema enum are
exhaustive on purpose: a schema change there should be a compile error in a
consumer that matches on it.

## Checks

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --no-default-features -- -D warnings
cargo test
cargo test --no-default-features
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps
cargo +1.89 test                 # the declared minimum Rust version
```

Clippy runs with `clippy::pedantic` on through `[lints]` in `Cargo.toml`;
the one `allow` in the crate says why it is there. `unsafe_code` is
forbidden and every public item is documented.

The tests read the repository's real files:

- every example outside `examples/negative/` validates and round-trips
  through the typed model with JSON equality;
- for every object in every example, a member no schema names is refused by
  the typed model exactly where the schema refuses it, and kept where the
  schema leaves the object open;
- for every member of every object in every example, removing it is refused
  by the typed model exactly where the schema refuses it, except for the
  schemas' conditional rules, which are listed in the test and which only
  the validator can see;
- every closed enum and every registry's seed list in the typed model is
  pinned to the schema's `enum` or `const` at its JSON Pointer, and the
  hand-coded extension and registry patterns are pinned to the schemas';
- every case in `examples/negative/` is rejected, each in its own test that
  asserts the reason the case states and nothing more, including the `0.3.2`
  case the schemas alone accept;
- the hurricane run passes the sequence checker, and each sequence rule has
  a test that breaks it.

## Licence

Apache License 2.0, as for the schemas, examples and tools of this
repository; see [`../../LICENSE`](../../LICENSE).
