---
# --- identity ----------------------------------------------------------------
skill_id: smart-stories/select-provider-by-context
name: select-provider-by-context
skill_version: 0.2.2
som_schema_version: ["1.0.0"]

# --- what it is --------------------------------------------------------------
description: >-
  Declares which provider the publishing house's policy resolves for a declared
  capability, given a condition over the story context, and displays the
  resolution, the rule that produced it and the default that stood behind it.
  It declares and displays only. It never assigns work, never invokes a
  provider, never sends anything to anyone, never judges what the resolved
  provider produces, and never withholds anything. A provider acting on a
  resolution is that provider's own act, taken by its own executor under its
  own reading of the same policy.
skill_type: REFERENCE
category: workflow
lifecycle: draft

# --- who stands behind it ----------------------------------------------------
publisher: smart-stories
origin: reference
owner_editorial: <name or team alias>
owner_engineering: <name or team alias>
licence: <SPDX identifier>

# --- where it lives, so a directory can index it and fetch it ----------------
skill_uri: <gs:// or https:// canonical location, version-pinned>
skill_content_sha: <sha256:... of the skill body>
level_2_uri: null
level_3_uri: null

# --- how an executor finds it: the advert ------------------------------------
# This is the row the global lookup table is built from. Machine-read, never prose.
recall:
  target_system_type: [transcription, verification]
  target_capability: null
  conditions:
    - kind: field
      path: "{{ config.trigger_path }}"
      op: equals
      value: "{{ config.trigger_value }}"
    - kind: field
      path: "{{ config.watched_field }}"
      op: equals
      value: "{{ config.selection_map[].when }}"
  recall_on: ["story.context", "asset.ingested", "asset.updated"]
  state_path: null

# --- what the executor puts on the bus on this skill's behalf ----------------
output_messages: ["skill.warning.raised"]
severity_range: [inform]

# --- behaviour this skill guarantees -----------------------------------------
auto_change_content: false
fail_closed: true

# --- operational -------------------------------------------------------------
migration_policy: GATED
disclosure_level: L2
depends: []
supersedes: null
superseded_by: null
---

# select-provider-by-context

**When a declared capability is warranted on a story, declare which provider the house's
policy resolves for it, and display the resolution, the rule that matched and the default
behind it. Do not send anything to anyone. Do not run anything. The named provider acting
is that provider's own act.**

Read `../docs/CONVENTIONS.md` first.
**Changed in 0.2.1, 20 August 2026.** One path. The first reuse scenario in section 3 still described the configured condition as story.category with values `politics` and `sport`. There is no `category` on `story.context`; it is `tags[].value` carrying IPTC media topic codes, `11000000` and `15000000`, which is what the demo configuration has used since 18 August.

**Changed in 0.2.0, 18 August 2026.** Three things. `som_schema_version` moves to `["0.3.2"]`, which as of 18 August is the version. Field paths in the advert and in the worked configurations were checked against the v0.3.2 story-context JSON Schema for the first time and the ones that did not resolve are corrected; see the pack's `SCHEMA-CONFORMANCE.md` for the full list and the evidence. And the shared policy index was chosen over executor-local policy, which settles the registration question this file used to leave open: the house registers one advert row per configured instance.

---

## 1. What it is (layer 1)

| | |
|---|---|
| Operation | Declares which provider the house's policy resolves for one declared capability on one story, and displays the resolution, the matched rule and the standing default |
| Condition type | A field on the story context, mapped by configuration to provider identifiers |
| One mechanism? | Yes, and the reasoning matters here the way it matters in `apply-clearance`, because the shortest description of this skill, that it picks the vendor, reads as routing. It is not routing. What is declared is which provider the policy resolves under the tested condition; the acting, which is the resolved provider running its own pipeline and every unresolved provider doing nothing, is each provider executor's own act, read off the same policy. There is no component that sends work to a vendor, there is no field where a supplier could be written into a task, and if any explanation of this skill requires such a component, the explanation is wrong. On the "and" test: declaring the resolution and displaying the rule that produced it are one declaration, not two, because a resolution without its rule is not reviewable and not auditable, and the display is the content of the declaration. It does not also gate the resolved provider's output, which is a withholding and therefore belongs to the hold machinery, reached over the bus. It does not also decide whether the capability is warranted in the first place, which is its sibling `enrich-on-condition`'s declaration, likewise reached over the bus |

