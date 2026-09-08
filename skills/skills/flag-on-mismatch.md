---
# --- identity ----------------------------------------------------------------
skill_id: smart-stories/flag-on-mismatch
name: flag-on-mismatch
skill_version: 0.2.2
som_schema_version: ["1.0.0"]

# --- what it is --------------------------------------------------------------
description: >-
  One configured field is watched on the story, and a mismatch is declared whenever
  the value a bound asset or rundown item is carrying differs from the value the story
  now holds. The declaration names both values, the old one and the current one, and
  names the asset carrying the old one. It declares only. It does not change content,
  it does not rewrite the stale value, it does not withhold anything, and it does not
  decide whether the item may run. Never auto-change is the whole promise: a tool that
  can see both values must not quietly reconcile them.
skill_type: REFERENCE
category: editorial
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
  target_system_type: [rundown, graphics, cms, playout]
  target_capability: null
  conditions:
    - kind: field_change
      path: "{{ config.watched_field }}"
      op: exists
      value: null
  recall_on: ["story.context", "story.version.published"]
  state_path: null

# --- what the executor puts on the bus on this skill's behalf ----------------
output_messages: ["skill.warning.raised"]
severity_range: [flag, inform]

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

# flag-on-mismatch

**When what the asset says stops matching what the story says, catch it. Do not fix it.**

Read `../docs/CONVENTIONS.md` first.
**Changed in 0.2.1, 20 August 2026.** Three things. Section 8's reusability claim and the section 9 declaring row named the watched field as premise.category, a path that does not resolve; it is `premise.actual_outcome`, which the worked configurations already use. asset.status is corrected to `assets[].status` in two places. The gate field this skill's warning is expected to reach is `editorial_gates[].gate_type`, not flag_type. And sections 2 and 10 now record proposal A as decided rather than printing both readings.

**Changed in 0.2.0, 18 August 2026.** Three things. `som_schema_version` moves to `["0.3.2"]`, which as of 18 August is the version. Field paths in the advert and in the worked configurations were checked against the v0.3.2 story-context JSON Schema for the first time and the ones that did not resolve are corrected; see the pack's `SCHEMA-CONFORMANCE.md` for the full list and the evidence. And the shared policy index was chosen over executor-local policy, which settles the registration question this file used to leave open: the house registers one advert row per configured instance.

---

## 1. What it is (layer 1)

| | |
|---|---|
| Operation | Declare that a bound asset carries a value the story no longer holds, and display both values |
| Condition type | `field_change`: the value differs from what the tool is carrying |
| One mechanism? | Yes. It raises. The raise-or-act test is answered in one word: **raise**. It declares a state and displays it, and it withholds nothing. The originating copy reads "flag and hold", and that sentence needs the word "and" because it is two skills. The hold is delivered by `hold-while-flagged` advertising against the flag declared here, chained over the bus. This is not tidiness: one skill occupies exactly one X category and must resolve at a single coordinate, so a skill that both declares a state and withholds something cannot be positioned, cannot be resolved, and therefore cannot be acted on by any executor. Observed behaviour is identical either way |

Category is `editorial` (X = 2) because the question being asked is an accuracy question: does what this asset says still match what the story holds to be true. That is an editorial value, not a regulatory obligation. Where a superseded figure would also breach a regulator's rule, a separate compliance skill at X = 1 covers that, and both bind at once without contest.

**Reuse test, three scenarios from different stories:**

