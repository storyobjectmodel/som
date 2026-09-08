---
# --- identity ----------------------------------------------------------------
skill_id: smart-stories/gate-by-scope
name: gate-by-scope
skill_version: 0.2.2
som_schema_version: ["1.0.0"]

# --- what it is --------------------------------------------------------------
description: >-
  One destination path's editorial gate is resolved for one telling of one asset, and whether
  that path is permitted to serve the fact now or must withhold it is declared. The question asked
  is always per path: does THIS path's gate permit THIS telling. It reads gate state only. It
  does not set, clear or grade a gate, it does not judge whether a fact is true, it does not
  change content, and it never resolves one answer for the whole story.
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
  target_system_type: [cms, playout, rundown, mam, distribution]
  target_capability: null
  conditions:
    - kind: field
      path: assets[].usage[].destination_id
      op: exists
    - kind: field
      path: editorial_gates[].gate_type
      op: equals
      value: "{{ config.gate_scope_kind }}"
    - kind: field_change
      path: editorial_gates[].status
      op: equals
      value: APPROVED
  recall_on: ["story.context", "telling.proposed", "story.gate.changed"]
  state_path: editorial_gates[].status

# --- what the executor puts on the bus on this skill's behalf ----------------
output_messages: ["skill.warning.raised"]
severity_range: [hold, inform]

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

# gate-by-scope

**Publish on a path while the gate resolved for that path reads `APPROVED`. Ask the question once per path, never once per story.**

Read `../docs/CONVENTIONS.md` first.
**Changed in 0.2.1, 20 August 2026.** The status vocabulary. Sections 4 to 9 still described gates
as moving between `clear` and `held`, words the schema does not carry, and still wrote
`editorial_gates[web]` as though the array were indexed by path name. `editorial_gates[].status`
is a closed enum of `PENDING`, `APPROVED` and `REJECTED`, and an entry is found by `gate_id`.
Both are corrected throughout, and `publish_when` is removed for the reason set out in section 7:
it was an expression in a syntax this library never defined, over states that do not exist.
`path_map` now resolves a destination to a `gate_id` rather than to a path name.

**Changed in 0.2.0, 18 August 2026.** Three things. `som_schema_version` moves to `["0.3.2"]`, which as of 18 August is the version. Field paths in the advert and in the worked configurations were checked against the v0.3.2 story-context JSON Schema for the first time and the ones that did not resolve are corrected; see the pack's `SCHEMA-CONFORMANCE.md` for the full list and the evidence. And the shared policy index was chosen over executor-local policy, which settles the registration question this file used to leave open: the house registers one advert row per configured instance.

---

## 1. What it is (layer 1)

| | |
|---|---|
| Operation | Resolve one destination path's gate into permit-or-withhold for one telling |
| Condition type | Gate state read at the scope of the path being published to |
| One mechanism? | Yes. The single act is a per-path gate read. The sentence needs no "and": nothing is declared about the fact itself, nothing is set on the gate, and no second path is considered in the same evaluation. The evaluation is repeated per path, which is repetition of one mechanism rather than a second mechanism |

The "and" test is also where this skill is separated from `hold-while-flagged`, because a
reviewer who reads both quickly will assume they duplicate each other. They do not, and the
difference is **what each reads**, not what scope either is configured at:

| | `hold-while-flagged` | `gate-by-scope` |
|---|---|---|
| What is read | A typed flag standing against an action | The gate bound to a destination path |
| Question | Is a flag of the configured type standing against this action? | Does the gate bound to this destination path permit this telling? |
| Answer varies with | Whether a flag of that type stands | Which path the telling is being served on |
| Scope | A config field on that skill, `story` or `link` | Link scope always, configured as `scope: destination-path` in every configured instance here |

Scope is not the distinguishing axis, and nothing in this file claims it is. `hold-while-flagged`
exposes `scope` as a config field, and the worked configuration 8b in that file is configured
`scope: link`, so a reader who separates the two by scope will separate them wrongly: a link-scoped
hold and a path's gate sit at the same coordinate and remain different questions. This skill is
link scoped always, and `hold-while-flagged` is the one that may be configured at story or at link
scope. Scope is also not X, so a configurable scope does not make a skill unpositionable: the
coordinate is set by `category` and `origin`, neither of which moves with scope.

Where both skills are recalled on one story, they are recalled against the same `editorial_gates[]`
state, and they co-bind by conjunction rather than competing: the unconfirmed figure is withheld on
a path if the flag stands or if that path's gate does not permit. Any one hold means held. That is
normal rather than an anomaly, because per-coordinate uniqueness is what makes co-binding safe.

