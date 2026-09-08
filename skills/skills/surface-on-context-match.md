---
# --- identity ----------------------------------------------------------------
skill_id: smart-stories/surface-on-context-match
name: surface-on-context-match
skill_version: 0.2.2
som_schema_version: ["1.0.0"]

# --- what it is --------------------------------------------------------------
description: >-
  A source is monitored continuously, and when an item from it reads as a match against
  the running story's context at or above a configured bar, that item is declared as a
  candidate and displayed as a source enrichment carrying its provenance: the source, the
  speaker, the timecode and the match confidence. It surfaces only. It never clears a
  held fact, never releases a gate, never withholds, blocks, delays or routes anything,
  never ranks the story's other material, and never changes content. It is additive and
  sits off the critical path: nothing waits for it. The clearance that may follow a
  surfaced item is a separate assertion by an authority, and the hold that releases does
  so because its own gate cleared, which is a different skill advertising against that
  state over the bus.
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
  target_system_type: [transcription, verification, discovery, markets_data, mam, planning]
  target_capability: null
  conditions:
    - kind: field
      path: "{{ config.monitored_source }}"
      op: active
    - kind: field
      path: "{{ config.match_field }}"
      op: contains
      value: "{{ config.match_against }}"
  recall_on: ["source.item.observed", "story.context"]
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

# surface-on-context-match

**When an item from a monitored source matches the running story's context, surface it as an
enrichment carrying its provenance, speaker, timecode and confidence. Do not clear anything.**

Read `../docs/CONVENTIONS.md` first.
**Changed in 0.2.1, 20 August 2026.** One thing. Sections 2 and 10 still printed the registration question as open after proposal A closed it. Both now record A as decided.

**Changed in 0.2.0, 18 August 2026.** Three things. `som_schema_version` moves to `["0.3.2"]`, which as of 18 August is the version. Field paths in the advert and in the worked configurations were checked against the v0.3.2 story-context JSON Schema for the first time and the ones that did not resolve are corrected; see the pack's `SCHEMA-CONFORMANCE.md` for the full list and the evidence. And the shared policy index was chosen over executor-local policy, which settles the registration question this file used to leave open: the house registers one advert row per configured instance.

---

## 1. What it is (layer 1)

| | |
|---|---|
| Operation | surface: one item from one monitored source is declared a match against one story's context and displayed as a source enrichment with its provenance |
| Condition type | context match: the item read against the story's context, at or above a configured match bar |
| One mechanism? | Yes. The sentence is "surface an item that matches the story's context, with its provenance", which needs no "and". The raise-or-act test resolves to **raise**: this declares a state (a candidate exists, here is who said it and exactly where) and withholds nothing at all. The editorial moment this is drawn from describes two things happening in sequence, the confirming figure being surfaced and the held figure then clearing, and those are two mechanisms. The clearance is a separate assertion by an authority against the held fact, and the hold releases because its own gate cleared, which belongs to `hold-while-flagged` and `gate-by-scope`. A skill that surfaced a candidate and also released a hold on the strength of it would occupy two positions and could not be resolved at a single coordinate. X is `workflow` because this governs how a discovery moves into a running story (what is watched, what reaches a producer, in what order), not whether the story may be published |

**Reuse test, three scenarios from different stories:**

1. **A live official press conference monitored against a running breaking-weather story.** The transcript stream is watched against `story.context`. When an authority speaks the confirming wind-speed figure, that one segment is surfaced as a source enrichment with the speaker's name, the timecode in the conference feed and the match confidence. Nothing about the story's held figure is touched by this skill.
2. **A markets story, the primary regulatory filings feed monitored against a running issuer story.** The same mechanism, a different corpus. When a filing whose body reads against the story's context is published, it is surfaced with the filing's issuer, its publication timestamp and paragraph reference in place of speaker and timecode, and its match confidence.
3. **A multi-day search and rescue at sea, monitored against a foreign-language coastguard broadcast the newsroom does not own and does not work in.** This library has not previously drawn a maritime story, and this goes further than the two above in three ways. The monitored corpus is in a language the story is not written in, so the read is cross-lingual and `confidence` would have to carry translation confidence as well as match confidence. That is the scenario's named blocker rather than a value someone sets: nothing in the schema says whether two confidences of different kinds may be combined into one number, or what scale the result would be on, so `config.match_threshold` cannot be configured honestly for this case until the group answers it (see the confidence-scale item in section 10). The source is not the newsroom's, so the provenance pointer is into a broadcast it cannot re-cut, which makes the timecode the only durable link back. And the story runs for days, so the same configured instance stays in place across many shifts and many producers, each of whom sees a surfaced item and the exact moment it came from rather than a note someone left.

