---
# --- identity ----------------------------------------------------------------
skill_id: smart-stories/apply-clearance
name: apply-clearance
skill_version: 0.2.2
som_schema_version: ["1.0.0"]

# --- what it is --------------------------------------------------------------
description: >-
  Declares, on a clearance assertion of a configured type, which member of a
  configured version set that clearance resolves to current and which other
  members of the same set it supersedes, and displays that resolution so every
  subscribing tool reads it off the one event. It declares only. It does not
  float, unfloat, publish, transmit or version anything itself, it never changes
  content of any kind, it withholds nothing and raises no hold, and it never
  treats a clearance on one set as a resolution of another.
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
  target_system_type: [playout, rundown, mam, cms, graphics]
  target_capability: null
  conditions:
    - kind: field
      path: compliance[].type
      op: equals
      value: "{{ config.clearance_type }}"
    - kind: field
      path: compliance[].status
      op: equals
      value: RESOLVED
    - kind: field_change
      path: compliance[].status
      op: equals
      value: RESOLVED
  recall_on: ["compliance.assertion", "story.context", "asset.context"]
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

# apply-clearance

**On a clearance of this type, declare which member of the named set that one event resolves to
current and which members it supersedes, and display it so every tool reads the same resolution
off the same event. Do not float it, do not publish it, do not change a frame of it.**

Read `../docs/CONVENTIONS.md` first.
**Changed in 0.2.1, 20 August 2026.** Four things, all found by a review of the 0.2.0 pack. The section 7 config row said the watched value lives at compliance[].clearance_type; the schema carries `compliance[].type` and the advert always tested that, so the row was describing a field that does not exist. Section 2 and section 10 still printed the registration question as open after proposal A had closed it; both now say what A decided and what it costs. The version-status note in section 10 said the author did not know whether v0.3.2 carries version fields: it has now been checked and it does not, so the note states that as fact. And the second reuse scenario is rewritten around a departure rather than a death, per a standing instruction that the earlier subject stays out of consortium and vendor material.

**Changed in 0.2.0, 18 August 2026.** Three things. `som_schema_version` moves to `["0.3.2"]`, which as of 18 August is the version. Field paths in the advert and in the worked configurations were checked against the v0.3.2 story-context JSON Schema for the first time and the ones that did not resolve are corrected; see the pack's `SCHEMA-CONFORMANCE.md` for the full list and the evidence. And the shared policy index was chosen over executor-local policy, which settles the registration question this file used to leave open: the house registers one advert row per configured instance.

---

## 1. What it is (layer 1)

| | |
|---|---|
| Operation | Declare, and display, the version resolution that one clearance carries: which member of a version set it names current, and which members of that same set it thereby supersedes |
| Condition type | A clearance assertion of a configured type, active on a configured scope, naming a member of a configured version set |
| One mechanism? | Yes, and the reasoning matters more here than anywhere else in the library, because the shortest description of this skill, that it sets the cleared package current, reads as acting. It is not acting. What is declared here is which package the clearance resolves to current and which alternates it supersedes; the floating, the unfloating and the flipping of version state are the executor's acts, in a vendor runtime, read off this declaration. Nothing produced here changes content, which is why `auto_change_content: false` holds without strain: version status is not content. The cleared package is byte for byte what it was before the clearance, the superseded ones are untouched, and what changed is which of them the room and the tools now hold to be current. On the "and" test: naming the winner and naming the losers is one declaration, not two, because a clearance resolves a set. There is no state of the world in which one member is current and the others are undecided, so splitting the sentence would split a single fact across two skills that would then have to be kept in step. A third thing often configured alongside this one, holding the unresolved count, is genuinely not this skill: that count stays held because its own gate has not cleared, which is `hold-while-flagged` and `gate-by-scope` territory, reached over the bus. `severity_range` is `[inform]` only and this skill withholds nothing at all. X is `workflow` because this governs how a story moves through approval and version resolution rather than whether a fact may lawfully be told, and that choice is defensible against `compliance`: the restriction that kept both packages off air before the clearance was the compliance question, and this is the operational resolution that follows it |

**Reuse test, three scenarios from different stories:**