1. **The hurricane upgrade.** A storm is upgraded to Cat 4 and accepted once on the story. A script in the rundown and a wall graphic built outside it both still read Cat 3. The mismatch is declared against every item carrying the old value, the moment the story moves.
2. **The scoreline that moved.** A goal is disallowed and the score on the story goes back to 1-1. A sports results graphic already rendered with 2-1 baked in carries the superseded value, and the mismatch is declared against it before it reaches the wall. Same skill, `watched_field` pointed at the score.
3. **The widened recall, syndicated out.** A food safety story is updated when the manufacturer widens the affected batch codes from one date range to three. A pre-cut explainer card, already syndicated to two partner sites under an ongoing feed, carries the narrower range. Each partner's own CMS executor, carrying the same library skill and the same advert, declares that its copy is behind. This library has not previously drawn this one: the asset is not in the newsroom, the tool declaring the mismatch is not the newsroom's, and the thing that has gone stale is a structured data field rather than a line of script.

One skill, three subjects, and the third one is a story this library has not previously drawn. What changes between them is a configured dot path and a bound scope. Nothing in the file knows what a hurricane, a scoreline or a batch code is.

---

## 2. Recall advert

The advert is declared in the frontmatter above so a tool can read it without parsing prose.
This section explains it in words for a human.

- **Who it is for:** any tool that carries a copy of a value taken from a story and can still emit or display it later. `rundown`, `graphics`, `cms` and `playout` cover the tool classes in scope. The common property is not the tool's job but its memory: it took a value once, it kept it, and it can still put it in front of an audience.
- **What wakes it:** `story.context` and `story.version.published`. A look happens whenever the story moves, not on a timer and not when the asset is opened. That is the whole difference from today, where a stale script is caught on the read-through if it is caught at all.
- **What it depends on:** one `field_change` condition on the path named by `config.watched_field`. The kind carries the comparison: it exists so that a story value differing from the value the tool is carrying causes a fresh look. `op: exists` narrows the look to snapshots where the configured path is actually present, so an absent field is quietly nothing rather than a mismatch against everything. Change time is not carried here, because History and Version Meta already put it on the bus.

**Unusual, and said here rather than left for a reviewer to find.** The condition path is config-templated: `{{ config.watched_field }}`, not a literal. This is deliberate, and it is what makes the file reusable rather than a hurricane skill. The consequence is real: the global lookup table cannot index a templated path as a single row. Proposal A settled that on 18 August: the house registers one advert row per configured instance, in its own registration layer. Repeated in section 10.

**Also unusual: `depends` is empty, and a reader expecting a dependency on `hold-while-flagged` will not find one.** Chaining is intended to happen through the bus, not through declared dependencies. The warning declared here goes on the bus, a second executor recalls `hold-while-flagged` against flag state, and the rundown item is withheld. Two skills, two tools, one bus, no integration between them. Declaring a dependency on the hold skill would invert the dependency, because a raise skill must never depend on what reads it, and the pair would form a cycle at the first change to either file. That chain is the assumption this library works to rather than a proven path: `hold-while-flagged` advertises on `editorial_gates[]` fields, this skill produces one `skill.warning.raised` with `blocks: []`, and `auto_change_content: false` forbids it from writing story state, so something outside this file has to carry a raised warning into gate state and nobody has yet said what. The gap is named in section 10.

---

## 3. Firing anchor

- **Evidential position:** any. The mismatch is an accuracy question about the copy an asset is carrying, and it is asked identically whether the story's value arrived PRIMARY, SECONDARY or TERTIARY. Filtering by evidential position here would let a mismatch through on the grounds of how the current value was sourced, which is a different question from whether the asset is behind.
- **Outlet / path:** the story's own path. The asset is reached through its binding to the story, not through an outlet, which is why a graphic built outside the rundown is in scope at all.
- **Scope:** `story:<id>`. The mismatch is a property of the story and the asset, not of a destination, so it holds on every path the asset could take. A per-destination variant, where the same asset is current for one link and stale for another, would be scoped `link:<id>`, and no configured instance in this library does that.
- **Compliance question, in one plain sentence:** if this went out now, would it state something the story no longer holds to be true?

---

## 4. Config surface (layer 2)

Every editorial choice sits here, set by the publishing house. Nothing here is in the skill.
Scope runs broadcaster, then site, then brand, the more specific overriding the default.

