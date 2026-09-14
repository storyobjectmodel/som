# SOM 1.0 — Open register

What 1.0 does not settle.

A 1.0 is a commitment to stability, not a claim of completeness. Every entry below is
a question the working group knows about and has deliberately left open rather than
guessed at. Nothing here blocks an implementation: the contract in `schema/` is stable
whatever these resolve to.

Entries are resolved by a change governed by [`compatibility-policy.md`](compatibility-policy.md)
like any other — most of them additively, in a 1.x release.

---

## 1. Three withdrawn fields, candidates for 1.1

Proposed during v0.3.2 and withdrawn at 1.0 because they were never ratified. Under
the compatibility policy, adding an optional field or an enum member is permitted in
any 1.x release, so each can return the moment it is settled — breaking nothing.

| Candidate | Open question |
|---|---|
| `assets[].voice_count` | The field is trivial; the rule attached to it is not. It carried "`voice_count: 1` MAY classify a package SECONDARY; absent or >1, the TERTIARY default stands" — an editorial classification rule (#22). Does that rule belong in the schema at all, or in the skill library where the rest of the classification logic lives? |
| `assets[].status` → `FINALIZING` | Named 13 July, taken to the consortium deck 15 July, never ratified. Intended for a derived output that is complete but still finishing — a transcription still growing after the parent media is CAPTURED. Question: is this distinct enough from `IN_PRODUCTION` to earn an enum member? |
| `transforms[].transform_id` | A stable handle so audit records can target a transform by identity rather than array position. Proposed 9 July. Application order remains the array position; this is identity, not order. Question: does anything actually need to reference a single transform, or is the array position sufficient in practice? |

## 2. Scheme-qualified tag matching

`tags[]` is ordered and scheme-qualified — `tags[0]` is primary, earliest match wins,
and a scheme is one of `newsroom`, `iptc-mediatopic`, or `com.{vendor}.{name}`.

Today the skill library matches on `tags[].value` regardless of scheme, so a
configured instance watching for `sport` matches whether the tag came from the
newsroom's own vocabulary or from IPTC.

Open question: should matching be scheme-qualified — `newsroom:sport` rather than
`sport`? Scheme-qualified matching is more precise and more work to configure. This is
a skill-library question, not a schema one; `tags[]` itself is unaffected either way.

## 3. The framing-treatment registry

`assets[].framing_treatment` is a governed per-newsroom registry key — UPPER_SNAKE for
canonical values, `x-<lowercase>` for vendor extensions, single-valued, on SECONDARY and
TERTIARY assets.

The field ships at 1.0. **The governed registry it keys into does not.** Until it is
published, a newsroom's canonical values are its own, and cross-newsroom agreement on
this field is by convention rather than by reference.

## 4. A negative conformance corpus

`examples/` is 17 messages that MUST validate. There is no published set of messages
that MUST be **rejected** — a missing `correlation_id`, a `topic` not beginning `som.`,
a non-UUID `message_id`, an unknown property where none is permitted.

Without one, conformance for the failure cases is self-asserted, and independent
implementations can diverge without anyone noticing. This matters most for format
assertion (`conformance.md` §4), which is the requirement implementations are most
likely to skip by default.

Roughly twenty small files. The highest-value single addition to this repository.

**Resolved, 12 September 2026.** `examples/negative/` holds twenty must-reject cases and
`tools/validate_negative.py` proves each is rejected. Adding a constraint to a schema now
comes with adding the case it rejects.

## 5. No worked envelope example

All 17 published examples are bare payloads. The envelope — which is required on every
message, and the one thing every implementation must get right — is exercised by no
example at all.

**Resolved, 12 September 2026.** `examples/story-context/hurricane-beat6-envelope.json`
is beat 6 of the hurricane run as it travels on the wire, envelope and all.

## 6. The generated schema reference

The field-by-field reference is generated from the schemas so that it cannot drift from
them. The generator predates the flat 1.0 layout: it hardcodes the three version
directories and computes a v0.3.1-to-v0.3.2 delta that no longer means anything.

Until it is reworked, `spec/` carries no generated reference and `schema/` is the only
authority on field-level detail.

## 7. Telling beat 08

The worked hurricane run ends with `usage[]` committed but never publishes the
`link.committed` that writes it. The run therefore exercises Story and Asset but never
Telling.

That is a new beat rather than a correction to the existing seven, and it is owed to
the working group member who shaped the run.

## 8. Source-of-truth inversion

Historically the schemas were authored outside this repository and vendored in, with a
drift gate to catch disagreement. This repository is now authoritative.

The remaining question is whether the published partner packs should be generated by CI
from this repository — which removes the whole class of drift rather than policing it.
A working-group decision, not a technical one.

## 9. Story-to-story relationships

`relations[]` ships in 1.0 as it was carried from v0.2: `relation_type` is an
unconstrained string with seven values that are convention only, and `target_story_id`
is optional, which permits a relationship to nothing. An implementer reading the schema
today will invent values, and two implementers will invent different ones.

A full model was worked through by the working group in August and offered for
ratification. It was not taken before the 1.0 cut and is recorded here rather than lost.
It proposed: a governed enum (`FOLLOW_UP`, `DEVELOPS`, `SIDEBAR`, `SPIN_OFF`, `RELATED`,
`CORRECTS`, `SUPERSEDES`, `SAME_STORY`); `target_story_id` required; an explicit
direction rule, where a relationship is read as "this story `relation_type` the target
story" and direction records where it was declared rather than any hierarchy;
withdrawal by state rather than deletion; a back-reference from a relationship to the
assertion that confirmed it; and a rule that only `CORRECTS` and `SUPERSEDES` may give
grounds for a declaration, inform only, one hop at a time, with any walk bounded because
relationships can legitimately form loops.

The open questions, each of which can be resolved additively under the compatibility
policy:

| Question | What turns on it |
|---|---|
| The governed enum itself | Without it, `relation_type` is free text and nothing is interoperable between two implementations. |
| Duplicate resolution | Two stories for one event resolve two different ways. Inside one owning system it is a merge: one story is kept, the other retires to `ARCHIVED` with the reason in the audit, and anything bound to it moves to the survivor keeping its `asset_id` and original timestamps. Across two owning systems there is no merge, only a `SAME_STORY` declaration from each side. Neither path is in the schema today. |
| A proposal does not declare its resolution | A `MATCH` in `assertions[]` carries the target identifier only, so a reviewer confirms without knowing whether they are confirming a merge or a relationship, and a consumer reading `CONFIRMED` cannot tell which to expect. |
| Cross-newsroom referencing form and `story_id` uniqueness scope | `target_story_id` works where identifiers have been exchanged directly. Until the general form is settled, cross-newsroom relationships are not interoperable, and the spec should say so plainly. |
| Cross-domain subscription | A newsroom can only act on another's correction if it is receiving that newsroom's snapshots. This is a conformance-floor question rather than a relations one. |
| The `FOLLOW_UP` and `DEVELOPS` boundary | A desk chooses between the two daily. One sentence each, or merge them, before they become synonyms in practice. |
| Re-proposal of a rejected match | Whether a `REJECTED` assertion on the same target and claim suppresses re-proposal, or whether de-duplicating is the deployment's job. Unstated, a confident matcher re-offers the same relationship every pass. |
| An umbrella or running story | Whether day stories declaring `DEVELOPS` back to a spine is sufficient, or whether the running-story case earns its own value, declared by the child rather than maintained as a list on the parent. |

There is a dependency worth naming: a skills executor cannot currently be recalled on a
relationship at all. The recall advert supports `field` and `field_change` conditions
only. Until a condition kind exists for a restriction carried through an editorial link,
`CORRECTS` is actionable in principle and unreachable in practice.

## 10. Who mints a story

The schema is deliberately silent on which system may create a story, and that silence
has been read both ways. This entry records the position the working group has been
operating to, so that it is either adopted or argued with rather than assumed.

What 1.0 already requires is narrow and unchanged: `story.context` is published by the
story owner, `story_id` is immutable, `sequence_number` increases, and
`originating_system` is re-stamped by whoever publishes, so that a correction is
attributed to the corrector rather than to whoever first minted.

The working position on top of that: minting authority is deployment policy, not a
standard-level rule, and one story has one owner. A newsroom may nominate whatever it
likes as the owner of a given class of story, including a light pass-through that mints
on a high-priority wire under a rule the newsroom wrote. What the position excludes is
an agency or a wire feed minting a story into a newsroom it cannot see into, and any
deployment in which two systems can both mint the same event. A wire arriving is not a
story existing; raw wires do not travel the bus, and a wire settles into a story as
referenced content in `content_refs[]` with its origin on `editorial_source[]`.

The question for the group is whether any of this belongs in the conformance floor.
Today an implementation can satisfy every rule in `conformance.md` and still produce two
stories for one event, and by entry 9 the reconciliation path for that is not in the
schema either.