One skill, three subjects, three monitored corpora and three notions of what a timecode is. Scenarios 1 and 2 are configuration and nothing else: the mechanism did not change between them. Scenario 3 needs no new mechanism either, and it is not served today, because the one thing it does need is an answer to an open schema question about what a single confidence number may combine. It is carried here as a stretch case with a named blocker, not as a case the skill serves unchanged. In none of the three is anything cleared, released or withheld.

---

## 2. Recall advert

The advert is declared in the frontmatter above so a tool can read it without parsing prose.
This section explains it in words for a human.

- **Who it is for:** the tools that hold a corpus and can read it against a story, plus the tool a producer reads the result in. `transcription` carries the live transcript and can point at a moment in it, `verification` reads that transcript for the claims made in it and discriminates the one claim that bears on the story, `discovery` runs a story against an archive or an external corpus, `markets_data` watches a filings or regulatory news feed against a running issuer story, `mam` holds the media the surfaced item points at and can land the conference recording so a cleared record has something to name, and `planning` is where a producer sees the candidate and decides what to do with it. These are broad system types, not named products, because whether the lookup table names a type or a precisely named product is not settled. Under the binding reading of targeting taken across this library, a tool type absent from this list cannot recall the skill at all, so the list is held to exactly the six executors section 11 gives build notes to: no more, and no fewer. Whether every one of those six spellings is a member of the v0.3 `system_type` enum is not something this file can verify, and section 10 says so rather than asserting it.
- **What wakes it:** `source.item.observed`, which is the topic a new item from a monitored source arrives on, and `story.context`, because a revision to the story's context changes what counts as a match and items already seen may need reading again against the revised context.
- **What it depends on:** two field conditions, composed with AND. The first tests that the source this configured instance monitors is `active`. The second tests that the item field named on this configured instance reads against the configured story-side field, which is `story.context` in every configured instance drawn so far. Both sides of both comparisons come from configuration, so no editorial choice sits in this file.

Four things about this advert are unusual and are named here rather than left for a reviewer to find.

**The paths are config-templated.** The monitored source and the item field are `{{ config.monitored_source }}` and `{{ config.match_field }}`, not literal dot paths, because the whole point of the skill is that the house chooses what is watched. A single-purpose skill advertises a literal path and the global lookup table indexes it as one row. This one cannot be indexed that way: the two templated paths are only known once configuration is read, so under proposal A, chosen 18 August, the house registers one advert row per configured instance in its own registration layer. Of the two, this file is written against the second, so a reader should expect the house to register one row per configured instance rather than a single row standing for the whole advert. Which of the two is correct is unresolved, and it is recorded again in section 10.

**The op set has no similarity operator.** The originating configuration writes the match as `item ~ story.context`, and `~` does not exist among `exists`, `absent`, `equals`, `contains` and `active`. `contains` is the nearest available and is used here as a coarse recall-time filter only. The actual scored read against the story's context, and the comparison against `config.match_threshold`, happen at the EVALUATE step in section 5, not at recall. This split is deliberate: it keeps the advert cheap enough to be matched against every message in a running newsroom, and it keeps the expensive read behind the gate.

**Nothing here reaches forward to a clearance.** The surfaced enrichment is a state on the story. Whether a held fact then clears against it is decided by an assertion from an authority, and whether a hold releases is decided by the gate that raised it. Those arrive by other skills advertising against the state this one wrote, over the bus: two skills, two tools, one bus, no integration between them. This file therefore has no knowledge of what may be cleared behind a surfaced item, which is why `depends` is empty.

**Continuous monitoring sits awkwardly against a field-driven advert.** Recall is triggered by messages on the bus, and a live transcript is a stream rather than a sequence of story field changes. This is a real edge for the recall model and is not assumed away here: the working position taken is that a transcription tool publishes each usable segment as its own `source.item.observed` message, so the stream becomes bus-observable and the ordinary field-driven advert applies unchanged. If a house instead runs the monitor inside its own executor and never puts segments on the bus, this skill is never recalled and the tool has invented a private mechanism. Recorded again in section 10.

---

## 3. Firing anchor

- **Evidential position:** any. The surfaced item is a candidate, so it has no established evidential position at the moment it is surfaced, and a skill that required `PRIMARY` would never be recalled at the moment this one is for. Assigning a position to the item, if it ever gets one, is a human act after the surfacing, not part of it.
- **Outlet / path:** the story's own context, not a destination path. No outlet is implied and no outlet is excluded, and the surfaced enrichment does not attach to any particular telling.
- **Scope:** `story:<id>`. A candidate discovered against the story's context is a property of the story at system level. Nothing here is destination specific; where a fact's availability differs by path, that is `gate-by-scope`'s question and `link:<id>` is its scope, not this skill's.
- **Compliance question, in one plain sentence:** "Is something being said right now, in a source we are watching, that this story needs to see, and can we point at exactly who said it and where?"