**The naming rule, applied to itself.** The name states the operation and the condition
type, never the editorial situation: `select-provider-by-context`, not
`use-vendor-a-for-politics`. A skill named for its scenario only ever does that scenario,
and that is a hardcode wearing a skill's clothes. The situation lives entirely in the
configured instance.

**Reuse test, three scenarios from different stories:**

1. **Two transcription services, one newsroom.** One service is stronger on podium audio
   and political vocabulary, the other on high-noise pitchside capture and squad lexicons.
   The configured condition is `tags[].value`, the map resolves the IPTC media topic
   `11000000` (politics) to the first and `15000000` (sport) to the second, and neither the producer asking for the transcript nor the workflow
   that carries the ask ever names a supplier. The provider name exists in the policy and
   in the audit trail, and nowhere else.
2. **Two synthetic media detectors on the verification desk.** One is visual-first, tuned
   for face and scene manipulation; the other is audio-first, tuned for cloned speech. The
   configured trigger is the state that already means a check is warranted on a loose clip,
   the watched field is the recorded media kind of that clip, and the map resolves
   audio-led material to the second detector and everything else to the first, with the
   first as default.
3. **One transcription capability, resolved on the language of the source, not the subject
   of the story.** A newsroom holds two transcription services: one demonstrably stronger on
   English-language audio, the other stronger on a second language the newsroom covers
   heavily. Incoming field audio carries a recorded source language. The configured watched
   field is that language tag, not the story category, and the map resolves each language to
   the service that transcribes it best, with the English service as default. The same skill,
   the same capability, a different watched field, and a selection nobody in the gallery has
   to remember: the story is not about language at all, but the audio is, and the policy reads
   the audio.

One skill, three subjects, three configured maps, and the third resolves on a different
watched field entirely. The mechanism knows nothing about politics, deepfakes or source
language: it knows a capability, a watched field, a map and a default.

---

## 2. Recall advert

The advert is declared in the frontmatter above so a tool can read it without parsing
prose. This section explains it in words for a human.

- **Who it is for:** any provider-side executor registered for a capability the house
  selects between: transcription services, and verification and detection tools. Each reads
  the same policy and the same resolution, and the one the resolution names acts while the
  others conformantly do nothing.
- **What causes a look:** a story context message, an asset arriving at ingest and an
  asset record changing. Those topic names are shared with a sibling advert in this
  library and carry the same status; see section 10.
- **What it depends on:** two field conditions, both supplied by the configured instance.
  The first is the trigger, the state that means the capability is warranted at all, for
  example an asset carrying media with no transcript yet recorded, or a loose clip
  recorded without an authenticity credential. The second is the watched field the map
  resolves over. This skill advertises against warranted state, never against the identity
  of whatever declared that state, per the chaining rule in `../docs/CONVENTIONS.md`: a
  house whose `enrich-on-condition` configured instance declares the warrant and whose
  configured instance of this skill resolves the provider has built the chain over the bus, and neither file
  names the other.

**Both condition paths are config-templated, and that has a consequence.** A
single-purpose skill advertises a literal dot path and the global lookup table indexes it
as one row. This skill advertises `{{ config.trigger_path }}` and
`{{ config.watched_field }}`, so the table cannot index it as a single row: either the
table resolves configuration before building its rows, or the house registers one advert
row per configured instance. The second is the working practice across this library, and
it is restated in section 10.

---

## 3. Firing anchor

- **Evidential position:** any. A provider resolution is about who does the work, not about
  the standing of the material the work is done on, and the trigger state in section 4 is
  where a house that only spends specialist providers on primary material narrows it.
- **Outlet / path:** the outlet whose configured instance resolves. Selection is an outlet-level
  judgement in the ordinary case, and a programme-level configured instance overrides the outlet map by
  ordinary specificity, read deterministically at configuration resolution rather than
  arbitrated at runtime.
- **Scope:** `story:<id>`. The resolution names who the policy resolves for a capability on
  this story, which is a fact about the story's work and not about any one destination, so
  `link:<id>` is not used here. A house wanting different providers per destination has
  two configured instances at two outlets, not one at link scope.