X is `compliance` and not `workflow` because the question is whether a fact may lawfully or
policy-permissibly be served on a given path. That a routing decision falls out of the answer is
a consequence, not the governance being applied.

**Reuse test, three scenarios from different stories:**

1. **A hurricane, the story it was written for.** The confirmed storm category is cleared to
   publish; an unconfirmed casualty figure is not. The web path releases the category while the
   broadcast path holds its whole item until its own gate clears.
2. **A watershed, a different kind of story entirely.** A distressing sequence in a court report
   carries a time-of-day gate. The linear path is not permitted before 21:00 and is permitted
   after it; the streaming path is permitted throughout. One gate state, read per path, three
   outcomes across the evening with no branch written into the skill.
3. **Betting integrity at a stadium, a story this library has not drawn yet.** A live athletics meeting is
   covered on the general sports site, on a broadcast rundown, and on an in-venue app served only
   to spectators inside the ground. Any fact derived from the unofficial trackside timing feed
   carries a gate that permits the general site and the rundown but withholds on the in-venue
   path, because in-venue latency advantage over the official feed is a market-integrity
   exposure. This library has not previously drawn a story where the narrowest audience is the
   most restricted one, and this skill needs no change to serve it: the gate is written per path
   and read per path.

One skill, three subjects, and in every one of them the varying thing is the destination rather
than the fact.

---

## 2. Recall advert

The advert is declared in the frontmatter above so a tool can read it without parsing prose.
This section explains it in words for a human.

- **Who it is for:** any tool that is itself a point of publication, because only such a tool has
  a path whose gate can be read. Digital CMS, playout, rundown, MAM and distribution or social
  publishers are targeted. A tool that merely holds an asset without serving it to a destination
  has no `link` to anchor on and is deliberately not targeted.
- **What wakes it:** a story context arriving, a telling being proposed to a destination, and a
  change of gate state. The third is the one that matters most for release: a path that has been
  held must be looked at again the moment its gate turns `APPROVED`, or an item stays held after the
  reason for holding it has gone.
- **What it depends on:** that the telling names a destination, that the story carries an
  `editorial_gates[]` entry scoped the way the configured instance expects, and, for the release
  path, that `editorial_gates[].status` has moved to `APPROVED`. `state_path` names that same
  field, `editorial_gates[].status`, because a message that does not carry the gate's status for
  the path cannot decide a hold that is already open on that path.

**The watershed shape, adapted to this skill.** The advert is written so that one skill serves
several outcomes without any branch inside the skill. The rows below are the hurricane. The
executor is told what to do for the case in front of it; the skill never chooses between cases:

| IF gate status reads | AND the gate resolved for the destination | AND moment | Executor recalls | Which tells it to |
|---|---|---|---|---|
| `APPROVED` | `eg-web` | any | this skill | RELEASE the confirmed category |
| `PENDING` | `eg-web` | any | the same skill | WITHHOLD the unconfirmed figure |
| `PENDING` | `eg-broadcast` | before clearance | the same skill | WITHHOLD the item at the gate |
| `APPROVED` | `eg-broadcast` | after clearance | the same skill | RELEASE the item |

Four outcomes, one advert, no branching. The skill is recalled identically in each row; what
differs is the path and the gate state the message carries.

**Unusual parts, named here rather than left for a reviewer.** Two of the three conditions carry
config-templated value (`{{ config.gate_scope_kind }}`), and the third names the enum value
`APPROVED` literally, because the permitting status is fixed by the schema and is not a house
choice. The
values, not the paths, are templated, so the lookup table can still index the rows; a house that
templates the paths themselves would lose that and would have to register one advert row per
configured instance. The `field_change` condition is present for release rather than for hold,
which reads backwards until you notice that a held path is only unheld by a change of gate state.

**`depends` is empty, deliberately.** Nothing here is read from another skill's output. Gate
state is read off the story itself, so this skill advertises against `editorial_gates[]` whoever
wrote the entry: a producer, a compliance desk, a legal hold, or another skill entirely. Chaining
happens through the bus, and a declared dependency on the skill that happens to have set a gate
today would invert the direction and create a cycle at the first change.

---

## 3. Firing anchor

- **Evidential position:** any. A gate binds a path regardless of whether the fact is carried at
  PRIMARY, SECONDARY or TERTIARY position, so filtering on evidential position here would
  silently unbind gates on the positions not listed.