1. **A verdict with staggered counts, live at transmission.** Two packages were built before the court ruled and both were held. The first count comes in, one clearance is asserted in court, and at that instant the correct package is declared current and its alternate superseded, everywhere at once, while the second count stays held by its own gate.
2. **A prepared package set for a long-serving public figure's departure, held for years and resolving on confirmation.** A newsroom holds three prepared packages: a career retrospective, a shorter news version, and a version written for a departure under pressure. Confirmation arrives from the authority the house has configured, one clearance names the version the circumstances call for, and the other two are superseded before anyone opens a rundown. The tempo is nothing like a verdict resolving at transmission, the mechanism is identical.
3. **A medicines regulator's licensing decision on a new treatment, a kind of story this library has not previously drawn.** Three explainer packages are cut in advance against the three outcomes the agency can reach: licensed, licensed with restrictions, refused. Each carries its own graphics set, its own patient-group reaction and its own digital explainer. The agency publishes at a stated hour, one clearance names the outcome, and the matching package plus its graphics set is declared current while the two unused sets are superseded in the same declaration. No court, no death, no live gallery, no breaking event, a three-way rather than two-way set, and an authority that is neither editorial nor legal but a regulator outside the newsroom entirely.

One skill, three subjects, three configured clearance types and three different version sets. The mechanism knows nothing about courts, careers or medicines: it knows a set, a clearance and a scope.

---

## 2. Recall advert

The advert is declared in the frontmatter above so a tool can read it without parsing prose.
This section explains it in words for a human.

- **Who it is for:** every tool that carries a notion of which version is the one to use. `playout` decides what goes to air at the moment of transmission, `rundown` decides what is floated into the live running order and what stays unfloated, `mam` carries the version state that every downstream tool resolves against, `cms` decides which package a digital path serves, and `graphics` decides which prepared set is the live one. Five broad types, one advert, because the resolution is the same fact in all five and only the act that follows it differs. Broad types are used deliberately rather than named products.
- **What wakes it:** `compliance.assertion`, which is the topic a clearance arrives on, and `story.context` and `asset.context`, which are the topics on which a clearance already asserted becomes visible to a tool that was not listening at the instant it landed. A tool joining late, or re-reading the story, reaches the same resolution from the same event.
- **What it depends on:** an entry in `compliance[]` whose `type` equals the configured value, with `status` reading `RESOLVED`. Those two compose with AND. The third condition is a field-change condition on that same `status`, and it is not part of the AND: it exists so that a clearance status differing from the one the tool is carrying causes a fresh look, which is what makes the staggered case work when a second count clears its own set minutes later.

Three things about this advert are worth naming rather than leaving to be discovered.

The first is that `state_path` is `null`. That field names the state a message must carry to decide an open hold, and this skill holds nothing: a resolution is a standing declaration, not something kept open pending an outcome. A later clearance is brought back for a fresh look by the field-change condition, not by a message carrying hold state.

The second is that the condition value is config-templated (`{{ config.clearance_type }}`) while the path is literal. The lookup table can therefore index this skill by the field but not by the value it tests, and a table indexing on `compliance[].type` will recall it for every clearance type and let the configured instance decide. Proposal A settled which of those it is on 18 August: the house registers one advert row per configured instance, in its own registration layer. Section 10 records what that costs.

The third is that `depends` is empty, and a reader who expects a dependency here should stop and read this paragraph. Nothing is depended on in either direction. What floats, unfloats or flips version state is the executor's act, not another skill's output, so there is nothing upstream to name. The audit trail over a resolution, the complete record of who cleared what, when, and what was suppressed, assembles itself from `som.system.audit` and from what the bus and the tools already put on it; it is infrastructure, not a skill, and it is certainly not a dependency this skill declares. Declaring one would invert the direction, since the audit reads this and not the other way round, and it would create a cycle at the first version change. Chaining happens over the bus.

---

## 3. Firing anchor

- **Evidential position:** any. A clearance does not resolve differently according to how strong the evidence for a package is. The prepared alternate that turns out to be right is often the thinner cut, and filtering on `evidential_position` here would mean a resolution that skipped the package the clearance actually named.
- **Outlet / path:** the path that owns the version decision. Where the resolution binds everywhere, which is the ordinary case and the whole point of declaring it once, the anchor is the story's own path and every outlet reads one declaration. Where a house resolves versions per destination, the anchor is that destination's `link` and other destinations are untouched.
- **Scope:** `story:<id>` when the resolution binds at system level, so every tool resolves the same member current; `link:<id>` when a house has configured version resolution per destination. The configured `scope` decides which, and the scope on the declaration must match the scope the clearance was asserted against.
- **Compliance question, in one plain sentence:** "If this went out now, is the version every tool is holding as current the one the clearance actually named?"

