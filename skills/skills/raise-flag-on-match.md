---
# --- identity ----------------------------------------------------------------
skill_id: smart-stories/raise-flag-on-match
name: raise-flag-on-match
skill_version: 0.2.2
som_schema_version: ["1.0.0"]

# --- what it is --------------------------------------------------------------
description: >-
  A named flag is declared on a story when a configured field matches a configured
  value, at mint or at ingest, including on a story that has no media yet. One
  configured instance watches one field and declares one flag, so a house running two
  postures on the same story runs two configured instances of this skill rather than
  one configured instance with two rules. It declares only. It never withholds, blocks,
  gates, delays or routes anything, it never ranks or enriches, and it never changes
  content. Any hold that follows a flag comes from a separate hold skill advertising
  against that flag, chained over the bus.
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
  target_system_type: [planning, rundown, mam, graphics, discovery]
  target_capability: null
  conditions:
    - kind: field
      path: "{{ config.match_field }}"
      op: equals
      value: "{{ config.match_value }}"
  recall_on: ["story.context", "asset.ingested"]
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

# raise-flag-on-match

**When a configured field on a story matches a configured value, declare the named flag,
at the moment the story exists. Do not withhold anything.**

Read `../docs/CONVENTIONS.md` first.
**Changed in 0.2.1, 20 August 2026.** Two small things. The flag-to-hold gap in section 10 described the gate entry as carrying a flag_type; the schema calls it `gate_type`. And section 10 records proposal A as decided rather than leaving the indexing consequence open.

**Changed in 0.2.0, 18 August 2026.** Three things. `som_schema_version` moves to `["0.3.2"]`, which as of 18 August is the version. Field paths in the advert and in the worked configurations were checked against the v0.3.2 story-context JSON Schema for the first time and the ones that did not resolve are corrected; see the pack's `SCHEMA-CONFORMANCE.md` for the full list and the evidence. And the shared policy index was chosen over executor-local policy, which settles the registration question this file used to leave open: the house registers one advert row per configured instance.

---

## 1. What it is (layer 1)

| | |
|---|---|
| Operation | raise: one named flag is declared on one story |
| Condition type | match: one configured field tested against one configured value |
| One mechanism? | Yes. The sentence is "declare a named flag when a configured field matches", which needs no "and". Nothing is withheld, nothing is routed, nothing is enriched. The hold that a house may want behind a flag is a second skill reading this one's output, not a second half of this one. X is `compliance` because the postures declared here (an indicative category, an unverified figure, a price-sensitive disclosure state) are the ones whose breach carries legal, regulatory or reputational consequence, not because the mechanism is regulatory in itself |

**Reuse test, three scenarios from different stories:**

1. **The hurricane, minted from the first wire alert.** The category is indicative and the casualty figures are unconfirmed, both before any footage exists. Two configured instances declare `indicative-category` and `UNVERIFIED` at mint, and every subscribing tool reads the posture and primes against it.
2. **A criminal verdict, minted from a court listing.** Reporting restrictions are recorded as active on the listing, so a configured instance watching the restriction field declares `REPORTING-RESTRICTED` at mint, well before a reporter files.
3. **A mid-cap issuer's profit warning, minted at 06:40 from a regulatory news service feed, ninety minutes before the market opens.** This library has not previously drawn a markets story. The issuer's disclosure state on the feed reads `pre-release`, so a configured instance watching that field declares `PRICE-SENSITIVE` at mint. The graphics desk primes a results template pack, the archive search opens on the company, and the business producer sees on the object itself why nothing may be published from it yet. When the announcement lands and the disclosure state moves to `released`, the declaration closes on its own terms.

One skill, three subjects, three different fields and three different flag names, all of it configuration. The mechanism did not change between them.

---

## 2. Recall advert

The advert is declared in the frontmatter above so a tool can read it without parsing prose.
This section explains it in words for a human.

