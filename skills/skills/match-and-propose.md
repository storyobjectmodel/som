---
# --- identity ----------------------------------------------------------------
skill_id: smart-stories/match-and-propose
name: match-and-propose
skill_version: 0.2.2
som_schema_version: ["1.0.0"]

# --- what it is --------------------------------------------------------------
description: >-
  Declares that a loose media asset, one carrying no story association, has a
  candidate association to a story at or above a broadcaster-set confidence
  threshold, and displays that candidate with the score it was given. It
  proposes only. It does not write the association, does not confirm it, does
  not rank the clip, does not enrich it and does not withhold anything pending
  a producer's decision. The confirmation is a human act recorded elsewhere.
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
  target_system_type: [multimodal, mam, discovery, ingest, rundown]
  target_capability: null
  conditions:
    - kind: field
      path: assertions[].assertion_type
      op: equals
      value: MATCH
    - kind: field
      path: "{{ config.confidence_path }}"
      op: exists
    - kind: field
      path: assertions[].review.state
      op: equals
      value: PENDING
    - kind: field
      path: lifecycle.phase
      op: equals
      value: "{{ config.active_story_state }}"
  recall_on: ["media.asset.ingested", "media.match.candidate", "story.context"]
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

# match-and-propose

**When loose media scores at or above the house threshold against a story, put the candidate
in front of a producer with its score. Do not attach it.**

Read `../docs/CONVENTIONS.md` first.
**Changed in 0.2.1, 20 August 2026.** Two things. media_asset.story_association and media_asset.match_candidate.confidence appeared in section 4, section 10, the never-auto-change eval row and the mistake-a-vendor-will-make trap. There is no `media_asset` object in `story.context`; what this skill leaves alone is `assertions[].review.state` at `PENDING`, and the house's own association field sits outside the story object entirely. And section 10 now records proposal A as decided rather than printing both readings.

**Changed in 0.2.0, 18 August 2026.** Three things. `som_schema_version` moves to `["0.3.2"]`, which as of 18 August is the version. Field paths in the advert and in the worked configurations were checked against the v0.3.2 story-context JSON Schema for the first time and the ones that did not resolve are corrected; see the pack's `SCHEMA-CONFORMANCE.md` for the full list and the evidence. And the shared policy index was chosen over executor-local policy, which settles the registration question this file used to leave open: the house registers one advert row per configured instance.

---

## 1. What it is (layer 1)

| | |
|---|---|
| Operation | proposes a candidate story association for a loose media asset |
| Condition type | a match score at or above a configured confidence threshold |
| One mechanism? | Yes, and the reason is not that the name is already in use. Matching a loose asset to a story at or above the configured threshold, and proposing that association for confirmation, are one mechanism rather than two, because the proposal is the form the match is declared in rather than a second operation performed on the match once it exists. The skill never commits the association; a human does. Take either half away and the other half stops meaning anything: a match declared without a proposal leaves nothing for anyone to confirm, and a proposal made without a match leaves nothing to propose. There is therefore no split that would leave two skills each still doing something. Nothing is written, nothing is confirmed, nothing is withheld, and the producer's confirmation is a human act recorded outside the skill entirely, which the skill neither waits on nor acts on |

**The "and" test, answered before the objection.** The name contains "and", and the rule is that a
sentence needing "and" describes two skills. Applied here the rule reports a false positive, because
the sentence that describes this skill does not need the word: "it displays a candidate association
scored at or above a threshold". Matching is the condition; proposing is the display of that
condition holding. There is no second operation to split off, because the only other verb in the
neighbourhood, confirming, belongs to a person and is recorded by whatever tool holds their
decision. The single X category the skill occupies (`workflow`, 3) is therefore unambiguous, which
is the thing the "and" test actually protects. The conformant form of the name would be
`propose-on-match`. The library name is kept because it is the contract vendors are already
building to, which is a reason to keep the name and not an argument that the mechanism is single;
that argument is the row above, and it holds whichever name the file carries. Section 10 records
both, and records the validator's warning as answered rather than waved through.