- **Compliance question, in one plain sentence:** "Who does the policy say does this work,
  which rule said so, and can the room see that answer without asking anyone?"

---

## 4. Config surface (layer 2)

Every editorial and commercial choice sits here, set by the publishing house. Nothing here
is in the skill. Scope runs broadcaster, then outlet, then programme, the more specific
overriding the default.

| Field | Required | Notes |
|---|---|---|
| `capability` | yes | The capability this configured instance resolves providers for, for example `transcription` or `synthetic-media-detection`. A name in the house's own vocabulary, because no governed registry of capability names exists; see section 10 |
| `trigger_path` / `trigger_value` | yes | The state that means the capability is warranted at all. This is read, never written, and never declared from here: whether work is warranted is a different declaration made elsewhere. A configured instance may point this at the state a warrant declaration materialised into, or directly at asset state |
| `watched_field` | yes | The story context field the map resolves over. `tags[].value` is one configured value of it, not the skill. `tags[]` is ordered and the first entry is primary, so a map matching more than one entry resolves on the earliest. Never point this at `tags[].label`, which is display only |
| `selection_map` | yes | Ordered rows of `when` value to `provider` identifier. First match resolves. This is where "the first service is better at politics" is written down: an editorial and commercial judgement, so it lives with the people who make it, as configuration, not code |
| `default_provider` | yes | Resolved when no row matches. A resolution off the default is still a resolution, declared as such, so the audit trail distinguishes a matched rule from a fallthrough |
| `providers` | yes | The registered provider identifiers this configured instance may resolve to, with the registration reference for each. A map row naming an unregistered provider is a configuration error surfaced at registration, not a runtime surprise |
| `instance_label` | yes | the editorial label for this configured instance |

**What is deliberately not on this surface: an approval gate.** Whether a human stands
between the resolved provider's output and the story is a withholding, and under the
one-mechanism rule a withholding cannot live in a skill that declares a resolution,
because the file would occupy two positions and resolve at no single coordinate. A house
wanting that gate configures the hold machinery against a review gate on the produced
output, reached over the bus, and a walkthrough surface presenting that as a single
boolean is presenting configuration sugar over that chain, not a field of this skill. See
section 10 for the gap that chain crosses.

The house also declares this skill in its own active set. Those values are copied straight
from the frontmatter above and are not a second set of choices: `skill_id`,
`skill_version`, `skill_type`, `disclosure_level`, `migration_policy`.

---

## 5. Runtime loop, specialised

on story.context snapshot, or asset.ingested / asset.updated carrying the configured fields:

1. **RESOLVE ANCHOR.** Read the story context and the outlet the executing runtime serves.
2. **GATE.** Does any configured instance at this anchor name a capability this runtime is
   registered to provide, or is this runtime a display surface for resolutions? If neither,
   exit quietly. Silence is a valid outcome and must not be an error.
3. **READ CONFIG.** Load the most specific configured instance in scope: programme over outlet over
   broadcaster, resolved deterministically. Verify every provider the map names appears in
   `providers`.
4. **EVALUATE.** Read the trigger. If the capability is not warranted, exit quietly. Read
   the watched field and resolve the map, first match wins, default on no match. This is a
   deterministic lookup, not a judgement: the judgement was made by the house when it wrote
   the map.
5. **DECIDE SEVERITY.** Always `inform`. Nothing here can hold.
6. **EMIT.** Declare the resolution: capability, resolved provider, matched rule or
   fallthrough, standing default, configured instance and version. The twelve-field warning, in
   the envelope.
7. **ACT, WHICH IS NOT THIS SKILL.** If the executing runtime is the resolved provider, its
   own act follows: run its own pipeline, publish its output as its own proposal with its
   own provenance, publish its own run report. If it is not the resolved provider, its act
   is to do nothing, and doing nothing is conformant. Neither act is done on this skill's
   behalf.