- **Who it is for:** the tools that prime against a freshly minted story before any media exists. `planning` opens a placeholder bound to the story, `rundown` carries the story slot, `graphics` surfaces a baseline template pack, `mam` registers archive and stock as references, and `discovery` opens a collection and runs the story against an archive. These are broad system types, not named products, because whether the table names a type or a product is not settled.
- **What wakes it:** `story.context`, which is the topic a mint or a context revision arrives on, and `asset.ingested`, which is the second moment a posture can first become knowable. Origination is not privileged: a wire, a planning desk, a digital CMS or a social desk can all publish the originating story context, with the rundown subscribing downstream, and the advert reads the same in every one of those directions.
- **What it depends on:** one field condition, `equals`, comparing the value at the configured watched path against the configured match value. Both sides of the comparison come from configuration, so no editorial choice sits in this file.

Two things about this advert are unusual and are named here rather than left for a reviewer to find.

**The path is config-templated.** The watched field is `{{ config.match_field }}`, not a literal dot path, because the whole point of the skill is that the house chooses what to watch. A single-purpose skill advertises a literal path and the global lookup table indexes it as one row. This one cannot be indexed that way, because the table cannot hold a config-templated path as a single row at all. The house therefore registers one advert row per configured instance, and that is how a human should expect this skill to appear in the table: two postures on one story read as two rows, not as one row carrying two rules. The alternative, a table that resolves configuration before it builds its rows, would remove that need, and nothing in this file would change if it were adopted. That question is unresolved, and it is recorded again in section 10.

**Nothing here reaches forward to a hold.** The flag declared by this skill is intended to be read as a state on the story, and the chain described across this library is that `hold-while-flagged` advertises against that state independently, over the bus. Two skills, two tools, one bus, no integration between them. That chain is the assumption this library works to, not a proven path: `hold-while-flagged` advertises on `editorial_gates[]` fields, this skill produces one `skill.warning.raised` with `blocks: []`, and `auto_change_content: false` forbids it from writing story state, so something outside this file has to carry a raised warning into gate state and nobody has yet said what. The gap is named in section 10. This file therefore has no knowledge of what may be withheld behind its flag, which is why `depends` is empty.

---

## 3. Firing anchor

- **Evidential position:** any. At mint there is no asset, so no evidential position has been established yet, and a skill that required `PRIMARY` would never be recalled at the moment this one is for. The same configured instance is recalled unchanged at ingest, when a position does exist.
- **Outlet / path:** the story's own context, not a destination path. No outlet is implied and no outlet is excluded.
- **Scope:** `story:<id>`. The posture is a property of the story at system level. Where a posture is genuinely destination specific, that is `gate-by-scope`'s question and `link:<id>` is its scope, not this skill's.
- **Compliance question, in one plain sentence:** "From the first moment this story exists, does it carry on its face the posture that anyone about to work it needs to see?"

---

## 4. Config surface (layer 2)

Every editorial choice sits here, set by the publishing house. Nothing here is in the skill.
Scope runs broadcaster, then site, then brand, the more specific overriding the default.

| Field | Required | Notes |
|---|---|---|
| `match_field` | yes | The dot path of the field watched, for example `lifecycle.phase` or `editorial_source[].credibility`. This is the value substituted into the advert's condition path |
| `match_value` | yes | The literal the watched field is compared against, for example `BREAKING`. Comparison is `equals` and is case sensitive, so a value drawn from a schema enum must be written in the enum's own casing |
| `flag_name` | yes | The name of the flag declared when the comparison holds, for example `UNVERIFIED`. Carried through to `affected_fields` and `detail` so the room reads a name rather than a rule number |
| `severity` | no | `flag` or `inform`, defaulting to `flag`. `hold` is not available to this skill at any configuration |
| `clearing_authority` | yes | The authority at or above which an assertion on the same scope closes the declaration, for example `duty-editor`. How two authorities are compared is not defined anywhere: see section 10 |
| `instance_label` | yes | the editorial label for this configured instance |

The house also declares this skill in its own active set. Those values are copied straight from
the frontmatter above and are not a second set of choices: `skill_id`, `skill_version`,
`skill_type`, `disclosure_level`, `migration_policy`.

**One story can carry two configured instances of this skill.** One configured instance watches
one field and declares one flag, so a house running two postures on the same story runs two
configured instances. The breaking weather case is exactly that: the hurricane is minted from
the first wire alert and two configured instances are recalled off that one snapshot, neither
aware of the other.

