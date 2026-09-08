# Shared conventions for the SOM skill library

Ten generic library skills, authored against SOM_SKILL_TEMPLATE_v0.1, covering
flagging, holding, gating, provenance, enrichment, matching, surfacing, provider
selection and clearance.

## "Configured instance", a locked term

The compound is one unit of vocabulary and is read as one. It names a single generic
skill file with one house's specific values loaded, watching one condition. A single
house runs several of them off the same file, one per condition it wants watched, which
is the whole point of a generic skill.

**Status: WORKING.** This is a position the library holds, not one any group has
confirmed. If a better word arrives it will be adopted.

The bare noun is banned, and the ban is older than this compound. "instance" alone was
banned in May as hopelessly overloaded. What was banned, precisely, was the schema's
`instances[]` array on the story object, the runtime crosspoint concept, which then split
into publication path, telling and delivery. That was a data-model decision about the
story object, and the compound here names a different concept: a skill file with a
configuration loaded against it. So this is a collision of senses rather than a breach of
the ban. A vendor reading cold will not hold that distinction, though. They meet a banned
word in a library that banned it and either stop trusting the prose or quietly merge the
two senses back together, and the second failure is the expensive one. So the bare noun
does not appear anywhere in the library, and the compound is always written in full.

Two candidates were tested and rejected. The reasons are worth keeping, because they
explain why the compound survived rather than something shorter.

**binding** fails on collision. Bind, binds, binding and bound are load-bearing seam
vocabulary throughout the library, over a hundred uses: gates binding a telling, assets
bound to a story, "every binding gate must permit, so any one hold means held".
Repurposing the word for the configuration concept would collide with the machinery the
architecture turns on, and a reader would have to decide which sense was meant every time
the word appeared.

**profile** fails on meaning. It is glossary-clean, used nowhere else in the library, and
"a house profile of the skill" reads naturally. But a profile implies one per house, a
settings bundle covering everything that house does. The concept is narrower: one skill,
one loaded configuration, one watched condition, and a house runs several of them. "one
advert row per profile" would mislead, because it reads as per-house when the truth is
per configured condition, and the row count is exactly what a vendor needs to get right.

Never write it bare. Not "the instance", not "another instance of it". Either the
compound in full, or a word that says more: the skill file, the loaded configuration, the
watched condition, the house's values. The config field name `instance_label` is a field
name rather than prose and is untouched by this.

## Frontmatter constants

These values are fixed across all ten files. Do not vary them per skill.

| Field | Value | Why |
|---|---|---|
| `publisher` | `smart-stories` | The namespace segment of every `skill_id`. |
| `skill_id` | `smart-stories/<name>` | Follows the warning schema's own worked example. |
| `skill_version` | `0.1.0` | Three numbers. No suffix. |
| `som_schema_version` | `["0.3.2"]` | **Corrected 18 August 2026.** This row previously said the move from `0.3.1` happened on 12 August and was enforced by the validator. The row said so; the ten files did not do it, and all ten still locked `0.3.1` on 18 August. They now lock `0.3.2`, which as of 18 August is the version. The validator does not check this row against the files, and until it does, this table is a statement of intent rather than of fact. Only `story.context`, the telling events and `delivery.media_available` changed. The envelope and `skill.warning.raised` did not, so no output contract moved and the whole change is on the read side. |
| `skill_type` | `REFERENCE` | Consistent with `origin: reference`. |
| `origin` | `reference` | Y = 5. The library is a shared set, adopted by choice, not a vendor product and not one house's rule. This is a fact about who made it, not a choice. |
| `lifecycle` | `draft` | Honest. Nothing here is ratified. |
| `owner_editorial` | `<name or team alias>` | Not ours to supply. Leave the marker. |
| `owner_engineering` | `<name or team alias>` | Same. |
| `licence` | `<SPDX identifier>` | Same. |
| `skill_uri` | `<gs:// or https:// canonical location, version-pinned>` | Same. A fake URI reads as a real one. |
| `skill_content_sha` | `<sha256:... of the skill body>` | Computed at build, not typed. |
| `level_2_uri` / `level_3_uri` | `null` | |
| `auto_change_content` | `false` | Only permitted value. |
| `fail_closed` | `true` | Only permitted value. |
| `migration_policy` | `GATED` | Chosen because none of these is certain enough for `HOT`. |
| `disclosure_level` | `L2` | Meaning "the deepest level the skill declares content for". Section 10 must say which of the two circulating definitions was meant. |
| `depends` | `[]` | See the chaining rule below. |
| `supersedes` / `superseded_by` | `null` | |
| `output_messages` | `["skill.warning.raised"]` | |

## The chaining rule: why `depends` is empty everywhere

Chaining happens through the bus, not through declared dependencies. A raise skill
does not depend on the hold skill that consumes its flag, because that inverts the
dependency and creates a cycle at the first change. A hold skill does not depend on
a particular raise skill either: it advertises against flag state, whoever wrote it.

Every file states this in section 2 and, where the reader would expect a dependency,
again in section 10.

## The raise-or-act split

`flag-on-mismatch` is authored as a **raise-only** skill, severity `flag` and `inform`,
even though the originating copy for these two skills reads "flag and hold".

The one-mechanism rule is not stylistic: a skill that both declares a state and
withholds something occupies two positions and cannot be resolved at a single
coordinate. The hold that copy describes is delivered by `hold-while-flagged`
advertising against the flag `flag-on-mismatch` declares, chained over the bus. Observed
behaviour is identical; the library keeps two resolvable skills instead of one
unresolvable one.