---

## 4. Config surface (layer 2)

Every editorial choice sits here, set by the publishing house. Nothing here is in the skill.
Scope runs broadcaster, then site, then brand, the more specific overriding the default.

| Field | Required | Notes |
|---|---|---|
| `monitored_source` | yes | The identifier of the one source this configured instance watches, for example `live-transcript:govops-presser-0914`. This is the value substituted into the advert's first condition path, and it is the cheap identity check at the gate |
| `match_field` | yes | The dot path of the item field read, for example `transcript.segment.text`. Substituted into the advert's second condition path |
| `match_against` | yes | The story-side field the item is read against. `tags[].value` in every configured instance drawn so far, which is the field that says what a story is about and the reason the skill is named for a context match rather than a keyword match. `story.context` is a message family and was never a field; that error is corrected in 0.2.0 |
| `match_threshold` | yes | The confidence at or above which an item is surfaced, for example `0.82`. This is the selectivity dial and the single most consequential value on this surface. Set low, the producer drowns; set high, the confirmation arrives after the bulletin. What scale the confidence is on is not governed anywhere: see section 10 |
| `surface_as` | yes | What the match is displayed as. `enrichment` in every configured instance drawn so far. A surfaced item is never displayed as a confirmation, a clearance or a verified fact |
| `carry` | yes | The provenance fields required on a surfaced item, for example `[provenance, speaker, timecode, confidence]`. An item that cannot carry one of these is still surfaced, with the missing field named as missing. Never surfaced silently without it |
| `max_surfaced_per_window` | no | A cap on how many items one configured instance surfaces in a configured window, defaulting to unset. A blunt second selectivity control for houses that would rather lose a late match than flood a producer |
| `clearing_authority` | yes | The authority at or above which an assertion on the same scope closes a surfaced item, by acknowledging or dismissing it, for example `output-editor`. This closes the surfacing only. It is not the authority that clears a held fact, which is a different assertion against a different object. How two authorities are compared is not defined anywhere: see section 10 |
| `instance_label` | yes | the editorial label for this configured instance |

The house also declares this skill in its own active set. Those values are copied straight from
the frontmatter above and are not a second set of choices: `skill_id`, `skill_version`,
`skill_type`, `disclosure_level`, `migration_policy`.

---

## 5. Runtime loop, specialised

```
on source.item.observed, or on story.context when the context itself is revised:
  1. RESOLVE ANCHOR   the running story named on the message, at story scope, together with the
                      one item carried on it. No asset and no evidential position is required
  2. GATE             three cheap checks, in this order, before anything expensive is read.
                      (a) Is the source on this message the one config.monitored_source names?
                      Almost every message in a running newsroom fails here, and that is the
                      ordinary case. (b) Does the anchored story still hold an open context, or
                      has it closed? A closed story is exited quietly. (c) Does the item share
                      any content-bearing term with the story's context at all? A cheap term
                      overlap, not a scored read. An item with no overlap is exited quietly and
                      is never scored. This step is where selectivity is won: the scored read in
                      step 4 is the expensive one, and on a live transcript it would otherwise
                      run on every segment of every stream for every configured story
  3. READ CONFIG      load the configured instance for this broadcaster / site / brand
  4. EVALUATE         read the item against the value at config.match_against, score the match,
                      and compare the score against config.match_threshold. Below the bar is a
                      non-match and is exited quietly: nothing is surfaced and nothing is logged
                      to the room. At or above the bar, assemble the provenance named in
                      config.carry and record which of those fields could not be obtained
  5. DECIDE SEVERITY  inform, always, in every configuration. Nothing here can reach flag or
                      hold, because nothing is withheld and no posture is being declared about
                      the story itself. A surfaced item is shown to a human, who decides
  6. EMIT             the twelve-field warning, causation_id on the triggering snapshot
  7. AWAIT CLEARANCE  an assertion on story scope from an authority at or above
                      config.clearing_authority, acknowledging or dismissing the surfaced item,
                      carrying its own provenance of who, when, scope and status. This closes
                      the surfacing. It is not, and must not be read as, the clearance of any
                      held fact the surfaced item happens to bear on
  8. RESOLVE / CLOSE  closed when a valid assertion acknowledges or dismisses it, or when the
                      story's context is closed. Closure is recorded, not erased, and the
                      provenance stays readable after closure so the pointer back into the
                      source survives the shift that found it
```

