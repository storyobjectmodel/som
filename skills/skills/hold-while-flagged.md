---
# --- identity ----------------------------------------------------------------
skill_id: smart-stories/hold-while-flagged
name: hold-while-flagged
skill_version: 0.2.2
som_schema_version: ["1.0.0"]

# --- what it is --------------------------------------------------------------
description: >-
  Withholding is declared against a named action on a named asset while a flag
  of a configured type stands on a configured scope, with release declared only
  once that flag is cleared by an assertion of equal or higher authority on the
  same scope. It withholds only. It does not raise the flag, does not judge
  whether the flag was correct, does
  not rank, edit, enrich or delete anything, and it never treats the absence of
  a flag on a later message as a clearance.
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
# This is the row the global lookup table is built from. Machine-read, never prose.
recall:
  target_system_type: [mam, playout, rundown, cms, compliance_hub]
  target_capability: null
  conditions:
    - kind: field
      path: editorial_gates[].gate_type
      op: equals
      value: "{{ config.gate_type }}"
    - kind: field
      path: editorial_gates[].status
      op: equals
      value: PENDING
    - kind: field_change
      path: editorial_gates[].status
      op: equals
      value: APPROVED
  recall_on: ["story.context", "asset.context", "compliance.assertion", "skill.warning.raised"]
  state_path: editorial_gates[].status

# --- what the executor puts on the bus on this skill's behalf ----------------
output_messages: ["skill.warning.raised"]
severity_range: [hold]

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

# hold-while-flagged

**While a flag of this type stands on this scope, hold the thing it names. Release it only
when an equal or higher authority clears the flag on the same scope. Do not fix it, do not
delete it, do not decide the flag was wrong.**

Read `../docs/CONVENTIONS.md` first.
**Changed in 0.2.1, 20 August 2026.** Two things. The file described the gate entry it advertises against as carrying a flag_type, in section 4, the loop, the section 6 payload, two eval rows and section 10. The schema's `editorial_gate` carries `gate_type`, which is what the advert and the config surface already used, so the prose was the only place the wrong name survived. And sections 2 and 10 now record proposal A as decided.

**Changed in 0.2.0, 18 August 2026.** Three things. `som_schema_version` moves to `["0.3.2"]`, which as of 18 August is the version. Field paths in the advert and in the worked configurations were checked against the v0.3.2 story-context JSON Schema for the first time and the ones that did not resolve are corrected; see the pack's `SCHEMA-CONFORMANCE.md` for the full list and the evidence. And the shared policy index was chosen over executor-local policy, which settles the registration question this file used to leave open: the house registers one advert row per configured instance.

---

## 1. What it is (layer 1)

| | |
|---|---|
| Operation | Withhold a named action on a named asset while a flag stands |
| Condition type | A standing flag of a configured type on a configured scope |
| One mechanism? | Yes. The sentence is "hold while flagged", with no "and" in it. Nothing is declared here: the flag already exists on the story, declared by whoever declared it, and what is added is the withholding. Category is `compliance` because the consequence of releasing a restricted detail is legal or regulatory, not a matter of house taste, and that choice is defensible against the alternative of `workflow`: routing decides where a thing goes, this decides whether it may go at all |

**Reuse test, three scenarios from different stories:**

1. **The hurricane, an unconfirmed casualty figure.** The confirmed category is cleared to publish; the unconfirmed casualty figure is not. The figure carries a flag on one publication path, and the hold keeps it off that path while the story goes out on every other path with the confirmed detail intact.
2. **The verdict, a court reporting restriction.** Two verdict packages are prepared before the court rules. One carries a court reporting restriction. The restriction sits on the asset, and the asset is held off air structurally, before anything reaches a rundown, so it cannot reach air by mistake.
3. **A conservation story this library has not previously drawn.** A wildlife unit files a piece on the first confirmed nesting site of a species thought extinct in the region. The national conservation authority attaches a location embargo to the precise coordinates, because publishing them invites egg collectors. The story runs in full; the sequence containing the readable GPS overlay is held while the embargo stands, and it releases when the conservation authority, not the newsroom, lifts it. No court, no crisis desk, no market, no rights holder, and the authority that declared the flag has no presence in the newsroom at all.

One skill, three subjects, three flag-type configurations. The situation is entirely in the configuration, and the mechanism knows nothing about hurricanes, courts or birds.

