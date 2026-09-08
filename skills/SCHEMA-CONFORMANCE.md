# Schema conformance audit · som-skill-library 0.2.2

**Date:** 18 August 2026, extended 20 August 2026
**Checked against:** `schema/story-context.schema.json` (SOM 1.0). The audits below were run against the v0.3.2 files dated 18 August; 1.0 withdrew `voice_count`, the `FINALIZING` status and `transforms[].transform_id`, none of which this library referenced.
**Method:** every literal path in every advert, every path in every worked configuration, and from 0.2.1 every backticked path-shaped token in the prose, resolved against the schema by walking `$defs`. Templated paths are unknowable and were skipped. The script is `scripts/check_paths.py`.

---

## Result

**Before: 6 advert paths and 19 worked-configuration paths did not resolve. After: 0.**

Nobody had run this check before. That is the finding, not the individual corrections, and it is the reason to keep the script in the pack rather than the answers.

---

## First, a correction I owe

A note went out from us on 18 August saying that a review claim, *"every library skill locks `som_schema_version 0.3.1`"*, was **stale**.

**It was accurate.** All ten library files locked `0.3.1`. What misled me was `CONVENTIONS.md`, which states the move to `0.3.2` happened on 12 August and is enforced by the validator. The conventions document said it; the files did not do it. I checked the document and reported it as though I had checked the files.

Anyone holding the earlier note should disregard that line. All eleven files now lock `["0.3.2"]`.

---

## Advert corrections

These are the machine-read rows. A skill advertising a path that does not resolve cannot be recalled correctly by anything.

| Skill | Was | Now | Why |
|---|---|---|---|
| `apply-clearance` | `compliance[].clearance_type` | `compliance[].type` | `compliance_flag` has no `clearance_type`. Its typed field is `type` |
| `apply-clearance` | `status` `active` `cleared` | `status` `equals` `RESOLVED` | The status enum is `ACTIVE`, `RESOLVED`, `WAIVED`. `cleared` is not a member |
| `gate-by-scope` | `telling.destination_id` | `assets[].usage[].destination_id` | The telling event carries `telling_id` and `link_id`, never `destination_id`. The destination a committed asset stands against is on the story |
| `gate-by-scope` | `editorial_gates[].scope` | `editorial_gates[].gate_type` | `editorial_gate` has no `scope`. The typed field is `gate_type` |
| `hold-while-flagged` | `editorial_gates[].flag_type` | `editorial_gates[].gate_type` | Same. There is no `flag_type` anywhere in the schema |
| `hold-while-flagged` | `status` `active` `standing` | `status` `equals` `PENDING` | The gate status enum is `PENDING`, `APPROVED`, `REJECTED` |
| `match-and-propose` | `media_asset.story_association` absent | `assertions[].assertion_type` equals `MATCH` | There is no `media_asset` object and no `story_association` field. A proposed association is an assertion of type MATCH, which is how v0.3.2 carries a proposal |
| `match-and-propose` | `story.lifecycle_state` | `lifecycle.phase` | Plus a new condition on `assertions[].review.state` equals `PENDING`, which is what makes a MATCH a proposal rather than a commitment |

---

## Worked-configuration corrections

These do not stop a skill being recalled. They stop a house configuring it, which is worse in a different way: the file reads as usable and is not.

| Skill | Was | Now |
|---|---|---|
| `apply-clearance` | `assets[].version_group` | `extensions.com.example.version_group` |
| `enrich-on-condition` | `asset.provenance.state` = `unverified` | `assets[].authenticity_credential.present` = `false` |
| `enrich-on-condition` | `asset.enrichments.*` | `assertions[]` |
| `enrich-on-condition` | `asset.audio.source_language_state` | `assets[].acquisition_state` |
| `flag-on-mismatch` | `premise.category` | `premise.actual_outcome` |
| `match-and-propose` | `media_asset.match_candidate.confidence`, `ingest.feed_item.match_score` | `assertions[].confidence` |
| `raise-flag-on-match` | `story_context.category_basis` = `indicative` | `lifecycle.phase` = `BREAKING` |
| `raise-flag-on-match` | `story_context.figures_status` = `unconfirmed` | `editorial_source[].credibility` = `UNVERIFIED` |
| `raise-flag-on-match` | `issuer_disclosure.state` = `pre-release` | `compliance[].type` = `PRICE_SENSITIVE` |
| `record-provenance-on-ingest` | `editorial_source[].provenance` | `editorial_source[].credibility` |
| `record-provenance-on-ingest` | `media_asset[]`, `media_asset[].provenance` | `assets[]`, `assets[].authenticity_credential` |
| `select-provider-by-context` | `assets[].transcript_recorded` = `false` | `assets[].acquisition_state` = `CAPTURED` |
| `select-provider-by-context` | `story.category`, values `politics` / `sport` | `tags[].value`, values `11000000` / `15000000` |
| `surface-on-context-match` | `match_against: story.context` | `match_against: tags[].value` |

---

## One correction that is a schema gap rather than a typo

**`apply-clearance` has no field to put a version set in.** The v0.3.2 asset object carries no version-status and no version-group property, and it is `additionalProperties: false`, so nothing can be added at the edge.

The file has been repointed at `extensions.com.{vendor}.version_group`, which works and is honest, and section 10 now says plainly that this skill cannot be configured against v0.3.2 without a vendor extension. The file previously said the author did not know whether version fields were settled. They are settled: they are absent.