---

## 4. Config surface (layer 2)

Every editorial choice sits here, set by the publishing house. Nothing here is in the skill.
Scope runs broadcaster, then site, then brand, the more specific overriding the default.

| Field | Required | Notes |
|---|---|---|
| `clearance_type` | yes | The value in `compliance[].type` this configured instance watches for: `verdict-count-clearance`, `departure-confirmation`, `licensing-decision`. A string the house owns, never a value this skill enumerates. The config key is named for what it means to the house; the field it is tested against is `compliance[].type`, which is what the schema carries |
| `version_set_field` | yes | The field that groups the alternate packages into one set, so the members are resolvable from the story rather than listed in the skill. A shared group identifier carried on each member. Which field this actually is depends on the deployment's schema: see section 10 |
| `current_status` | yes | The status word the house uses for the member the clearance names, conventionally `current`. Declared, never written by this skill |
| `superseded_status` | yes | The status word the house uses for the other members of the same set, conventionally `superseded` |
| `scope` | yes | `story` or `link`. Decides whether the resolution binds everywhere or on one destination, and decides the scope a clearance must be asserted against to count |
| `clearing_authority` | yes | The lowest authority whose clearance this configured instance honours, on the same scope |
| `authority_scale` | yes | The house's own ordered list of authority levels, most senior first, used to decide whether a clearing assertion is of equal or higher authority. This field exists because the model does not define how authority is compared. See section 10 |
| `re_resolution` | yes | What a later clearance naming a different member of the same set means: `equal-or-higher` (a new resolution, honoured only from an authority at or above the one that made the standing resolution) or `ignore` (the first clearance is final). The default assumed here is `equal-or-higher`. See section 10 |
| `instance_label` | yes | the editorial label for this configured instance |

The house also declares this skill in its own active set. Those values are copied straight from
the frontmatter above and are not a second set of choices: `skill_id`, `skill_version`,
`skill_type`, `disclosure_level`, `migration_policy`.

---

## 5. Runtime loop, specialised

```
on a compliance assertion, or on story or asset context carrying one:
  1. RESOLVE ANCHOR   the version set on this story identified by config.version_set_field,
                      and the scope the clearance is asserted against
  2. GATE             exit unless the message carries a compliance[] entry whose
                      compliance[].type equals the configured value, status RESOLVED, on the
                      configured scope. Every other message in a running newsroom stops here,
                      which is most of them
  3. READ CONFIG      load the configured instance for this broadcaster / site / brand
  4. EVALUATE         read the whole set in one pass off this one event: the member the
                      clearance names, and every other member carrying the same group value.
                      The resolution is a function of that single clearance and the set
                      membership it names, and of nothing else. No second tool is consulted,
                      no other executor's state is read, and nothing is negotiated between
                      tools. Every subscriber evaluating the same event against the same set
                      reaches the same resolution independently, which is what makes it the
                      same instant everywhere at once
  5. DECIDE SEVERITY  inform, always. Nothing is withheld and nothing is contested, so there
                      is no lesser or louder reading. This is why severity_range has one value
  6. EMIT             the twelve-field warning, causation_id on the triggering snapshot,
                      blocks empty, non_overridable false, keyed on the clearance event
  7. AWAIT CLEARANCE  nothing is awaited. This declaration is made on a clearance rather than
                      held pending one. What can arrive later is a further clearance naming a
                      different member of the same set, handled under config.re_resolution
  8. RESOLVE / CLOSE  the declaration stands as the resolution of that set. It is replaced only
                      by a valid re-resolution, and the previous resolution stays visible in
                      history rather than being erased. A clearance on a different set on the
                      same story closes nothing here and resolves nothing here
```

**Skill specific traps:**