**Skill specific traps:**

- The mistake a well-meaning vendor will make is treating a surfaced item as a clearance. It is not one. A surfaced enrichment says an authority said this, here, at this moment, with this confidence. It does not say the story's held figure is now good. An executor that releases a hold, unflags a figure or starts serving a withheld fact because a high-confidence item was surfaced has performed a clearance nobody asserted and has collapsed two skills into one.
- The second mistake is surfacing on every loose resemblance. A live transcript is a continuous stream, and a producer working a breaking story will read the first three surfaced items and then stop reading. One relevant confirmation is the product; fifty near-misses is not a lesser version of it, it is a worse outcome than surfacing nothing at all, because it trains the room to ignore the panel that the one real confirmation will arrive in. `config.match_threshold` and the term-overlap check at the gate exist for exactly this, and a vendor who softens either to improve recall has made the skill useless without making it fail.
- The third mistake is putting this on the critical path. None of the tools that carry this skill sits between the story and air. An executor that holds a story, delays a rundown item or waits on a monitor before letting anything move has made an additive discovery mechanism into a blocking one. Nothing waits for this skill, and nothing is missing if it never surfaces anything.
- `detail` must carry the item as spoken or written, the speaker or issuer, the timecode or equivalent pointer, the source identifier, the confidence score, the threshold it cleared, and what closes the surfacing. A message saying only that a relevant item was found is annoying rather than actionable, because the producer cannot go and hear it. Where a field named in `config.carry` could not be obtained, `detail` names it as missing rather than omitting it, so a producer can tell the difference between a clip with no timecode and a clip whose timecode nobody bothered to carry.
- Fail closed: config that is missing or unreadable, or a story context that cannot be read, is treated as the condition holding, and the candidate is surfaced at the loudest severity this skill actually has, which is `inform`, with `detail` saying which value could not be read rather than asserting a match score that was never computed. Nothing is silently dropped. The tension is real and is not glossed here: for a deliberately selective skill, failing open in this direction costs noise, and noise is the exact failure mode section 5 is otherwise built to avoid. The trade is accepted because the alternative is a monitor that goes quiet when its config breaks and looks identical to a monitor that is working and finding nothing. Recorded again in section 10.
- What does NOT clear it: a later, better-scoring item from the same source; the end of the press conference or the closing of the feed; the passage of time; the held fact clearing by some other route; and an assertion from below `config.clearing_authority` or on a different scope. None of those close a surfacing.

---

## 6. Output contract

`skill.warning.raised`, severity `inform`. This skill has no other severity in any configuration.

```json
{
  "topic": "som.skill.warning.raised",
  "originating_system": { "system_id": "<tool>", "system_type": "<the tool's own type, one this skill's advert targets>" },
  "correlation_id": "<the story's chain>",
  "causation_id": "<the snapshot that triggered evaluation>",
  "timestamp": "<on the envelope, never in the payload>",
  "payload": {
    "warning_id": "wrn-....",
    "skill_id": "smart-stories/surface-on-context-match",
    "skill_version": "0.2.2",
    "story_id": "story-...",
    "scope": "story:...",
    "severity": "inform",
    "rule_id": "<the configured instance label>",
    "non_overridable": false,
    "affected_fields": ["<config.match_against>"],
    "detail": "Surfaced from <monitored_source>: \"<item as spoken or written>\", <speaker or issuer>, <timecode or equivalent pointer>, confidence <score> against a bar of <match_threshold>. Missing provenance: <fields from carry that could not be obtained, or none>. Closed by an assertion at or above <clearing_authority> on this story. This is a candidate for a human to read. It clears nothing.",
    "blocks": [],
    "skill_warning_ref": null
  }
}
```

`blocks` is empty and `non_overridable` is false, permanently, in every configuration. Nothing is withheld here, so there is nothing to list and nothing to override. `affected_fields` names the story-side field the item was read against, not the story field a human might later change on the strength of it.

---

## 7. Priority position

| Axis | This skill |
|---|---|
| X category | `workflow` (3) |
| Y type / source | `reference` (5) |
| Z precedence | unset. The publishing house sets it, nobody else |

Because nothing is withheld, this skill contributes no gate, contests no action and outranks nothing: several configured instances of it can watch several sources against the same story and co-bind without conflict, which is what happens when a transcription tool, a claim-analysis tool and an archive discovery tool all carry it during one press conference. Where a hold skill separately binds the fact a surfaced item bears on, holds combine by conjunction: every binding gate must permit, so any one hold means held, and no volume of surfaced candidates changes that.