**Fail closed, read for a skill with no hold authority.** An unreadable watched field means
the default is resolved and the declaration states the gap loudly in `detail`: the room
learns the resolution was a fallthrough on missing state, not a match. An unreadable
trigger means the capability cannot be shown warranted, so nothing is declared and the
executor's own run report records the failure. An unresolvable default, or a map naming an
unregistered provider, means no resolution is declared and the gap is stated: a guessed
provider at transmission is worse than a visible absence. Nothing is ever silently passed
and no declaration is ever a guess.

---

## 6. Output contract

`skill.warning.raised`, severity `inform`.

```json
{
  "topic": "som.skill.warning.raised",
  "originating_system": { "system_id": "<tool>", "system_type": "<the tool's own type, one this skill's advert targets>" },
  "correlation_id": "<the story's chain>",
  "causation_id": "<the snapshot that triggered evaluation>",
  "timestamp": "<on the envelope, never in the payload>",
  "payload": {
    "warning_id": "wrn-3c91aa",
    "skill_id": "smart-stories/select-provider-by-context",
    "skill_version": "0.2.2",
    "story_id": "story-chancellor-0810",
    "scope": "story:story-chancellor-0810",
    "severity": "inform",
    "rule_id": "nr-transcription-politics",
    "non_overridable": false,
    "affected_fields": [],
    "detail": "Provider resolution for capability transcription: provider-a, matched on tags[].value = 11000000 (Politics) under configured instance nr-transcription-provider v3, default provider-a standing unused. The resolution is a declaration of what the policy names. It assigns nothing: the named provider acts under its own executor, and every other registered provider conformantly does not.",
    "blocks": [],
    "skill_warning_ref": null
  }
}
```

`blocks` is empty because nothing is withheld from here, and `non_overridable` is false for
the same reason. The resolved provider's transcript or verdict is published by
that provider's own executor as its own message with its own provenance, and its run report
is likewise its own: neither travels on this skill's behalf, and the resolution above is
what lets an auditor line all three up afterwards without inference.

---

## 7. Priority position

| Axis | This skill |
|---|---|
| X category | `workflow`, so X = 3 |
| Y type / source | `reference`, so Y = 5 |
| Z precedence | unset. The publishing house sets it, nobody else |

At (3, 5) this contests nothing in `compliance` or `editorial`, which resolve first, and a
resolution is not a gate: where a hold binds the same asset or the same story, what is
served is governed by conjunction, every binding gate must permit, and this declaration
contributes no gate to that conjunction. Two configured instances of this skill matching the same
story at different scopes are not a priority contest either: the more specific configured instance
resolves at configuration read, deterministically, before priority is ever consulted, which
is inheritance and not arbitration.

---

## 8. Worked configurations

**8a. Two transcription services, outlet scope.**

```yaml
instance_label: nr-transcription-provider
capability: transcription
trigger_path: assets[].acquisition_state
trigger_value: CAPTURED
watched_field: tags[].value
selection_map:
  - when: "11000000"   # IPTC Media Topic: Politics
    provider: provider-a
  - when: "15000000"   # IPTC Media Topic: Sport
    provider: provider-b
default_provider: provider-a
providers: [provider-a, provider-b]
```

A chancellor's press conference resolves `provider-a` on the politics row; a
cup final flash interview resolves `provider-b` on the sport row; a story carrying neither
category resolves the default and the declaration says so. Editing one map row and
replaying the same snapshot yields a different resolution with the story, the skill and
every executor's code unchanged, which is the whole point of the exercise.

**8b. Two synthetic media detectors, verification desk.**

```yaml
instance_label: nr-synthetic-check-provider
capability: synthetic-media-detection
trigger_path: assets[].authenticity_credential.present
trigger_value: false
watched_field: assets[].asset_type
selection_map:
  - when: AUDIO
    provider: detector-y
default_provider: detector-x
providers: [detector-x, detector-y]
```

The trigger is the state that already means a check is warranted, a loose clip recorded
without an authenticity credential, mirroring the derived trigger convention already in
vendor use. The two configurations differ in tool type, in trigger family and in watched field, and share
only the mechanism: a capability, a map, a default, a declaration. The audio clip of a
cloned voice reaches the audio-first detector because the policy says so, in writing, with
the rule named in the declaration, and not because anyone in the newsroom knew which
detector to pick.

---

## 9. Eval set