**How this is distinguished from `gate-by-scope`: by what is read, not by scope.** The closest file
in the library is `gate-by-scope`, and a reviewer reading both quickly will look for the difference
in the scope coordinate and find the wrong answer there. The difference is the question each one
reads. A typed flag is read here: is a flag of the configured type standing against this action? A
path's gate is read there: does the gate bound to this destination path permit this telling? This
skill may be configured at `story:<id>` or at `link:<id>`, while `gate-by-scope` is `link:<id>`
scope always, so scope is not the distinguishing axis, and the `scope` field in section 4 and the
link-scoped configured instance in 8b are correct rather than an exception. Where both bind the
same telling, both are recalled against the same `editorial_gates[]` state, and they co-bind by
conjunction: the unconfirmed figure is withheld on a path if the flag stands or if the path's gate
does not permit. Any one hold means held, so nothing has to be arbitrated between the two.

---

## 2. Recall advert

The advert is declared in the frontmatter above so a tool can read it without parsing prose.
This section explains it in words for a human.

- **Who it is for:** any tool that can be the last thing between a held asset and an audience. A MAM can keep an asset uncommittable, a playout system can refuse to take it to air, a rundown can refuse to accept it into a running order, a digital CMS can refuse to publish it on a path, and a compliance hub can show that the hold is standing and who could lift it. Five tool types, one advert, because the hold is the same fact in all five and only the blocked action differs.
- **What causes a look:** a message carrying story or asset context, a message carrying a compliance assertion, and a raised warning. A clearance is itself an assertion, so the same topics that bring the hold into being also bring the release. `skill.warning.raised` is listed because a warning raised by some other skill is one of the things a flag can arrive behind: a raise skill declares a mismatch or a match, the warning goes on the bus, and a fresh look at gate state is taken here. That look finds a hold only if the raised warning has been materialised into an `editorial_gates[]` entry by something outside both files, which is the gap recorded in section 10. Listing the topic makes the chain mechanically possible; it does not close the gap.
- **What it depends on:** an entry in `editorial_gates[]` whose `gate_type` equals the configured value, with `status` reading `PENDING`. Those two compose with AND. The third condition is a field-change condition on the same `status`, and it is not part of that AND: it exists so that a status differing from the one the tool is carrying causes a fresh look at an open hold rather than leaving the hold standing on stale state. `state_path` names that same field, because a message that does not carry the flag's status cannot decide a hold that is already open.

Two things about this advert are worth naming rather than leaving to be discovered.

The first is that `depends` is empty, and deliberately so. This skill advertises against flag state, whoever declared it. It does not depend on `flag-on-mismatch`, `raise-flag-on-match`, or any other raise skill, and it must not, for two reasons. Depending on the raise skill would invert the dependency, since the raise skill is the thing that reads nothing and the hold skill is the thing that reads it, and it would create a cycle the first time either one changed version. It would also make the hold narrower than the fact it enforces: a court reporting restriction typed in by a lawyer, imported from a wire, or written by a skill nobody in this library wrote must hold exactly the same way. Chaining happens through the bus, not through declared dependencies.

The second is that the condition value is config-templated (`{{ config.gate_type }}`) even though the path is literal. The lookup table therefore cannot index this skill by the value it tests, only by the field. Proposal A settled that on 18 August: the house registers one advert row per configured instance, in its own registration layer. Section 10 records what that costs.

---

## 3. Firing anchor

- **Evidential position:** any. A restriction does not weaken with evidential position. A reporting restriction attached to a tertiary illustration binds exactly as hard as one attached to the primary package, so nothing here is filtered on `evidential_position`.
- **Outlet / path:** whoever owns the blocked action. Where the flag binds the story everywhere, the anchor is the story's own path and every outlet reads the same hold. Where the flag binds one destination, the anchor is that destination's `link`, and the other destinations are not held.
- **Scope:** `story:<id>` when the flag binds at system level, so no path may take the action; `link:<id>` when the flag binds one destination, so that path is held and the rest are not. The configured `scope` decides which, and the scope on the warning must match the scope the clearance is asserted against.
- **Compliance question, in one plain sentence:** "If this were released now, would something still under a standing restriction reach an audience?"

---

## 4. Config surface (layer 2)