| Field | Required | Notes |
|---|---|---|
| `watched_field` | yes | The dot path on the story whose value is compared. `premise.actual_outcome` in both worked configurations in section 8. This is the field that makes the file reusable, and `premise.actual_outcome` is one configured value of it, not the skill |
| `bound_scope` | yes | Which of the tool's own things are compared: `rundown-item`, `asset`, `package`, `card`. The executor binds these; the skill only names the class |
| `compare_via` | yes | How the embedded value is obtained: `bound_field` where the asset re-resolves a bound field, or `version_history` where the asset compares against the version it was built from. The wall graphic configuration in section 8b uses `version_history` |
| `declared_severity` | no | `flag` (default) or `inform`. `flag` where the item could still be scheduled or published as it stands, `inform` where it demonstrably cannot |
| `clearing_authority` | yes | The authority and scope that may clear a declaration from this configured instance. See section 10: how authority is compared is not defined |
| `instance_label` | yes | the editorial label for this configured instance |

The house also declares this skill in its own active set. Those values are copied straight from
the frontmatter above and are not a second set of choices: `skill_id`, `skill_version`,
`skill_type`, `disclosure_level`, `migration_policy`.

---

## 5. Runtime loop, specialised

```
on story.context or story.version.published carrying the configured watched field:
  1. RESOLVE ANCHOR   the story's current value at config.watched_field, and the value
                      embedded in each thing of class config.bound_scope that this
                      executor holds and has bound to this story_id
  2. GATE             does this tool hold anything bound to this story_id, and does the
                      configured path appear in the snapshot? If either is no, exit
                      quietly. This is the cheap check, and without it every executor
                      evaluates this skill against every message in the newsroom
  3. READ CONFIG      load the configured instance for this broadcaster / site / brand
  4. EVALUATE         obtain the embedded value by config.compare_via, and compare it
                      against the story's current value. Equal means nothing happens
  5. DECIDE SEVERITY  flag where the item can still be scheduled, played or published as
                      it stands; inform where it demonstrably cannot. Never hold: the
                      hold is intended to arrive from hold-while-flagged over the bus,
                      and the carrier of that chain is an open item (section 10)
  6. EMIT             the twelve-field warning, causation_id on the triggering snapshot
  7. AWAIT CLEARANCE  an assertion of equal or higher authority on the same scope, either
                      recording that the asset now carries the current value or that the
                      difference is intended (an archive clip legitimately quoting the
                      superseded figure, for example)
  8. RESOLVE / CLOSE  closes when a later snapshot shows the embedded value matching the
                      current value, or when a valid clearance is recorded against it
```

**Skill specific traps:**

- **The mistake a well-meaning vendor will make: fixing it.** A data-driven graphics system that can re-resolve a bound field is one line of code away from writing the current value into the asset and reporting success. That is auto-change, it is forbidden, and it is worse than the stale graphic because nobody in the gallery knows the wall changed. Re-resolve into a *variant offered beside the original*, never over it.
- **The second mistake: waiting to be opened.** A tool that compares only when an operator opens the asset has rebuilt the read-through, which is the failure this skill exists to remove. The comparison belongs on the story moving.
- **What must go in `detail`:** both values, named as old and current, plus which asset carries the old one and what will clear it. "Mismatch detected" is annoying. "This rundown item reads Cat 3; the story now holds Cat 4; clears when the item is updated or a producer records the difference as intended" is actionable.
- **Fail closed:** where the configured instance is missing or unreadable, or where the embedded value cannot be obtained by the configured `compare_via`, the condition is treated as holding and the loudest severity this skill has, `flag`, is declared, with `detail` saying which of the two could not be read. An unreadable value is never an all clear. This skill has no hold authority, so fail-closed here produces a flag rather than a hold: see section 10.
- **What does NOT clear it:** the story changing again (a second change to the same field leaves the asset just as stale); an operator acknowledging the warning without updating anything; the item being moved down the rundown; the asset being re-rendered from the same superseded source. A clearance from a lower authority does not clear it either, and the attempt stays visible.

