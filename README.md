# Story Object Model 1.0

An open standard for story context in content production.
Home: [storyobjectmodel.com](https://storyobjectmodel.com). Governance and how to take
part: [`GOVERNANCE.md`](GOVERNANCE.md). Who made it: [`CONTRIBUTORS.md`](CONTRIBUTORS.md).

A newsroom runs on systems that each hold part of a story: the planning system knows
what it is about, the media store knows what was shot, the rundown knows where it
airs, the compliance engine knows what may not be said. None of them share a way to
say *this is the same story, and here is what is true about it right now*.

SOM is that shared way. It is a small set of JSON message families published on a bus,
so that any system can say what it knows and any other system can act on it without a
point-to-point integration.

SOM 1.0 carries forward the contract proven at IBC as v0.3.2, with three fields
withdrawn that had never been ratified. It is a **clean break**: `1.0.0` is the only
version on the wire, and v0.3.2 traffic is not 1.0 traffic. See
[`spec/migration-from-v0.3.2.md`](spec/migration-from-v0.3.2.md) — the migration is
small and mechanical.

## The seven message families

Everything normative in this repository is in [`schema/`](schema/). Nothing else is.

| Family | File | Published by |
|---|---|---|
| Message envelope | `envelope.schema.json` | every publisher |
| `story.context` | `story-context.schema.json` | the story owner |
| `som.telling.*` | `telling-event.schema.json` | the exposure publisher |
| `som.link.*` | `link-event.schema.json` | the committing system |
| `delivery.media_available` | `delivery-media-available.schema.json` | the media store (MAM / TAMS) |
| `som.system.audit` | `system-audit.schema.json` | any governance actor |
| `skill.warning.raised` | `skill-warning.schema.json` | a skills executor |

Each schema is identified by a stable URL that also serves it:

```
https://storyobjectmodel.com/schema/1.0/story-context.schema.json
```

That identifier will not change for the life of 1.x. What may change, and what would
require a 2.0, is set out in [`spec/compatibility-policy.md`](spec/compatibility-policy.md).

## Start here

0. [`spec/introduction.md`](spec/introduction.md) — why this exists and the six
   principles behind it, in plain words. Read this if you are not an implementer.
1. [`spec/conformance.md`](spec/conformance.md) — what it means to be SOM 1.0 conformant.
   Shorter than you expect: most families are opt-in.
2. [`examples/hurricane-run/`](examples/hurricane-run/) — one story told across seven
   snapshots, which is the clearest picture of how SOM is meant to be used.
3. [`schema/story-context.schema.json`](schema/) — the family you will almost certainly
   implement first.

## Validating

```
pip install jsonschema rfc3339-validator
python3 tools/validate.py                          # every example against the schemas
python3 tools/validate_negative.py                 # every negative case is rejected
python3 tools/som_lint.py schema                   # the schemas against their own claims
python3 tools/validate_sequence.py examples/hurricane-run   # a story over time
```

Four different questions. `validate_negative.py` asks whether the schemas actually
constrain: `examples/negative/` holds twenty messages that must be rejected, each with
its reason, so that an implementation in any language can be checked against the same
list. Nineteen are rejected by the schemas; the twentieth, `som_version: "0.3.2"`, is
rejected by the wire-version rule in `spec/conformance.md` §3, which the schema
deliberately does not encode. Without `rfc3339-validator`, Python's `jsonschema` silently
accepts a malformed `timestamp`; that case is in the corpus, so the check fails loudly if
the package is missing.

`validate.py` asks whether a message is legal.
`som_lint.py` asks whether the schema agrees with what it says about itself.
`validate_sequence.py` asks whether a sequence of snapshots of one story holds
together — which no single message can answer, and where the expensive bugs live.

**Implementations must assert `format`.** Most JSON Schema validators treat `format`
as an annotation and ignore it unless told otherwise, which means a `message_id` of
`"not-a-uuid"` passes by default. See `spec/conformance.md`.

## Repository layout

```
schema/      the seven families. NORMATIVE. Nothing else here is.
examples/    18 worked messages, one directory per family, plus negative/ (20 must-reject)
tools/       validators
skills/      som-skill-library 0.2.2 — a shared vocabulary for newsroom automation
spec/        introduction and principles, conformance, compatibility policy,
             migration, open register, glossary, version history
```

The skill library versions separately from the standard, and deliberately. `schema/` is
the normative contract and carries a stability promise; the library is a vocabulary
layered on top of it that is still moving — its skills are marked `lifecycle: draft`.
A newsroom can implement SOM 1.0 completely without using the library at all.
[`spec/compatibility-policy.md`](spec/compatibility-policy.md) §5 sets out what that
separation means.

## Licence

Two licences, split by what the file is for.

| Covers | Licence |
|---|---|
| `schema/`, `examples/`, `tools/`, `skills/` | Apache License 2.0 — [`LICENSE`](LICENSE) |
| `spec/`, and the prose in this file | CC BY 4.0 — [`LICENSE-SPEC`](LICENSE-SPEC) |

Apache 2.0 covers the schemas deliberately. They are consumed by machines and compiled
into products, and Apache 2.0 carries an express patent grant — which is the assurance
a vendor implementing an interoperability standard actually needs.

## Not in 1.0

Named so that nobody assumes otherwise:

- **Demo configuration.** The IBC walkthrough's configured skill runs name individual
  vendors and their products. Held pending sign-off from each.
- **Integration guide and FAQ.** Expected in a later point release.
- **Reference implementation.** A .NET sample implementation follows, in a separate
  repository.
- **Generated schema reference.** The generator predates the flat 1.0 layout.

Open questions that 1.0 deliberately does not settle are recorded in
[`spec/open-register.md`](spec/open-register.md) rather than left implicit — including
the three fields withdrawn at 1.0 and the three that would most improve the next
release. A 1.0 is a commitment to stability, not a claim of completeness.