- **Outlet / path:** the destination of the telling under evaluation, and only that one. Each
  outlet reads its own path, at the point it publishes, against the same story state.
- **Scope:** `link:<id>`. Not `story:<id>`, because the answer is destination-specific: the same
  story state resolves to release on one path and withhold on another at the same instant, so a
  story-scoped warning would be false on one of the two paths whichever way it was written. The
  `link` is the telling of an asset to a destination, and it is the only object in the model that
  identifies both the thing being served and the path it is being served on, which is exactly the
  pair the gate question needs.

  This is the clearest example of the link-scoped case in the library, and worth reading as the
  reference for it. Most skills here answer a question about the story and are correctly
  `story:<id>` scoped, so a reviewer meeting `link:<id>` for the first time should meet it here:
  the test for link scope is not "is a destination involved" but "can two destinations reading
  identical story state at the same moment be entitled to different answers". For the
  configured instances described in this file they can, by design, because the thing being read is
  bound to the path. That is a fact about those configured instances and not the line between this
  skill and `hold-while-flagged`, which exposes `scope` as a config field and is itself configured
  at `link` scope in its own worked configuration. The two are separated by what each reads, per
  section 1, and a house may configure `hold-while-flagged` at story or at link scope, while
  every configured instance of this skill is link scoped.
- **Compliance question, in one plain sentence:** "If this went out on this path right now, would
  it serve something this path is not currently cleared to carry?"

---

## 4. Config surface (layer 2)

Every editorial choice sits here, set by the publishing house. Nothing here is in the skill.
Scope runs broadcaster, then site, then brand, the more specific overriding the default.

| Field | Required | Notes |
|---|---|---|
| `instance_label` | yes | The editorial label for this configured instance, carried into `rule_id` on the warning. For example `house-storm-web-path`. |
| ~~`publish_when`~~ | removed | **Removed in 0.2.1.** It was written as an expression, `gate(path) == clear`, in a syntax this library never defined and no executor could parse, over a status vocabulary the schema does not carry. With `clear_state` gone there is nothing left for it to configure. The permit test is fixed and is described below the table. A house that wants a richer test wants a different skill. |
| `scope` | yes | The scope kind the gate is read at. `destination-path` in every configured instance in this file. A configured instance that reads a gate bound to one path and then declares one answer for the whole story is a configuration error, not a variant, and should be rejected rather than coerced. |
| `gate_scope_kind` | yes | Which class of `editorial_gates[]` entry binds this configured instance, for example `editorial_clearance` or `legal_restriction`. Matched by the second advert condition. |
| ~~`clear_state`~~ | removed | **Removed in 0.2.0.** `editorial_gates[].status` is a closed enum of `PENDING`, `APPROVED` and `REJECTED`, so the permitting value is `APPROVED` and is not a house choice. A house whose own vocabulary says `released` maps it at its edge; the story object carries the enum. |
| `path_map` | yes | How this tool's destinations resolve to gates: each `assets[].usage[].destination_id` this tool serves, mapped to the one `editorial_gates[].gate_id` that governs it. Several destinations may map to one gate, which is how every syndicated feed comes to sit behind the web gate. A destination the map does not name is held. |
| `record_permits` | no | Boolean, `false` unless the house sets it. Whether a permitted read is put on the record as an `inform`. With it off, only holds are declared and a permitted path produces nothing at all. With it on, every permitted telling on every path is declared, which is a positive audit trail bought with message volume. The cost is set out in section 10 and the expected counts are in the eval set. |
| `blocks_label` | no | House wording for what is withheld, used in `blocks[]`. Defaults to the affected field names. |

**The permit test, which is fixed rather than configured.** Take the destination the telling is
proposed to. Resolve it through `path_map` to one `gate_id`. Find the single `editorial_gates[]`
entry carrying that `gate_id` and of the configured `gate_scope_kind`. Read `status`. The path is
permitted when and only when `status` is `APPROVED`. `PENDING` withholds, `REJECTED` withholds,
a destination `path_map` does not name withholds, and an absent or unreadable entry withholds.
There is no expression language here and none is planned: the schema closed the enum, so the test
has one shape.

Note that `editorial_gates[]` is an array and the entries carry no path field, so a gate is found
by `gate_id` and never by writing something like `editorial_gates["web"]`. Earlier drafts of this
file did write it that way; it does not resolve and never did.

Whether a path with no readable gate entry is permitted is **not** configurable. It is held. See
section 10, where that choice and its editorial cost are set out.