- The mistake a well-meaning vendor will make is to read the resolution as an instruction to publish. It is not one. Declaring a package current says which version is the one to use; it says nothing about whether that version may be served. The cleared package still passes every binding gate on the same telling, and any one hold standing against it still means held. An executor that takes a resolution as a release has invented a clearance nobody asserted.
- The second mistake is to reach the resolution by asking another tool. The whole value of the mechanism is that the rundown, the MAM, the graphics store and the digital path each read the same clearance event and move together, with nothing point to point between them. An executor that waits for the MAM to flip before it floats has rebuilt by hand the racing and hoping this replaces, and it will be the slow one at transmission.
- The third mistake is to resolve by changing something: re-rendering the cleared package, trimming the superseded one, or deleting the alternate. `auto_change_content` is false. Version status is not content, and an executor that edits to express a resolution has broken the guarantee the whole model rests on. The superseded package must stay intact and retrievable, because the audit trail needs a record of what was suppressed, not an absence where it used to be.
- What makes the resolution idempotent is that the declaration is keyed on the clearance event's own identity together with the set identifier and the scope, never on the message that carried it, the topic it arrived on, or the executor that observed it. The same clearance seen twice, seen in both clouds, or seen again on a later `story.context` re-read, yields one resolution, because the key is the same in every case and the computed outcome is the same in every case. This matters more here than in most of the library: a double resolution at the moment of transmission, one tool flipping twice while another flips once, is exactly the disaster this mechanism exists to rule out.
- What must go in `detail` for the message to be actionable: the clearance event, the member declared current, every member declared superseded by name, the version set, the scope, the asserting authority, and what would change the resolution. A message that says a clearance was applied without naming the losers is unusable, because the loser is the package a producer needs to be certain is not going out.
- Fail closed: if the clearance assertion cannot be read, or the set membership cannot be read in full, no resolution is declared for any member of the set. An `inform` is declared instead, which is the loudest severity this skill actually has, and `detail` names the value that could not be read and the set left unresolved. Fail closed here does not mean a hold is produced, because this skill has no hold authority; it means an unreadable value is treated as the condition holding and stated loudly rather than passed over. A partial resolution, one member declared current while the alternates could not be read and so were left not superseded, is the worst available outcome at transmission and is never declared.
- What does NOT resolve a set: the passage of time; a package simply appearing or disappearing from a later snapshot; a clearance of a different configured type; a clearance asserted at a different scope, so a `link` clearance resolves nothing at `story`; a clearance from below `clearing_authority` on `authority_scale`; and a clearance naming a member of a different version set on the same story, which is precisely the staggered second count and is a separate resolution of its own.

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
    "warning_id": "wrn-3d90a7",
    "skill_id": "smart-stories/apply-clearance",
    "skill_version": "0.2.2",
    "story_id": "story-verdict-0412",
    "scope": "story:story-verdict-0412",
    "severity": "inform",
    "rule_id": "house-verdict-count-resolution",
    "non_overridable": false,
    "affected_fields": ["assets[4].version_status", "assets[5].version_status", "compliance[3].status"],
    "detail": "Resolution declared on clearance cmp-3 (verdict-count-clearance), asserted at story scope by court-reporter. In version set vg-verdict-count-1: asset-pkg-count-guilty is current; asset-pkg-count-not-guilty is superseded. Read this resolution off this event; do not negotiate it with another tool. Nothing is released by it: gates binding this telling still apply. Changed only by a further clearance on this same set at story scope from court-reporter or above.",
    "blocks": [],
    "skill_warning_ref": null
  }
}
```

`blocks` is empty and `non_overridable` is false, permanently, in every configuration. Nothing is
withheld here, so there is nothing to list and nothing for anyone to override. `affected_fields`
names the version status field of every member of the set, winner and losers together, because the
resolution is one declaration over the whole set and a reader must be able to see the losers named.

---

## 7. Priority position

| Axis | This skill |
|---|---|
| X category | `workflow` (3) |
| Y type / source | `reference` (5) |
| Z precedence | unset. The publishing house sets it, nobody else |

This skill contests nothing and no claim is made here that it outranks any other skill. X and Y
decide only which skill an executor follows when two skills clash on the same action, and a
declaration that withholds nothing makes no claim on that contest. Enforcement is the separate
question and is answered by conjunction: where several gates bind the same fact on the same
telling, the value is served only when every binding gate permits, so any one hold means held.
This skill contributes no gate. That is the load-bearing point at transmission: declaring the cleared
package current does not lift the hold standing on the unresolved count, and it does not lift any
hold standing on the cleared package either. A resolution says which version, and the gates say
whether at all. The two co-bind without arbitration because they answer different questions.

---

## 8. Worked configurations

### 8a. A verdict with staggered counts, resolving live at transmission

```yaml
instance_label: house-verdict-count-resolution
clearance_type: verdict-count-clearance
version_set_field: extensions.com.example.version_group
current_status: current
superseded_status: superseded
scope: story
clearing_authority: court-reporter
authority_scale: [legal, duty-editor, output-editor, court-reporter]
re_resolution: equal-or-higher
```

What this produces: the first count comes in, the reporter in court asserts one clearance, and at
that instant the correct package is current in the MAM and floated into the live rundown while the
wrong one is superseded and stays unfloated, on every path at once. The second count, a different
version set, stays held by its own gate and is untouched by this resolution.

### 8b. Which edit of a recut package goes current as approvals land

```yaml
instance_label: house-recut-approval-resolution
clearance_type: recut-approval
version_set_field: extensions.com.example.version_group
current_status: current
superseded_status: superseded
scope: story
clearing_authority: compliance-editor
authority_scale: [legal, compliance-editor, output-editor, producer]
re_resolution: equal-or-higher
```

What this produces: a package is recut twice during the afternoon after a complaint, and each time
an approval lands the approved edit becomes the current one for every tool that might serve it
while the edit it replaces is superseded and stays retrievable. Nobody has to remember which cut
the digital path picked up an hour ago.

What changed between 8a and 8b: the tempo (one instant at transmission against a working
afternoon), the authority and the scale it is ranked on (a reporter in court against a compliance
editor), and the nature of the alternate set. In 8a the set is mutually exclusive outcome packages
built in advance, only one of which can ever be right, and the losers were always going to lose. In
8b the set is successive edits of the same piece, the alternates are the earlier versions, and the
set grows as the afternoon goes on rather than being complete before the first clearance. What
stayed the same: the mechanism, the advert, the one-event rule, the scope coordinate, `inform`,
the empty `blocks`, the fact that no content is changed by either configured instance, and the
file itself. Only the configured instance differs.

---

## 9. Eval set

Minimum five. The skill is not finished until a declaring case and a clearance case both pass.

| Case | Input | Expected |
|---|---|---|
| declaring | A `compliance[]` entry of type `verdict-count-clearance`, active at `cleared`, at story scope, naming `asset-pkg-count-guilty` in version set `vg-verdict-count-1`, configured instance 8a loaded | One `skill.warning.raised`, severity `inform`, `rule_id` `house-verdict-count-resolution`, `detail` carrying both the member declared current and every member declared superseded by name, plus the clearance event and the asserting authority. `blocks` empty, `non_overridable` false |
| non declaring | The story carries clearances, but none of the configured `clearance_type`, or one of the right type asserted at `link` scope while the configured instance is set for `story` | Nothing at all on the bus. The gate step exits quietly and the case asserts that no declaration was produced |
| clearance | The configured clearance asserted at story scope by `court-reporter`, which is at the configured `clearing_authority` on `authority_scale`, with provenance of who, when, scope and status | The resolution is declared and stands: the named member current, every other member of that set superseded, in one declaration off the one event |
| wrong clearance | The same clearance attempted by a gallery director who is not on `authority_scale`, or by a role below `clearing_authority`, or asserted at `link` scope against a `story`-scope configured instance, or naming a member of a different version set | Not applied. No resolution is declared and any standing resolution is unchanged. The rejected attempt stays visible in `compliance[]` with its own provenance, so the room and the audit can see who tried and why it did not count |
| idempotency | The same clearance event delivered twice, then re-read on a later unrelated `story.context`, then observed again after an executor restart | Exactly one resolution, keyed on the clearance event identity together with the set identifier and the scope. No second declaration, and no flip-back. This case is run at transmission tempo deliberately: a double resolution here is the failure this skill exists to rule out |
| fail closed | The clearance entry is present but its named subject is unreadable, or `version_set_field` is missing, or the set membership can be read only in part | `inform`, the loudest severity this skill has, with `detail` naming the value that could not be read and the set left unresolved. No member is declared current and no member is declared superseded. A partially resolved set is never declared, and silence is never the outcome |
| never auto-change | Any declaring case, and the re-resolution in the staggered case | Assert that no message produced on this skill's behalf changed, re-rendered, trimmed, moved or deleted any content. The superseded package is byte for byte what it was and is still retrievable. Only version status was declared, and only warnings were added |
| staggered clearance | The second count clears fifteen minutes later, asserting a clearance of the same configured type at story scope naming a member of a different version set on the same story | A second, separate resolution over that second set only. The first resolution is untouched, neither reopened nor superseded, and no member of the first set changes status. The two resolutions coexist on one story |
| multicloud | The same single clearance event observed independently by an executor in each of two clouds, each with its own installation of 8a | One resolution, identical in both, reached independently and not by either executor reading the other. The idempotency key is the event, not the observation, so the second observation adds nothing rather than declaring a second time |

---

## 10. Open items

- **"Set the cleared package current" reads as acting, and it is not acting. This is the scoping question this skill turns on, so it is stated here as well as in section 1.** Under the passive convention agreed on 29 July 2026, skills declare and display and executors act. What this file declares is which member of a version set a clearance resolves to current and which members it supersedes, and it displays that resolution; the floating into a live rundown, the leaving unfloated, and the flipping of version state are acts of a vendor runtime reading the declaration. `auto_change_content: false` holds without strain, and the reason is worth writing down rather than assuming: no message produced under this skill changes any content. Version status is not content. The cleared package is unchanged by being declared current, and the superseded ones are unchanged by being declared superseded, which is also why the audit trail can still show what was suppressed. If a reviewer reads the short description of this skill and concludes that it mutates something, this is the line that answers them.
- **How authority is compared is not defined anywhere, and this skill turns on it as hard as any hold does.** The rule adopted throughout this library is that a declaration closes only on an assertion of equal or higher authority on the same scope, so a clearance is honoured only from an authority at or above the one it resolves against, on that same scope. That is this library's working position, not a rule any group has ratified, and no vendor should read it as settled or hard-code enforcement against it. How two authorities are compared is not defined anywhere either, so the authority value is a house-local string and the executor's comparison of it is house-local too. Nothing in the schema orders a court reporter against an output editor, nothing says whether authority is a property of a person, a role, a system or an assertion, and nothing says what to do when the asserting authority does not appear on the comparing house's scale at all. What was assumed here: authority is a role identifier carried on the clearance's own provenance, ranked against an ordered list the house supplies as `authority_scale`, most senior first, with `clearing_authority` naming the lowest rank honoured, and an unlisted or unresolvable authority treated as below it so that no resolution is declared. That assumption is doing real work and it is local. If the group defines a comparison rule, `authority_scale` should be deleted from the config surface, `clearing_authority` read against the group's comparison rather than the house's, the rule read from wherever the group puts it, every configured instance re-checked against the new ordering, and the wrong clearance eval row in section 9 rewritten against whatever comparison lands.
- **Ordering: what a later clearance for the same set means is not settled by any of the documents, and a configuration keyed `on: first-clearance-event` does not settle it either.** A verdict whose counts resolve at staggered moments is an ordinary situation, so a later clearance arriving for a set that already has a standing resolution is a real case and not a hypothetical. Two readings are available: the first clearance is final and any later one for the same set is ignored, or a later one is a new resolution. What was assumed here: the first clearance resolves the set, and a later clearance naming a different member of the same set is a new resolution, honoured only from an authority at or above the one that made the standing resolution, with the previous resolution recorded rather than erased. The assumption is exposed as `re_resolution` in the config surface, with `equal-or-higher` as the assumed default and `ignore` available, so a house that reads it the other way can configure that without editing this file. Note that a clearance for a *different* set on the same story is not this case at all: that is the staggered second count, and it is an independent resolution, which is what the staggered eval row proves.
- **v0.3.2 carries no version-status and no version-group field on the asset, and this was checked rather than assumed.** The earlier draft of this note said the author did not know. It is now known: `asset` is `additionalProperties: false` and neither field exists under any name. So a resolution has nowhere in the story object to name, and the worked examples and `affected_fields` reach `extensions` instead, with the group field pointed there through `config.version_set_field`. That is a vendor extension standing in for a schema gap, which is workable for a demo and is not a position to build a product on. Closing it is a decision for the group, not a fix for this file: either the asset carries a version status and a version-group identifier, or this skill is configured against an extension forever and no two houses' configurations are comparable.
- **The `and` test was resolved by treating the whole clearance as one resolution, and the accompanying hold was deliberately left out of this skill.** The originating configuration does three things: set current, set alternates superseded, hold the unresolved count. The first two are one declaration, because a clearance resolves a set and there is no state in which the winner is named and the losers are not. The third is not this skill. The unresolved count stays held because its own gate has not cleared, which is `hold-while-flagged` and `gate-by-scope` reached over the bus, and folding it in here would give this skill both a declaring and a withholding half, two positions, and no single coordinate at which it could be resolved. `severity_range` is `[inform]` only and nothing here can reach `hold`. Anyone reading that configuration and looking for the hold in this file should find this note rather than conclude something is missing.
- **The name does not follow `<operation>-<condition-type>` and is kept anyway.** `apply-clearance` reads as an operation plus an event rather than an operation plus a condition type; the conformant form would be something closer to `resolve-version-on-clearance`. The library name is the contract vendors are building to, so it is retained and recorded here rather than silently fixed. This is the same treatment `match-and-propose` gets in `CONVENTIONS.md`. The word `apply` should also not be read as licence to act, for the reasons in the first item above.
- **Fail closed reads unusually here, and the reading is stated rather than left to be inferred.** The template's illustration of fail-closed is a hold, and this skill has no hold authority, so it cannot produce one. At the skill layer, fail closed means an unreadable value is treated as the condition holding, which produces the loudest severity the skill actually has: `inform`. Concretely, an unreadable clearance or an unreadable set membership means no resolution is declared for any member and the gap is stated loudly in `detail`. It does not mean a hold appears, and it does not mean a partial resolution is declared, which at transmission would be worse than either. Safe state at the executor layer, where nothing can be resolved at a coordinate and the executor does not act, is a different mechanism at a different layer and is not what `fail_closed: true` means in this file.
- **`depends` is empty on purpose.** What floats, unfloats or flips version state is the executor's act rather than another skill's output, so there is nothing upstream to declare. The audit trail over a resolution assembles itself from `som.system.audit` and from what the bus and the tools already put on it: it is infrastructure, not a skill, and naming it here would invert the direction, since the audit reads this rather than the other way round. The hold on the unresolved count is likewise not a dependency: it stands on its own gate and would stand identically if this skill were never installed.
- **`disclosure_level: L2` means the deepest level for which this file declares content**, that is, frontmatter and body through the config surface, with vendor build notes as the third layer. Two definitions of the field are in circulation, the other being how much of its reasoning a skill reveals. The other reading was not intended here, and if the group settles on it this value needs revisiting rather than reinterpreting.
- **Whether `target_system_type` binds or merely advises is open.** The advert is written as though it binds, per the working position, which is why five types are listed explicitly: under a binding reading, a tool that is not named cannot pick the resolution up, and a resolution one of the five tools cannot see is the exact failure this skill exists to remove. Under an advisory reading the list would be a hint and off-target tools such as an audit or oversight hub could read the resolution too, which for this skill would be harmless and arguably useful, since nothing here withholds anything. Broad types are used rather than named products, because naming a product is a bet on the second open targeting question. Whether `cms`, `graphics` and `rundown` are members of the deployment's v0.3 `system_type` enum under those spellings should be checked before registration.
- **The system that emits on this skill's behalf carries the tool's own system type, and `skill_worker` is withdrawn.** An executor is a sidecar to a vendor tool, and there is no newsroom-scoped class of skill-running system for one to belong to, so the executor that puts this declaration on the bus is the playout, rundown, MAM, CMS or graphics tool's own executor, and it names that tool's own type in `originating_system.system_type`. Earlier drafts of this library emitted `skill_worker` in the section 6 payload, a value whose only meaning was "a thing that runs skills", which implied precisely the class that does not exist; it is withdrawn, and a playout tool emitting as `skill_worker` had erased its own type in order to speak. Whether the emitting tool's type is the right value for `originating_system.system_type` on a skill warning, as against some value naming the executor role rather than the tool the executor sits beside, is not settled in the schema. The position taken here follows from a decision taken outside the schema, that tools have executors and newsrooms do not, and the schema has not caught up with it. One consequence should be said plainly rather than discovered: this is one of the two places this file names a system type, the other being the advert's `target_system_type` in the frontmatter, and the two now agree by construction, because the tool that may recall this skill is the tool that emits on its behalf.
- **The alternate set is grouped by a shared field rather than by a relationship, because there is no relationship condition kind available to use.** The honest description of an alternate set is a set of related assets, and `derived_from` is exactly the sort of editorial link that would express "these are versions of the same thing". A relationship or cascade condition kind has been proposed but is not in the condition set, so this skill reaches the members through a configured group field instead. It works, and it is a workaround: a house that does not carry a group identifier on its packages cannot configure this skill today. If the relationship or cascade condition returns, this advert should be revisited.
- **The condition value is config-templated, so the lookup table cannot index this skill by the value it tests.** The path is literal, which is better than the fully templated case, but a table indexing on `compliance[].type` recalls this skill for every clearance type and lets the configured instance decide. Under proposal A, chosen 18 August, the house registers one advert row per configured instance rather than the table resolving configuration for it. The cost is that the registration layer has to exist and has to be kept honest, which is the work A now owes; the benefit is that conformance is provable, because every row a house holds names the instance it came from.
- **`recall_on` is a proposed name, not an agreed one**, and of the three topics listed only `story.context` has a worked precedent. `compliance.assertion` and `asset.context` are plausible rather than ratified and should be reconciled with whatever the bus actually publishes before registration. This matters more here than elsewhere, because a clearance that arrives on a topic this advert does not list is a resolution nobody sees at the moment of transmission.
- **`skill_uri` is left as a placeholder** because it is not this author's to supply and because whether it is required at all is unsettled: one position treats it as mandatory on every registered skill, the other as optional. A plausible-looking invented URI reads as a real one to the next person. `owner_editorial`, `owner_engineering`, `licence` and `skill_content_sha` are left for the same reason, the last being computed at build rather than typed.
- **The `skill_id` format is one of four in circulation.** Nothing agreed so far picks one, and `<publisher>/<name>` is used because it is the form with a worked precedent in the warning schema's own example, not because the question is settled. If the group picks another, this changes.

---

## 11. Vendor build notes (layer 3)

| Platform | What your executor does |
|---|---|
| Playout | Takes the resolution as the answer to "which item is the one to take" at the moment of transmission, and reads it against the live context at that moment rather than once at load time, because a clearance can land seconds before the take. The superseded item stays in the schedule and stays previewable; it is simply not the one. Your executor does not treat the resolution as permission to transmit: gates binding the telling still apply, and any one hold still means held |
| Rundown | Floats the item the resolution names current into the live running order and leaves the alternates unfloated, off the one event, without waiting to hear from any other system. Show on the item which clearance resolved it and who asserted it, because in a live gallery the question is always "says who". A rundown that floats the right item but cannot say why has done half the job |
| MAM versioning | Flips the named member to your current-version state and marks the other members of the same group superseded, so every downstream tool resolving a version through you gets the right one without asking anybody. Keep the superseded versions intact and retrievable: they are what the audit record of what was suppressed is made of, and deleting them to express a resolution is a content change you are not entitled to make |
| Digital CMS | Serves the member declared current on its paths and stops offering the superseded one, at the point it publishes, against the same story state every other destination is reading. Where the house has configured `link` scope, decide for your own destination only. Do not express a resolution by publishing an edited version with the superseded material removed: that is a content change, and it is not yours to make |
| Graphics | Makes the prepared set belonging to the current package the live one and retires the alternate sets in the same movement, off the same clearance. In the three-way cases, two sets retire at once, so build for a set rather than for a pair. A graphics store that flips a moment after playout has already taken the item is the visible version of the failure this exists to remove |

The instinct this skill exists to interrupt is the producer at the moment of transmission telling
each downstream system separately that the count has come in: floating the right package by hand
here, suppressing the wrong one by hand there, racing the clock and hoping every tool follows.
Nothing here is point to point between tools. Every tool reads the same resolution off the one
clearance event and moves at the same instant, everywhere at once, or the gap between them is
visible instead of hidden.