**Reuse test, three scenarios from different stories:**

1. **A hurricane, breaking.** A citizen clip arrives loose, with no attribution and no
   story. A multimodal tool scores it against the stories currently on the bus and the candidate
   association to the active hurricane story is displayed at 0.91 for a producer to confirm.
2. **A trial verdict.** An agency stills package lands in the MAM overnight under a generic feed
   title, with no story on it. It is scored against the running trial story ahead of the evening
   programme, and the candidate is displayed to the intake producer with the score and the story
   it matched.
3. **Not drawn yet: a regional station's tape re-digitisation.** Thousands of untitled reels come
   off a multi-year archive programme with nothing but a shelf code. Each is scored against the
   standing anniversary and retrospective stories the house keeps open, and every candidate above the
   archive threshold is displayed for an archivist to confirm, one reel at a time, over months.
   This library has not previously drawn this case, and the skill needs no change to serve it:
   a different corpus, a different threshold, a different confirming role, the same mechanism.

One skill, three subjects: breaking news, a court story and an archive backlog.

---

## 2. Recall advert

The advert is declared in the frontmatter above so a tool can read it without parsing prose.
This section explains it in words for a human.

- **Who it is for:** `multimodal`, `mam`, `discovery`, `ingest` and `rundown` executors.
  `mam`, `ingest` and `discovery` are the tools that hold loose media, see it arrive, or compare
  it against what is running. `multimodal` is the tool that computes the score in the first place
  and publishes it at the configured path, so it is recalled on the same message it produced.
  `rundown` is where the producer works and where the proposal is read, which matters because
  targeting is written here as though it binds: a tool type absent from this list cannot recall
  the skill at all, so the surface the proposal has to reach is named rather than assumed. A
  playout or graphics executor holds nothing loose, computes no score and is not where a candidate
  is read, so it would recall this skill to no purpose. These are broad system types, not named
  products, because whether the lookup table names a type or a precisely named product is not
  settled, and the list is held to exactly the five executors section 11 gives build notes to.
- **What wakes it:** `media.asset.ingested` (a clip lands), `media.match.candidate` (a matching
  tool publishes a score against a story) and `story.context` (the set of stories a clip could
  belong to has changed).