```yaml
# First configured instance: the category is indicative
instance_label: house-breaking-indicative-category
match_field: lifecycle.phase
match_value: BREAKING
flag_name: indicative-category
severity: flag
clearing_authority: duty-editor

# Second configured instance: the figures are unconfirmed
instance_label: house-breaking-unverified-figures
match_field: editorial_source[].credibility
match_value: UNVERIFIED
flag_name: UNVERIFIED
severity: flag
clearing_authority: output-editor
```

What this produces: before any footage exists the story carries `indicative-category`
and `UNVERIFIED` on its face at once, the casualty and wind-speed figures are marked as
unconfirmed from the moment the story exists, and the sources are logged in `editorial_source[]`
with the wire recorded as `TRUSTED`. The two configured instances differ only in the watched
field, the matched value, the flag name and the clearing authority. They are configuration, not
code, and they are two rows in the house's active set rather than one row with two rules.

---

## 5. Runtime loop, specialised

```
on story.context or asset.ingested:
  1. RESOLVE ANCHOR   the story named in the message, at story scope. No asset is required
  2. GATE             does the snapshot carry story context at all, and is config.match_field
                      present within it? Absent context or an absent watched field means exit
                      quietly, which is the ordinary case on most messages in a running newsroom
  3. READ CONFIG      load the configured instance for this broadcaster / site / brand
  4. EVALUATE         compare the value at config.match_field against config.match_value
  5. DECIDE SEVERITY  config.severity, defaulting to flag. inform where the house has configured
                      the posture as advisory. Nothing here can reach hold
  6. EMIT             the twelve-field warning, causation_id on the triggering snapshot
  7. AWAIT CLEARANCE  an assertion on story scope from an authority at or above
                      config.clearing_authority, carrying its own provenance of who, when,
                      scope and status
  8. RESOLVE / CLOSE  closed by a valid clearance, or when a later snapshot shows the watched
                      field no longer matching. Closure is recorded, not erased
```

**Skill specific traps:**

- The mistake a well-meaning vendor will make is treating the flag as an instruction to withhold. It is not one. A declared flag changes what is displayed on the object and what other skills may advertise against; it withholds nothing by itself, and an executor that quietly stops serving a story because a flag is present has invented a hold nobody configured.
- The second mistake is waiting for media. This is recalled against a story with no asset attached, and an executor that resolves the anchor by looking for an asset will find nothing and conclude wrongly that the message is not for it.
- `detail` must carry the flag name, the watched path, the value found, the value matched against, and what clears it. A message saying only that a flag was raised is annoying rather than actionable, because the producer cannot tell from it what would make it go away.
- Fail closed: config that is missing or unreadable, or a watched field that is present but whose value cannot be read, is treated as the condition holding, and the declaration is made at the loudest severity available here, which is `flag`. `detail` says the value could not be read rather than asserting a match that was never observed. A watched field that is genuinely absent is a non-match, not an unreadable value, and is handled at the gate. That line between absent and unreadable is drawn by this author, not by the specification: see section 10.
- What does NOT clear it: a later snapshot that simply omits the field, an assertion from below `config.clearing_authority`, an assertion on a different scope, and the passage of time. None of those close a declaration.

---

## 6. Output contract

`skill.warning.raised`, severity `flag` (or `inform` where the house has configured the posture as advisory).

```json
{
  "topic": "som.skill.warning.raised",
  "originating_system": { "system_id": "<tool>", "system_type": "<the tool's own type, one this skill's advert targets>" },
  "correlation_id": "<the story's chain>",
  "causation_id": "<the snapshot that triggered evaluation>",
  "timestamp": "<on the envelope, never in the payload>",
  "payload": {
    "warning_id": "wrn-....",
    "skill_id": "smart-stories/raise-flag-on-match",
    "skill_version": "0.2.2",
    "story_id": "story-...",
    "scope": "story:...",
    "severity": "flag",
    "rule_id": "<the configured instance label>",
    "non_overridable": false,
    "affected_fields": ["<config.match_field>"],
    "detail": "<flag_name> declared: <match_field> reads <value found>, which matches <match_value>. Cleared by an assertion at or above <clearing_authority> on this story.",
    "blocks": [],
    "skill_warning_ref": null
  }
}
```

