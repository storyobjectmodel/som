---
# --- identity ----------------------------------------------------------------
skill_id: smart-stories/record-provenance-on-ingest
name: record-provenance-on-ingest
skill_version: 0.2.2
som_schema_version: ["1.0.0"]

# --- what it is --------------------------------------------------------------
description: >-
  At the moment a source enters a story, whether that is a wire alert at mint or a
  media asset at ingest, this skill declares that the source's provenance is to be
  recorded structurally: what it carries, and equally what it does not carry. A
  C2PA chain read as present is recorded; a chain that is absent is recorded as
  absent, not passed over in silence. Credibility states such as TRUSTED and
  UNVERIFIED are recorded as declared priors, and they are non-ordinal: they answer
  different questions and are not a ranking. It records only. It does not judge,
  rank, block or enrich.
skill_type: REFERENCE
category: compliance
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
recall:
  target_system_type: [mam, ingest, verification, discovery, archive]
  target_capability: null
  conditions:
    - kind: field_change
      path: "{{ config.subject_path }}"
      op: exists
    - kind: field
      path: "{{ config.record_path }}"
      op: absent
  recall_on: ["story.context", "source.registered", "media.ingested"]
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

# record-provenance-on-ingest

**When a source enters the story, record what it came with. If it came with nothing,
record that too. Do not judge it.**

Read `../docs/CONVENTIONS.md` first.
**Changed in 0.2.1, 20 August 2026.** One thing. Sections 2 and 10 still printed the registration question as open after proposal A closed it on 18 August. Both now say what A decided, that the house registers one advert row per configured instance in its own registration layer, and what that obliges.

**Changed in 0.2.0, 18 August 2026.** Three things. `som_schema_version` moves to `["0.3.2"]`, which as of 18 August is the version. Field paths in the advert and in the worked configurations were checked against the v0.3.2 story-context JSON Schema for the first time and the ones that did not resolve are corrected; see the pack's `SCHEMA-CONFORMANCE.md` for the full list and the evidence. And the shared policy index was chosen over executor-local policy, which settles the registration question this file used to leave open: the house registers one advert row per configured instance.

---

## 1. What it is (layer 1)

| | |
|---|---|
| Operation | Record. The provenance a source arrived with is declared and displayed at the moment that source enters the story, stating what the source carries and, equally, what it does not carry: a chain read as present is stated present, and a chain that is absent is stated absent rather than passed over in silence. The record itself is written by the executor, under its own system identity, off that declaration |
| Condition type | Ingest. A source or asset entry appearing on the story that has no provenance record against it yet |
| One mechanism? | Yes. One thing is declared: what a source came with. Presence and absence are not two operations, they are two values of one declaration, and treating them as two is precisely the failure this skill exists to remove. The sentence needs no "and", and in particular the sentence is not "declare and write": the writing is the executor's, which is why `auto_change_content: false` holds here without an exception of any kind |

The absence is the point. Today provenance is a human judgement: an operator eyeballs
a clip, decides whether to trust it, and the absence of a chain is invisible, because
nothing anywhere records that a clip arrived with no provenance at all. A story then
carries two silences that look identical: the silence of a source nobody checked, and
the silence of a source that had nothing to check. Here both are written down. The
honesty sits in the data rather than in someone remembering to mention it.

`compliance` is the defensible category (X = 1) because authenticity and provenance
are the compliance job that raw material carries. What a story can later say about
where its evidence came from is a regulatory and legal exposure, not a matter of house
style, so the choice survives an audit by the deploying house.

**Reuse test, three scenarios from different stories:**

1. **A hurricane story, at mint.** The story is minted from the first wire alert.
   The wire is logged in `editorial_source[]` and the house's standing prior for that
   wire, `TRUSTED`, is recorded against it. Compliance posture is set before any
   footage exists.
2. **The same hurricane story, at media ingest.** Two clips arrive. An agency clip
   is attributed and carries a C2PA chain, which is recorded present. A citizen clip
   arrives loose with none, and its chain is recorded absent. Both are `PRIMARY`; the
   agency clip is `TRUSTED` on an institutional prior, the citizen clip `UNVERIFIED`
   because nothing is yet known about it.