Every editorial choice sits here, set by the publishing house. Nothing here is in the skill.
Scope runs broadcaster, then site, then brand, the more specific overriding the default.

| Field | Required | Notes |
|---|---|---|
| `gate_type` | yes | The value in `editorial_gates[].gate_type` this configured instance watches for. `COURT_REPORTING_RESTRICTION`, `FLAGGED_FIGURE`, `SPECIES_LOCATION_EMBARGO`. A string the house owns, never a value this skill enumerates. Renamed from `flag_type` in 0.2.0 to match the schema field it reads |
| `blocked_action` | yes | What is withheld while the flag stands: `air`, `publish`, `commit`, `distribute`. This becomes the entries in `blocks` on the warning, and it is what an executor translates into its own refusal |
| `scope` | yes | `story` or `link`. Decides whether the hold binds everywhere or on one destination only, and decides the scope a clearance must be asserted against to count |
| `clears_by` | yes | The clearance rule in the house's own words, for example "equal-or-higher authority, same scope". Displayed in `detail` so the room can see what would lift the hold without asking anyone |
| `authority_scale` | yes | The house's own ordered list of authority levels, most senior first, used to decide whether a clearing assertion is of equal or higher authority than the declaring one. This field exists because the model does not define how authority is compared. See section 10 |
| `instance_label` | yes | the editorial label for this configured instance |

The house also declares this skill in its own active set. Those values are copied straight from
the frontmatter above and are not a second set of choices: `skill_id`, `skill_version`,
`skill_type`, `disclosure_level`, `migration_policy`.

---

## 5. Runtime loop, specialised

```
on a message carrying story or asset context, a compliance assertion, or a raised warning:
  1. RESOLVE ANCHOR   the asset or action the message names, and the scope it sits at
  2. GATE             exit unless some editorial_gates[] entry at the configured scope
                      carries the configured gate_type. No entry of that type, no evaluation
  3. READ CONFIG      load the configured instance for this broadcaster / site / brand
  4. EVALUATE         is that entry still standing, and has any clearance assertion on the
                      same scope been made by an authority of equal or higher rank on
                      authority_scale than the authority that declared it
  5. DECIDE SEVERITY  hold, always. There is no lesser reading of a standing restriction,
                      which is why severity_range has one value
  6. EMIT             the twelve-field warning, causation_id on the triggering snapshot,
                      blocks carrying the configured blocked_action against this scope,
                      non_overridable true
  7. AWAIT CLEARANCE  a clearance assertion of equal or higher authority, on the same scope,
                      against this flag
  8. RESOLVE / CLOSE  the warning closes when the flag no longer stands. The withheld action
                      becomes available again. Nothing is published as a consequence of
                      closing: releasing a hold is not an instruction to release the asset
```

**Skill specific traps:**

- The mistake a well-meaning vendor will make is to render the hold as a warning banner with a "publish anyway" control beside it. `non_overridable` is true. The whole value of the hold is that the restricted package is uncommittable while the flag stands, not that someone is reminded about it. A hold a producer can click past is the manual practice this replaces.
- The second mistake is to resolve the hold by changing something: stripping the restricted field, trimming the clip, or deleting the asset. `auto_change_content` is false. What is withheld is an action, never the content, and an executor that edits to release has broken the guarantee the whole model rests on.
- What must go in `detail` for the message to be actionable: the flag type, the authority that declared it, the scope it binds at, the action being withheld, and in plain words what would clear it. "Held off air: court reporting restriction declared by legal at story scope. Clears on a clearance assertion at story scope by legal or above." A hold that does not say what lifts it produces a phone call, which is exactly the manual step this exists to remove.
- Fail closed: an unreadable configuration, an unreadable `editorial_gates[].status`, or an `authority_scale` that does not contain the declaring authority is all treated as the flag standing. The hold is declared, and `detail` says which value could not be read. Silence on an unreadable restriction is the one outcome that cannot be allowed.
- What does NOT clear it: the passage of time; the flagged entry simply not appearing in a later snapshot; a clearance asserted at a different scope, so a `link` clearance never lifts a `story` hold; a clearance by an authority below the declaring one on `authority_scale`; an executor restart; and a second flag of a different type being cleared. Each of those is a way a hold has been lost in practice, and each is written into the eval set.

---

## 6. Output contract

`skill.warning.raised`, severity `hold`.