---

## 6. Output contract

`skill.warning.raised`, severity `flag` (or `inform` where the configured instance says so).

```json
{
  "topic": "som.skill.warning.raised",
  "originating_system": { "system_id": "<tool>", "system_type": "<the tool's own type, one this skill's advert targets>" },
  "correlation_id": "<the story's chain>",
  "causation_id": "<the snapshot that triggered evaluation>",
  "timestamp": "<on the envelope, never in the payload>",
  "payload": {
    "warning_id": "wrn-....",
    "skill_id": "smart-stories/flag-on-mismatch",
    "skill_version": "0.2.2",
    "story_id": "story-...",
    "scope": "story:...",
    "severity": "flag",
    "rule_id": "<the configured instance label>",
    "non_overridable": false,
    "affected_fields": ["premise.actual_outcome"],
    "detail": "Rundown item itm-4471 reads 'Cat 3'. The story now holds 'Cat 4' as of version 7. Clears when the item is updated to the current value, or when a producer of equal or higher authority on this scope records the difference as intended.",
    "blocks": [],
    "skill_warning_ref": null
  }
}
```

`blocks` is empty and `non_overridable` is false, and both are load-bearing rather than left over. Nothing is withheld here. The withholding is `hold-while-flagged`'s message, with its own `blocks` and its own `non_overridable: true`, referencing this warning through `skill_warning_ref`.

---

## 7. Priority position

| Axis | This skill |
|---|---|
| X category | `editorial` (2) |
| Y type / source | `reference` (5) |
| Z precedence | unset. The publishing house sets it, nobody else |

At (2, 5) this does not contest anything in `compliance`, so a regulatory or legal skill resolving on the same telling is followed first, and this declaration sits alongside it rather than against it. It contests nothing in `workflow`, `appearance` or `capability`, which resolve after it. No gate is contributed from here, because nothing is withheld from here. Where a hold skill binds the same telling, holds combine by conjunction: every binding gate must permit, so any one hold means held, and this flag is the state such a gate reads rather than a gate itself.

---

## 8. Worked configurations

### 8a. Breaking weather: a rundown script carrying a superseded category

```yaml
instance_label: house-accuracy-premise-mismatch-rundown
watched_field: premise.actual_outcome
bound_scope: rundown-item
compare_via: bound_field
declared_severity: flag
clearing_authority: producer-on-story
```

What this produces: the hurricane is upgraded to Cat 4 and accepted once, and a rundown item still reading Cat 3 is marked as behind the moment the story moves, with both figures shown. The rundown tool detects that its own item is behind. The withholding that keeps it off air is intended to arrive from `hold-while-flagged`, recalled by the same tool against this flag, so the item goes to producer review rather than to the read. What carries this flag into the gate state that hold skill reads is an open item, named in section 10.

### 8b. Breaking weather: a wall graphic built outside the rundown carrying the same superseded category

```yaml
instance_label: house-accuracy-premise-mismatch-asset
watched_field: premise.actual_outcome
bound_scope: asset
compare_via: version_history
declared_severity: flag
clearing_authority: producer-on-story
```

What this produces: the same upgrade reaches a wall graphic built from a brief and an email, with Cat 3 baked in. Bound to the story, the graphics tool introspects its own asset's data, finds it carries the superseded figure, and the operator is shown a refreshed Cat 4 variant beside the stale one. `assets[].status` stays `READY` until the operator acts, because nothing here changes the asset and nothing here withholds it.

**What changed between 8a and 8b, and what did not.** The honest claim is **one skill, one watched field, two bindings**. The skill file is the same file and the watched field is the same path, `premise.actual_outcome`. What differs is the binding: `bound_scope` is a rundown item in one and an asset outside the rundown in the other, and `compare_via` is `bound_field` in one and `version_history` in the other.