3. **A slow corruption investigation, not previously drawn by this library.** Leaked
   documents reach the desk over eight months through an intermediary, some with a
   signed custody statement from the originating registry and some with nothing but the
   intermediary's word. Each document's custody statement is recorded present or absent
   as it lands, so that a year later the lawyers can see which parts of the file were
   ever traceable, without asking anyone to remember.

One skill, three kinds of source (a wire feed, a video clip, a leaked document) and one
operation: record what came with the source, including the fact that nothing did.

---

## 2. Recall advert

The advert is declared in the frontmatter above so a tool can read it without parsing
prose. This section explains it in words for a human.

- **Who it is for:** tools that see material at the boundary, where provenance is still
  cheaply legible: a MAM, an ingest tool, a verification tool, a discovery tool, an
  archive. A tool that meets the asset later meets it stripped of the container metadata
  the chain lived in, which is why targeting sits at the point of arrival.
- **What wakes it:** `story.context`, plus the topics that carry a new source or asset
  entry onto the story, `source.registered` and `media.ingested`.
- **What it depends on:** a subject appearing at the configured `subject_path` that was
  not carried before, and no provenance record yet standing at the configured
  `record_path`.

Two things about this advert are unusual and are better said here than found later.

**The advert deliberately does not test for the presence of provenance.** The obvious
draft advertises `op: exists` against the C2PA field, and it is wrong. A clip arriving
with no chain would then never be recalled at all, the executor would look at nothing,
and the absence would stay exactly as invisible as it is today. The condition is
therefore on the arrival of the subject, never on what the subject happens to carry.