- **What it depends on:** three field conditions, composed with AND. The asset carries no story
  association (no `assertions[]` entry of type `MATCH` already confirmed against it, which is what
  "loose" means; the house's own association field sits outside the story object). A candidate
  score is present at the path the house's tools publish it to. The story the score was computed
  against is in the lifecycle state the house counts as matchable.

**The threshold is not in the advert, and cannot be.** Two reasons, and both matter. Editorially,
a number like 0.85 is a decision about how much doubt a newsroom will put in front of a producer,
which is the house's call and not the library's; a literal in the skill would mean the skill was
carrying an editorial choice it has no standing to make. Mechanically, the advert's operator set
is `exists`, `absent`, `equals`, `contains` and `active`, none of which expresses "at or above".
So the threshold sits in the config surface as `confidence_threshold` and the comparison happens
at the EVALUATE step of the runtime loop, against the configured instance. The advert's job is
only to notice that a scored candidate exists at all.

**The advert uses a config-templated path, and that has a consequence.** The second condition
advertises `{{ config.confidence_path }}` rather than a literal dot path, because where a
confidence score lives in the schema is not settled (see section 10) and each house's matching
tools publish it where their own integration put it. A single-purpose skill advertising a literal
path is indexed by the global lookup table as one row. This one cannot be indexed that way at all,
because there is no literal path to key the row on until configuration has been read. The house
therefore registers one advert row per configured instance, and that is how a reader should expect
this skill to appear in the table: the two configurations in section 8 read as two rows, one keyed
on one house's candidate-score path and one on another's, rather than
as one row standing for the whole advert. Proposal A settled that on 18 August, and nothing in
this file changes under it. It is recorded again in section 10, and it is a design
consequence rather than a mistake. The third condition also carries a config-templated value,
`{{ config.active_story_state }}`, which is the ordinary preferred form and raises no indexing
question.

**`state_path` is null and `depends` is empty, on purpose.** The skill never holds, so there is
no open hold for a later message to decide, which is what `state_path` exists for. And nothing is
declared as a dependency because chaining happens over the bus, not through declared dependencies.
What the proposal chains into arrives the same way everything else does: a MAM skill advertising
against a confirmed association writes it once a producer has confirmed, and a provenance skill
advertising against the clip's arrival records what the clip carries. Neither is named here.
Declaring them would invert the dependency and create a cycle at the first change.

---

## 3. Firing anchor

- **Evidential position:** any. A loose clip usually arrives with no evidential position at all,
  which is precisely the case this skill is for. Requiring one before a candidate could be
  displayed would suppress exactly the matches most worth showing a human.
- **Outlet / path:** the media asset's own path in the tool that holds it. The proposal is
  attached to the candidate story's chain, so it surfaces where the story is being worked on and
  not only where the clip is filed.
- **Scope:** `story:<id>` of the candidate story. The association is a system-level fact about
  which story a clip belongs to, not a per-destination one, so `link:<id>` is not used here.
- **Compliance question, in one plain sentence:** "There is media in the building that probably
  belongs to this story: has anybody been told, and does the record show how confident the machine
  was when it said so?"

---

## 4. Config surface (layer 2)

Every editorial choice sits here, set by the publishing house. Nothing here is in the skill.
Scope runs broadcaster, then site, then brand, the more specific overriding the default.

| Field | Required | Notes |
|---|---|---|
| `confidence_threshold` | yes | The score at or above which a candidate is displayed. Broadcaster-set, never a literal in the skill. A number between 0 and 1, for example `0.85` for a live breaking story where a producer is watching, higher where nobody is |
| `confidence_path` | yes | The dot path where this house's matching tools publish the score. Templated into the advert, because the schema home for a confidence score is not settled |
| `instance_label` | yes | The editorial label for this configured instance. Travels as `rule_id` on the warning |
| `active_story_state` | yes | Which story lifecycle state counts as matchable: the running set, the planned set, or a house-defined standing set |
| `candidate_corpus` | no | Which stories the loose media is compared against. Defaults to whatever the matching tool was pointed at; set it where a house wants archive and planned stories in or out |
| `max_candidates` | no | How many candidates are displayed per asset. Defaults to one. A house that wants a producer to choose between two near-equal scores raises it |
| `confirming_role` | no | The role whose recorded decision closes a proposal. Read alongside the authority comparison gap in section 10 |

The house also declares this skill in its own active set. Those values are copied straight from
the frontmatter above and are not a second set of choices: `skill_id`, `skill_version`,
`skill_type`, `disclosure_level`, `migration_policy`.

---

## 5. Runtime loop, specialised

```
on media.asset.ingested | media.match.candidate | story.context:
  1. RESOLVE ANCHOR   the loose media asset named on the message, and the candidate
                      story its score was computed against
  2. GATE             exit quietly unless the asset carries no story association AND a
                      candidate score is present at the configured path. Most media on a
                      newsroom bus is already associated, so this is the cheap check that
                      keeps the skill out of the way
  3. READ CONFIG      load the configured instance for this broadcaster / site / brand
  4. EVALUATE         compare the candidate score against confidence_threshold. At or
                      above: the condition holds. Below: nothing, quietly
  5. DECIDE SEVERITY  inform, always. This skill has no louder severity to reach for,
                      because it withholds nothing and asserts nothing about the clip
                      beyond how well it scored
  6. EMIT             the twelve-field warning, causation_id on the triggering snapshot
  7. AWAIT CLEARANCE  a producer's recorded confirmation or rejection of the candidate,
                      held by whichever tool the house records decisions in
  8. RESOLVE / CLOSE  the proposal closes when the asset carries a confirmed association,
                      or when the candidate is recorded as rejected. It does not close by
                      itself
```

**Skill specific traps:**

- **The mistake a well-meaning vendor will make: attaching the clip.** A score of 0.97 is not
  permission. This skill declares a candidate and displays it; `assertions[].review.state` is
  left at `PENDING` by it, at any score, in any configuration, and the house's own association
  field is not written by it either. A
  vendor whose integration sets the association when the threshold is met has built a different
  product and should not carry this skill.
- **The second version of the same mistake: holding the clip back until someone confirms.** The
  skill withholds nothing. A loose clip stays exactly as available, or as unavailable, as it was
  before the candidate was displayed. `severity_range` is `[inform]` for this reason and lists
  nothing else.
- **What must be in `detail`:** both numbers, the score as computed and the threshold it was
  compared against, plus which clip and which story, plus the sentence that closes it. A
  proposal that says "match found" and not "0.91 against a threshold of 0.85" gives a producer
  nothing to disagree with.
- **Fail closed:** if the threshold config or the score itself cannot be read, the condition is
  treated as holding and the candidate is displayed anyway, at `inform`, with `detail` saying the
  value could not be read and which one. Fail-closed here does not mean a hold, because this skill
  has no hold to produce: it means the loudest severity the skill actually has. Silence is the
  dangerous failure mode for a proposal, because a dropped match looks identical to no match, and
  nobody investigates a match that was never displayed.
- **What does NOT close it:** a second matching tool publishing a lower score; the story moving on;
  the clip being viewed, downloaded or filed; time passing. Only a recorded decision on the
  candidate, made by an authority the house counts as equal or higher on the same scope.

---

## 6. Output contract

`skill.warning.raised`, severity `inform`.

```json
{
  "topic": "som.skill.warning.raised",
  "originating_system": { "system_id": "<matching-tool>", "system_type": "<the tool's own type, one this skill's advert targets>" },
  "correlation_id": "<the candidate story's chain>",
  "causation_id": "<the snapshot that carried the candidate score>",
  "timestamp": "<on the envelope, never in the payload>",
  "payload": {
    "warning_id": "wrn-....",
    "skill_id": "smart-stories/match-and-propose",
    "skill_version": "0.2.2",
    "story_id": "story-...",
    "scope": "story:...",
    "severity": "inform",
    "rule_id": "<the configured instance label>",
    "non_overridable": false,
    "affected_fields": ["assertions[0].review.state"],
    "detail": "Loose asset ugc-4417 scored 0.91 against story-2291, at or above the configured threshold of 0.85. Proposed for producer confirmation. The association has not been written. Closes when a confirmation or a rejection is recorded on this candidate.",
    "blocks": [],
    "skill_warning_ref": null
  }
}
```

`blocks` is empty and `non_overridable` is false, both because this skill withholds nothing.
`affected_fields` names the association field as the field the proposal is about, which is not the
same as claiming the skill sets it.

---

## 7. Priority position

| Axis | This skill |
|---|---|
| X category | `workflow` (3) |
| Y type / source | `reference` (5) |
| Z precedence | unset. The publishing house sets it, nobody else |

The category is defensible rather than merely stated: the question the skill answers is where a
piece of media belongs in the ingest-to-output process and who gets told about it, which is
operational process. It is not compliance, because nothing here has legal consequence and nothing
is withheld; it is not editorial, because it makes no claim about whether the clip is any good or
whether it may be used. It contests nothing: it displays a candidate and asks for a person. A
compliance skill that withholds the same clip resolves ahead of it at X, and this proposal
remaining visible alongside that hold is not a contradiction, because a proposal is not a release.
This skill contributes no gate at all, so it takes no part in the conjunction at the enforcement
point where binding gates each have to permit.

---

## 8. Worked configurations

### 8a. Loose user-generated media matched to an active breaking story

```yaml
instance_label: hurricane-ugc-loose-match
confidence_threshold: 0.85
confidence_path: assertions[].confidence
active_story_state: running
candidate_corpus: active-stories
max_candidates: 1
confirming_role: producer
```

What this produces: two clips arrive, handled honestly. The agency clip comes attributed and
carries its own C2PA chain, so it is not loose and this skill stays quiet about it. The citizen
clip comes with nothing. A multimodal tool scores it against the stories on the bus, the hurricane
story comes back at 0.91, and the producer sees a line saying this clip probably belongs to the
hurricane story, scored 0.91 against a threshold of 0.85, awaiting their confirmation. The clip is
not attached to the story, and the producer can disagree without undoing anything.

### 8b. An overnight agency feed item against tomorrow morning's planned story

```yaml
instance_label: overnight-feed-planned-match
confidence_threshold: 0.93
confidence_path: assertions[].confidence
active_story_state: planned
candidate_corpus: planned-stories-next-24h
max_candidates: 3
confirming_role: intake-editor
```

What this produces: a feed item lands at 03:40 under a generic agency slug with no story on it.
The ingest-side matcher, working from feed metadata and transcript text rather than vision, scores
it against the stories planned for the morning programme. Three candidates above 0.93 are waiting
for the intake editor when they arrive, each with its score, none of them attached to anything.

**What changed and what did not.** Changed: the corpus (running stories to planned stories in a
24 hour window), the threshold (0.85 to 0.93, because nobody is watching at 03:40 and a wrong
candidate would sit unchallenged for hours), the tool and therefore the path the score is
published to, the number of candidates shown, and the role that closes them. Unchanged: the
mechanism, which is a candidate above a configured threshold displayed with its score, and the
refusal, which is that in both configurations the association is never written by this skill and
nothing is withheld while the candidate waits.

---

## 9. Eval set

Minimum five. The skill is not finished until a declaring case and a clearance case both pass.

| Case | Input | Expected |
|---|---|---|
| declaring | loose asset, no story association, candidate score 0.91 against a running story, threshold 0.85 | one `inform` warning, `detail` carrying both values (0.91 and 0.85), the clip id and the story id, and `assertions[].review.state` still `PENDING` afterwards |
| non declaring | the asset already carries a story association, so the gate does not match | nothing at all on the bus, quietly |
| below threshold | same asset, candidate score 0.62 against a threshold of 0.85 | nothing on the bus, quietly. A near miss is not a quieter proposal, it is no proposal |
| clearance | the configured confirming role records a decision on the candidate, on the same scope | the warning closes, the decision and its provenance recorded alongside it. Nothing is released by the closure, because nothing was withheld: the closure retires a display |
| wrong clearance | an automated matching tool, or a role the house has not given the confirming authority, records a confirmation on the candidate | not applied, the proposal stays open, and the attempt stays visible with its own provenance in the compliance array. The cost of a wrongly applied clearance here is a candidate that stops being displayed, not a release, which is why this row is thinner than it is for a holding skill |
| confirmation | a producer confirms the candidate and a MAM skill, advertising against that confirmed state, writes the association | the proposal closes; assert that the association was written by the other skill's executor and that no message carrying this `skill_id` wrote it |
| idempotency | the same snapshot twice, in both clouds | one warning. The candidate is identified by asset, story and score snapshot, so a redelivery does not produce a second proposal |
| fail closed | `confidence_threshold` missing, or the score unreadable at the configured path | `inform`, the loudest severity this skill has, with `detail` naming which value could not be read. Never silence: a dropped match is indistinguishable from no match |
| never auto-change | any declaring case | assert no message produced by this skill changed any content, and specifically that `assertions[].review.state` is still `PENDING` and the house's association field is unchanged from before evaluation to after |

---

## 10. Open items

- **The name contains "and", and the validator warns on it.** The warning is a fair reading of the
  name and a wrong reading of the skill, and both halves are recorded here rather than left for a
  reviewer to reconstruct. The mechanism is genuinely single: the proposal is the form the match
  is declared in, not a second operation performed on the match, and neither half stands alone,
  since a match declared without a proposal leaves nothing to confirm and a proposal made without
  a match leaves nothing to propose. Section 1 makes that argument in full. What is at fault is
  the name, not the skill: the conformant form under `<operation>-<condition-type>` is
  `propose-on-match`, which would carry the same mechanism without inviting the question. The
  library keeps `match-and-propose` because it is what vendors have already built against and
  renaming it silently would break that contract. So the honest position is that the name is a
  compromise between the one-mechanism rule and an existing vendor contract, not a name the rule
  endorses: the rule is satisfied in substance, and the name is retained for a reason that has
  nothing to do with the rule. If the contract can be renegotiated at a version boundary,
  `propose-on-match` is the name to move to, and nothing in this file would change but the name.
- **The advert carries a config-templated path.** `{{ config.confidence_path }}` cannot be indexed
  by the global lookup table as a single row, because there is no literal path to key that row on
  until configuration has been read. Proposal A settled that on 18 August: the house registers one
  advert row per configured instance, in its own registration layer. Stated in section 2 in the
  same terms, which for the two configurations in section 8 means two rows and not one. Nothing in
  this file changes under A, because this is what it always assumed. What A obliges is the
  registration layer itself, and it is stated in both places so nobody discovers the dependency by
  watching a table fail to build.
- **Where a confidence score lives in the schema is not settled**, as far as this file knows. The
  path is config-templated for that reason. `assertions[].confidence` is what the demo houses
  configure it to and it resolves, but nothing says it is the canonical home. Do not read any
  configured value in section 8a as a claim about the schema: it is one house's value in a worked
  example. If the group settles a canonical home, the path becomes a literal and the indexing
  question above disappears with it.
- **Fail-closed for a skill that cannot hold.** `fail_closed: true` is required, but this skill has
  no hold to produce. The reading applied throughout holds the two layers apart:
  fail-closed at the skill layer means treating an unreadable value as the condition holding,
  which produces the loudest severity the skill actually has, here `inform`. Safe state at the
  executor layer is a different mechanism at a different layer and is not what this row is doing.
  Assumed, not settled.
- **Clearance semantics are thinner here than the template assumes.** The template's clearance and
  wrong-clearance cases are written for a skill that withholds something, where clearance releases
  it. Nothing is withheld here, so clearance means a display is retired and wrong clearance means
  a display is wrongly retired. Both eval rows are written honestly on that reading. Alongside it,
  the rule adopted throughout this library is that a declaration closes only on an assertion of
  equal or higher authority on the same scope. That is this library's working position, not a rule
  any group has ratified, and no vendor should read it as settled or hard-code enforcement against
  it. **How two authorities are compared is not defined** anywhere either, so the authority value
  is a house-local string and the executor's comparison of it is house-local too: here the house
  names the confirming role in `confirming_role` and its own configuration decides what counts as
  equal or higher. If the group defines a comparison rule, that config field is where it lands,
  and the wrong-clearance eval row in section 9 is rewritten against whatever comparison lands.
- **`disclosure_level: L2`** is used in the sense of "the deepest level the skill declares content
  for", which is layer 2, the config surface. It is not the other definition in circulation, which
  is about how much of its reasoning a skill reveals. Two definitions are live and neither 29 July
  document mentions the field.
- **Targeting is written as though it binds.** `target_system_type` names `multimodal`, `mam`,
  `discovery`, `ingest` and `rundown` on the working position that an executor recalls a skill only
  when the target matches its own type. That reading is what fixes the list: a type absent from the
  advert cannot recall the skill, so a build note in section 11 addressed to a type the advert does
  not name would describe work no executor could be recalled to do. The two lists are therefore held
  identical at five types, and `multimodal` and `rundown` are named because the tool that computes
  the score and the tool the producer reads the proposal in genuinely carry this skill, not to pad
  the list. Broad types are used rather than named products, because naming a product is a bet on the
  second open targeting question. Sibling skills in this library use all five spellings, but being in
  circulation is not the same as being in the v0.3 `system_type` enum, and this file does not assert
  that any of the five is a member: the spellings should be checked against the deployment's enum
  before registration. Under an advisory reading of the target list the behaviour would change in one
  specific way worth naming: an off-target audit or oversight hub, one that section 2 says would
  recall this skill to no purpose under the binding reading, would pick it up and display the same
  candidates, which is harmless for an inform-only proposal and would arguably be an improvement. So this skill is a poor
  test case for the binding question rather than evidence either way.
- **The emitting executor carries the tool's own `system_type`, and the `skill_worker` type this
  library used to emit here is withdrawn.** An executor is a sidecar to a vendor tool, and no
  newsroom-scoped class of skill-running system exists, so the thing that puts this skill's proposal
  on the bus is a multimodal analysis tool, a MAM, a discovery tool, an ingest tool or a rundown, and
  section 6 now says so: `<matching-tool>` is a `multimodal` system, and the placeholder is kept
  because the identifier is the house's to supply. The sample previously carried a `skill_worker`
  system type, a value whose only reason to exist was that the emitter ran skills, which is the
  newsroom-executor shape under another name. It is withdrawn rather than reinterpreted, and no file
  in this library should be read as having such a class. What is not settled is whether the emitting
  tool's own type is the right value for `originating_system.system_type` on a skill warning at all,
  as against some value naming the executor role that raised it. The schema does not answer that, and
  the position taken here follows from a decision taken outside the schema, that tools have executors
  and newsrooms do not, which the schema has not caught up with. Worth naming plainly: this is one of
  the two places this file names a system type, the other being the advert's `target_system_type`,
  and after this change the two agree by construction, because the tool that may recall this skill is
  the tool that emits on its behalf.
- **`depends` is empty and stays empty.** The reader expects a dependency here, because a proposal
  obviously chains into something: a MAM committing the association after confirmation, or a
  provenance skill recording what the clip carries. Neither is declared. Both arrive by advertising
  against the state this skill's output leaves on the bus. Declaring them would invert the
  dependency and create a cycle at the first change.
- **`recall_on` is a proposal name**, not an agreed one. The field replaced an older active-voice
  name under the passive convention agreed on 29 July 2026, and no replacement name has been agreed
  anywhere. It is not this skill's to settle.
- **`skill_uri`, `skill_content_sha`, `owner_editorial`, `owner_engineering` and `licence` are left
  as markers.** They are not this author's to supply, and whether `skill_uri` is required at all is
  unsettled: one position treats it as mandatory on every registered skill, the other as optional. A
  plausible invented URI would read to the next person as a real one.
- **The `skill_id` format is one of four in circulation.** `<publisher>/<name>` is used because it is
  the form with a worked precedent in the warning schema's own example, not because the question is
  settled.

---

## 11. Vendor build notes (layer 3)

| Platform | What your executor does |
|---|---|
| Multimodal analysis | You already score media against candidate stories. Publish the score to the path the house configured and stop there. Recall of this skill tells you the candidate is displayable at that score, not that you may attach it. Your confidence value is the whole payload of the proposal, so publish the number, not a verdict word |
| MAM | You hold the loose asset. Your executor recalls this skill when an asset in your store has no story association and a score exists, and it displays the candidate against the story. Your integration writes `story_association` only when a separate skill, advertising against a recorded confirmation, tells it to. The write is a different recall from this one |
| Discovery | You watch what is running and what is arriving. Your executor recalls this skill to put a scored candidate in a producer's surface next to the story it matched. The score is displayed with the threshold it was compared against; a candidate shown without its numbers is not this skill's output |
| Ingest | You see the clip first, and often before anyone has decided what it is. Your executor recalls this skill on `media.asset.ingested`, gates on the association being absent, and displays the candidate. You do not delay the ingest, quarantine the asset or wait for a confirmation, because nothing here withholds anything |
| Rundown | You are where the producer is. The proposal reaches you attached to the candidate story's chain, and the confirmation the producer gives you is recorded by you and put back on the bus. That recorded decision is what closes the proposal; this skill has no other way to be closed |

The five rows above are exactly the five types the advert targets, which is deliberate: under the
binding reading of targeting, a build note addressed to a type the advert does not name would
describe work no executor could be recalled to do.

The instinct this skill exists to interrupt: a tool that is confident enough attaches the clip
itself and tells nobody, so that the newsroom's record shows an association nobody chose and a
score nobody saw.