`blocks` is empty and `non_overridable` is false, permanently, in every configuration. Nothing is withheld here, so there is nothing to list and nothing to override.

---

## 7. Priority position

| Axis | This skill |
|---|---|
| X category | `compliance` (1) |
| Y type / source | `reference` (5) |
| Z precedence | unset. The publishing house sets it, nobody else |

Because nothing is withheld, this skill contests no action and outranks nothing at the point of declaration: two configured instances of it on the same story co-bind without conflict, which is exactly what happens at mint in the hurricane. Where a hold skill later binds against a flag declared here, holds combine by conjunction: every binding gate must permit, so any one hold means held.

---

## 8. Worked configurations

### 8a. Breaking weather: the category is indicative and not yet confirmed

```yaml
instance_label: house-breaking-indicative-category
match_field: lifecycle.phase
match_value: BREAKING
flag_name: indicative-category
severity: flag
clearing_authority: duty-editor
```

What this produces: the hurricane story is minted from the first wire alert, and before any footage exists it carries `indicative-category` on its face, so the planning placeholder, the graphics template pack and the archive search all prime knowing the category may move. The second configured instance on the same story, `house-breaking-unverified-figures`, is set out in section 4.

### 8b. A pre-market profit warning: the issuer's disclosure is not yet released

```yaml
instance_label: house-markets-pre-release-disclosure
match_field: compliance[].type
match_value: PRICE_SENSITIVE
flag_name: PRICE-SENSITIVE
severity: flag
clearing_authority: business-editor
```

What this produces: a mid-cap issuer's profit warning is minted at 06:40 from a regulatory news service feed, ninety minutes before the market opens. The issuer's disclosure state reads `pre-release`, so the story carries `PRICE-SENSITIVE` from the moment it exists. The graphics desk primes a results template pack, the archive search opens on the company, and the business producer sees on the object itself why nothing may be published from it yet. When the announcement lands and the disclosure state moves to `released`, the declaration closes on its own terms, without anyone asserting a clearance.

What changed between 8a and 8b: the story, the originating feed, the watched field, the matched value, the flag name, the clearing authority and the set of tools that prime against the posture. A breaking weather story minted from a wire and worked by the assignment desk, and a markets story minted from a regulatory feed and worked by a business producer, share no subject matter, no vocabulary and no desk. What stayed the same: the mechanism, the advert shape, the `story:<id>` scope, the severity ceiling and the fact that neither configured instance withholds anything. One skill, two situations, all of the difference in configuration.

---

## 9. Eval set

Minimum five. The skill is not finished until a declaring case and a clearance case both pass.

| Case | Input | Expected |
|---|---|---|
| declaring | A breaking weather story minted from a wire alert, `story_context.figures_status` reads `unconfirmed`, the `house-breaking-unverified-figures` configured instance from section 4 loaded | One `skill.warning.raised`, severity `flag`, `rule_id` is `house-breaking-unverified-figures`, `detail` carries both the value found (`unconfirmed`) and the value matched against (`unconfirmed`), plus the clearing authority. `blocks` empty |
| non declaring | Same story, `story_context.figures_status` reads `confirmed`, or the field is absent from the snapshot | Nothing at all on the bus, quietly. No warning, no closure, no heartbeat |
| clearance | An `output-editor` assertion on `story:<id>` states the figures are confirmed, with provenance of who, when, scope and status | The declaration closes, the closure is recorded, and the original declaration stays visible in history |
| wrong clearance | A `journalist` assertion, below `output-editor`, attempts the same closure | Not applied, the declaration stands, and the attempt remains visible with its provenance. Nothing is silently discarded |
| idempotency | The same minted snapshot delivered twice, in both clouds, both executors recalling `house-breaking-unverified-figures` | Exactly one warning. The second is recognised by `causation_id` on the same snapshot |
| fail closed | `clearing_authority` missing from the configured instance, or `story_context.figures_status` present but unreadable | `flag`, the loudest severity this skill has, and `detail` says the value or the config could not be read rather than asserting a match that was not observed |
| never auto-change | Any declaring case, and the two configured instances from section 4 loaded together on one story | Assert that no message produced on this skill's behalf altered any field of the story, any asset, or any config. Only warnings were added |