The house also declares this skill in its own active set. Those values are copied straight from
the frontmatter above and are not a second set of choices: `skill_id`, `skill_version`,
`skill_type`, `disclosure_level`, `migration_policy`.

---

## 5. Runtime loop, specialised

```
on a telling being proposed, a story context arriving, or a gate status changing:
  1. RESOLVE ANCHOR   the link: this asset, to this destination, on this path. One link,
                      one evaluation. A message naming several destinations yields several
                      evaluations, never one shared answer
  2. GATE             does this tool own the destination named on the link, and does the
                      story carry any editorial_gates[] entry of the configured
                      gate_scope_kind? If either is no, exit quietly
  3. READ CONFIG      load the configured instance for this broadcaster / site / brand
  4. EVALUATE         resolve the destination through path_map to one editorial_gates[].gate_id,
                      read THAT entry only, and permit when its status is APPROVED. Every
                      other status, and an absent entry, withholds
  5. DECIDE SEVERITY  the path is not permitted, so the telling is withheld: hold. The path
                      is permitted: nothing at all unless record_permits is on for this
                      configured instance, in which case inform, so the positive read is
                      on the record and the audit can show why a release was allowed
                      rather than only why a hold was applied
  6. EMIT             the twelve-field warning, scope link:<id>, causation_id on the
                      triggering snapshot
  7. AWAIT CLEARANCE  only a gate status change on this same path, asserted at equal or
                      higher authority than the assertion that set it, closes a hold
  8. RESOLVE / CLOSE  the hold closes when that gate entry reads APPROVED. It closes for
                      this path alone, and leaves every other path's warning standing
```

**The loop run twice on one story state, the hurricane.** This is an illustration of steps
1 to 8 rather than a second mechanism. One story, one instant, two configured instances of this
same skill:

| | Digital CMS configured instance | Broadcast configured instance |
|---|---|---|
| `instance_label` | `house-storm-web-path` | `house-storm-broadcast-path` |
| `path_map` | `cms-web-path: eg-web`, `syndication-partners: eg-web` | `rundown-primary: eg-broadcast`, `playout-tx-a: eg-broadcast` |
| Gate read at this instant | the `editorial_gates[]` entry with `gate_id: eg-web` reads `status: APPROVED` | the entry with `gate_id: eg-broadcast` reads `status: PENDING` |
| What this produces | The page goes live carrying the confirmed storm category, and the unconfirmed casualty figure is withheld in place | The rundown holds its item at the gate and playout does not take it, until the broadcast gate clears, at which point the item releases with no further human action |

Everything else in the two configured instances is identical: the same `scope`, the same
`gate_scope_kind`, the same story, the same story state,
read at the same instant. Only the `path_map` differs, and with it which gate entry is read. Two
paths, two answers, and neither tool had to be told what the other was doing. That is one situation
worked twice, which is why it sits here as an illustration rather than in section 8, where two
genuinely different situations are required.

**Skill specific traps:**

- **The mistake a well-meaning vendor will make:** resolving the gate once, caching the answer as
  a property of the story, and applying it to every destination the tool serves. It reads as an
  efficiency and it is the exact failure this skill exists to prevent. The gate is a property of
  the pair (story state, path), so a cache keyed on story alone is wrong the moment a second path
  appears.
- **The second version of the same mistake:** reading the gate for the path a tool usually
  publishes to rather than for the path of the link in hand. A rundown that also feeds a website
  has two paths and must read two gates.
- **What must go in `detail` to be actionable:** which path was read, which gate entry was read
  for that path, the `gate_id` read, the status found, that `APPROVED` was required, and what
  would clear it.
  A message saying only "held" tells a producer nothing about which of their outlets is affected,
  which is the one thing they need at that moment.
- **Fail closed:** an unreadable gate, an unreadable config, or a destination that `path_map`
  cannot resolve is treated as the condition holding. That produces a hold, the loudest severity
  this skill has, with `detail` naming what could not be read. Silence is not available: a path
  whose gate cannot be read is not a path that has been cleared.
- **What does NOT clear it:** a gate clearing on a different path. A flag being cleared elsewhere
  on the story. An operator override at the publishing tool, because a hold is non-overridable.
  The item simply being late. Another skill declaring the fact confirmed, which changes the fact's
  status and not this path's permission.

---

## 6. Output contract

`skill.warning.raised`, severity `hold` when the path is not permitted. Severity `inform` when the
path is permitted and `record_permits` is on for the configured instance; where it is off, which is
the default, a permitted path produces no message. The payload below is the hurricane on the
broadcast path.