The originating copy calls the wall graphic configuration "the same skill and config" as the rundown item configuration, and read strictly that is not what these two configured instances are, so the claim is not made here in that form. The two config blocks are not byte-identical and nothing in this file says they are. The difference is a binding difference rather than a rule difference: the rule being applied is the same in both, namely does the value this thing carries still match the value the story holds, and the two lines that differ say only which class of thing the executor compares and by what route it obtains that thing's copy of the value. A rundown item re-resolves a bound field; a rendered wall graphic has no bound field left to re-resolve and is compared against the version it was built from. The question asked and the answer declared are identical; only the route to the embedded value is not.

Said plainly, this is the sharpest reusability proof in this library. The wall graphic in 8b sits outside the rundown entirely, and no MOS integration ever reached it: it was built from a brief and an email by a different vendor's tool, and nothing in thirty years of rundown integration would have caught it. It is served by the same library file, off the same advert, as the rundown item MOS has owned for those thirty years. Overstating that as byte-identical config invites a reviewer to check and find the two differing lines, which costs more than the stronger and true claim gains.

---

## 9. Eval set

Minimum five. The skill is not finished until a declaring case and a clearance case both pass.

| Case | Input | Expected |
|---|---|---|
| declaring | Story moves `premise.actual_outcome` from `Cat 3` to `Cat 4`; the tool holds a bound item embedding `Cat 3` | One `skill.warning.raised`, severity `flag`, `detail` naming both `Cat 3` and `Cat 4` and the item carrying the old one, `causation_id` on the triggering snapshot |
| non declaring | Same story change, but this tool holds nothing bound to the story; and separately, a bound item already embedding `Cat 4` | Nothing on the bus at all, in both variants. The gate exits quietly |
| clearance | The item is updated to `Cat 4` and a producer of the configured authority records it on the same scope | The warning closes, and the closure carries the provenance of who cleared it and when |
| wrong clearance | A user below the configured authority, or one asserting on a different scope, attempts to clear | Not applied, the warning stays open, and the attempt remains visible with its provenance |
| idempotency | The same snapshot delivered twice, in both clouds | One warning, not two. Re-evaluation of an unchanged snapshot yields the existing `warning_id` |
| fail closed | Configured instance missing, or the embedded value unreadable by the configured `compare_via` | `flag`, the loudest severity this skill has, with `detail` saying which of the two values could not be read. Never silence |
| never auto-change | Every declaring case above, replayed with the asset store under observation | Assert that no message produced by this skill changed any content: the rundown item's text, the graphic's embedded value and `assets[].status` are byte-identical before and after. The refreshed variant offered in 8b is a new object beside the original, and the original is untouched. This is the flagship case for this file, and a build that fails it is not shipping this skill |

---

## 10. Open items