---

## 10. Open items

- **`disclosure_level: L2` is ambiguous.** Two definitions circulate: the deepest level the skill declares content for, and how much of its reasoning the skill reveals. `L2` here means **the deepest level the skill declares content for**, which is the config surface in section 4. If the group settles on the reasoning definition instead, this value is re-examined and probably changes, because the reasoning revealed by this skill is a single field comparison.
- **The advert's condition path is config-templated.** `{{ config.match_field }}` is a good design with an unresolved consequence: the global lookup table cannot index it as one row the way it indexes a literal dot path. Proposal A settled that on 18 August: the house registers one advert row per configured instance, in its own registration layer, which for the hurricane minted from a wire alert means two rows for the two configured instances in section 4. Assumed here: one row per configured instance. If the table resolves configuration instead, nothing in this file changes, but the house's registration step does.
- **Whether `target_system_type` binds or merely advises is not resolved.** It is written here as though it binds, per the working position. Under an advisory reading this skill would behave differently in one respect worth naming: a compliance hub, an audit tool or an oversight desk is not in the target list and would not be recalled, yet those are precisely the tools a house might want reading a posture declared at mint. If targeting becomes advisory, expect the target list to stay as it is and the off-target readers to pick it up anyway.
- **The emitting executor carries the tool's own `system_type`, and the `skill_worker` type this library used to emit here is withdrawn.** An executor is a sidecar to a vendor tool, and no newsroom-scoped class of skill-running system exists, so the thing that puts this skill's declaration on the bus is a planning tool, a rundown, a MAM, a graphics system or a discovery tool, and section 6 now says so. The sample payload previously carried a `skill_worker` system type, a value whose only reason to exist was that the emitter ran skills, which is the newsroom-executor shape under another name. It is withdrawn rather than reinterpreted, and no file in this library should be read as having such a class. What is not settled is whether the emitting tool's own type is the right value for `originating_system.system_type` on a skill warning at all, as against some value naming the executor role that raised it. The schema does not answer that, and the position taken here follows from a decision taken outside the schema, that tools have executors and newsrooms do not, which the schema has not caught up with. Worth naming plainly: this is one of the two places this file names a system type, the other being the advert's `target_system_type`, and after this change the two agree by construction, because the tool that may recall this skill is the tool that emits on its behalf.
- **The clearance authority comparison rule is undefined.** The rule adopted throughout this library is that a declaration closes only on an assertion of equal or higher authority on the same scope, so a flag closes only that way. That is the working position this library adopts, not a rule any group has ratified, and no vendor should read it as settled or hard-code enforcement against it. How two authorities are compared is not defined anywhere either, so `config.clearing_authority` is a house-local string and the executor's comparison of it is house-local too. The wrong-clearance eval case is written against the adopted rule, and if the group settles the comparison differently that row is rewritten and `clearing_authority` is read against the group's rule rather than the house's.
- **Nothing has been written down that carries a flag declared here into the gate state a hold reads.** This is the gap in the flag-to-hold chain, and it is named here rather than papered over. `hold-while-flagged` advertises on `editorial_gates[]` fields: a `gate_type`, a `status`, a scope. This skill produces exactly one output, `skill.warning.raised`, with `blocks: []`, and `auto_change_content: false` forbids it from writing story state, so a warning raised here does not become an `editorial_gates[]` entry by itself. Something between the two has to materialise it, and nothing written down says what. The assumption taken across this library is that the executor materialises a raised warning into gate state on the story, which keeps the two skills independent and keeps the writing of state out of the skill layer. If the group decides instead that the bus does it, or that a third skill or a gate service does it, nothing in this file's advert, config surface or output contract changes, but the chain acquires an owner that would need naming here and the idempotency question moves to that owner. The bus side of the chain is in place: `hold-while-flagged` carries `skill.warning.raised` in its `recall_on`, so a warning raised here does reach it, and the chain works as described from that point on. What is unowned is the step before, the materialisation into `editorial_gates[]`. Until one of those readings is agreed, that step should be read as assumed, not as proven.
- **`depends` is empty, deliberately, and this is where a reader expects a dependency.** The behaviour a house may want behind these flags (a hold) is intended to come from `hold-while-flagged` advertising against the flag this skill declares, chained over the bus, subject to the carrier gap named above. Declaring that as a dependency would invert it: a raise skill would depend on the skill that consumes its output, which creates a cycle at the first change. This skill depends on what it reads, and it reads only story context and its own config, neither of which is a skill.
- **`state_path` is `null` because this skill never holds.** That is legal by reading rather than by statement: the field is described as the state a message must carry to decide an open hold, and there is no open hold here. Nobody has written down that `null` is correct for a non-holding skill.
- **`fail_closed: true` reads oddly for a skill with no hold authority.** The template's illustration of fail-closed is a hold that says why, which is not available here. The reconciliation applied in section 5 is that fail-closed at the skill layer means treating an unreadable value as the condition holding, producing the loudest severity this skill actually has, which is `flag`, while safe state at the executor layer handles the case where nothing resolves at all. The line drawn between an absent field (non-match, exit quietly) and an unreadable value (treated as a match) is this author's, not the specification's, and a reviewer may reasonably draw it elsewhere.
- **`recall_on` is a proposal, not an agreed name.** It replaced the older active-voice name for the same field under the passive convention agreed on 29 July 2026, and no replacement name has been agreed anywhere. Separately, `story.context` is the topic used across this library for a mint or context revision, but `asset.ingested` is a plausible ingest-side topic rather than a topic anyone has ratified. If the ingest topic is named differently, this advert's second entry changes and nothing else does.
- **Flag names are house configuration and their casing is not governed.** Sections 4 and 8 carry `indicative-category` in lower case and `UNVERIFIED` and `PRICE-SENSITIVE` in upper case because that is how the originating configuration reads. Nothing in the schema constrains the vocabulary or the casing of `flag_name`, so two houses can declare the same posture under different names and no skill will notice.
- **`skill_uri`, `skill_content_sha`, `owner_editorial`, `owner_engineering` and `licence` are left as markers.** They are not this author's to supply, and whether `skill_uri` is required at all is unsettled: one position treats it as mandatory on every registered skill, the other as optional. A plausible invented URI would read to the next person as a real one.
- **The `skill_id` format is one of four in circulation.** `<publisher>/<name>` is used because it is the form with a worked precedent in the warning schema's own example, not because the question is settled.

