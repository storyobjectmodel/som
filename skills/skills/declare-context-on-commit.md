---
# --- identity ----------------------------------------------------------------
skill_id: smart-stories/declare-context-on-commit
name: declare-context-on-commit
skill_version: 0.2.2
som_schema_version: ["1.0.0"]

# --- what it is --------------------------------------------------------------
description: >-
  Where a link commits an asset to a destination, the story context that applies on that
  link is declared and displayed: which story the destination is carrying, and the
  configured story fields that describe its editorial posture. What is declared names the
  link, the destination, the asset and the values read from the story at that moment. It declares only.
  It does not choose a presentation, colour a surface, drive a label, route anything, withhold
  anything, or decide whether the destination may carry the asset at all. What a downstream
  surface does with it is the executor's, and this file holds no opinion on it.
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
  target_system_type: [playout, mam, graphics]
  target_capability: null
  conditions:
    - kind: field
      path: "assets[].usage[].state"
      op: equals
      value: "committed"
    - kind: field
      path: "{{ config.match_field }}"
      op: equals
      value: "{{ config.match_value }}"
  recall_on: ["story.context", "asset.context", "link.committed"]
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

# declare-context-on-commit

**Where a link commits an asset to a destination, say which story that link is carrying and what the story says about itself. Decide nothing else.**

**Status: PROPOSED.** This skill is in no release of som-skill-library and ships in no package. The library is ten skills; this is a candidate eleventh. Anything that describes it as the eleventh library skill is wrong, and the walkthrough now labels it accordingly.

Read `../docs/CONVENTIONS.md` first. It is carried in this pack unchanged from the library's first release, because this file is authored to those conventions and proposed as an addition to that library.

**Changed in 0.2.1, 20 August 2026.** One path. Section 4 and the section 9 declaring row wrote the committed-state field as usage[].state, which does not resolve from the root of `story.context`; it is `assets[].usage[].state`. Nothing else in the file moves, and the finding in section 10 is unchanged.

**What changed in 0.2.0.** `automation` is removed from `target_system_type`. The rest of the file is unchanged in substance; sections 2, 8, 9, 10 and 11 are updated to match, and a third worked configuration uses `tags[]`, which landed on `story.context` on 18 August.

---

## 1. What it is (layer 1)

| | |
|---|---|
| Operation | Declare the story context that applies at a destination, and display the configured fields read from the story |
| Condition type | `field`: an asset stands committed to a destination, and a configured story field carries a configured value |
| One mechanism? | Yes. It declares a state and displays it. Nothing is withheld, nothing is presented, nothing is changed. The sentence needs no "and": the destination is carrying a story, and here is what the story says. The presentation that a reader imagines next, a coloured tile or a label under a picture, belongs to the executor and is described in section 11 rather than governed here. A file that both declared the context and specified the treatment would occupy two X categories, workflow and appearance, and could not be resolved at a single coordinate |

Category is `workflow`, so X = 3, because the question being asked is where a story has got to and what is carrying it. Routing and handoff are named in the workflow category directly. It is deliberately not `appearance`, even though the first reader will picture a coloured tile, because the appearance decision is the executor's and this file must not take it.

**Reuse test, three scenarios from different stories:**

1. **The gallery surface.** A cut package stands committed to a programme output. The destination surface has never had any way to know which story the picture belongs to, and the declaration gives it the slug, the story type and the priority the desk set.
2. **The syndication desk.** A clip stands committed to a partner feed under an ongoing supply agreement. The partner's operations view carries no editorial context at all today. The same skill, `match_field` pointed at the compliance posture rather than the priority, declares which embargo state applies to what that feed is carrying.
3. **The shared venue feed, not drawn before.** One asset stands committed to two destinations at once, a broadcaster's programme output and a venue's in-house screens, and the two destinations carry different editorial postures on the same material because one is bound by a rights window the other is not. An operator standing in front of the venue surface needs to see the posture that applies to the destination in front of them, not the posture that applies to the asset in general. This library has not drawn a case before where the same asset carries two different declared contexts at the same moment, and the destination scope in the configuration is what makes it expressible.

One skill, three subjects, and the third one turns on a distinction the library has not previously needed: context declared per destination rather than per asset.