```json
{
  "topic": "som.skill.warning.raised",
  "originating_system": { "system_id": "<tool>", "system_type": "<the tool's own type, one this skill's advert targets>" },
  "correlation_id": "<the story's chain>",
  "causation_id": "<the snapshot that triggered evaluation>",
  "timestamp": "<on the envelope, never in the payload>",
  "payload": {
    "warning_id": "wrn-8f21c4",
    "skill_id": "smart-stories/hold-while-flagged",
    "skill_version": "0.2.2",
    "story_id": "story-verdict-0412",
    "scope": "story:story-verdict-0412",
    "severity": "hold",
    "rule_id": "house-reporting-restriction-hold",
    "non_overridable": true,
    "affected_fields": ["editorial_gates[].gate_type", "editorial_gates[].status", "compliance[].status"],
    "detail": "Held off air. Court reporting restriction (court-reporting-restriction) declared by legal at story scope, status standing. Clears on a clearance assertion at story scope by legal or above on the house authority scale. Not cleared by time, by a link-scope clearance, or by the flag disappearing from a later snapshot.",
    "blocks": ["air:story-verdict-0412", "commit:asset-pkg-guilty-cut"],
    "skill_warning_ref": null
  }
}
```

`blocks` is populated because this is a hold, and it names the configured `blocked_action` against
the scope it binds at, so an executor knows what to refuse without interpreting prose.
`non_overridable` is true for the same reason.

---

## 7. Priority position

| Axis | This skill |
|---|---|
| X category | `compliance`, so X = 1 |
| Y type / source | `reference`, so Y = 5 |
| Z precedence | unset. The publishing house sets it, nobody else |

This skill contests nothing, and no claim is made here that it outranks any other skill. X and Y
decide only which skill an executor follows when two skills clash on the same action, and a hold
is not a claim on that contest. Enforcement is the separate question, and it is answered by
conjunction: where several gates bind the same fact on the same telling, the value is served only
when every binding gate permits, so any one hold means held. This skill contributes one such gate.
Multiple binding rules from different categories are normal rather than an anomaly, and a hold
declared here standing alongside a hold declared by `gate-by-scope` off the same gate state needs
no arbitration between them, because both must permit before anything is served.

---

## 8. Worked configurations

### 8a. Court reporting restriction: the prepared hold

```yaml
instance_label: house-reporting-restriction-hold
gate_type: COURT_REPORTING_RESTRICTION
blocked_action: air
scope: story
clears_by: equal-or-higher authority, same scope
authority_scale: [regulator, legal, duty-editor, producer]
```

What this produces: both verdict packages are built before the court rules, and the one carrying
the restricted detail is uncommittable in the MAM and unairable in playout while the flag stands,
so it cannot reach a rundown by anyone's mistake.

### 8b. Unconfirmed figure: the hold on one publication path

```yaml
instance_label: house-unconfirmed-figure-hold
gate_type: FLAGGED_FIGURE
blocked_action: publish
scope: link
clears_by: equal-or-higher authority, same link
authority_scale: [legal, duty-editor, output-editor, producer]
```

What this produces: the confirmed hurricane category publishes everywhere, and the unconfirmed
casualty figure is held on the destination whose gate has not cleared, so the same story goes out
differently on each path without anyone remembering to hold the figure per outlet.

What changed between 8a and 8b: the flag type, the blocked action (`air` against `publish`), the
authority scale the house supplies, and, most importantly, the scope coordinate. 8a binds at
`story:<id>`, so no path may take the action and one clearance lifts it for everybody. 8b binds at
`link:<id>`, so the hold exists once per destination, each destination decides for itself at the
point it publishes against the same story state, and a clearance on one link lifts nothing on
another. What stayed the same: the mechanism, the condition it reads, the clearance rule, the
severity, `non_overridable`, and the fact that the skill file itself is identical in both. Only the
configured instance differs.

---

## 9. Eval set

Minimum five. The skill is not finished until a declaring case and a clearance case both pass.

