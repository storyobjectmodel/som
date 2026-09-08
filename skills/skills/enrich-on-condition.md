---
# --- identity ----------------------------------------------------------------
skill_id: smart-stories/enrich-on-condition
name: enrich-on-condition
skill_version: 0.2.2
som_schema_version: ["1.0.0"]

# --- what it is --------------------------------------------------------------
description: >-
  Declares that a named enrichment is warranted for an asset whenever a
  configured condition holds of that asset, and displays which enrichment and
  why. The condition and the enrichment are both set by the publishing house:
  provenance recorded as unverified is one configured instance, not the skill.
  It declares and displays only. It does not run the enrichment, which is the
  executor's work. It does not judge the result the enrichment publishes back,
  and it does not withhold anything pending that result. It never holds, never
  blocks and never changes content.
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
  target_system_type: [mam, verification, multimodal, transcription, ingest]
  target_capability: null
  conditions:
    - kind: field
      path: "{{ config.condition_path }}"
      op: equals
      value: "{{ config.condition_value }}"
  recall_on: ["asset.ingested", "asset.updated", "story.context"]
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

# enrich-on-condition

**When a configured condition holds of an asset, declare that a named enrichment is
warranted for it and display which one and why. Do not run it, do not judge what it
finds, and do not withhold anything while it is pending.**

Read `../docs/CONVENTIONS.md` first.
**Changed in 0.2.1, 20 August 2026.** Two things. Sections 2 and 10 still printed the registration question as open after proposal A closed it, and now say what A decided and what it obliges. And three eval-set rows tested asset.provenance.state and asset.enrichments.synthetic_media_analysis, neither of which exists; they now test `assets[].authenticity_credential.present` and `assertions[]`, which is what the worked configurations in section 8 have always configured.

**Changed in 0.2.0, 18 August 2026.** Three things. `som_schema_version` moves to `["0.3.2"]`, which as of 18 August is the version. Field paths in the advert and in the worked configurations were checked against the v0.3.2 story-context JSON Schema for the first time and the ones that did not resolve are corrected; see the pack's `SCHEMA-CONFORMANCE.md` for the full list and the evidence. And the shared policy index was chosen over executor-local policy, which settles the registration question this file used to leave open: the house registers one advert row per configured instance.

---

## 1. What it is (layer 1)

| | |
|---|---|
| Operation | Declares one named enrichment warranted for one asset, and displays the reason |
| Condition type | A field condition, whose path and value are supplied by configuration |
| One mechanism? | Yes. The declaration and the display of the reason are one act, not two: the reason is the content of the declaration, and a declaration without it is not actionable by the executor that has to choose an enrichment. Nothing else is attached. It does not also judge what the enrichment returns, and it does not also hold the asset until the enrichment lands. Both of those are separate skills reading the state this one's enrichment wrote |

**The name says "run", and this skill does not run anything.** Vendors are building to the
name, so the name stays. What the skill does under the passive convention agreed 29 July
2026 is narrower and honest: it DECLARES that an enrichment is warranted for this asset
under this condition, and DISPLAYS which enrichment and why. It is easy to read the name as
a promise that the skill runs an enrichment whenever a condition holds, and that reading is
wrong: the executor, a vendor runtime, is what actually runs the analysis, and it publishes
what it found back onto the bus as its own enrichment with its own provenance. Recorded
again in section 10, because a reader who meets the name before the body will assume
otherwise.

**The "and" test, explicitly:** the sentence is "declare that an enrichment is warranted".
There is no "and". It does not also judge the enrichment's result: a result that comes back
saying a clip is probably synthetic is a state some other skill advertises against. It does
not also hold anything pending the result: nothing is withheld here, which is why
`severity_range` is `[inform]` alone.

**Reuse test, three scenarios from different stories:**

1. **The hurricane.** Two clips arrive for the same landfall story. The agency clip
   carries a C2PA chain and is recorded `TRUSTED`. The citizen clip arrives loose with no
   chain and is recorded `UNVERIFIED`, its absence recorded rather than hidden. The
   configured condition is provenance recorded as unverified, and the enrichment declared
   warranted is synthetic-media analysis, on that clip only.