| Case | Given | Expect |
|---|---|---|
| declaring | Politics snapshot, transcript warranted, configuration 8a in scope | One `inform` warning declaring `provider-a`, rule `nr-transcription-politics`, default named as standing |
| non declaring | Snapshot on a story where no configured instance in scope names a warranted capability | No declaration, quiet exit, no error recorded |
| clearance | A valid clearance assertion lands for an unrelated gate on the same story | No effect on the standing resolution. Resolutions are not cleared; they are recomputed when policy or context changes, and a policy version change re-declares with the previous resolution recorded, not erased |
| wrong clearance | An assertion attempts to name a different provider for the same task without any policy change | Not honoured and not declared against. The policy is the only source of a resolution; the authority comparison that governs clearances elsewhere is a working position and nothing in this skill reads it |
| idempotency | The same snapshot delivered twice | One declaration, keyed on `correlation_id` plus scope plus `rule_id` |
| fail closed | Watched field unreadable; separately, default unresolvable | Default resolved with the gap stated loudly in `detail`; no resolution declared and the gap stated, with the executor's own run report carrying the failure |
| never auto-change | Any declaring run | No story state written, no content changed, no provider invoked from here. The resolved provider's output arrives later as that provider's own proposal |
| quiet non-selection | A registered provider's executor reads a resolution naming a different provider | It does nothing, publishes nothing, and that is conformant behaviour, not an error state |
| map edit replay | Configuration 8a with the politics row edited to `provider-b`, same snapshot replayed | A different resolution, same story, same skill, unchanged executor code on every side |

These are specifications an executor is expected to satisfy. Nothing has run them and no
results exist; they are the contract, not evidence of conformance.

---

## 10. Open items

- **"Select" reads as acting, and it is not acting. This is the scoping question this skill
  turns on, so it is stated here as well as in section 1.** Under the passive convention of
  29 July 2026, what this file declares is which provider the policy resolves; the acting
  is each provider executor's own, including the act of doing nothing when unnamed. There
  is no component that sends work to a vendor anywhere in this design, and an
  implementation that adds one has built the thing this library exists to make unnecessary.
- **Capability names are ungoverned, and this is the file where that bites hardest.** No
  registry of capability names exists, so `capability` is a house-vocabulary string, two
  houses may name the same capability differently, and a provider registering across houses
  reconciles the spellings itself. The frontmatter's `target_capability` is the field that
  would carry this properly, and it is `null` across this library precisely because its
  status is unclear. If the group governs capability naming, `capability` here should read
  from that registry and this item closes.
- **The approval gate is deliberately absent, and the chain that delivers it crosses a named
  gap.** A house gating the resolved provider's output configures the hold machinery
  against a review gate on that output. What materialises the produced output's pending
  state into the `editorial_gates[]` entry a hold reads is the same carrier gap named by
  the raise and hold files: nothing in the model says what performs that write, and this
  file does not pretend the chain completes itself. A walkthrough surface presenting the
  gate as one boolean on a config surface is faithful as sugar and misleading as architecture if
  read without this note.
- **How clearance authorities are compared is not defined anywhere.** It touches this file
  only at the wrong-clearance row, and the treatment adopted is this library's working
  position, not a rule any group has ratified: no assertion outside the policy resolves a
  provider, whatever its authority.
- **Whether `target_system_type` is binding or advisory is open.** The advert is written to
  the binding reading, the stricter one. Under it, the two listed types are the population
  that can be recalled at all. Both `transcription` and `verification` are in circulation
  in sibling adverts; the advert names only spellings already in use, so no unverified enum
  member is advertised here.