```json
{
  "topic": "som.skill.warning.raised",
  "originating_system": { "system_id": "playout-01", "system_type": "playout" },
  "correlation_id": "corr-storm-0912",
  "causation_id": "snap-storm-0912-0041",
  "timestamp": "<on the envelope, never in the payload>",
  "payload": {
    "warning_id": "wrn-7f2c9a41",
    "skill_id": "smart-stories/gate-by-scope",
    "skill_version": "0.2.2",
    "story_id": "story-storm-0912",
    "scope": "link:lnk-broadcast-rundown-item-14",
    "severity": "hold",
    "rule_id": "house-storm-broadcast-path",
    "non_overridable": true,
    "affected_fields": ["assertions[].value", "editorial_gates[].status"],
    "detail": "Destination 'playout-tx-a' resolves through path_map to gate_id 'eg-broadcast', whose editorial_gates[] entry reads status 'PENDING'; APPROVED is required. The unconfirmed casualty figure is withheld on this path. Released only by that entry moving to 'APPROVED' at equal or higher authority than the assertion that set it. The gate resolved for the web destinations is a different entry, evaluated separately, and is not affected by this warning.",
    "blocks": ["link:lnk-broadcast-rundown-item-14", "assertions[].value"],
    "skill_warning_ref": null
  }
}
```

`blocks` is populated because this is a hold, and it names both the telling withheld and the
field withheld on it: the link alone would not tell a producer what part of the item is the
problem, and the field alone would not tell them which outlet is affected. `non_overridable` is
`true`, permanently, for the hold severity. The `inform` case, declared only where the house has
turned `record_permits` on, carries the same shape with an empty `blocks` array and
`non_overridable: false`, because nothing is withheld when a path permits.

---

## 7. Priority position

| Axis | This skill |
|---|---|
| X category | `compliance`, X = 1 |
| Y type / source | `reference`, Y = 5 |
| Z precedence | unset. The publishing house sets it, nobody else |

This skill contests nothing, and no claim is made here that it outranks or is outranked by any
other skill: X and Y decide only which skill an executor follows when two skills clash on the same
action, and a deploying house's overrides and conditionals are not visible from inside a skill
file.

Enforcement is a different question from selection, and for holds it is answered by conjunction.
Where this skill contributes a gate, every binding gate must permit before the value is served, so
any one hold means held. A permit declared here does not release anything that another binding
gate is holding, and the `inform` recorded on a permitted path, where `record_permits` is on, is a
statement about this path's gate alone. The two phrasings withdrawn on 29 July 2026 do not apply and must not be
reintroduced in a house's own notes on this skill.

Co-binding is normal rather than an anomaly: a `link` commonly carries this skill's per-path gate
alongside a story-scoped hold from `hold-while-flagged`, and per-coordinate uniqueness is what
makes that safe. The audit records the most terminal outcome word over the locked
`som.system.audit` action set, WITHHELD over SUPPRESSED over CLEARED, which is a rating over a
closed set and does not change what is served.

---

## 8. Worked configurations

The hurricane is worked in section 5, where two configured instances of one situation illustrate
the loop. The two configured instances below are deliberately not that: they are drawn from
different situations, which is what this section is for and where the reuse claim in section 1 is
checked.

### 8a. A watershed restriction on a court report, linear against streaming

```yaml
instance_label: house-watershed-linear-path
scope: destination-path
gate_scope_kind: WATERSHED_RESTRICTION
path_map:
  playout-tx-a: eg-linear
  rundown-primary: eg-linear
  ott-player: eg-streaming
record_permits: false
blocks_label: "held until the watershed"
```

What this produces: a distressing sequence in a court report carries a time-of-day gate. Before
21:00 `eg-linear` reads `PENDING`, so the sequence is withheld on that path
and the item runs without it. `eg-streaming` reads `APPROVED` all evening, so the same
sequence carries there throughout. At 21:00 `eg-linear` moves to `APPROVED` and the
sequence releases on the `field_change` alone, with no new telling proposed and nobody at the
transmission tool doing anything. One gate state, read per path, three outcomes across the evening.

### 8b. A market-integrity restriction at a stadium, withheld on the in-venue path

```yaml
instance_label: house-trackside-timing-in-venue
scope: destination-path
gate_scope_kind: MARKET_INTEGRITY_RESTRICTION
path_map:
  cms-web-live: eg-general-web
  rundown-primary: eg-broadcast
  venue-app-spectator: eg-in-venue
record_permits: true
blocks_label: "withheld in venue"
```