| Case | Input | Expected |
|---|---|---|
| declaring | An `editorial_gates[]` entry with `gate_type` matching the config, `status` reading `PENDING`, at the configured scope | one `skill.warning.raised`, severity `hold`, `non_overridable` true, `blocks` naming the configured action against that scope, and `detail` carrying both the flag type and the clearance rule |
| non declaring | The story carries gates, but none whose `gate_type` matches the config, or one that matches at `link` scope while the configured instance is set to `story` | nothing at all on the bus. The gate step exits quietly and the case asserts that no warning was produced |
| clearance | A clearance assertion against the same flag, at the same scope, by an authority equal to or above the declaring one on `authority_scale` | the flag no longer stands, the warning closes, the withheld action becomes available. No content is released as a side effect of closing |
| wrong clearance | A clearance assertion by an authority below the declaring one, or at `link` scope against a `story`-scope hold, or against a different flag on the same story | not applied. The hold stands, `blocks` is unchanged, and the rejected attempt remains visible in `compliance[]` with its own provenance so the room can see who tried and why it did not count |
| idempotency | The same snapshot delivered twice, in both clouds, and the same standing flag re-read on a later unrelated message | one warning, keyed on the flag entry and the scope, not one per message. Re-reading a hold that is already open updates nothing |
| fail closed | Config missing, `authority_scale` absent, or `editorial_gates[].status` unreadable | `hold`, the loudest and only severity this skill has, with `detail` naming the value that could not be read. Never silence, and never a downgrade to a flag |
| never auto-change | Any declaring case, plus the clearance case | assert that no message produced under this skill changed, trimmed, redacted or deleted any content. The only thing withheld is an action |

---

## 10. Open items