Every file that touches this says so plainly in section 10. Nobody should discover it
by reading the code.

### `flag-on-mismatch`: one skill, one watched field, two bindings

The same skill and the same configuration reach two situations: an asset held inside a
rundown, and an asset held outside any rundown, each bound by a different executor. It is
tempting to describe the second as running "the same skill and config" as the first. Read
strictly, that is not what the two configured instances are: they share the skill and the
watched field, and they differ in how each tool holds its own copy of the value and what
the tool is bound to. The honest claim, and the one the files make, is **one skill, one
watched field, two bindings**.

That is still the sharpest reusability claim the library makes, because the second asset
sits outside the rundown entirely and no MOS integration ever reached it. Overstating it
as byte-identical config invites a reviewer to check and find the difference, which costs
more than the stronger, true claim gains.

## `hold-while-flagged` versus `gate-by-scope`: what distinguishes them

These two are the closest pair in the library, and both may be recalled against the same
story, so the distinction has to be stated once, here, and never contradicted in a skill
file.

They are distinguished by **what is read**, not by scope.

- `hold-while-flagged` reads a **typed flag**: is a flag of the configured type standing
  against this action?
- `gate-by-scope` reads a **path's gate**: does the gate bound to this destination path
  permit this telling?

`hold-while-flagged` may be configured at `story:<id>` or `link:<id>` scope, because a
hold can legitimately bind a whole story or one telling of it. `gate-by-scope` is
`link:<id>` scope always, because a per-path publish permission has no meaning at story
level; a configured instance that declares one answer for the whole story is a
configuration error and should be rejected rather than coerced. Scope is still not the
distinguishing axis, since a link-scoped hold and a path's gate sit at the same
coordinate and remain different questions, and no file may claim scope is what separates
them. Scope is also not X, so a configurable scope does not make a skill unpositionable.

Where both bind against the same `editorial_gates[]` state on one story, they co-bind by
conjunction: an unconfirmed figure is withheld on a path if the flag stands or the path's
gate does not permit. Any one hold means held. That is normal, not an anomaly, because
per-coordinate uniqueness makes co-binding safe.

## The flag-to-hold chain, and the gap in it

Three files depend on a chain: a raise skill declares a flag, and `hold-while-flagged`
withholds against it. `raise-flag-on-match` and `flag-on-mismatch` each produce one output,
`skill.warning.raised`, and `auto_change_content: false` forbids either from writing story
state. So something has to carry a raised warning into the `editorial_gates[]` entry that
`hold-while-flagged` reads.

Nobody has said what. This is a genuine gap in the model, not an oversight in these files,
and every file that depends on the chain must name it in section 10 rather than write as
though the chain completes itself.

## Axis assignments

X is declared by the author and is auditable. Each choice must be defensible in
section 1, not merely stated.

| Skill | `category` (X) | `severity_range` | Reasoning |
|---|---|---|---|
| `raise-flag-on-match` | `compliance` (1) | `[flag, inform]` | The postures it declares at mint, indicative-category and UNVERIFIED, are compliance postures. It declares only; it withholds nothing. |
| `hold-while-flagged` | `compliance` (1) | `[hold]` | Withholding against a standing flag. Breach carries legal or regulatory consequence. |
| `flag-on-mismatch` | `editorial` (2) | `[flag, inform]` | Accuracy: what an asset says against what the story now holds true. Editorial values, not regulation. |
| `gate-by-scope` | `compliance` (1) | `[hold, inform]` | Whether a fact may be served on a given path. Per-destination clearance is a compliance question, not a routing one. |
| `record-provenance-on-ingest` | `compliance` (1) | `[inform]` | Authenticity and provenance obligation. Records only. |
| `enrich-on-condition` | `workflow` (3) | `[inform]` | When an enrichment runs in the ingest-to-output process. |
| `match-and-propose` | `workflow` (3) | `[inform]` | Association and handoff to a producer for confirmation. |
| `surface-on-context-match` | `workflow` (3) | `[inform]` | Discovery against a running story, surfaced for a human. |
| `select-provider-by-context` | `workflow` (3) | `[inform]` | Which provider the house's policy resolves for a declared capability. It resolves and displays; it never assigns work and never invokes anything. |
| `apply-clearance` | `workflow` (3) | `[inform]` | Version and approval resolution. It resolves which package is current; it never changes content. |

## Naming notes to carry, not to silently fix

Two library names do not follow `<operation>-<condition-type>`:

- `match-and-propose` contains "and". The conformant form is `propose-on-match`. The
  library name is the contract vendors are building to, so it is kept, and section 1
  defends why this is one mechanism rather than two: the proposal is the display of
  the match, and the producer's confirmation is a human act outside the skill.
- `apply-clearance` reads as operation-plus-event rather than operation-plus-condition-type.
  Kept for the same reason. Noted in section 10.

Never rename a library skill silently. The name is what a vendor built against.

## Language

Skills declare and display. Executors recall, rank and act. Write "the skill declares",
"the executor recalls", "the advert matches". Never "the skill fires", "the skill acts",
"the skill subscribes", "the skill emits".

"Most restrictive wins" and "story owner takes precedence" were formally withdrawn on
29 July 2026. Holds combine by conjunction: every binding gate must permit, so any one
hold means held.

Severity words and category and origin values are lower case, permanently.
`migration_policy` and `disclosure_level` are upper case.
