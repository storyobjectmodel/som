# SOM 1.0 — Conformance

What a system must do to call itself SOM 1.0 conformant, and — just as important —
what it does not have to do.

The key words MUST, MUST NOT, SHOULD, SHOULD NOT and MAY are to be interpreted as
described in RFC 2119.

## 1. The floor is lower than the schema pack suggests

There are seven message families. **A conformant system does not implement seven
families.** It implements the envelope, plus whichever families it has something to
say about or something to do with.

A system is **SOM 1.0 conformant** if:

1. Every message it publishes is a valid envelope (§2), and
2. every payload it publishes validates against the schema for the `message_type` it
   declares, and
3. it ignores what it does not recognise (§5).

That is the whole floor. A planning system that only ever publishes `story.context`
and only ever reads `skill.warning.raised` is fully conformant.

Implementations SHOULD state which families they produce and which they consume. That
statement, not the size of the implementation, is what an integrator needs.

## 2. The envelope

Every SOM message is an envelope with the payload inside it. The envelope is defined by
`schema/envelope.schema.json` and is not optional.

- `message_type` is the **only** parsing discriminator. Implementations MUST select the
  payload schema from `message_type` and MUST NOT infer it from topic, filename,
  publisher identity, or payload shape.
- `correlation_id` is REQUIRED and links every message about one story lifecycle. It is
  the natural partition key.
- `message_id` MUST be a UUID. UUIDv7 is RECOMMENDED, because it sorts by time.
- `topic` MUST begin with `som.`
- `timestamp` is the authoritative time of the event. Skill output carries its timestamp
  here, not in the payload.

## 3. The wire version

`som_version` records the schema pack a payload conforms to. For this release that
value is `"1.0.0"`.

- Producers MUST emit the version of the pack they conform to. For SOM 1.0 that is
  `"1.0.0"`; a future 1.1 producer emits `"1.1.0"`.
- Consumers MUST NOT branch on `som_version`. It is informative. `message_type`
  identifies the payload family; `som_version` never does.
- `"0.3.2"` is **not** SOM 1.0. Three fields were withdrawn before ratification, so a
  v0.3.2 payload may carry a field that 1.0 disallows. Consumers MUST NOT treat
  `"0.3.2"` as conformant to this specification. See `migration-from-v0.3.2.md`.

A consumer that switches behaviour on this field will break at 1.1, which is by
definition a version increment that changes nothing it depends on.

## 4. Format assertion is REQUIRED

**This is the conformance requirement most implementations will get wrong by default.**

JSON Schema treats `format` as an annotation. Most validators therefore ignore it
unless explicitly configured, and a message like this passes with zero errors:

```json
{ "message_id": "NOT-A-UUID", "correlation_id": "also-not-a-uuid",
  "timestamp": "not-a-timestamp" }
```

A conformant implementation MUST enable format assertion for `uuid` and `date-time`.
Two implementations that disagree about whether `message_id` has to be a UUID are not
interoperable, however well each validates on its own.

| Language | What this requires |
|---|---|
| Python | `jsonschema` — pass `format_checker=`; install `rfc3339-validator` for `date-time` |
| Node | `ajv` v8 (`ajv/dist/2020`) plus `ajv-formats` |
| Go | `santhosh-tekuri/jsonschema/v6` with format assertion enabled |
| C# | a Draft 2020-12 validator with format validation switched on |

`tools/validate.py` in this repository asserts formats. An implementation that passes
where `tools/validate.py` fails is not conformant.

## 5. Unknown fields

- `extensions` carries vendor fields, reverse-domain namespaced (`com.{vendor}.*`).
  Consumers MUST ignore extensions they do not recognise, and MUST NOT reject a
  message for carrying them.
- A consumer MUST NOT fail on a `message_type` it does not handle. It ignores it.

This is what allows 1.x to add message families without breaking anything already
deployed.

## 6. `story.context` is a snapshot, never a delta

A `story.context` message is the complete state of the story at that moment.

A producer MUST NOT omit fields it did not modify. Omission means *absent*, not
*unchanged* — a writer that sends only its own fields silently erases every other
system's work, and this is the single most damaging error a SOM implementation can
make.

Across a sequence of snapshots for one story:

- `story_id` MUST be immutable. A revision is the next snapshot, not a new story.
- `sequence_number` MUST increase.
- `updated_at` MUST move forward.
- `originating_system` MUST be re-stamped by whoever publishes, so that a correction is
  attributed to the corrector and not to whoever first minted the story.

`tools/validate_sequence.py` checks all four across a run.

## 7. Demonstrating conformance

```
python3 tools/validate.py
python3 tools/validate_sequence.py examples/hurricane-run
```

Every example in `examples/` MUST validate against the published schemas, with formats
asserted. An implementation can demonstrate conformance by validating the same corpus
with its own validator and getting the same result.

A negative corpus — messages that MUST be rejected — is not yet published. Until it is,
conformance for the failure cases is self-asserted, and implementers should treat §4 as
the place they are most likely to differ from each other without noticing.