- **How authority is compared is not defined anywhere, and this skill turns on it.** The rule adopted throughout this library is that a flag clears only by an assertion of equal or higher authority on the same scope. That is this library's working position, not a rule any group has ratified, and no vendor should read it as settled or hard-code enforcement against it. How two authorities are compared is not defined anywhere either, so the authority value is a house-local string and the executor's comparison of it is house-local too. Nothing in the schema orders `legal` above `duty-editor`, nothing says whether authority is a property of a person, a role, a system or an assertion, and nothing says what happens when the declaring authority is not on the comparing house's scale at all. What was assumed here: authority is a role identifier carried on the assertion's provenance, ranked against an ordered list the house supplies as `authority_scale`, most senior first, with an unlisted or unresolvable authority treated as lower than the declaring one so the hold stands. That assumption is doing real work, and it is local. If the group defines a comparison rule, `authority_scale` should be deleted from the config surface and the rule read from wherever the group puts it, every house's configured instances would need re-checking against the new ordering, and the wrong clearance eval row in section 9 would need rewriting against whatever comparison lands. This is the first item because it is the crux of the skill: everything else here degrades gracefully, and this one silently changes who can lift a hold.
- **Two readings of what releases a hold stand in the library at once, and this file does not resolve them.** This file's reading is that a hold releases only on a clearance assertion of equal or higher authority on the same scope, which is why the passage of time stands in section 5's list of things that do not clear it. `gate-by-scope` carries the other reading in its worked configuration 8a, a watershed hold whose entire release condition is a time of day: before 21:00 the linear path's gate does not read `permitted`, at 21:00 it does, and the withheld sequence releases on the `field_change` alone with nobody asserting anything and nobody at the transmission tool doing anything, which that file's eval set records as designed behaviour rather than as a defect. Those two cannot both be right as written. Nothing is settled here, and no reader should take this line as having settled it: declaring a disagreement is the whole of what this item does, because settling a contradiction in the files before the group has agreed is exactly how the previous one shipped. What is agreed underneath both readings, and is not in question anywhere: clearing a flag is a human editorial act, and no person clears their own flag. What is open is narrower than the disagreement looks. It is whether taking the hold off is a separate act needing its own authority, or the mechanical consequence of the flag having gone, which anything reading the condition may observe. Under the first answer this file stands exactly as written, and the watershed configuration needs an account of how a clock reaching 21:00 comes to count as an assertion by an authority, which is written nowhere. Under the second answer this file changes in two places: the "what does NOT clear it" list in section 5 loses the passage-of-time entry, since a condition that reads a clock is cleared by the clock, and the wrong clearance row in section 9 narrows to the human act, asserting only that a person of insufficient authority cannot clear the flag and no longer that the hold cannot come off without a person. Until the group answers, a reader of both files should treat this as declared and not as decided.
- **A flag a raise skill declares reaches `editorial_gates[]` only if something materialises it, and nobody has said what.** `flag-on-mismatch` and `raise-flag-on-match` each produce exactly one output, `skill.warning.raised`, with `blocks` empty, and `auto_change_content: false` forbids either of them from writing story state. This skill reads `editorial_gates[]`, so the chain described here completes only if a raised warning is turned into an `editorial_gates[]` entry by something that is in neither file: the executor that recalled the raise skill and acted on its warning, or the bus itself as a property of carrying the warning. What was assumed here: the executor does it, as part of acting on a warning it recalled, and `skill.warning.raised` is carried in `recall_on` so that a look is taken on the warning as well as on story context, asset context and compliance assertions. If the group decides the bus materialises gate state instead, the conditions and `recall_on` in this file are unaffected, but the assumption stated in section 2 changes and section 11 gains a step for every executor that currently has none. If the group decides that neither does it, and a raised warning is never materialised into gate state, then for flags this library raises the chain does not exist at all: this skill still holds correctly against a restriction typed in by a lawyer or arriving from a wire, and the flag-and-hold chain would need a mechanism nobody has written. Nothing is invented here past that gap, and no file in the library should be read as having closed it.
- **`depends` is empty on purpose, and a reviewer expecting `flag-on-mismatch` there should read this line.** This skill advertises against flag state, whoever declared it, not against a particular raise skill. Naming `flag-on-mismatch` or `raise-flag-on-match` in `depends` would invert the dependency, since the reader would be declaring a dependency on the thing that reads nothing, and it would create a cycle at the first version change. It would also narrow the hold to flags this library raised, when a restriction typed in by a lawyer or arriving from a wire must hold identically. Chaining happens over the bus. Nothing is lost by the empty list except the illusion of a declared contract.
- **This skill supplies the hold half of the "flag and hold" behaviour that is usually described as a single step.** That description reads as one action because that is how it appears from outside. It is authored as two skills because a skill that both declares a state and withholds something occupies two positions and cannot be resolved at a single coordinate. `flag-on-mismatch` declares the mismatch at `editorial`, this skill withholds against the resulting flag at `compliance`, and the two meet on the bus rather than inside one file. The experience in use is identical; the library keeps two resolvable skills instead of one unresolvable one. Anyone looking for a single skill should find this note rather than conclude something is missing.
- **An unresolved second count is delegated here, and to `gate-by-scope`, by `apply-clearance`.** That skill resolves which package is current and withholds nothing at all, so the count that has not cleared stays held because a gate of its own still stands, read here off the same `editorial_gates[]` state and reached over the bus rather than by declaration, and anyone looking for that hold inside `apply-clearance` should find this line instead.
- **`disclosure_level: L2` means the deepest level for which this file declares content**, that is, frontmatter and body through the config surface, with vendor build notes as the third layer. Two definitions of the field are in circulation, the other being how much of its reasoning a skill reveals. The other reading was not intended here, and if the group settles on it this value needs revisiting rather than reinterpreting.
- **Whether `target_system_type` binds or merely advises is open, and it matters more for this skill than for most.** The advert is written as though it binds, per the working position. `compliance_hub` is listed explicitly for exactly that reason: under a binding reading, an oversight tool that is not named cannot pick the hold up, and a hold nobody can see is close to a hold that does not exist. Under an advisory reading the target list would be a hint, off-target tools such as an audit desk or an oversight function could pick the hold up anyway, and the explicit inclusion would be harmless but unnecessary. Whether `compliance_hub` is a member of the deployment's v0.3 `system_type` enum under that spelling should be checked before this file is registered; the broad type is used deliberately rather than a named product, since naming a product is a bet on the second open targeting question.
- **The emitting executor carries the tool's own `system_type`, and the `skill_worker` type this library used to emit here is withdrawn.** An executor is a sidecar to a vendor tool, and no newsroom-scoped class of skill-running system exists, so the thing that puts this skill's warning on the bus is a playout system, a MAM, a rundown, a CMS or a compliance hub, and section 6 now says so. The sample payload previously carried a `skill_worker` system type, a value whose only reason to exist was that the emitter ran skills, which is the newsroom-executor shape under another name. It is withdrawn rather than reinterpreted, and no file in this library should be read as having such a class. What is not settled is whether the emitting tool's own type is the right value for `originating_system.system_type` on a skill warning at all, as against some value naming the executor role that raised it. The schema does not answer that, and the position taken here follows from a decision taken outside the schema, that tools have executors and newsrooms do not, which the schema has not caught up with. Worth naming plainly: this is one of the two places this file names a system type, the other being the advert's `target_system_type`, and after this change the two agree by construction, because the tool that may recall this skill is the tool that emits on its behalf.
- **The condition value is config-templated, so the lookup table cannot index this skill by the value it tests.** The path is literal, which is better than the fully templated case, but a table indexing on `editorial_gates[].gate_type` will recall this skill for every gate type and let the configured instance decide. Proposal A settled that on 18 August: the house registers one advert row per configured instance, in its own registration layer. Stated in section 2 as well.
- **A restriction carried by a related asset needs a relationship condition, and there is no such condition kind available.** A clip cut from a held package, linked by `derived_from`, is the obvious next case, and it is not expressible with `field` and `field_change` alone. Nothing here approximates it: this skill holds only what carries the flag itself. If the relationship or cascade condition returns, this is the skill that should take it, one hop at a time.
- **`recall_on` is a proposed name, not an agreed one**, and of the four topics listed only `story.context` has a worked precedent. `asset.context` and `compliance.assertion` are plausible rather than ratified, and should be reconciled with whatever the bus actually publishes before registration. `skill.warning.raised` is the one output topic with a ratified payload, so the name is safe, but whether a warning raised by one executor is republished on a topic a second executor takes a look on is a bus question this file cannot answer, and it is part of the materialisation gap recorded above.
- **Fail-closed reads cleanly here and that is worth saying, because it does not everywhere.** At the skill layer, fail closed means an unreadable value is treated as the condition holding, which produces the loudest severity the skill has. This skill has exactly one severity, so the reading is unambiguous. Safe state at the executor layer, where nothing can be resolved at a coordinate and the executor does not act, is a different mechanism at a different layer and is not what `fail_closed: true` means here.
- **`skill_uri` is left as a placeholder** because it is not this author's to supply and because whether it is required at all is unsettled: one position treats it as mandatory on every registered skill, the other as optional. A plausible-looking invented URI reads as a real one to the next person. `owner_editorial`, `owner_engineering`, `licence` and `skill_content_sha` are left for the same reason, the last being computed at build rather than typed.
- **The `skill_id` format is one of four in circulation.** Nothing agreed so far picks one, and `<publisher>/<name>` is used because it is the form with a worked precedent in the warning schema's own example, not because the question is settled. If the group picks another, this changes.