What this produces: a live athletics meeting is covered on the general sports site, on a broadcast
rundown, and on an in-venue app served only to spectators inside the ground. Any fact derived from
the unofficial trackside timing feed carries a gate that reads `open` for the general site and the
rundown and does not for the in-venue path, because a latency advantage over the official feed
inside the ground is a market-integrity exposure. The narrowest audience is the most restricted
one, which inverts the usual assumption that a smaller audience is a safer one, and the skill needs
no change to serve it because the gate is written per path and read per path. This house turns
`record_permits` on for this configured instance, because an integrity audit wants the permitted
reads as well as the withheld ones and has accepted the traffic that buys.

**What changed and what did not.** What changed: the situation, from a watershed on a court report
to market integrity at a sports meeting; the `gate_scope_kind`, from `watershed_restriction` to
`market_integrity_restriction`, so a different class of gate entry binds; the `path_map` and the set of gates it names, from
linear against streaming to three paths of differing audience breadth; `record_permits`, off in 8a
and on in 8b; the `blocks_label`; and what releases the hold, which in 8a is the clock and in 8b is
a decision about the timing feed with no scheduled release at all.

What stayed the same: the skill file itself, the mechanism (resolve the destination to a path,
read that gate's entry, permit only on `APPROVED`), `scope: destination-path`, the
fail-closed treatment of a path whose gate cannot be read, `non_overridable` on every hold, and the
absence of any branch inside the skill for either case. Only the configured instance differs.

---

## 9. Eval set

Minimum five. The skill is not finished until a declaring case and a clearance case both pass.

| Case | Input | Expected |
|---|---|---|
| declaring | Telling proposed to `playout-tx-a`, which `path_map` resolves to `eg-broadcast`; that `editorial_gates[]` entry reads `status: PENDING` | one `hold`, scope `link:lnk-...`, `non_overridable: true`, populated `blocks`, and `detail` carrying the `gate_id` read, the status found and that `APPROVED` was required |
| non declaring | Telling proposed to a destination `path_map` resolves to a path with no gate entry of the configured `gate_scope_kind`, and no gate of that kind on the story at all | nothing, quietly. The gate step exits at condition two before config is read |
| clearance | `eg-broadcast` is asserted to `APPROVED` by the compliance desk that set it to `PENDING` | the `hold` on `link:lnk-broadcast-...` closes; where `record_permits` is on an `inform` records the permitted read and where it is off nothing replaces the closed hold; no other path's warning is touched |
| wrong clearance | A playout operator sets an override at the transmission tool to release the held item | not applied, the hold stands, `non_overridable: true` is unchanged, and the attempt remains visible in the record with its provenance |
| idempotency | The same snapshot delivered twice, in both clouds | one warning. Deduplication on (`skill_id`, `scope`, `causation_id`), which is per link rather than per story |
| fail closed | `path_map` cannot resolve the destination, or the gate entry is unreadable | a `hold`, the loudest severity this skill has, with `detail` naming which destination or which gate could not be read. Never silence |
| never auto-change | Any declaring case | assert that no message produced carried a content change: the item, the figure and the story are byte-identical before and after |
| per path divergence | One story state, `record_permits: true`. Telling A to `cms-web-live`, resolving to `eg-web`, which reads `APPROVED`. Telling B to `playout-tx-a`, resolving to `eg-broadcast`, which reads `PENDING`. Both evaluated at the same instant | two warnings, two scopes: an `inform` on `link:` A and a `hold` on `link:` B. A releases while B stays held. Neither warning names the other path, and clearing B later leaves A untouched. With `record_permits: false` the same input yields one message, the `hold` on B, and A releases silently |
| per path release ordering | Continuing the case above, `eg-broadcast` then moves to `APPROVED` | the `hold` on B closes on the `field_change` alone, with no new telling proposed and no human action at the tool. A's `inform` is not reissued |
| permit volume | One story with four destination paths, ten tellings proposed on each across a publishing day, every gate reading `APPROVED` throughout, so forty permitted evaluations and no holds | with `record_permits: false`, the default, zero messages. With `record_permits: true`, forty `inform` messages, one per permitted telling per path. The case exists so a house sees the number before turning the field on rather than after |
| gate absent for one path only | `web` has a gate entry of the configured kind; `broadcast` has none, and `path_map` resolves the destination | `web` resolves normally; `broadcast` produces a `hold` under fail-closed, with `detail` saying the path has no gate rather than that the path was denied |

---

## 10. Open items

- **A gate absent for a path is treated as held, and that has an editorial cost.** Fail-closed
  requires that an unreadable value is treated as the condition holding, so a destination whose
  path carries no gate entry publishes nothing. This is a real consequence and not a technicality:
  a house that adds a new outlet and forgets to add a gate row for it will find that outlet silent
  and will read the silence as a system fault rather than as a missing gate. The assumption made
  here is that absent means held. If the group decides absent should mean clear, this skill's fail
  closed eval case and the `path_map` requirement in section 4 both change, and the failure mode
  inverts from a silent outlet to an uncleared publication, which is the worse of the two.
- **The positive read is volume, and it is off by default for that reason.** An `inform` on every
  permitted telling on every path is a defensible audit intent and an undefended traffic figure: a
  skill that raises something on every message is worse than one that never raises, because the
  holds it does declare are read in a stream of routine permits. The four-path day in the eval set
  is forty messages from one story, and a house running two hundred stories a day with eight paths
  each is in the low tens of thousands of `inform` messages for a bus that also carries everything
  else. Nobody has measured what that costs on a real deployment, and no retention or sampling
  policy for `inform` exists anywhere in the model. What was decided here: `record_permits`
  defaults to off, so the reference declares holds only and a house opts into the positive trail
  one configured instance at a time, as 8b does. If the group settles a sampling rule, or decides
  audit requires the positive read unconditionally, the default flips and the eval counts change
  with it.
- **`disclosure_level: L2` means "the deepest level the skill declares content for".** Two
  definitions are in circulation, the other being how much of its own reasoning a skill reveals.
  The first was meant. Under the second reading the value would need reconsidering, because the
  `detail` string specified in section 5 reveals a good deal of the evaluation.
- **Targeting is written as though it binds.** `target_system_type` lists publication-capable
  tools, on the working position that an executor acts only when its own type matches. Under an
  advisory reading an off-target tool such as a compliance hub or an audit service would also
  recall this skill. That would be harmless and arguably useful for this skill in particular,
  since a per-path hold is exactly the thing an oversight desk wants to see across all outlets at
  once, but it is not what is assumed here.
- **The system that emits on this skill's behalf carries the tool's own system type, and
  `skill_worker` is withdrawn.** An executor is a sidecar to a vendor tool, and there is no
  newsroom-scoped class of skill-running system for one to belong to, so the executor that puts
  this declaration on the bus is the CMS, playout, rundown, MAM or distribution tool's own
  executor, and it names that tool's own type in `originating_system.system_type`. That is why the
  section 6 payload, whose `system_id` names a real playout tool, now reads `playout` rather than
  the `skill_worker` this library emitted in earlier drafts: a value whose only meaning was "a
  thing that runs skills" implied exactly the class that does not exist, and it is withdrawn.
  Whether the emitting tool's type is the right value for `originating_system.system_type` on a
  skill warning, as against some value naming the executor role rather than the tool the executor
  sits beside, is not settled in the schema. The position taken here follows from a decision taken
  outside the schema, that tools have executors and newsrooms do not, and the schema has not caught
  up with it. Worth stating rather than leaving to be found: this is one of the two places this
  file names a system type, the other being the advert's `target_system_type`, and the two now
  agree by construction, because the tool that may recall this skill is the tool that emits on its
  behalf.
- **How clearance authority is compared is not defined.** The rule adopted throughout this library
  is the stated one: a declaration closes only on an assertion of equal or higher authority on the
  same scope, so a gate clears only that way, where the scope is the path. That is this library's
  working position, not a rule any group has ratified, and no vendor should read it as settled or
  hard-code enforcement against it. What makes one authority equal to or higher than another is
  not specified anywhere either, so the authority carried on a gate assertion is a house-local
  string and the executor's comparison of it is house-local too. The wrong clearance eval case is
  written against a plain reading (a transmission operator is not equal to the compliance desk
  that set the gate) and would need rewriting if a formal comparison rule lands, together with
  whatever house-local field the deployment uses to name the clearing authority.
- **Whether `link` scope resolution is settled in the schema is doubtful.** The warning payload
  admits `story:<id> | link:<id>`, so link scope is legal. What is not pinned down is how a link
  id is minted, whether one telling always corresponds to exactly one link, and whether a link
  survives a destination being re-pointed. This skill assumes one link per telling per
  destination, stable for the life of that telling. If a link turns out to be coarser, for example
  one per destination across many tellings, the deduplication key in the idempotency case and the
  per-path independence in the divergence case both need re-examining. Given that this is the
  library's clearest link-scoped skill, that gap matters more here than anywhere else.
- **`depends` is empty, which will look like an omission.** It is not. Gate state is read off the
  story, not off any particular skill's output, so there is nothing to depend on. In particular
  this skill does not depend on `hold-while-flagged` and `hold-while-flagged` does not depend on
  it; the two read the same gate and flag state independently and are chained, when they are
  chained at all, through the bus.
- **`recall_on` topics beyond `story.context` are proposals.** `telling.proposed` and
  `story.gate.changed` name what needs to cause a look, not topics anyone has ratified. The name
  `recall_on` itself is also a proposal, replacing the older active-voice name under the 29 July
  convention.
- **Fail-closed at the skill layer is a different mechanism from safe state at the executor
  layer.** Here fail-closed means treating an unreadable value as the condition holding, which for
  this skill produces a hold. Safe state is the separate case where the priority model cannot
  resolve an unambiguous skill at a coordinate at all, in which case nothing is loaded and the
  executor does not act. Both can occur on the same path and they are not the same event.
- **A gate inherited through an editorial link would need the relationship condition.** If a clip
  is `derived_from` an asset whose path is gated, whether the derived clip's telling inherits that
  gate is a cascade question. A relationship condition kind has been proposed but is not in the
  condition set, so nothing here approximates it with field conditions. A house needing inherited
  gates today must write the derived asset its own gate entry.
- **The name does not follow `<operation>-<condition-type>` and is kept anyway.** `gate-by-scope`
  reads as an operation plus the dimension the operation runs on rather than an operation plus a
  condition type; the conformant form would be closer to `hold-on-path-gate`. It passes the test
  that matters most, since it names a mechanism and not a news situation, so nothing about it makes
  the skill single-use. The library name is the contract vendors are building to, so it is retained
  and recorded here rather than silently fixed. This is the same treatment `match-and-propose` and
  `apply-clearance` get.
- **`skill_uri` is left as a placeholder** because it is not this author's to supply and because
  whether it is required at all is unsettled: one position treats it as mandatory on every
  registered skill, the other as optional. A plausible-looking invented URI reads as a real one to
  the next person. `owner_editorial`, `owner_engineering`, `licence` and `skill_content_sha` are
  left for the same reason, the last being computed at build rather than typed.
- **The `skill_id` format is one of four in circulation.** Nothing agreed so far picks one, and
  `<publisher>/<name>` is used because it is the form with a worked precedent in the warning
  schema's own example, not because the question is settled. If the group picks another,
  this changes.

---

## 11. Vendor build notes (layer 3)

| Platform | What your executor does |
|---|---|
| Digital CMS | For each publish or schedule action, resolve the target channel to a gate path and read that path's gate before the action commits. Publish the fields the path permits and withhold the rest in place, so the page goes live with the confirmed category and without the unconfirmed figure. Do not hold the whole page because one field is held, and do not resolve the gate at story load and reuse it at publish time; a gate can move between the two |
| Playout | Treat the gate as a condition on take, evaluated at the transmission item's own path. A held item is not taken and is not silently dropped from the list either: it stays visible as held with its reason. When the gate clears, the item becomes takeable with no operator action, because the release is the gate change and not a new instruction |
| Rundown | Hold at the gate, per item, and show the producer which path is holding and what would clear it. An item held on the broadcast path while the same story is live on the web is the normal case, not an inconsistency to reconcile. Never propagate one item's hold to the story's other items |
| MAM | You are usually the source of the asset rather than a destination, so recall applies to you only where you serve a path directly, for example a public media portal or a partner delivery. Where you do, evaluate per delivery target. Where you do not, carry the gate state with the asset so the publishing tools downstream can read it, and do not summarise it into a single cleared or not-cleared flag on the asset |
| Social and distribution | Each account, feed or partner endpoint is its own path. Resolve every one through `path_map` rather than treating "social" as a single destination, because a wire partner and an owned account are not entitled to the same answer. A scheduled post must re-read its path's gate at the moment of posting, not at the moment of scheduling |

The instinct this skill exists to interrupt is the natural one, and it is why per-path holds are
manual and fragile today: making one publish decision for the whole story, then remembering to
apply it everywhere. Somebody has to hold the figure in their head across every outlet, or each
system has to be told separately, and one slip publishes a held figure somewhere. Here nobody
remembers anything and nobody is told: every outlet reads the gate for its own path off one story
state, and the answers are allowed to differ.