2. **An overnight partner feed on a foreign-ministry statement.** An incoming satellite feed
   from a partner broadcaster is ingested with its source language unrecorded. The
   configured condition is the recorded language state, and the enrichment declared
   warranted is automated language identification, so that a transcription pass is not
   attempted against a guess.
3. **A weekly agricultural commodities report, a kind of story this library has not previously
   drawn.** A supplied price chart arrives from a data provider as a flat image with no
   recorded units or currency on the axes. The configured condition is the missing
   axis-label state, and the enrichment declared warranted is a chart-extraction pass that
   reads the axes before the graphic is cleared for reuse on air.

One skill, three subjects, three different enrichments, no branching inside the skill. The
subject lives entirely in the house's configuration.

---

## 2. Recall advert

The advert is declared in the frontmatter above so a tool can read it without parsing prose.
This section explains it in words for a human.

- **Who it is for:** tools that can either read the state the condition tests or perform the
  enrichment that is declared warranted. Typically that is the MAM holding the asset
  record, the ingest tool that first publishes the fields, and the verification, multimodal
  and transcription tools that carry the enrichments themselves. Targeting names broad
  system types, not named products.
- **What wakes it:** an asset arriving at ingest, an asset record changing, and a story
  context message. Those are the three moments at which the tested field can newly take, or
  newly fail to take, the configured value.
- **What it depends on:** exactly one field condition, whose path and value are both
  supplied by the configured instance. `provenance == unverified` is one configured instance.
  It is not the skill.

**The condition path is config-templated, and that has a consequence.** A single-purpose
skill advertises a literal dot path and the global lookup table indexes it as one row. This
skill advertises `{{ config.condition_path }}`, so the table cannot index it as a single
row. Proposal A settled that on 18 August: the house registers one advert row per
configured instance, in its own registration layer. Section 10 records what that costs.
It is a good design with a named cost, not a mistake.

**The operator is fixed at `equals` while the path and value are not.** The advert's `op`
must be a literal from the schema enum, so a house that wants to enrich on the absence of
something configures the recorded state field that carries the absence, for example a
provenance state recorded as `unverified`, rather than pointing the condition at the missing
field itself. This suits the model well, because the absence being recorded rather than
hidden is the point of the unverified-provenance configured instance, but it is a real limit
on what can be configured. Noted again in section 10.

**`depends` is empty, and it stays empty.** Chaining happens through the bus. If the result
of the enrichment should later hold something, that arrives as a second skill advertising
against the state this skill's enrichment wrote, recalled by a different executor on a later
message. Declaring a dependency on that consumer would invert the dependency and create a
cycle at the first change. This skill depends on what it reads and never on what reads it.

**`state_path` is null,** because this skill never holds, so there is no open hold for a
later message to decide. Idempotency is decided against the enrichment record on the asset,
described in section 5, not through `state_path`.

---

## 3. Firing anchor

- **Evidential position:** any, by default. An enrichment can be worth doing on supporting
  material as readily as on the clip at the top of the story, and in the hurricane configured
  instance both clips sit at `PRIMARY` anyway. A house that wants to spend its compute only
  on primary evidence narrows this through `applies_to_position` in section 4.
- **Outlet / path:** the asset's own record within the story in the ordinary case. Where a
  configured instance sits at link scope, the declaration attaches to the destination path
  whose requirement warranted the enrichment. In neither case does it attach to the content
  of the telling.
- **Scope:** `story:<id>` where the configured condition is read from asset or story state.
  That is the ordinary case and it covers both worked configurations in section 8.
  `link:<id>` where the configured condition is read from a destination's own requirement
  state, because an enrichment warranted by one outlet's requirement is not warranted for
  every path the story takes.
  The dual scope is kept rather than simplified away, and the reason is a configuration a
  reader can check. A house serving the same package to a broadcast rundown and to an
  on-demand platform whose accessibility obligation demands captions configures
  `condition_path` at the on-demand link's caption state, `condition_value` at the state
  that records captions as not yet present, and `enrichment` at automated caption
  generation. The rundown path carries no such obligation. Declared at story scope, that
  configured instance would spend a caption pass on every telling in order to satisfy one of
  them, and would close for a path that never needed it. Link scope is doing real work there,
  which is why it stays.
  The dual scope costs nothing positionally: `CONVENTIONS.md` records that scope is not X,
  so a configurable scope does not make a skill unpositionable, and this skill's X stays
  `workflow` whichever scope a configured instance is declared at.
