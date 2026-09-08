# Version history

The releases of the Story Object Model, newest first.

This records the **standard**. The skill library versions separately and keeps its own
changelog in [`../skills/README.md`](../skills/README.md).

---

## 1.0 — 12 September 2026

The first stable release. Published at IBC 2026 in Amsterdam by the Accelerator project
SMART STORIES.

1.0 carries forward the contract proven at IBC as v0.3.2 and closes it. From this point
the [compatibility policy](compatibility-policy.md) applies: nothing is removed from
`schema/` until 2.0.

**Three fields withdrawn.** Each was proposed during the v0.3.2 window and never
ratified, and shipping an unratified field in 1.0 would have locked it in until a major
version. All three remain candidates for 1.1, where re-adding them is additive and
breaks nothing. See [`open-register.md`](open-register.md).

| Withdrawn | Was |
|---|---|
| `assets[].voice_count` | optional integer, carrying an unsettled classification rule |
| `assets[].status` → `FINALIZING` | enum member |
| `transforms[].transform_id` | optional string, a stable audit handle |

None appeared in any published example, and nothing in the skill library referenced
them.

**A clean version break.** `som_version` is `"1.0.0"` and nothing else. Because fields
were withdrawn, a v0.3.2 payload may carry a field 1.0 disallows, so the two do not
denote the same contract. Consumers must not treat `"0.3.2"` as conformant. See
[`migration-from-v0.3.2.md`](migration-from-v0.3.2.md).

**Identifiers rebased and made resolvable.** Every schema is now identified by a URL
that also serves it, on the project's own domain:

```
https://storyobjectmodel.com/schema/1.0/<family>.schema.json
```

The former identifiers were never resolvable and carried `-proposed` in the identity of
a shipping standard. The new ones are stable for the life of 1.x.

**A flat schema directory.** The three version directories are gone; the seven families
sit in `schema/`, one file per family, no version in the filename. The v0.3.2
"effective pack" had spanned three directories, because v0.3.2 revised only three
families and the rest stayed authoritative at an earlier version. That inheritance is
now invisible.

**Format assertion is now required of implementations.** `format` is an annotation in
JSON Schema and most validators ignore it unless configured, so a `message_id` that is
not a UUID passed everywhere by default. Conformant implementations must assert it —
see [`conformance.md`](conformance.md) §4.

**Published apparatus.** A conformance statement, a compatibility policy, a migration
note and an open register, none of which existed before 1.0.

---

## v0.3.2 — 18 August 2026

The IBC demo target, and the shape 1.0 ratifies. Six changes to `story.context`, one to
`delivery.media_available`, none to the envelope.

- **`tags[]`** — subject-matter tagging: ordered, scheme-qualified (`newsroom` /
  `iptc-mediatopic` / `com.{vendor}.{name}`), additive and optional. Added because every
  other axis carried lifecycle, priority, compliance or provenance and none of them said
  what a story is *about*, so a skill trigger policy had no field to resolve on and was
  falling back to substring matching on headline text.
- **`assertions[]`** — claims about content (`FACT_CHECK` / `DETECTION` / `MATCH`) with a
  structured review state. **`ai_enrichments[]` is hard-rejected**: it narrowed to
  `assertions[]`, and generative outputs became Assets. Breaking for anyone on v0.3.1.
- **`assets[].provenance`** — authorship-general provenance, `HUMAN` or `MODEL`, never
  AI-keyed. Principle ratified 29 June.
- **`asset_type` gains `TRANSCRIPT`** (locked at v0.3.1 on 29 June, carried here because
  the 30 June re-cut omitted it) plus the promoted generative outputs `SUMMARY`,
  `SOCIAL_POST`, `ARTICLE`, `ANALYSIS`.
- **`story_type` gains `ORPHAN`** — a minimal shell story holding an unmatched clip,
  filterable by the `story_type` predicate.
- **`editorial_gate.blocks[]` contract restored** to `{kind: ASSET|PHASE, ref}`; the bare
  string form deprecated.
- **`delivery.media_available` gains a locator branch**, mirroring `media_refs[]`'
  `anyOf(source|locator)` onto the arrival event. Locked v0.3.1 required a `tams://`
  source, so a non-TAMS clip could be referenced by an asset but its arrival could not be
  announced. Media in SOM is any media, not just TAMS.

The SOM-048 `0.2.0` wire freeze was retired on 12 August 2026.

---

## v0.3.1 — 30 June 2026

Locked, with a Source re-key on 29 June.

- **The source disambiguation.** One overloaded word was carrying four jobs. `source` on
  the envelope became `originating_system`; `sources[]` on the story became
  `editorial_source[]`.
- **Three families proposed**: `som.link.*` (§3.1), `som.telling.*` (§3.2), and
  `som.system.audit`. The link is the Asset↔Destination connection; `usage[]` is
  maintained from link events alone, under idempotent-upsert rules. Telling carries the
  ratified event-pair semantics: `exposure_start` / `exposure_end` immutable and
  event-stamped, `scheduled_start` mutable.
- **A shared human-review record**, replacing the `ai_enrichments[].human_reviewed`
  boolean, which had no way to express rejection.

---

## v0.3 — June 2026

The envelope lock, decision #18.

- `correlation_id` REQUIRED, linking every message about one story lifecycle.
- `topic` REQUIRED, and it must begin `som.`
- Skill output timestamps live on the envelope, never in the payload.
- `message_type` established as the only parsing discriminator.
- JSON-LD `@context` deferred, but permitted for forward compatibility.