---

## 11. Vendor build notes (layer 3)

| Platform | What your executor does |
|---|---|
| MAM | Holds both prepared sets by reference and keeps the flagged one uncommittable. The asset stays `CAPTURED` and fully browsable, and it is committed nowhere: no export, no send-to-playout, no publish target accepts it while the flag stands. Your executor watches for the gate entry, recalls this skill, and applies its own uncommittable state. It does not move, trim or version the asset to achieve that |
| Playout | Refuses the take. A held item may sit in the schedule and may be previewed, and it will not go to air. Your executor reads the hold against the live context at the point of transmission, not once at load time, because a restriction can be declared after an item is loaded and a clearance can arrive seconds before transmission |
| Rundown | Refuses acceptance. This is the earliest useful refusal: the point is that the restricted package never reaches a running order at all, so a rundown that accepts the item and marks it red has done less than a rundown that declines it. Show the reason and who could clear it, on the item |
| Digital CMS | Holds per path. Where `scope` is `link`, your executor decides for its own destination, at the point it publishes, against the same story state every other destination is reading. The rest of the story publishes normally. Do not resolve the hold by publishing a version with the flagged element removed: that is a content change, and it is not yours to make |
| Compliance hub | Displays the standing hold, the flag type, the declaring authority, the scope, the blocked action, and what would clear it. Also displays rejected clearance attempts, because a lower authority trying and failing is itself a thing an auditor needs to see. Your executor withholds nothing here; it is the window on the fact |

The instinct this skill exists to interrupt is the belief that a restriction is safe once a
competent person has been told about it. Today the held figure lives in someone's head across
every outlet, and the prepared restricted package is held off air by a producer knowing not to
play it. Both work until one shift change, one gap between producers, one busy gallery, and
then one slip puts it out. The restriction sits on the asset instead, and the asset is held
structurally, so remembering is no longer part of the mechanism.