**Why this is not `raise-flag-on-match` with a second condition.** The first question a reviewer asks is whether an existing skill and a longer configuration would do. Three things say no, and they are worth having together rather than spread through the file. The X category differs: `raise-flag-on-match` is `compliance` at X = 1 and this is `workflow` at X = 3, so the two resolve at different coordinates and cannot be the same file. The scope differs: `raise-flag-on-match` reads a field on the story and declares at story scope, and this declares at `link:<id>` because the same asset committed to two destinations carries two different answers. And the two destinations case in section 9 cannot be expressed at all by a story-scoped skill, because it turns on the same story yielding different declared values on different links at the same instant. A second condition on an existing skill gets none of those three.

---

## 2. Recall advert

The advert is declared in the frontmatter above so a tool can read it without parsing prose.
This section explains it in words for a human.

- **Who it is for:** tools that hold or hand on a committed asset and have an operator surface in front of a human. `playout`, `mam` and `graphics`, all three of which exist in the library's targeting vocabulary today and all three of which carry such surfaces.

  **`automation` was named in 0.1.0 and has been removed.** It appears in the schema's `originating_system.system_type` enum and not in the library's `target_system_type` vocabulary, so advertising for it produced a row the lookup table could not index. A skill that advertises for a type the table cannot carry is not reaching that type; it is stating a wish. Removing it makes the advert honest and moves the whole of the problem to where it actually lives, which is that a plant control system has no home in the targeting vocabulary at all. Section 10 sets that out and it remains the reason this file exists.
- **What wakes it:** a story snapshot, an asset snapshot, or the commitment of an asset to a destination.
- **What it depends on:** that `assets[].usage[].state` reads `committed`, which the schema maintains only from link events, and that a configured field on the story carries a configured value.

Two parts of this advert are unusual and are named here rather than left for a reviewer to find.

`link.committed` is used as a recall topic and is not among the fourteen topics the current table indexes. The library's own position is that `recall_on` names are proposals rather than ratified names, so a new one is not a breach, but an executor built against today's table will not see it. The other two topics are enough on their own, and an executor that reads only `story.context` and `asset.context` still reaches every case in section 8. The third topic buys promptness, not coverage.

The commitment test reads `usage[]` rather than a link event directly. That is deliberate. `usage[]` is the asset side back edge index and is committed-only, so reading it keeps this skill on the story object and out of the business of correlating events.

The second condition watches a config-templated path, `{{ config.match_field }}`, which the lookup table cannot index as a single row. The house registers one row per configured instance, one for each condition it wants watched. The first condition is a literal path, so a table that indexed only that one would return this skill against every committed asset in the building, which is why both conditions belong in the advert rather than one in the advert and one in section 5.

---

## 3. Firing anchor

- **Evidential position:** any. What is being carried matters here; how it was gathered does not.
- **Outlet / path:** the destination's path. The declaration is about one destination and is meaningless without it.
- **Scope:** `link:<id>`, always. This is destination specific by construction. A story-scoped configuration of this skill would be declaring that a story is being carried somewhere without saying where, which answers no question anyone has.
- **Compliance question, in one plain sentence:** if someone were looking at this destination now, could they tell which story it is carrying and what the story currently says about itself?

---

## 4. Config surface (layer 2)

Every editorial choice sits here, set by the publishing house. Nothing here is in the skill.
Scope runs broadcaster, then site, then brand, the more specific overriding the default.

| Field | Required | Notes |
|---|---|---|
| `instance_label` | yes | The editorial label for this configured instance. Rides on the warning as `rule_id` |
| `destination_scope` | yes | Which destinations this configured instance covers. A destination id, or a pattern the house maintains. A configured instance that covered every destination would declare against a partner feed and a gallery surface in the same words, which is the mistake section 11 names |
| `declared_fields` | yes | The story fields carried in the declaration. `slug`, `story_type`, `priority.level` and `premise.premise_changed` are the obvious four, and a house may declare fewer. Declaring more than a surface can display is noise |
| `match_field` | yes | The story field tested to decide whether this configured instance concerns the story at all. Since 18 August this may be `tags[].value`, the subject-matter field on `story.context`, which is the first path a house can use to scope a configured instance by what a story is *about* rather than by its lifecycle state. Read the array-path warning in section 11 before configuring it |
| `match_value` | yes | The value `match_field` must carry |
| `redeclare_on_change` | no | Whether a further declaration is made when a declared field changes while the asset stands committed. Defaults to true. A house that sets it false gets one declaration at commitment and a surface that goes stale, which is a choice it should make knowingly |