**The watched paths are config-templated.** `subject_path` and `record_path` are set by
the house, because the thing being ingested at mint (a source entry) and the thing being
ingested at media ingest (an asset's provenance block) are different paths in the same
schema. The lookup table cannot index `{{ config.subject_path }}` as one row, so
under proposal A, chosen 18 August, the house registers one advert row per configured
instance in its own registration layer. That is a named cost of a good design rather
than a mistake, and it is repeated in section 10.

`depends` is empty and stays empty. Anything that wants to act on what this skill records
advertises against the recorded state over the bus, whoever wrote it. Declaring a
dependency in the other direction inverts it and creates a cycle at the first change.

---

## 3. Firing anchor

- **Evidential position:** any. Clips arriving at media ingest are `PRIMARY`, raw
  material whose compliance job is authenticity and provenance, but a wire alert logged
  at mint is not reached by that constraint. Narrowing the anchor to `PRIMARY` would
  leave the least traceable material unrecorded in exactly the cases that matter most.
- **Outlet / path:** the story's own source and asset arrays. Nothing here attaches to a
  destination's telling; a provenance record is true of the source regardless of where
  the story goes.
- **Scope:** `story:<id>`. This is system level. A record of what a source carried is not
  destination specific, so `link:<id>` is never the scope for this skill.
- **Compliance question, in one plain sentence:** for every source this story is standing
  on, does the record say what that source arrived with, including that it arrived with
  nothing?

---

## 4. Config surface (layer 2)

Every editorial choice sits here, set by the publishing house. Nothing here is in the
skill. Scope runs broadcaster, then site, then brand, the more specific overriding the
default. Broadcaster is grounded in `newsroom_id`, brand in `framing_treatment`, and site
is the executor's deployment context rather than a schema field.

| Field | Required | Notes |
|---|---|---|
| `subject_path` | yes | The schema path of the thing whose provenance is recorded. `editorial_source[]` at mint; `assets[]` at media ingest |
| `record_path` | yes | Where the executor writes the record, under its own system identity, off what this skill declares. Nothing on this skill's behalf writes there. Used by the gate as well: a subject that already carries a record here is not looked at again |
| `record_kind` | yes | The label for what is being looked for: `source-credibility` at mint, `c2pa-chain` at media ingest. It names the question, never the answer |
| `prior_credibility` | no | A map from subject class to the credibility state the house holds as a standing prior, for example a wire it has a long relationship with recorded as `TRUSTED`. Where the house declares no prior for a class, `UNVERIFIED` is recorded, which states that nothing is yet known, not that something scored badly |
| `instance_label` | yes | the editorial label for this configured instance |

Two constraints on what a house may configure here, both load bearing.

**There is no field that suppresses the absence case.** A house chooses what is watched
and what its priors are. It cannot configure away the declaration that a source arrived
with nothing, nor the record the executor writes off it. That is the skill, not a setting.

**Nothing here ranks one credibility state above another,** because the states are not on
a scale. `TRUSTED` records an institutional prior about who sent the material.
`UNVERIFIED` records that no such prior exists yet. They answer different questions, so
there is no configuration surface for comparing them and none is coming.

The house also declares this skill in its own active set. Those values are copied straight
from the frontmatter above and are not a second set of choices: `skill_id`,
`skill_version`, `skill_type`, `disclosure_level`, `migration_policy`.

---

## 5. Runtime loop, specialised

```
on a source or media asset appearing on a story:
  1. RESOLVE ANCHOR   the subject that arrived at config.subject_path: a source entry
                      at mint, an asset's provenance block at media ingest
  2. GATE             a record already stands at config.record_path for this subject?
                      then exit quietly. This is the cheap check, and it is a record
                      test, never a provenance test
  3. READ CONFIG      load the configured instance for this broadcaster / site / brand
  4. EVALUATE         read what the subject carries: a chain, an institutional prior,
                      or nothing. All three are findings, and all three are declared
  5. DECIDE SEVERITY  inform, in every case. Presence and absence are declared at the
                      same volume, because this skill has no other volume
  6. EMIT             the twelve-field warning, causation_id on the triggering snapshot,
                      carrying the declaration and displaying it. The executor writes
                      the record at config.record_path under its own system identity,
                      off this declaration. Nothing is written on this skill's behalf
  7. AWAIT CLEARANCE  a correction or supersession of the declared provenance, asserted
                      by an equal or higher authority on the same scope
  8. RESOLVE / CLOSE  closed when a superseding record on the same subject and scope is
                      carried. The superseded record stays visible in the array
```

**Skill specific traps:**

- **The absence of a chain is a finding, not a non-event.** This is the loudest thing in
  this file. A clip that arrives with nothing is declared exactly as fully as a clip that
  arrives with a complete chain, and the record the executor writes off that declaration
  says "no chain present", never nothing at all. A vendor whose executor quietly returns
  early when it finds no C2PA data has rebuilt, inside their own product, the invisibility
  this skill exists to remove. Silence about a source is not the same fact as a source with
  nothing to say.
- **Declaring is this skill's half; writing is the executor's.** What this skill does is
  declare the provenance found (or found missing) and display it. What puts a record at
  `config.record_path` is the executor, acting under its own system identity off that
  declaration, exactly as section 11 describes for each platform. The distinction is not
  pedantry: it is what keeps `auto_change_content: false` true without a carve-out, and it
  is what keeps the operation single rather than "declare and write". An executor that
  treats the warning itself as the write has collapsed the two halves and broken the
  guarantee the eval set asserts.
- **Do not narrow the advert to the presence of provenance.** The same mistake one layer
  up, described in section 2. If the recall condition tests the chain field, the absent
  case never reaches the executor at all.
- **What must go in `detail`:** which subject arrived, what was looked for
  (`record_kind`), what was found stated as present or absent in those words, and which
  authority would supersede the record. A record that says "provenance checked" is not
  actionable; a record that says "c2pa-chain: absent, on citizen clip mda-0442,
  superseded only by an equal or higher authority on story scope" is.
- **Credibility is recorded, not computed.** The state written comes from the house's
  declared prior in `prior_credibility`, or is `UNVERIFIED` where no prior exists. It is
  never inferred from the material, never derived from the presence of a chain, and never
  compared with another source's state.
- **Fail closed:** an unreadable provenance block, or configuration that cannot be loaded,
  is treated as the condition holding. What is declared, and what the executor's record
  then states, is that the value could not be read, at `inform`, which is the loudest
  severity this skill actually has. Fail
  closed here does not mean a hold appears, because this skill has no hold authority at
  all. Safe state at the executor layer, where an executor that cannot resolve a skill
  simply does not act, is a different mechanism at a different layer and is not this. See
  section 10.
- **What does NOT clear it:** an operator's opinion that a clip looks genuine; a later
  enrichment attaching context; downstream use of the asset in a package; the asset being
  archived. And nothing clears a record of absence by making it silent again. A record of
  absence closes only when a superseding record on the same subject and scope states that
  a chain is now present, and the earlier record of absence remains in the array with its
  own provenance of who, when and scope.

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
    "skill_id": "smart-stories/record-provenance-on-ingest",
    "skill_version": "0.2.2",
    "story_id": "story-...",
    "scope": "story:...",
    "severity": "inform",
    "rule_id": "house-media-ingest-provenance",
    "non_overridable": false,
    "affected_fields": ["assets[0].authenticity_credential"],
    "detail": "c2pa-chain looked for on mda-0442 (citizen clip, PRIMARY): absent. Credibility recorded UNVERIFIED, no prior declared for class citizen. Superseded only by an equal or higher authority on story scope.",
    "blocks": [],
    "skill_warning_ref": null
  }
}
```

`blocks` is empty and stays empty: nothing is withheld by this skill, so there is nothing
to name. `non_overridable` is false for the same reason, because it is only true of a
hold.

---

## 7. Priority position

| Axis | This skill |
|---|---|
| X category | `compliance` (1) |
| Y type / source | `reference` (5) |
| Z precedence | unset. The publishing house sets it, nobody else |

It contests nothing at the gate. It withholds nothing, contributes no binding gate, and a
record of absence never by itself means held. Where this skill's record matters to a
decision, the deciding is done by some other skill at its own coordinate, advertising
against the state recorded here and reached over the bus. Where gates do contribute to a
telling, holds combine by conjunction: every binding gate must permit, so any one hold
means held. That rule is stated here for orientation; this skill is not one of those
gates.

---

## 8. Worked configurations

### 8a. A wire source at mint, logged TRUSTED

```yaml
instance_label: house-wire-source-provenance
subject_path: editorial_source[]
record_path: editorial_source[].credibility
record_kind: source-credibility
prior_credibility:
  wire: TRUSTED