- **The originating copy says "flag and hold"; this file declares only.** Both worked configurations in section 8 are described as "flag and hold" and "flag + alert operator", and both are authored here as raise-only, `severity_range: [flag, inform]`. The hold is intended to be delivered by `hold-while-flagged` advertising against the flag declared here, chained over the bus, subject to the carrier gap in the next item. The assumption is the one-mechanism rule: a skill that both declares a state and withholds something occupies two positions and cannot be resolved at a single coordinate, so no executor can act on it. If the group decides a single skill may carry both, this file and `hold-while-flagged` collapse into one, `severity_range` gains `hold`, and `blocks` becomes required. Nothing observable changes either way.
- **Nothing has been written down that carries a flag declared here into the gate state a hold reads.** This is the gap in the flag-to-hold chain both worked configurations depend on, and it is named here rather than papered over. `hold-while-flagged` advertises on `editorial_gates[]` fields: a `gate_type`, a `status`, a scope. This skill produces exactly one output, `skill.warning.raised`, with `blocks: []`, and `auto_change_content: false` forbids it from writing story state, so a warning raised here does not become an `editorial_gates[]` entry by itself. Something between the two has to materialise it, and nothing written down says what. The assumption taken across this library is that the executor materialises a raised warning into gate state on the story, which keeps the two skills independent and keeps the writing of state out of the skill layer. If the group decides instead that the bus does it, or that a third skill or a gate service does it, nothing in this file's advert, config surface or output contract changes, but the chain acquires an owner that would need naming here and the idempotency question moves to that owner. The bus side of the chain is in place: `hold-while-flagged` carries `skill.warning.raised` in its `recall_on`, so a warning raised here does reach it, and the chain works as described from that point on. What is unowned is the step before, the materialisation into `editorial_gates[]`. Until one of those readings is agreed, that step should be read as assumed, not as proven.
- **`depends` is empty and stays empty.** A reader expecting `hold-while-flagged` in `depends` should read section 2. Depend on what you read, never on what reads you: declaring the hold skill here would invert the dependency and create a cycle at the first change to either file. The hold skill does not depend on this one either, because it advertises against flag state whoever declared it.
- **The condition path is config-templated, and the indexing consequence is now settled.** `{{ config.watched_field }}` is not a literal dot path, and the global lookup table cannot index it as a single row. Proposal A settled that on 18 August: the house registers one advert row per configured instance, in its own registration layer. That is the reading this file always assumed, because it keeps the table's rows literal and lets `bound_scope` differ per row, which is exactly what the two worked configurations need: they read as two rows rather than one row the executor discriminates on. What A costs is a registration layer that has to exist and be kept honest; what it buys is that conformance is provable, because every row names the instance it came from.
- **`disclosure_level: L2` means "the deepest level the skill declares content for".** Two definitions are in circulation, the other being how much of its own reasoning a skill reveals. The first was meant: sections 1 to 3 are layer 1, section 4 is layer 2, section 11 is layer 3, and `level_2_uri` and `level_3_uri` are null because it is all in this one file.
- **Targeting is written as though it binds.** `target_system_type` lists four types and the assumption is the deterministic base: an executor acts on this skill only where the conditions match and its own type is in that list. Under an advisory reading the behaviour would differ materially here, because off-target tools such as a compliance hub, an audit service or an oversight desk could also recall it and declare the same mismatch. That would widen who declares, which makes the idempotency case a question about more than the two clouds it currently covers. Not resolved.
- **The system that emits on this skill's behalf carries the tool's own system type, and `skill_worker` is withdrawn.** An executor is a sidecar to a vendor tool, and there is no newsroom-scoped class of skill-running system for one to belong to, so the executor that puts this declaration on the bus is the rundown, graphics, CMS or playout tool's own executor, and it names that tool's own type in `originating_system.system_type`. Earlier drafts of this library emitted `skill_worker` in the section 6 payload, a value whose only meaning was "a thing that runs skills", which implied exactly that class; it is withdrawn, and a rundown tool emitting under it had given up its own type in order to speak. Whether the emitting tool's type is the right value for `originating_system.system_type` on a skill warning, as against some value naming the executor role rather than the tool the executor sits beside, is not settled in the schema. The position taken here follows from a decision taken outside the schema, that tools have executors and newsrooms do not, and the schema has not caught up with it. Worth naming rather than leaving to be found: this is one of the two places this file names a system type, the other being the advert's `target_system_type`, and the two now agree by construction, because the tool that may recall this skill is the tool that emits on its behalf.
- **How clearance authority is compared is not defined.** The rule adopted throughout this library is that a declaration closes only on an assertion of equal or higher authority on the same scope, so a flag clears only that way, with `clearing_authority` naming that level per configured instance. That is this library's working position, not a rule any group has ratified, and no vendor should read it as settled or hard-code enforcement against it. Raising is open to anyone, with provenance recorded, and there is no write-authority layer. What is undefined is the comparison itself: nothing states how one authority is ranked against another, or what "the same scope" means when a producer asserts on the story and an operator asserts on the asset, so `config.clearing_authority` is a house-local string and the executor's comparison of it is house-local too. The wrong-clearance eval case is written against the adopted rule, and if the group settles the comparison differently that row is rewritten and `clearing_authority` is read against the group's rule rather than the house's.
- **`state_path` is null, which is legal by reading rather than by statement.** The field carries the state a message must carry to decide an open hold, and nothing is held from here, so there is nothing to decide. No document says in so many words that null is permitted for a skill that never holds.
- **`fail_closed: true` reads differently for a skill with no hold authority.** The template's illustration of fail-closed is a hold that says why, and that is not available here. The reading applied is that fail-closed at the skill layer means treating an unreadable value as the condition holding, which produces the loudest severity this skill actually has, `flag`. Safe state at the executor layer, where nothing can be resolved at all and the executor does not act, is a different mechanism at a different layer.
- **`recall_on` topic names are working names.** `story.context` has a worked precedent; `story.version.published` does not, and `recall_on` is itself a proposal rather than an agreed replacement for the older active name. If the topic vocabulary settles differently, only this line of the advert changes.
- **`bound_scope` names a class, and the binding is the executor's.** The skill names what class of thing to compare and cannot know what this tool has bound to the story. Whether a tool with no binding to the story should be able to recall the skill at all is the same question as binding-versus-advisory targeting, above.
- **`skill_uri`, `skill_content_sha`, `owner_editorial`, `owner_engineering` and `licence` are left as markers.** They are not this author's to supply, and whether `skill_uri` is required at all is unsettled: one position treats it as mandatory on every registered skill, the other as optional. A plausible invented URI would read to the next person as a real one.
- **The `skill_id` format is one of four in circulation.** `<publisher>/<name>` is used because it is the form with a worked precedent in the warning schema's own example, not because the question is settled.