The house also declares this skill in its own active set. Those values are copied straight from
the frontmatter above and are not a second set of choices: `skill_id`, `skill_version`,
`skill_type`, `disclosure_level`, `migration_policy`.

---

## 5. Runtime loop, specialised

```
on story.context / asset.context / link.committed:
  1. RESOLVE ANCHOR   the destination named in usage[].destination_id, and the asset carrying it
  2. GATE             does usage[].state read committed? if not, exit quietly
  3. READ CONFIG      load the configured instance for this broadcaster / site / brand
  4. EVALUATE         does destination_scope cover this destination, and does
                      match_field carry match_value on the story?
  5. DECIDE SEVERITY  inform, always. This skill has no other severity
  6. EMIT             the twelve-field warning, causation_id on the triggering snapshot
  7. AWAIT CLEARANCE  nothing is withheld, so nothing waits. A declaration is closed by
                      the commitment ending, not by an authority
  8. RESOLVE / CLOSE  when usage[].state stops reading committed, or the link is withdrawn
```

**Skill specific traps:**

- The mistake a well meaning vendor will make is treating this as permission to drive the surface directly, so that a declaration arriving is wired to a tile turning red. That inverts the model. The declaration says what the story holds; the treatment is the executor's own, configured in the product, and two products reading the same declaration are expected to present it differently.
- The second mistake is declaring per asset rather than per destination. An asset committed to two destinations under two configured instances yields two declarations, and collapsing them to one loses exactly the distinction scenario 3 in section 1 turns on.
- What must go in `detail` for the message to be actionable: the destination, the story, and each declared field with the value read, so that a human reading the warning alone can tell what a surface should be showing without opening the story.
- Fail closed: where the configuration is missing or unreadable, or a declared field cannot be read from the story, a declaration is made at `inform` whose `detail` says which field could not be read. Silence would leave a surface confidently displaying a stale context, which is worse than a surface that says it does not know.
- What does not close it: a declared field changing value. That yields a further declaration where `redeclare_on_change` is true, and the earlier one stands as the record of what was true at commitment.

---

## 6. Output contract

`skill.warning.raised`, severity `inform`, published on `som.skills.events` per the 10 August decision paper, pick 5. If that pick has not landed in the library, this reverts to `som.skill.warning.raised` and the note in section 10 stands.

```json
{
  "topic": "som.skills.events",
  "originating_system": { "system_id": "<tool>", "system_type": "automation" },
  "correlation_id": "<the story's chain>",
  "causation_id": "<the snapshot that triggered evaluation>",
  "timestamp": "<on the envelope, never in the payload>",
  "payload": {
    "warning_id": "wrn-....",
    "skill_id": "smart-stories/declare-context-on-commit",
    "skill_version": "0.2.2",
    "story_id": "story-...",
    "scope": "link:0190a000-0000-7000-8000-00000000aa01",
    "severity": "inform",
    "rule_id": "<the configured instance label>",
    "non_overridable": false,
    "affected_fields": ["assets[].usage[].destination_id"],
    "detail": "Destination PGM-A is carrying story harbour-fire-2026-0913. slug HARBOUR-WAREHOUSE-FIRE, story_type ACTIVE, priority.level URGENT, premise.premise_changed true.",
    "blocks": [],
    "skill_warning_ref": null
  }
}
```

`blocks` is empty and stays empty. This skill has no hold authority and `non_overridable` reads false, which is correct for a declaration that withholds nothing.

---

## 7. Priority position

| Axis | This skill |
|---|---|
| X category | `workflow` |
| Y type / source | `reference` |
| Z precedence | unset. The publishing house sets it, nobody else |

It outranks nothing and is outranked by every compliance and editorial declaration standing on the same story, which is the right way round: a surface being told what it is carrying must never resolve ahead of a hold being applied to the thing it is carrying. This skill contributes no gate, so it takes no part in the conjunction by which holds combine.