```

What this produces: the story is minted from the first wire alert and its compliance
posture is already set before any footage exists. The wire stands in `editorial_source[]`
recorded `TRUSTED`, and the figures it carries stand `UNVERIFIED` alongside it, which is a
statement that nothing has been checked yet rather than a doubt about the wire.

### 8b. A C2PA chain on an ingested clip, recorded present or absent

```yaml
instance_label: house-media-ingest-provenance
subject_path: assets[]
record_path: assets[].authenticity_credential
record_kind: c2pa-chain
prior_credibility:
  agency: TRUSTED
  citizen: UNVERIFIED
```

What this produces: two clips, handled honestly. The agency clip arrives attributed
carrying a C2PA chain, and the chain is recorded present. The citizen clip arrives loose
with none, and the chain is recorded absent. Both are `PRIMARY` evidence; the agency clip
is `TRUSTED` on an institutional prior and the citizen clip `UNVERIFIED` because nothing
is known of it yet. Neither of those states outranks the other.

**What changed between 8a and 8b:** the kind of source (a wire feed against a video file),
the path watched, and the question asked of it (a declared credibility prior against an
embedded cryptographic chain). **What stayed the same:** the operation, the anchor, the
severity, and the rule that whatever is not there is written down as not there. One
provenance skill, two postures, configured rather than forked.

---

## 9. Eval set

Minimum five. The skill is not finished until a declaring case and a clearance case both
pass.

| Case | Input | Expected |
|---|---|---|
| declaring | An agency clip arrives at ingest carrying a readable C2PA chain, no record standing at `record_path` | One `inform` warning. `detail` carries both values: `c2pa-chain` looked for, found present, credibility recorded `TRUSTED` from the declared prior for class agency |
| absence | A citizen clip arrives at ingest with no C2PA data of any kind | One `inform` warning of the same shape, `detail` stating `c2pa-chain: absent` and credibility `UNVERIFIED`. Assert explicitly that the outcome is a record, not silence, and that its shape and severity match the declaring case. This row is the point of the skill |
| non declaring | A message carries no new entry at `subject_path`, or the subject already carries a record at `record_path` | Nothing at all on the bus, quietly |
| clearance | For a record-only skill, clearance means correction or supersession of what was recorded: a verification desk with story-scope authority asserts that the citizen clip's chain has since been located and validated | The earlier record is closed as superseded. The superseding record stands, and the original record of absence remains visible in the array with its own provenance |
| wrong clearance | A single operator with narrower scope asserts that the citizen clip is fine and its absence record should go away | Not applied. The record stands, and the attempt itself stays visible with its own provenance of who, when and scope. Nothing is deleted |
| idempotency | The same ingest snapshot presented twice, in both clouds | One warning and one record. The gate at step 2 is a test for an existing record, so the second pass exits quietly |
| fail closed | Configuration missing, or a provenance block present but unreadable | An `inform` warning, which is the loudest severity this skill has, with `detail` saying what could not be read and why. Assert that no `hold` is produced under any input: a skill with no hold authority cannot fail closed into one |
| never auto-change | Any declaring case, including the absence case, replayed with the story, the source array and the asset store under observation | Assert that no message carrying this `skill_id` changed any content: the story, the source and the asset are byte-identical before and after. Only warnings were added. The provenance record itself is written by the executor under its own system identity, so it is not a counter-example and must not be treated as one |

---

## 10. Open items

- **Fail closed, for a skill that cannot hold.** The template illustrates `fail_closed`
  with a hold, and this skill has no hold authority. The reading applied throughout is
  that fail closed at the skill layer means an unreadable value is treated as the
  condition holding, which produces the loudest severity the skill actually has: here
  that is `inform`, and it is stated that way in section 5 and in the fail-closed eval
  row. Safe state at the executor layer, where nothing resolves and the executor does
  not act, is a separate mechanism at a separate layer and is not reachable from this
  skill. This is written down rather than left implicit so that no reviewer concludes it
  was overlooked. If the group later decides that fail closed must always escalate
  severity, `severity_range` here would have to grow, and that would be a change to what
  this skill claims it can do.
- **Clearance semantics are thinner here than the template assumes.** The template's
  clearance case presumes something is being withheld and can be released. This skill
  withholds nothing, so "clearance" has been written as correction or supersession of the
  recorded provenance by an equal or higher authority on the same scope. That is an
  assumption, not a ratified reading, and a reviewer may reasonably say a record-only
  skill should have no clearance case at all.
- **How authority is compared is undefined.** The rule adopted throughout this library is
  that a declaration closes only on an assertion of equal or higher authority on the same
  scope, so a record is superseded only that way. That is this library's working position,
  not a rule any group has ratified, and no vendor should read it as settled or hard-code
  enforcement against it. Nothing states how two authorities are compared either, so the
  authority named on a superseding assertion is a house-local string and the executor's
  comparison of it is house-local too. The wrong-clearance eval case reads a narrower
  operator scope as losing to a story-scope verification desk. If the comparison rule lands
  elsewhere, that row changes, along with whatever house-local field the deployment uses
  to name the clearing authority.
- **`disclosure_level: L2` means the deepest level the skill declares content for.** Two
  definitions are in circulation, the other being how much of its reasoning a skill
  reveals. The first is the one meant here.
- **Targeting is written as though it binds.** `target_system_type` is treated as a
  deterministic base: an executor acts only where the conditions match and its own type
  matches. Under an advisory reading, off-target tools such as a compliance hub or an
  audit tool would also pick this skill up, which for a record-only skill would be
  harmless and arguably useful, so nothing in the body would need to change. Separately,
  only `mam` among the listed system types is confirmed against the v0.3 enum by the
  material to hand; `ingest`, `verification`, `discovery` and `archive` are written as
  the broad types this skill is for and should be checked against the enum before the
  advert is registered. Naming a product rather than a type would be a bet on an open
  question, so no product is named.
- **The emitting executor carries the tool's own `system_type`, and the `skill_worker`
  type this library used to emit here is withdrawn.** An executor is a sidecar to a
  vendor tool, and no newsroom-scoped class of skill-running system exists, so the thing
  that puts this skill's record on the bus is a MAM, an ingest tool, a verification
  tool, a discovery tool or an archive, and section 6 now says so. The sample payload
  previously carried a `skill_worker` system type, a value whose only reason to exist was
  that the emitter ran skills, which is the newsroom-executor shape under another
  name. It is withdrawn rather than reinterpreted, and no file in this library should be
  read as having such a class. What is not settled is whether the emitting tool's own
  type is the right value for `originating_system.system_type` on a skill warning at
  all, as against some value naming the executor role that raised it. The schema does
  not answer that, and the position taken here follows from a decision taken outside the
  schema, that tools have executors and newsrooms do not, which the schema has not
  caught up with. Worth naming plainly: this is one of the two places this file names a
  system type, the other being the advert's `target_system_type`, and after this change
  the two agree by construction, because the tool that may recall this skill is the tool
  that emits on its behalf.
- **Whether `editorial_source[]` and the C2PA fields are settled in the v0.3 schema is
  not known here, and it is not asserted.** Both appear in the material to hand and both
  are used above as configured paths rather than as literals in the advert, which limits
  the damage if the names differ: the house repoints `subject_path` and `record_path`
  without a new skill version. What is genuinely unresolved is whether the schema carries
  a place to record a chain as absent, as opposed to simply leaving a chain field empty.
  Those are different facts, this skill depends on the difference, and if the schema
  cannot express it then the schema, not the skill, is what needs changing.
- **The config-templated path has an indexing consequence, settled by proposal A,** as set
  out in section 2: the lookup table cannot index `{{ config.subject_path }}` as one row, so
  the house registers one advert row per configured instance in its own registration layer. The mint posture and the media ingest posture are two
  configured instances of this skill on the same story, so this is not hypothetical here.
- **`state_path` is `null`** because this skill never holds and there is no open hold
  state for a message to decide. That is legal by reading rather than by explicit
  statement in the reference.
- **`depends` is empty by rule.** At media ingest this skill sits alongside
  `match-and-propose` and `enrich-on-condition` on the same clips, which reads like a
  dependency and is not one. Chaining happens through the bus, and a declared dependency
  would invert at the first change.
- **`recall_on` is a proposal name,** replacing the older active form under the passive
  convention agreed 29 July 2026. A reviewer preferring a different name is raising a
  live disagreement, not finding an error.
- **`skill_uri`, `skill_content_sha`, the owners and the licence are left as markers.**
  They are not this author's to supply, and a plausible invented value would read as a
  real one. Whether `skill_uri` is required at all is disputed between two reference
  tables, and the placeholder is left deliberately rather than filled with something
  plausible.
- **The `skill_id` format is one of four in circulation.** `<publisher>/<name>` is used
  because it is the form with a worked precedent in the warning schema's own example, not
  because the question is settled.
- **Nothing states what carries a declaration into the record at `record_path`.** This
  skill declares the provenance found and displays it; `auto_change_content: false`
  forbids it from writing anything; and section 11 asks each vendor's executor to write
  the record under its own system identity. That division is the only reading under which
  the guarantee and the purpose both survive, and it is the reading applied here. What is
  not written down anywhere is the mechanism by which a `skill.warning.raised` becomes a
  provenance record on the asset. This is the same gap `CONVENTIONS.md` names for the
  flag-to-hold chain, met here at ingest instead of at the gate, and it is a gap in the
  model rather than in this file. A house whose executor does not close it gets warnings
  and no records.

---

## 11. Vendor build notes (layer 3)

Here the matching, detection and ingest tools each compute and publish what they find,
rather than one tool computing on another's behalf.

| Platform | What your executor does |
|---|---|
| MAM | On ingest, read the asset's embedded C2PA data and record the chain present against the asset, with its claim generator and signature status. Where a file arrives with no such data, write the record anyway, stating the chain absent. The attributed agency clip and the loose UGC clip take the same code path and produce the same shape of record; only the value differs |
| Ingest | At the boundary where the file is still whole, before transcode strips container metadata, capture what the source arrived with. This is the last cheap moment: what is not read here is not recoverable later without going back to the sender |
| Verification | Do not let a verification verdict overwrite the arrival record. The two are separate facts: what a clip came with, and what your tooling later concluded about it. Publish the second as its own record referencing the first, so that a later validation of the citizen clip supersedes rather than erases the record that it arrived with nothing |
| Discovery | Loose media matched to a running story arrives with a match confidence and, separately, with whatever provenance it carries. Publish both. A high match confidence is a statement about which story the clip belongs to, and says nothing whatever about whether the clip is authentic |
| Archive | Carry the provenance record forward with the asset, including the records of absence, and keep superseded records rather than compacting them away. A year on, the fact that a clip once had no chain is the fact somebody will need |

The instinct this skill exists to interrupt is recording only what is there.