---

## 11. Vendor build notes (layer 3)

You are building what your executor does with this skill, not what the skill does. The skill declares a posture on a story that may have no media at all; your executor reads that posture and primes itself, deciding what it loads, holds or searches.

| Platform | What your executor does |
|---|---|
| Planning | Opens a placeholder bound to the story id the moment the context arrives, and shows the declared flags on it, so the assignment desk briefing a crew sees `indicative-category` before anyone is dispatched. No media is expected and none is waited for |
| Rundown | Carries the story slot with its posture attached, so the flags travel with the item into the running order rather than being re-derived when copy lands. A slot with a declared flag is a slot the producer can see the reason for |
| Graphics | Surfaces the baseline template pack for the declared posture and holds off building anything specific: an indicative category means the category-bearing template is prepared but the value is not burned in. Nothing is rendered on the strength of a flag |
| MAM | Registers archive and stock as references against the story id and records the posture alongside them, so material associated before any new footage exists is associated with the story's stated state at the time |
| Discovery | Opens a collection on the story id and runs the story context against the archive immediately, on video or on transcripts, so results are waiting rather than being requested. The declared flags are carried into the collection as context for whoever reviews the results |

The instinct this skill exists to interrupt is the instinct to wait: tools today wait for footage to arrive or for a human to brief them, and nothing works the story until media or copy lands. Here the story exists first, with its posture on it, and every tool can prime against it.