---

## 8. Worked configurations

### 8a. The gallery surface at the stand

```yaml
instance_label: house-gallery-context
destination_scope: ["dest-gallery-pgm-a", "dest-gallery-pgm-b"]
declared_fields: ["slug", "story_type", "priority.level", "premise.premise_changed"]
match_field: "story_type"
match_value: "ACTIVE"
redeclare_on_change: true
```

An operator standing in front of that output can tell which story it is carrying, that the story is active, that the desk has it at urgent, and that its premise has changed since the package was cut. None of which was available to that surface before.

### 8b. The partner feed under a supply agreement

```yaml
instance_label: house-syndication-posture
destination_scope: ["dest-partner-feed-*"]
declared_fields: ["slug", "compliance[].type", "compliance[].status"]
match_field: "compliance[].status"
match_value: "ACTIVE"
redeclare_on_change: true
```

### 8c. Scoping by subject, using the field that did not exist until 18 August

```yaml
instance_label: house-courts-desk-context
destination_scope: ["dest-gallery-pgm-*"]
declared_fields: ["slug", "story_type", "compliance[].type"]
match_field: "tags[].value"
match_value: "02000000"
redeclare_on_change: true
```

`02000000` is the IPTC Media Topic for Crime, Law and Justice. This configured instance concerns itself only with court stories, and declares the compliance posture alongside the slug on every programme output, because a reporting restriction on a court case is the thing an operator in front of that surface most needs to see and is the thing least likely to reach them today.

The reason this configuration could not be written before is worth stating: until 18 August nothing on `story.context` said what a story was *about*. Every axis carried lifecycle, priority, compliance or provenance. A house wanting to scope by subject had to enumerate skills per story in `skills_config`, which puts routing in the NCS, or substring-match the headline, which is how a safety gate once matched `graphic` against the word "graphics".

**What changed across 8a, 8b and 8c:** the destination scope, the fields declared and the field tested. **What stayed the same:** the skill file, the commitment gate, the per destination scoping and the severity. One declares an editorial posture to a gallery, one a compliance posture to a partner, one a court restriction scoped by subject, and nothing in the file knows the difference between them.

**A note on ordering, because 8c depends on it.** `tags[]` is ordered and the first entry is primary. A configured instance whose `match_value` matches more than one entry resolves on the earliest, so a story that is genuinely both courts and politics resolves the same way every time. A house pointing `match_field` at `tags[].value` is relying on that guarantee and should know it is relying on it.

---

## 9. Eval set

Minimum five. The skill is not finished until a declaring case and a clearance case both pass.

| Case | Input | Expected |
|---|---|---|
| declaring | An asset with `assets[].usage[].state` committed to a destination in scope, on a story where `match_field` carries `match_value` | One `inform` declaration, `scope` reading `link:<id>`, and `detail` naming the destination, the story and every declared field with its value |
| non declaring | The same asset, committed to a destination outside `destination_scope` | Nothing at all on the bus, quietly |
| clearance | The link is withdrawn, so `assets[].usage[].state` no longer reads committed | The declaration closes. Nothing waited on an authority, because nothing was withheld |
| wrong clearance | An operator surface attempts to close the declaration while the asset stands committed | Not applied. The declaration stands, and the attempt stays visible in the trail |
| idempotency | The same snapshot twice, in both clouds | One declaration |
| fail closed | `declared_fields` names a field the story does not carry | One `inform` declaration whose `detail` names the field that could not be read, rather than silence |
| never auto-change | Any declaring case | Assert that no message produced here changed any content, any story field, any destination or any presentation |
| two destinations | One asset committed to two destinations under two configured instances | Two declarations, one per destination, each carrying its own scope and its own declared values |
| array-valued watched field | `match_field` configured to `tags[].value` on a story carrying two tags, the second of which is the configured value | The condition holds and one declaration is made. An executor whose path resolver walks objects only will resolve this to null and declare nothing, silently, which is why this case is here rather than assumed |
| ordered resolution | `match_field` configured to `tags[].value` on a story whose first tag matches one configured instance and whose second matches another | The earliest matching entry resolves. Both configured instances are consulted; the primary tag decides |

