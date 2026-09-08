# Two targeting vocabularies, and the gap this skill sits in

**Status:** finding, not a proposal. Raised while authoring `declare-context-on-commit` for the EVS stand demo.

**Updated 18 August 2026, pack 0.2.0.** The skill no longer names `automation` in its advert. That was removed because advertising for a value the lookup table cannot index reaches nothing and reads as coverage. **The finding below is unchanged by that removal.** The two vocabularies still disagree, they still share only `graphics` and `archive`, and a plant control system still has no home in either the targeting vocabulary or, therefore, the table. What changed is that the skill file no longer contains the contradiction; the gap is now stated rather than embodied.

**One thing the choice of proposal A does to this.** The shared policy index was chosen on 18 August over executor-local policy. Under the alternative, a vendor could have carried a policy locally whatever the table said, which would have made this gap survivable by working around it. Under A the row is the mechanism, so no row means no reach, and this finding gets more expensive rather than less.

---

## What was found

There are two lists of system types in circulation and they do not agree.

**The schema's enum.** `originating_system.system_type` in the SOM v0.3.2 envelope, which every publisher on the bus sets on every message:

```
ncs · mos_device · graphics · automation · wire_service · ai_agent ·
compliance_engine · editorial_dashboard · archive · prompter · camera ·
audio · skill_worker · custom
```

**The library's vocabulary.** `recall.target_system_type` across the ten skills, which is what `lookup-table.json` and `docs/LOOKUP-TABLE.md` index and what an executor matches its own type against:

```
rundown · mam · cms · playout · graphics · planning · discovery ·
transcription · verification · multimodal · ingest · archive ·
distribution · markets_data · compliance_hub
```

Fourteen values and fifteen values. **Two of them appear in both:** `graphics` and `archive`.

The two lists are not describing the same thing badly. They are describing two different things: the schema's enum says what kind of system published a message, and the library's vocabulary says what kind of tool a skill is for. Those overlap heavily in practice and a reader will assume they are one list, which is the failure worth heading off.

## Why it matters here rather than in the abstract

`declare-context-on-commit` is written for a plant control system: a router, a multiviewer, a tally. That is `automation`, which exists in the schema's enum and does not exist in the library's vocabulary. So the skill advertises a target type the lookup table cannot index, and no row is returned for the tool the skill was written for.

The nine other skills never hit this, because all nine target editorial tooling, and the library's vocabulary covers editorial tooling well. The EVS stand demo is the first thing anyone has built that reaches past the newsroom into the plant, and it found the edge on the first try.

## The three ways out, and what each costs

**A. Add `automation` to the library's vocabulary.** One value, and this skill's row appears. Cheap, and it does not settle the larger question, which means the next plant-side skill asks for `router` and the one after that asks for `multiviewer` and nobody has decided whether those are one type or three.

**B. Index the schema's enum instead.** Coherent, since every message already carries a `system_type` from that list and an executor could match its own declared type without a second vocabulary to maintain. The cost is real: the ten existing adverts all rewrite, `rundown` has no home in the enum, and the enum's granularity is wrong for skills targeting in places, since `mam` and `cms` both land in `custom`.

**C. Maintain a declared mapping between them.** Honest about the fact that they are two different questions, and it puts the mapping somewhere a vendor can read. It is also a third artefact to keep current, and the library's own README is clear about what happens to a hand-maintained index: it drifts, silently, and a skill that is not recalled looks exactly like a story with nothing to declare.

## What was assumed here, so it can be undone

This skill targets the library's vocabulary, because that is what the table indexes today, and its advert now names only values from that vocabulary: `playout`, `mam`, `graphics`. Until 0.2.0 it also named `automation`, a value from the schema's enum, sitting in an advert indexed against the other list. That was a deliberate inconsistency at the time and it has been removed rather than defended, because an advert nothing can index states a wish. The cost of removing it is that the skill now advertises for three tool types that are adjacent to the one it was written for, and reaches the plant control system itself through none of them. That is the gap, stated plainly instead of embodied in a value.

The alternative considered and rejected was having the control system present its executor as `playout`, which would make the row resolve immediately. It was rejected because it is a misrepresentation that then gets indexed, and because it hides exactly the finding worth surfacing.

## The question for the group

> Does `target_system_type` draw from the schema's `system_type` enum, from a separate functional vocabulary the library maintains, or from a declared mapping between the two?

Any of the three is workable. What is not workable is the current position, where two lists exist, share two values, and nothing says which one an executor should match itself against.