- **The emitting executor carries the tool's own `system_type`, and the `skill_worker` type
  this library used to emit here is withdrawn.** An executor is a sidecar to a vendor tool,
  and no newsroom-scoped class of skill-running system exists, so the thing that puts this
  resolution on the bus is a transcription service's or a verification tool's own executor,
  and section 6 now carries that tool's own type. The sample payload previously carried a
  `skill_worker` system type, a value whose only reason to exist was that the emitter ran
  skills, which is the newsroom-executor shape under another name; it is withdrawn rather
  than reinterpreted, and it is named in full here because a vendor whose own sample carries
  that value will search for exactly that string. What is not settled is whether the emitting
  tool's own type is the right value for `originating_system.system_type` on a skill warning
  at all, as against some value naming the executor role that raised it. The schema does not
  answer that, and the position taken here follows from a decision taken outside the schema,
  that tools have executors and newsrooms do not, which the schema has not caught up with. On
  the applied position this is one of the two places this file names a system type, the other
  being the advert's `target_system_type`, and the two agree by construction, because the tool
  that may recall this skill is the tool that emits on its behalf. A counter-reading is live
  and is not resolved: it holds `skill_worker` to be a capacity marker rather than a
  component, what an executor reports when it emits in its skill-evaluating capacity rather
  than in the tool's own right, with `system_id` preserving which tool's executor evaluated
  the skill and the type deliberately never targetable, so adverts target native types, here
  `transcription` and `verification`, and the emission type differs from the advert type by
  design. Under that reading the two lists do not agree by construction, which is the opposite
  of what this item and the other nine files say. This file follows the applied position for
  consistency across the library rather than because the counter-reading has been answered,
  and the counter-reading remains this file's author's to settle. One question neither reading
  answers: whether a vendor's own `skill.run.completed` carries the same value in its
  `originating_system`. This file declares nothing about that message and takes no position on
  it.
- **Both condition paths are config-templated**, so the lookup table cannot index this
  skill as a single row and the house registers one advert row per configured instance,
  per the working practice restated in the README.
- **`recall_on` topic names are working names.** `story.context` has a worked precedent;
  `asset.ingested` and `asset.updated` are shared with a sibling advert in this library and
  carry the same unratified status, and the two adverts should move together if the topic
  vocabulary settles differently.
- **The publish topic in section 6 follows this library's convention and is being
  reconciled** against the worked precedent in the locked specification's examples. If the
  reconciliation lands the other way, one string in the contract changes and nothing else.
- **What `fail_closed: true` means for a skill with no hold authority** is stated in
  section 5 rather than left to be inferred: the most restrictive output this skill
  possesses is a loud `inform` naming the gap, never a guessed resolution and never a
  silent pass. A hold appearing is not a possible reading here.
- **`state_path` is `null` by reading rather than by statement.** A resolution is
  recomputed per snapshot and nothing standing is tracked between messages, so no state
  dependency exists to name. Whether `null` is legal by statement is unsettled; this file
  relies on the reading.
- **`disclosure_level: L2` means the deepest layer for which this file declares content**,
  frontmatter and body through the config surface, with vendor build notes as the third
  layer. Two definitions of the field are in circulation; the other was not intended here.
- **`skill_uri` is left as a placeholder** because it is not this author's to supply and
  whether it is required at all is unsettled. `owner_editorial`, `owner_engineering`,
  `licence` and `skill_content_sha` are left for the same reason, the last computed at
  build rather than typed.
- **The `skill_id` format is one of four in circulation.** `smart-stories/select-provider-by-context`
  uses the form with a worked precedent in the warning schema's own example, not because
  the question is settled. If the group picks another, this changes.
- **`depends` is empty on purpose.** The warrant this skill's trigger reads may be declared
  by a sibling skill, by ingest state, or by anything else the house configures, and the
  chain is built over the bus against state, never against a named skill, per the chaining
  rule. Naming a sibling here would invert the dependency and break at the first change.

---

## 11. Vendor build notes (layer 3)

Written in each platform's own terms. Each row names the instinct this skill exists to
interrupt.

| Platform | Build note |
|---|---|
| Transcription | Your executor reads the resolution and the policy behind it. If it names you, run your own pipeline, publish the transcript as your own proposal with your own provenance, and publish your own run report naming the matched rule and whether you were the default. If it does not name you, do nothing and report nothing, and treat that as success. The instinct interrupted: waiting to be called. Nothing calls you, and building a listener for an assignment message means building against a message that does not exist |
| Verification | The two-detector configuration resolves you per clip, not per account, so being demonstrably best at cloned speech is enough to own that branch without displacing the visual-first incumbent. A cheap-then-expensive handoff inside your own pipeline is your territory and invisible to the spec. The instinct interrupted: bidding for the whole desk. The map buys branches, and your claim data is what the house reviews the map against at contract time |