---

## 10. Open items

- **The two targeting vocabularies do not agree, and this skill sits on the seam.** The envelope's `originating_system.system_type` enum in SOM v0.3.2 reads `ncs, mos_device, graphics, automation, wire_service, ai_agent, compliance_engine, editorial_dashboard, archive, prompter, camera, audio, skill_worker, custom`. The library's `target_system_type` vocabulary reads `rundown, mam, cms, playout, graphics, planning, discovery, transcription, verification, multimodal, ingest, archive, distribution, markets_data, compliance_hub`. Only `graphics` and `archive` appear in both. This file targets the library's vocabulary, because that is what the lookup table indexes. Whether the table should index the schema enum, the functional vocabulary, or a mapping between them is unresolved and is larger than this skill.
- **A plant control system has no row, and 0.2.0 stops pretending otherwise.** Version 0.1.0 named `automation` in the advert, which reads as coverage and is not coverage: a value that exists only in the schema's enum cannot be indexed by a table built from the library's vocabulary, so the row was never going to exist. Removing it makes the file internally consistent and leaves the gap where it belongs, on the executor side. A control and monitoring layer still cannot be reached by any skill in this library, and the gallery case in 8a still has nothing to run on that class of tool.
- **The word for it is not ours to pick.** `automation` was the nearest value in the schema's enum, but in this industry automation usually means playout automation, a scheduler running a channel, and a control and monitoring layer driving routers, multiviewers and tally is a different animal. Whatever value is added should be the one the vendor would use for themselves, asked rather than guessed. Until that answer arrives no value should be added, and this file should not name one.
- **The workaround is worse than the gap.** Having a control system present its executor as `playout` would be a misrepresentation that then gets indexed, and it would resolve at a coordinate nobody intended. Naming the gap costs one vocabulary value. Working around it costs the meaning of the vocabulary.
- **Subject-matter conditions became possible on 18 August, and this file now uses one.** `tags[]` landed on `story.context`: ordered, scheme-qualified, `{scheme, value, label?}` where scheme is `newsroom`, `iptc-mediatopic` or `com.{vendor}.{name}`. Configuration 8c depends on it. Two consequences worth naming. `label` is display only and a configured instance must never test it, because an opaque identifier is what makes two houses comparable and a human-readable label is not. And a reference path resolver was found on the same day to walk objects only, which meant an array-valued watched field resolved to null and a configured instance pointed at `tags[].value` would have declared nothing, silently and forever. That is a vendor-side defect rather than a skill-side one, and it is in section 11 because every executor has the same code to check.
- **The shared policy index is the decided mechanism, as of 18 August.** Proposal A, policy files plus a generated lookup table plus house-registered configured instances, was chosen over executor-local policy on the grounds of provable conformance. This file was already written to A and nothing in it changes. What changes is that the advert row is now the load-bearing artefact rather than one of two candidate designs, which raises the cost of the vocabulary gap above rather than lowering it: under the alternative, an executor could have carried this policy locally whatever the table said.
- **`som_schema_version` is not a hedge.** This file declares `1.0.0`, and its field paths are checked against the published SOM 1.0 story-context schema on every library run. Older review notes citing `0.3.1` or `0.3.2` are stale and should not be used to date this file.
- **`link.committed` is a proposed recall topic.** The library's position is that `recall_on` names are proposals rather than ratified names. This one is not among the fourteen the current table indexes, and an executor built against today's table will reach the same cases through `story.context` and `asset.context`, later.
- **A telling may be the better anchor, and the counter-reading is worth stating.** This file anchors on commitment because a link is what `usage[]` records and what a destination stands against. The counter-reading is that a tally reflects on-air state, on-air state is derived from tellings and never stored on the asset, and so a skill written for a surface that shows what is on air should anchor on `telling.started` instead. Both readings are defensible. Anchoring on the link was chosen because a destination is carrying an asset from the moment it is committed, whether or not it is exposed, and a surface that only knew about exposure would be dark during preparation, which is when an operator most needs to know what is loaded. If the group prefers the telling anchor, the advert changes and section 3 changes with it; nothing else in the file does. The likelier answer is that a gallery surface wants both, the link for what is loaded and the telling for what is on air, and that the second is a configuration of a telling-anchored sibling rather than a change to this file. A source lock, which must not fire on a loaded source and must fire on a live one, can only be built on the telling, and that is the clearest case anyone has put for the sibling existing.
- **Targeting binds.** Pick 4 of the 10 August decision paper proposes that `target_system_type` binds, with advisory targeting expressed by omitting the field, and this file is written to that pick. The advisory reading would have dissolved the `automation` problem above by letting an off target executor pick the skill up anyway; if pick 4 stands, that escape is closed and the vocabulary gap has to be answered on its merits.
- **The advert is config-templated, so it cannot be indexed as one row.** `{{ config.match_field }}` is itself configuration, and fixing the warning would mean writing one house's field name into a library skill. The house registers one row per configured instance instead. This is the same answer the other seven config-templated skills in the library give, and it is answered here rather than fixed for the same reason.
- **How clearance authorities are compared is undefined, and this file assumed its way past it.** Nothing is withheld here, so no authority is needed to release anything, and the wrong clearance eval case turns on that: an attempt to close a standing declaration is not applied because the declaration closes on the commitment ending rather than on anyone's say so. That assumption holds only while this skill carries no hold authority. Were a house to argue that a declaration should be dismissible by an operator, the comparison rule the rest of the library waits on would apply here too.
- **`disclosure_level: L2`** is meant here as the deepest level for which this skill declares content. Two definitions are in circulation and this file states which one was meant.
- **`skill_uri` is left as a marker**, and whether it is required at all is unresolved: one field table in circulation marks it required and another marks it optional, which decides whether a skill may exist without declaring a home. Which of the four circulating `skill_id` formats is canonical is unresolved in the same way; the `<publisher>/<name>` form used here follows the warning schema's own worked example, the only one of the four with a worked precedent.
- **No dependency is declared**, in line with the library's chaining rule. Nothing in this file depends on another skill, and no other skill in the library reads what this one declares. It is the only skill in the set whose output is intended for a human looking at a surface rather than for another skill or a producer's queue, which is worth a reviewer's attention: if that is thought to be outside what `skill.warning.raised` is for, this skill needs a different output message and there is no other message with a ratified payload.