- **Compliance question, in one plain sentence:** "Is there something we should have found out
  about this asset before it went any further, given what we already know about it?"

---

## 4. Config surface (layer 2)

Every editorial choice sits here, set by the publishing house. Nothing here is in the skill.
Scope runs broadcaster, then site, then brand, the more specific overriding the default.

| Field | Required | Notes |
|---|---|---|
| `instance_label` | yes | The editorial label for this configured instance, carried into `rule_id`. For example `hurricane-unverified-synthetic-check` |
| `condition_path` | yes | The dot path the condition tests. For example `assets[].authenticity_credential.present`. This is the value that is templated into the advert |
| `condition_value` | yes | The value that warrants the enrichment. For example `false` |
| `enrichment` | yes | The name of the enrichment declared warranted. For example `synthetic-media-analysis`. A vendor executor maps this name onto its own capability |
| `enrichment_record_path` | yes | Where a completed or in-flight record of this enrichment is written on the asset. Read by the gate, so the same enrichment is not declared warranted twice |
| `clearing_authority` | yes | The authority and scope that may clear a declaration from this configured instance. See section 10: how authority is compared is not defined |
| `applies_to_position` | no | Which evidential positions are in scope. Defaults to all |
| `reason_template` | no | House wording for `detail`. Defaults to the condition path, the value read and the enrichment name |

The house also declares this skill in its own active set. Those values are copied straight from
the frontmatter above and are not a second set of choices: `skill_id`, `skill_version`,
`skill_type`, `disclosure_level`, `migration_policy`.

---

## 5. Runtime loop, specialised

```
on asset.ingested | asset.updated | story.context:
  1. RESOLVE ANCHOR   the asset referenced by the message, and the story it sits in
  2. GATE             is an enrichment named `enrichment` already recorded at
                      `enrichment_record_path` for this asset, completed or in flight,
                      under this `instance_label`? If so, exit quietly. This is the
                      cheap read that keeps a busy ingest from paying twice
  3. READ CONFIG      load the configured instance for this
                      broadcaster / site / brand
  4. EVALUATE         read `condition_path` on the asset. The condition holds when the
                      value equals `condition_value`, or when the value cannot be read
                      at all (see fail closed, below)
  5. DECIDE SEVERITY  `inform`, always. This skill has no other severity. Nothing is
                      withheld, and nothing is pending on it
  6. EMIT             the twelve-field warning, causation_id on the triggering snapshot,
                      naming the enrichment and the value that warranted it
  7. AWAIT CLEARANCE  the enrichment being recorded at `enrichment_record_path`, or an
                      assertion at or above `config.clearing_authority` on the same scope
                      that the enrichment is not warranted for this asset, carrying its
                      own provenance of who, when, scope and status
  8. RESOLVE / CLOSE  the declaration closes when either of those is recorded. It does
                      not close on what the enrichment found, because this skill does
                      not read that
```

**Skill specific traps:**

- The mistake a well-meaning vendor will make is to treat the declaration as the enrichment
  and write the analysis result back into the field the condition read. The tested field is
  the input and must be left alone. The result belongs at `enrichment_record_path` as a
  separate record with its own provenance, or the next evaluation reads its own output and
  the condition oscillates.
- The second mistake is to run the enrichment and stay silent about it. The declaration is
  what makes the spend visible and auditable, and the whole claim of the unverified-provenance
  configured instance is that the absence was recorded rather than hidden.