---

## 8. Worked configurations

### 8a. A live official press conference monitored against a running breaking-weather story

```yaml
instance_label: house-live-conference-confirmation
monitored_source: live-transcript:govops-presser-0914
match_field: transcript.segment.text
match_against: tags[].value
match_threshold: 0.82
surface_as: enrichment
carry: [provenance, speaker, timecode, confidence]
max_surfaced_per_window: 3
clearing_authority: output-editor
```

What this produces: while the conference runs, the transcript is read segment by segment against the story's context, and almost every segment is discarded at the gate without being scored. When the authority speaks the confirming wind-speed figure, that one segment is surfaced as a source enrichment on the story, naming the speaker, the timecode in the conference feed and the confidence, so a producer can hear the exact moment rather than take someone's word for the number. The held figure on the story is untouched by this. An output editor then asserts the clearance against the held fact, and the hold releases because its own gate cleared.

### 8b. A markets story, the primary regulatory filings feed monitored against a running issuer story

```yaml
instance_label: house-markets-filing-watch
monitored_source: filings-feed:rns-primary
match_field: filing.body_text
match_against: tags[].value
match_threshold: 0.90
surface_as: enrichment
carry: [provenance, speaker, timecode, confidence]
max_surfaced_per_window: 2
clearing_authority: business-editor
```

What this produces: a running story about a mid-cap issuer's profit warning has the primary regulatory news service feed watched against it. Most filings that day concern other issuers and are discarded at the gate. When a filing whose body reads against the story's context is published, it is surfaced as a source enrichment naming the issuing entity in place of a speaker, the publication timestamp and paragraph reference in place of a conference timecode, and the match confidence. The business producer opens the filing at the exact paragraph. Nothing about the story's disclosure posture is changed by the surfacing.

What changed between 8a and 8b: the monitored corpus (a live spoken transcript against a published document feed), the item field read, the match bar (raised to 0.90 because a filings feed is structured and a false positive on a markets story is expensive), the clearing authority, and what a timecode means, which generalises to the finest pointer back into the source that the source itself supports. The tool carrying the skill changed too, from a `transcription` executor to a `markets_data` one, both of which the advert targets, so both configured instances can be recalled as described. What stayed the same: the mechanism, the advert shape, the story scope, the severity ceiling of `inform`, the four provenance fields carried on every surfaced item, and the refusal to clear anything. Both configured instances surface a candidate and hand the decision to a human, and neither knows about the other.

---

## 9. Eval set

Minimum five. The skill is not finished until a declaring case and a clearance case both pass.

| Case | Input | Expected |
|---|---|---|
| declaring | Live transcript segment carrying the confirming wind-speed figure arrives on `source.item.observed` from `live-transcript:govops-presser-0914`, scores 0.91 against the story's context, configured instance 8a loaded | One `skill.warning.raised`, severity `inform`, `rule_id` is `house-live-conference-confirmation`, `detail` carries both values in the comparison (the score of 0.91 and the bar of 0.82) alongside the segment text, speaker, timecode and source. `blocks` empty, `non_overridable` false |
| non declaring | A segment from a different feed the configured instance does not name, or a segment on the named feed sharing no content-bearing term with the story's context | Nothing at all on the bus, quietly. Exited at the gate, no scored read performed, no heartbeat, no "evaluated and found nothing" message |
| clearance | An `output-editor` assertion on `story:<id>` acknowledges the surfaced item, with provenance of who, when, scope and status | The surfacing closes and the closure is recorded. Assert separately that nothing else moved: no held fact cleared, no hold released, no gate changed state as a result of this closure. Closing the surfacing is the whole of what this clearance does |
| wrong clearance | A `journalist` assertion, below `output-editor`, attempts the same closure | Not applied, the surfaced item stands, and the attempt remains visible with its provenance. Nothing is silently discarded |
| idempotency | The same transcript segment snapshot delivered twice, in both clouds, both executors recalling configured instance 8a | Exactly one warning. The second is recognised by `causation_id` on the same snapshot. A near-identical later segment from the same speaker is a different snapshot and is a different item, not a duplicate |
| fail closed | `match_threshold` missing from the configured instance, or the story context present but unreadable, with a candidate item in hand | `inform`, the loudest severity this skill has, and `detail` says which value could not be read rather than asserting a score that was never computed. The candidate is surfaced for a human rather than dropped. Assert the noise cost is visible: the message must be distinguishable from a genuine above-bar match |
| never auto-change | Any declaring case, 8a and 8b together on one story | Assert that no message produced on this skill's behalf altered any field of the story, any asset, any hold, any gate or any config. Only warnings were added |
| near miss below the match bar | A segment on the named feed sharing terms with the story's context and scoring 0.74 against a bar of 0.82 | Nothing on the bus, quietly. The scored read ran and found a non-match, and a non-match is silence, not a low-confidence surfacing. Assert that fifty such segments across one conference produce fifty silences and not one message |
| provenance completeness | An above-bar segment whose timecode cannot be obtained because the feed dropped its timing, with `carry` naming timecode | One `skill.warning.raised`, severity `inform`, carrying the segment, speaker and confidence, and naming timecode explicitly as missing in `detail`. Assert it is not surfaced silently without the timecode and is not suppressed for lacking one |