---

## 11. Vendor build notes (layer 3)

| Platform | What your executor does |
|---|---|
| Rundown | On the story moving, walk the items you hold against this story and compare the value each one embeds at the configured path. Where an item is behind, mark it for producer review and put the warning on the bus with both figures in `detail`. Do not edit the script. The item text is the producer's, and an item that quietly changed under a producer is worse than one that was flagged and left alone |
| Graphics | Compare the value your rendered asset carries with the story's current value, by whichever route your executor has. A data-driven executor re-resolves the bound field against the current story and compares what comes back with what the asset carries; where they differ, it builds the refreshed variant and offers it to the operator *beside* the stale one, both visible, neither selected. A template-based executor cannot re-resolve, so it introspects its own asset's data against the version that asset was built from, compares that with the story's current value, and where they differ marks the asset as carrying a superseded value and flags it for rebuild. Prompt or offer only: leave `assets[].status` at `READY` and let the operator choose. Re-resolving straight into the live asset is auto-publishing a change nobody approved, and changing nothing yourself is the correct outcome on both routes, because the declaration is the same one either way |
| Digital CMS | Compare the value embedded in published and scheduled pieces, including structured data fields and syndicated copies you still feed, against the story's current value. Flag the piece and surface both values to the desk. Do not push a correction, do not silently update the live page, and do not suppress the piece: withholding is `hold-while-flagged`'s message, not yours |
| Playout | Compare the value carried by items already in the playlist or on the server against the story's current value, and declare the ones that are behind, with enough lead time for a human to act. Declaring is the whole of your job here. Pulling the item from the playlist is a hold, it arrives from a different skill, and taking that decision yourself removes the producer from a call that is theirs |

The instinct this skill exists to interrupt is the helpful rewrite. A tool that can see both the stale value and the current one is one line of code away from reconciling them and reporting success, and the gallery would never know the wall changed. `auto_change_content: false` is not decoration here, it is the whole promise of the skill: never auto-change.