- What must go in `detail` for the message to be actionable rather than annoying: the path
  that was read, the value that was found (or the fact that it could not be read), the name
  of the enrichment declared warranted, and what closes the declaration. A message saying
  only "enrichment warranted" makes the newsroom go looking.
- Fail closed: where `condition_path` is missing, unreadable or malformed, or the configured
  instance itself cannot be loaded, the condition is treated as holding. Concretely, if the
  provenance state of a clip cannot be read, the clip is treated as warranting the
  synthetic-media analysis rather than as safe to skip. That produces `inform`, which is the
  loudest severity this skill actually has, with `detail` saying which value could not be
  read. Fail closed here does not mean a hold, because this skill has no hold authority.
- What does NOT clear it: the enrichment coming back with a reassuring result does not clear
  anything, because there is nothing here to clear once the enrichment is recorded, and a
  worrying result is a separate skill's business. A producer clicking past the display does
  not clear it either. An assertion from below `config.clearing_authority`, or on a
  different scope, does not clear it. Re-ingesting the same asset does not clear it: the
  gate at step 2 sees the existing record and exits quietly.

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
    "warning_id": "wrn-....",
    "skill_id": "smart-stories/enrich-on-condition",
    "skill_version": "0.2.2",
    "story_id": "story-...",
    "scope": "story:...",
    "severity": "inform",
    "rule_id": "hurricane-unverified-synthetic-check",
    "non_overridable": false,
    "affected_fields": ["assets[0].authenticity_credential.present"],
    "detail": "assets[0].authenticity_credential.present reads false, so synthetic-media-analysis is warranted for this asset. Closes when a DETECTION assertion is recorded in assertions[], or when an assertion of equal or higher authority on this scope records that it is not warranted.",
    "blocks": [],
    "skill_warning_ref": null
  }
}
```

`blocks` is empty and `non_overridable` is false, permanently for this skill. Both are
required to be so: nothing is withheld, so there is nothing to list, and an advisory display
that could not be overridden would be a hold wearing a quieter word.

---

## 7. Priority position

| Axis | This skill |
|---|---|
| X category | `workflow` (3). When an enrichment happens in the ingest-to-output process is an operational question, not a regulatory one. The result of the enrichment may well raise a compliance question later, but that is a different skill at a different coordinate |
| Y type / source | `reference` (5), from `origin`. A shared reference set, adopted by choice |
| Z precedence | unset. The publishing house sets it, nobody else |

This skill contributes no binding gate, so it neither outranks nor is outranked in any
enforcement question: it withholds nothing and there is nothing for it to contest. Where
gates do combine, every binding gate must permit, so any one hold means held, and this skill
adds nothing to that combination. A compliance skill holding the same asset stands whether or
not an enrichment has been declared warranted for it, and the declaration stands whether or
not the asset is held.

---

## 8. Worked configurations

### 8a. Synthetic-media analysis on a clip whose provenance is unverified

```yaml
instance_label: hurricane-unverified-synthetic-check
condition_path: assets[].authenticity_credential.present
condition_value: false
enrichment: synthetic-media-analysis
enrichment_record_path: assertions[]
applies_to_position: [PRIMARY]
clearing_authority: producer-on-story
```

What this produces: two clips on the same story, handled honestly. Against the agency clip,
attributed and carrying its C2PA chain, nothing appears. Against the citizen clip, which
arrived loose, a line says that its provenance reads unverified and that synthetic-media
analysis is warranted for it. The verification tool then runs that analysis and records both
what it found and that no provenance chain is present, as its own enrichment.

### 8b. An overnight partner feed with no recorded source language

```yaml
instance_label: intake-unlabelled-language-id
condition_path: assets[].acquisition_state
condition_value: CAPTURED
enrichment: automated-language-identification
enrichment_record_path: assertions[]
applies_to_position: [PRIMARY, SECONDARY]
clearing_authority: intake-editor
```

What this produces: an inbound satellite feed of a foreign-ministry statement lands
overnight with its source language unrecorded, and a line against it says that automated
language identification is warranted before anything is transcribed against a guess.

What changed between 8a and 8b: the condition path, the value that warrants the enrichment,
the enrichment itself, the tool that will perform it, the breadth of evidential positions
in scope, and the authority whose assertion may close the declaration. What stayed the same: the skill, its advert, its one mechanism, its category, its
single severity of `inform`, the gate that keeps it from declaring the same enrichment twice,
and the division of labour in which the skill declares and displays while a vendor executor
runs the work and publishes back what it found.

---

## 9. Eval set

Minimum five. The skill is not finished until a declaring case and a clearance case both pass.

| Case | Input | Expected |
|---|---|---|
| declaring | An asset arrives with `assets[].authenticity_credential.present` reading `false`, no record at `assertions[]` for this enrichment, configured instance `hurricane-unverified-synthetic-check` loaded | One `skill.warning.raised` at `inform`, `rule_id` the `instance_label`, `affected_fields` naming the tested path, `detail` carrying both the value read (`unverified`) and the enrichment name (`synthetic-media-analysis`), `blocks` empty, `non_overridable` false |
| non declaring | The sibling agency clip on the same story, `assets[].authenticity_credential.present` reading `true`, same configured instance loaded | Nothing at all on the bus. No message, no empty declaration, no "checked and fine" record |
| clearance | The producer who owns the asset in this story, who is at the configured instance's `clearing_authority` of `producer-on-story`, records on the same scope that the enrichment is not warranted, for example because the C2PA chain was supplied late and the provenance state was corrected to `trusted`; or the verification tool records a completed result at `enrichment_record_path` | The declaration closes. The assertion itself, with its provenance of who, when, scope and status, stays visible in the compliance array. Nothing is released by the closure, because nothing was withheld |
| wrong clearance | An automated housekeeping tool that does not appear on the house's authority scale at all, or an operator ranked below the configured instance's `clearing_authority`, or an assertion made at `link` scope against a `story`-scope configured instance, asserts that the enrichment is not warranted | Not applied. The declaration stays open, and the attempt stays visible with its own provenance. The assumed rule is that a declaration closes only by an assertion at or above `config.clearing_authority` on the same scope; how authority is compared is undefined, so this case encodes the assumption rather than a settled rule (section 10) |
| idempotency | The same ingest snapshot for the citizen clip is delivered twice, once in each cloud, and a third time as a replayed `asset.updated` carrying no change to the tested field | Exactly one declaration and exactly one enrichment run. Step 2 reads `enrichment_record_path` and finds the in-flight or completed record written by the first pass, and exits quietly on the second and third. The executor additionally derives its dedupe key from (`story_id`, asset identifier, `skill_id`, `instance_label`, `enrichment`) so that two clouds racing before either record is written still resolve to one declaration. This case matters more here than elsewhere: a duplicate declaration is a duplicate synthetic-media analysis, and that is money and compute spent twice on one clip |
| fail closed | `assets[].authenticity_credential.present` is absent, malformed, or the configured instance cannot be loaded | `inform`, the loudest severity this skill has, with `detail` saying which value could not be read and that the enrichment is warranted on that basis. The unreadable state is treated as the condition holding, so the enrichment is declared warranted rather than skipped. Not a hold: this skill has no hold authority |
| never auto-change | Any declaring case above, replayed with a full field-level diff of the asset record before and after | No message produced by this skill changed any content. The tested field is untouched, `assets[].authenticity_credential.present` still reads `unverified`, and the only new state is written by the executor's own enrichment record under its own system identity |

---

## 10. Open items

- **The name says "run" and the skill does not run, and the name also names no condition
  type.** Two faults in one name, recorded together because one decision covers both. The
  name predates the passive convention agreed 29 July 2026, and it reads as though the
  skill runs an enrichment itself. The resolution assumed here is that the skill declares
  that an enrichment is warranted for this asset under this condition and displays which
  one and why, and that the vendor executor runs the analysis and publishes the result back
  as its own enrichment. Separately, `<operation>-<condition-type>` wants a condition type
  in the second half, and "condition" is not one: it is a placeholder that every field
  condition in the library satisfies equally, so the name does not narrow anything. It
  passes the primary test, in that it names a mechanism rather than a news situation, which
  is why it is not a blocking fault. The name is kept unchanged because the library name is
  the contract vendors are building to, so a silent rename would break it, and this is the
  same treatment `match-and-propose` and `apply-clearance` get in `CONVENTIONS.md`. If the
  group would rather the name matched the behaviour on either count, the conformant form is
  closer to `declare-enrichment-on-field-state`, and that is a library-wide decision, not
  this file's.
- **The condition path is config-templated, and the lookup table cannot index it as a single
  row.** Proposal A, chosen 18 August, settles it: the house
  registers one advert row per configured instance, in its own registration layer, which is
  what this file always assumed. Nothing in this file changes. What A obliges is that the
  registration layer exists and stays honest, because a row that names no instance cannot be
  checked, and provable conformance was the reason A was chosen over executor-local policy.
- **The advert's operator is fixed at `equals`.** `op` must be a literal from the schema enum,
  so a condition on the absence of something must be configured against a recorded state value
  such as `unverified` or `unrecorded`, not against the missing field itself. That fits this
  library, where recording the absence is the point, but it is a real limit and a house with
  an unrecorded absence cannot configure this skill against it.
- **Fail closed does not mean a hold here.** The template's illustration of fail closed is a
  hold that says why, and this skill has no hold authority, so it cannot do that. The reading
  applied here is that fail closed at the skill layer means treating an unreadable value as
  the condition holding, which produces the loudest severity this skill actually has,
  `inform`. Safe state at the executor layer is the separate mechanism that covers the case
  where nothing can be resolved at all. Worth naming the cost: fail closed here spends money
  rather than withholding anything, because an unreadable provenance state results in an
  enrichment being run that might not have been needed. That is the deliberate trade, and a
  house that cannot afford it should be changing its ingest so the field is always
  published, not weakening the rule.
- **Clearance semantics are thinner for a non-holding skill than the template assumes.** The
  template's clearance and wrong-clearance cases are written for a skill that withholds
  something, where clearing releases it. Nothing is withheld here, so closing a declaration
  releases nothing and only stops the display. The cases are written honestly on that basis,
  and the more natural closure is the enrichment simply being recorded. Alongside that, the
  rule adopted throughout this library is that a declaration closes only on an assertion of
  equal or higher authority on the same scope, with `clearing_authority` in section 4 naming
  that level per configured instance so an executor has something concrete to compare an
  assertion against. That rule is this library's working position, not a rule any group has
  ratified, and no vendor should read it as settled or hard-code enforcement against it. How
  two authorities are compared is not defined anywhere either, so `config.clearing_authority`
  is a house-local string and the executor's comparison of it is house-local too. The
  wrong-clearance eval row is written against the adopted rule, and if the group settles the
  comparison differently that row is rewritten and `clearing_authority` is read against the
  group's rule rather than the house's. Both gaps are live.
- **`depends` is empty by design.** A reader expecting a dependency on whatever consumes the
  enrichment result will not find one. Declaring it would invert the dependency and create a
  cycle at the first change. A hold on what the enrichment found arrives as a second skill
  advertising against the state this skill's enrichment wrote, chained over the bus, with no
  integration between the two tools.
- **`disclosure_level: L2` means the deepest level for which this skill declares content**,
  which is section 4's config surface plus the vendor build notes at layer 3 being pointers
  rather than a body. Two definitions of the field are in circulation, the other being how much
  of its reasoning a skill reveals, and neither of the 29 July documents mentions the field.
  This file means the first.
- **Targeting is written as though it binds.** `target_system_type` is listed as a base an
  executor must match against its own type. If targeting turns out to be advisory, this skill
  behaves differently in one specific way: an off-target compliance hub or audit tool would
  pick up the declaration and display it, which is harmless here and arguably desirable, since
  a record of what enrichment was declared warranted and why is exactly what an audit wants.
  Whether the table names broad types or precisely named products is also unsettled; broad
  types are used here.
- **The system type and enrichment names are not schema-controlled vocabulary.** The values in
  `target_system_type` are read as broad types from the v0.3 `system_type` enum, and any name
  in that list not present in the enum is an error in this file to be corrected against the
  enum rather than a proposed extension. `enrichment` in the config surface is a house string
  that each vendor executor maps onto its own capability; there is no agreed registry of
  enrichment names, and two houses may well use different words for the same analysis.
- **The system that emits on this skill's behalf carries the tool's own system type, and
  `skill_worker` is withdrawn.** An executor is a sidecar to a vendor tool, and there is no
  newsroom-scoped class of skill-running system for one to belong to, so the executor that puts
  this declaration on the bus is the MAM, verification, multimodal, transcription or ingest
  tool's own executor, and it names that tool's own type in `originating_system.system_type`.
  Earlier drafts of this library emitted `skill_worker` in the section 6 payload, a value whose
  only meaning was "a thing that runs skills", which implied exactly that class; it is
  withdrawn, and a tool emitting under it had given up its own type in order to speak. Whether
  the emitting tool's type is the right value for `originating_system.system_type` on a skill
  warning, as against some value naming the executor role rather than the tool the executor
  sits beside, is not settled in the schema. The position taken here follows from a decision
  taken outside the schema, that tools have executors and newsrooms do not, and the schema has
  not caught up with it. Worth naming honestly: this is one of the two places this file names a
  system type, the other being the advert's `target_system_type`, and the two now agree by
  construction, because the tool that may recall this skill is the tool that emits on its
  behalf.
- **`state_path` is null** because this skill never holds, which is legal by reading rather
  than by statement. If a future revision requires a non-null path for every skill, the
  idempotency record at `enrichment_record_path` is the value this skill would supply.
- **`skill_uri`, `skill_content_sha`, `owner_editorial`, `owner_engineering` and `licence`
  are left as markers.** They are not this author's to supply, and whether `skill_uri` is
  required at all is unsettled, one position treating it as mandatory on every registered
  skill and the other as optional, so the placeholder is left deliberately. A plausible
  invented URI would read to the next person as a real one.
- **The `skill_id` format is one of four in circulation.** `<publisher>/<name>` is used
  because it is the form with a worked precedent in the warning schema's own example, not
  because the question is settled.

---

## 11. Vendor build notes (layer 3)

| Platform | What your executor does |
|---|---|
| Verification | Your executor is recalled when an asset's provenance state matches the configured value, reads the configured instance for that house, and runs the analysis it names, typically synthetic-media analysis on a clip that arrived without a chain. Publish what you found back onto the bus as your own enrichment, with your own provenance, method and confidence, and record the absence of a provenance chain as a finding rather than as nothing. You are not being asked whether the clip is usable: that judgement belongs to a skill reading your result |
| Multimodal | You are recalled on the same messages, usually for a detection or description pass. Write your output to the configured `enrichment_record_path` as a new record. Do not merge it into the field the condition read, because your next evaluation would then be reading your own output and the condition will oscillate between warranted and not |
| MAM | You hold the asset record, so your part is mostly to make the advert matchable: expose the field the condition reads, including when it records an absence, and expose the enrichment ledger the gate is idempotent against. Accept the enrichment records other tools publish as separate records with their own provenance, and keep them beside the source field rather than folded into it |
| Transcription | You appear on both sides of this skill. You are frequently the enrichment declared warranted, for a transcript or an alignment pass, and you are also a common reason for a second configured instance of this skill, because an unrecorded source language is exactly the sort of condition a house configures on. Do not chain the two inside your own product: wait for a language identification result to be recorded, and let a second recall against that recorded state bring you the transcription work |
| Ingest | You are the earliest point at which the condition can be read, so publish the fields it depends on at ingest, including the ones whose value is that nothing is known. A field you have not published reads as unreadable, and fail closed means that warrants the enrichment, so silence at ingest costs the house compute downstream |

The instinct this skill exists to interrupt is the one that goes either enrich everything
because compute is cheap, or enrich nothing until a human thinks to ask: both take the
decision away from the story and put it in a budget line or a queue, and this skill puts it
back where the condition actually is.