---

## 10. Open items

- **This skill surfaces and does not clear, and the editorial moment it is drawn from contains both.** That moment reads as one: the confirming figure arrives and the held figure clears. Those are two mechanisms and are authored as two skills. This one declares that an item from a monitored source matches the story's context and displays it with its provenance. The clearance of the held fact is a separate assertion by an authority against that fact, and the hold releases because its own gate cleared, which is `hold-while-flagged` and `gate-by-scope` territory reached over the bus. A producer sees the same sequence; the library keeps three resolvable skills instead of one unresolvable one. Nobody should have to discover this by reading the runtime loop.
- **Clearance semantics are thinner here than the template assumes, and the authority comparison rule is undefined.** The template's clearance and wrong-clearance cases are written for a skill that withholds something, where clearing means the withheld thing is served. This skill withholds nothing, so "clearance" means only that a surfaced candidate has been acknowledged or dismissed by a human and stops asking for attention. That is a weaker act than the word suggests, and the eval rows in section 9 are written honestly against it rather than borrowed from a holding skill. Underneath that sits the unresolved question shared by the whole library. The rule adopted throughout this library is that a declaration closes only on an assertion of equal or higher authority on the same scope, and that is the library's working position, not a rule any group has ratified: no vendor should read it as settled or hard-code enforcement against it. How two authorities are compared is not defined anywhere either, so `config.clearing_authority` is a house-local string and the executor's comparison of it is house-local too. The wrong-clearance eval row is written against the adopted rule, and if the group settles the comparison differently that row is rewritten and `clearing_authority` is read against the group's rule rather than the house's.
- **`fail_closed: true` reads oddly for a skill with no hold authority, and here it works against the skill's own design.** The reconciliation applied here holds the two layers apart: fail-closed at the skill layer means treating an unreadable value as the condition holding, which produces the loudest severity this skill actually has, which is `inform`, while safe state at the executor layer handles the case where nothing resolves at all. In practice that means an unreadable story context or unreadable match config causes the candidate to be surfaced for a human rather than silently dropped. The honest cost: this is a skill whose whole value is selectivity, and failing in the loud direction spends exactly the resource the skill is built to conserve. A broken configured instance can therefore surface a great deal for a producer to wade through. The trade is accepted because a monitor that goes quiet when its config breaks is indistinguishable from a monitor that is working and finding nothing, and the second is a normal state. A reviewer may reasonably prefer the quiet failure, and if the group settles it that way, section 5's fail-closed line and the fail-closed eval row both change.
- **The advert's condition paths are config-templated.** `{{ config.monitored_source }}` and `{{ config.match_field }}` are a good design with an unresolved consequence: the global lookup table cannot index them as one row the way it indexes a literal dot path. Proposal A settled that on 18 August: the house registers one advert row per configured instance, in its own registration layer, which during a press conference watched by three tools means three rows. Assumed here: one row per configured instance. If the table resolves configuration instead, nothing in this file changes, but the house's registration step does.
- **Continuous monitoring is a genuine edge for the recall model.** Recall is triggered by messages on the bus and the advert is field-driven, but a live transcript is a stream, not a sequence of field changes on a story. The working position taken here is that the monitoring tool publishes each usable segment as its own `source.item.observed` message, which makes the stream bus-observable and lets the ordinary field-driven advert apply unchanged. This is not stated anywhere and is not assumed away: it is an assumption, and it has a visible failure mode. A house that runs the monitor privately inside an executor and never puts segments on the bus will never have this skill recalled, and the tool will have built a private mechanism that no audit can see. This sits close to the unresolved question of request-triggered skills, which has the same shape: something that happens outside the bus needs to appear on the bus before any advert can match it, and nobody has said how.
- **`disclosure_level: L2` is ambiguous.** Two definitions circulate: the deepest level the skill declares content for, and how much of its reasoning the skill reveals. `L2` here means **the deepest level the skill declares content for**, which is the config surface in section 4. That reading is worth flagging twice for this skill, because the second definition is unusually live here: a match score and a threshold are reasoning, and they are published in `detail` on purpose so a producer can judge a candidate. If the group settles on the reasoning definition, this value is re-examined and probably rises.
- **Whether `target_system_type` binds or merely advises is not resolved.** It is written here as though it binds, per the working position, and that reading has a consequence this file now honours: a tool type absent from the advert cannot recall the skill, so a build note in section 11 addressed to a type the advert does not target would describe something that cannot happen. The two lists are therefore held identical at six types, and the `verification` and `markets_data` entries are there because the claim-analysis executor and the markets-feed executor of section 11 genuinely carry this skill, not to pad the list. Under an advisory reading this skill would behave differently in one respect worth naming: a compliance hub, an audit desk or a standards tool is not in the target list, yet a record of which candidates were surfaced against a story, at what confidence, and which a human then ignored is exactly the material an oversight function wants after the fact. If targeting becomes advisory, expect the target list to stay as it is and the off-target readers to pick it up anyway.
- **The op set has no similarity operator, so the advert under-specifies the match.** The originating configuration writes `item ~ story.context`, and `~` is not among `exists`, `absent`, `equals`, `contains` and `active`. `contains` is used as a coarse recall-time filter and the real scored read happens at EVALUATE. This is a deliberate split and it works, but it means the advert alone does not tell a reader what the match actually is, and two executors could implement the scored read differently and disagree about the same segment. Same skill plus same values is supposed to give the same decision in any vendor's tool, and for the scored read it does not, quite. A similarity or threshold condition kind has been suggested as a fourth condition type and has no shape yet.
- **The confidence scale is not governed.** `match_threshold` is written here as a value between 0 and 1 because that is the range the tooling in circulation expresses a match score in, and nothing in the schema says what scale a confidence is on, whether two vendors' scores are comparable, or whether a translation confidence and a match confidence may be combined into one number as scenario 3 in section 1 would require. Two houses can configure the same-looking bar and get very different selectivity. Scenario 3 is written in section 1 as a stretch case blocked on this question rather than as a case the skill serves today, because until the combination is either allowed or forbidden there is no honest value to put in `config.match_threshold` for a cross-lingual monitor. If the group allows the combination, scenario 3 becomes configuration like the other two and nothing in this file changes; if it forbids it, a cross-lingual monitor needs two thresholds and this config surface grows a field.
- **`depends` is empty, deliberately, and this is where a reader expects a dependency.** What the surfaced enrichment chains into (a clearance asserted against a held fact, a gate that then releases) arrives by other skills advertising against the state this one wrote, over the bus. Declaring those as dependencies would invert the relationship: a surfacing skill would depend on the skills that consume its output, which creates a cycle at the first change. This skill depends on what it reads, which is the story's context, the monitored source and its own config, none of which is a skill.
- **`state_path` is `null` because this skill never holds.** That is legal by reading rather than by statement: the field is described as the state a message must carry to decide an open hold, and there is no open hold here. Nobody has written down that `null` is correct for a non-holding skill.
- **`recall_on` is a proposal, not an agreed name, and neither topic is ratified.** The field replaced an older active-voice name under the passive convention agreed on 29 July 2026, and no replacement name has been agreed anywhere. `story.context` is the topic used across this library for a mint or context revision. `source.item.observed` is a plausible name for an item arriving from a monitored source rather than one anyone has ratified, and it is the topic the continuous-monitoring assumption above rests on. If the item topic is named differently, this advert's first entry changes and nothing else does.
- **None of the six targeted `system_type` values can be verified against the v0.3 enum from here, and this file does not claim any of them can.** `discovery`, `mam`, `planning`, `verification` and `transcription` are at least in circulation, because sibling skills in this library use those spellings: `transcription` is the spelling `enrich-on-condition` targets, and it is named here because this skill needs a tool type that holds a live transcript and can point at a moment in it. `markets_data` is not in circulation: it is named because the skill needs a tool type that watches a filings or regulatory news feed against a running issuer story, and worked configuration 8b is carried by exactly that tool. Being in circulation is not the same as being in the v0.3 enum, so all six spellings should be checked against the deployment's enum before registration. If the enum has no member for `markets_data`, the honest fix is to fall back to `discovery`, which is the broad type nearest it, and let `target_capability` distinguish, which reopens the unresolved question of whether `target_capability` is a governed field with an agreed vocabulary at all, and `target_capability` is left `null` here precisely because its status is unclear. If the enum has no member for `transcription`, that is a question for the library rather than for this file alone, because a sibling advert is registered against the same spelling and would have to move with it. Broad types are used throughout and no named product appears in the advert, because naming a product is a bet on the second open targeting question.
- **The emitting executor carries the tool's own `system_type`, and the `skill_worker` type this library used to emit here is withdrawn.** An executor is a sidecar to a vendor tool, and no newsroom-scoped class of skill-running system exists, so the thing that puts this skill's surfaced candidate on the bus is a transcription tool, a verification tool, a discovery tool, a markets-data tool, a MAM or a planning tool, and section 6 now says so, carrying the spelling of the tool that runs worked configuration 8a. The sample payload previously carried a `skill_worker` system type, a value whose only reason to exist was that the emitter ran skills, which is the newsroom-executor shape under another name. It is withdrawn rather than reinterpreted, and no file in this library should be read as having such a class. What is not settled is whether the emitting tool's own type is the right value for `originating_system.system_type` on a skill warning at all, as against some value naming the executor role that raised it. The schema does not answer that, and the position taken here follows from a decision taken outside the schema, that tools have executors and newsrooms do not, which the schema has not caught up with. Worth naming plainly: this is one of the two places this file names a system type, the other being the advert's `target_system_type`, and after this change the two agree by construction, because the tool that may recall this skill is the tool that emits on its behalf. The enum caveat in the item above therefore now reaches section 6 as well as the advert, since a spelling that is not a member of the v0.3 enum is not a member in either place.
- **`skill_uri`, `skill_content_sha`, `owner_editorial`, `owner_engineering` and `licence` are left as markers.** They are not this author's to supply, and whether `skill_uri` is required at all is unsettled: one position treats it as mandatory on every registered skill, the other as optional. A plausible invented URI would read to the next person as a real one.
- **The `skill_id` format is one of four in circulation.** `<publisher>/<name>` is used because it is the form with a worked precedent in the warning schema's own example, not because the question is settled.