That is a decision for the group, not a fix for a skill file. Either the schema gains a version-set shape, or version resolution stays vendor-namespaced and the conformance claim in proposal A does not reach it.

---

## Three corrections that changed the demo

The walkthrough describes what each skill reads. Three of its descriptions named paths that had just been corrected, so the walkthrough moved with the library.

- `gate-by-scope` now reads `assets[].usage[].destination_id` in the skills column, on demo 1 beat 10 and on the closing Cuez beat before it was cut.
- The `enrich-on-condition` run on demo 1 beat 5 now names `editorial_source[].credibility` explicitly rather than the bare word `credibility`.
- Build stamp `2026-08-18k` at the time of the 0.2.0 audit. The current build is `2026-08-20a`, and `demo-skills/` and `configured-instances/` are generated from it.

Every skill condition displayed in the walkthrough now resolves in the schema, and all 36 beat states still validate with zero errors.

---

## The 0.2.1 pass, 20 August 2026

A review of the shipped 0.2.0 pack found nine things. Four of them were one failure repeated: **a summary layer moved and the files underneath it did not, and every validator passed throughout.** The walkthrough page said the library was v0.1.0 in four places while a paragraph on the same page said 0.2.0. `SCHEMA-CONFORMANCE.md` said build 18k, `demo-skills/` said 18l and the walkthrough said 18m. Eight of the ten skill files still printed proposal A as an open question ten days after it was decided. `SYSTEM-TYPE-VOCABULARIES.md` had its opening patched and its closing section left describing a contradiction that had been removed.

**The path finding.** Six field paths did not resolve, and the 0.2.0 audit could not have seen any of them, because all six lived in prose, worked examples and eval-set rows rather than in adverts or config blocks, which is all `check_paths.py` read:

| File | Path that did not resolve | Corrected to |
|---|---|---|
| `flag-on-mismatch` | `premise.category`, in the section 8 reusability claim and the section 9 declaring row | `premise.actual_outcome` |
| `flag-on-mismatch` | `asset.status` | `assets[].status` |
| `match-and-propose` | `media_asset.story_association` and `media_asset.match_candidate.confidence` | `assertions[].review.state`, `assertions[].confidence`; the house's association field is outside the story object |
| `select-provider-by-context` | `story.category`, values `politics` and `sport` | `tags[].value`, IPTC media topics `11000000` and `15000000` |
| `enrich-on-condition` | `asset.provenance.state`, `asset.enrichments.synthetic_media_analysis`, in three eval rows | `assets[].authenticity_credential.present`, `assertions[]` |
| `declare-context-on-commit` | `usage[].state` | `assets[].usage[].state` |
| `apply-clearance` | `compliance[].clearance_type`, in the section 7 config row | `compliance[].type` |
| `hold-while-flagged` | `editorial_gates[].flag_type`, in six places of prose | `editorial_gates[].gate_type` |

**So the script changed rather than only the files.** `check_paths.py` now runs a second sweep over every backticked token in the body that has the shape of a field path. A token whose first segment is a `story.context` property must resolve. A token whose first segment is not is also an error, unless that root is named in the script's `OUT_OF_SCHEMA` table with a reason, so adding an exemption is a decision somebody makes rather than a silence. Bus topics are exempt, and the topic set is read from the skills' own `recall_on` lists rather than hard-coded. It also fails when it finds no files to check: `check_paths.py .` used to pass on an empty glob.

**`gate-by-scope` was describing a vocabulary that does not exist.** Sections 4 to 9 had gates moving between `clear` and `held`. `editorial_gates[].status` is a closed enum of `PENDING`, `APPROVED`, `REJECTED`. The file also wrote `editorial_gates[web]`, indexing an array by a path name, and specified `publish_when` as `gate(path) == clear`, an expression in a syntax this library never defined. The permit test is now fixed and described rather than configured: resolve the destination through `path_map` to a `gate_id`, read that one entry, permit on `APPROVED` and nothing else. `publish_when` is removed for the same reason `clear_state` was removed in 0.2.0.

**Two new scripts.** `build_lookup_table.py` generates `lookup-table.json` and `docs/LOOKUP-TABLE.md` from the skill frontmatter, so the table that is step 1 of both on-ramps is in the pack and cannot drift from the adverts it projects. `check_cover_claims.py` reads the claims a cover makes about the files underneath it, the declared library version, the declared build stamp and the declared decision, and fails when they disagree. That is the class of error that produced four of the nine findings and that every existing validator passed straight through.

---

## What the check does not cover

**Item-side paths.** `surface-on-context-match` reads `transcript.segment.text` and `filing.body_text` off the observed item, not off the story. Those are not story-context paths and were not checked. There is no schema for a monitored-source item, which is worth someone's attention.

**Templated paths.** Every `{{ config.* }}` is unknowable until a house configures it. That is the point of a generic skill and also the hole in this audit: **a house can still configure a path that does not resolve, and nothing will tell it.** The paths are corrected in the worked configurations; nothing enforces them at registration.

That is the next thing to build, and under proposal A it is part of the house registration layer that A now owes. A registration step that resolves each configured path against the schema before writing the row would close it, and it is roughly the same forty lines as the script in this pack.

---

## Reproducing

```
python3 scripts/check_paths.py skills/
```

Exits non-zero on any non-resolving literal path. Worth putting in CI on the same commit as the skills, which is exactly the argument proposal A rests on.