---

## 11. Vendor build notes (layer 3)

| Platform | What your executor does |
|---|---|
| Playout | You already know which asset is loaded against which output. Read `usage[]` on the story rather than inferring commitment from your own playlist, and declare per output, not per event |
| MAM | You hold the asset and its `usage[]` back edge. You are frequently the only tool that can see the same asset committed to two destinations at once, which makes you the natural place for the two destinations eval case |
| Graphics | A renderer bound to a story already reads the fields this skill declares. Declare what the destination is carrying; do not use the declaration to change what you render |

**A note for plant control, which the table above deliberately does not address.** A control and monitoring layer knows the destination, the multiviewer tile and the tally state, and none of that belongs in a declaration. This skill cannot reach that class of tool at present: its type is not in the targeting vocabulary, and 0.2.0 stopped claiming otherwise by removing `automation` from the advert. There is no build note for it above because under the binding reading of targeting a build note addressed to an unnamed type describes work no executor could be recalled to do. When a value for what such a tool actually is has been agreed and added, this is the skill that surface has been missing, and nothing in this file changes except one entry in the advert.

**One thing every executor must check, whatever the platform.** If a configured instance points `match_field` at `tags[].value`, or at any other array-valued path, your path resolver has to walk arrays and not only objects. A resolver that walks objects only returns null for an array-valued path, the condition reads as not holding, and the configured instance declares nothing for the rest of its life without producing a single error. That defect was found in a reference engine on 18 August, the day the field landed. It is silent, it looks exactly like a story with nothing to declare, and the eval case named "array-valued watched field" in section 9 exists to catch it. Run it before you configure a subject-scoped instance, not after.

The instinct this skill exists to interrupt is the reflex to wire a declaration straight to a treatment, so that a story going urgent turns a tile red inside the same piece of code that read the story. The moment those are one thing, the house has lost the ability to change what urgent looks like without a vendor release, and the vendor has lost the ability to present it in the way its own operators expect.