---

## 11. Vendor build notes (layer 3)

You are building what your executor does with this skill, not what the skill does. The skill declares that one item from a source you are monitoring reads as a match against a running story, and displays it with the pointer back to where it came from. Your executor holds the corpus, does the reading, and hands a candidate to a human. None of you is on the critical path, and nothing in the story waits for any of you.

| Platform | What your executor does |
|---|---|
| Transcription | Monitors the live transcript against the running story, discards almost every segment at the gate without scoring it, and publishes the one segment that reads against the story's context as a source enrichment with the speaker as diarised, the timecode in the feed and the match confidence. Your timecode is the product, not the text: a segment surfaced without a pointer back into the audio is a paraphrase |
| Verification, the claim analysis case | Reads the transcript against the story and surfaces the one relevant claim rather than every claim in the conference. Your job is the discrimination, not the extraction. Publish the claim with its speaker, timecode, confidence and status, and publish it as a claim that was made, never as a claim that is true. You are not clearing the story's held figure and nothing you publish should read as though you were |
| MAM | Lands the conference media against the story and registers it under a name, so that when a cleared record later points at the source it points at an asset that exists and can be pulled, rather than at a moment in a stream nobody kept. You are also the reason a timecode surfaced during the conference still resolves to something the next day |
| Discovery, the archive case | Runs the same mechanism against a corpus instead of a stream: a running story's context read against the archive, surfacing the one relevant match with its accession reference, its date and its confidence in place of speaker and timecode. Expect to be configured with a higher bar than a live monitor, because an archive will always return something |
| Markets data | Watches a filings or regulatory news feed against a running issuer story and surfaces the one filing that reads against it, carrying the issuing entity, the publication timestamp and the paragraph reference. A false positive on a markets story is expensive, so your bar is set high and your silence is a correct answer. This is the executor of worked configuration 8b |
| Planning | You are where the producer works the running story, so you are where the candidate is read. It reaches you as an enrichment on the story carrying the source, the speaker or issuer, the pointer back into the source and the confidence against the bar it cleared, and you display it as a candidate, never as a confirmation. You score nothing and you clear nothing: the acknowledgement or dismissal a producer records with you closes the surfacing, and it closes the surfacing only, never the held fact the item happens to bear on |

The six rows above are exactly the six types the advert targets, which is deliberate: under the
binding reading of targeting, a build note addressed to a type the advert does not name would
describe work no executor could be recalled to do.

The instinct this skill exists to interrupt is the instinct to relay the fact without the pointer back to where it was said: today someone watches the press conference, hears the figure and passes it on by phone call and memory, and the link to the exact moment in the source is lost at the first retelling.
