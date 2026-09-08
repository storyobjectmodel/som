# som-skill-library 0.2.2 · plus one proposed skill

**8 September 2026.** Ten library skills at 0.2.2, one proposed eleventh, the generated lookup table, and the audits that produced them. Published as part of SOM 1.0.

| | |
|---|---|
| Library | Ten skills, `lifecycle: draft`, `origin: reference`, `som_schema_version: ["1.0.0"]` |
| Proposed | `declare-context-on-commit`. Not in the library, ships in no package, candidate eleventh |
| Table | `docs/LOOKUP-TABLE.md` and `lookup-table.json`, generated from the skill frontmatter |
| Configured | Demo configuration (`demo-skills/`, `configured-instances/`) is **not published in SOM 1.0** — it names individual vendors and is held pending their sign-off |
| Validators | `validate_som_skill.py`, `check_library.py`, `check_paths.py`, `check_cover_claims.py`, `build_lookup_table.py --check` |
| Status | 11/11 pass, 0 errors. 0 non-resolving paths. 0 cover claims disagreeing with the files |

---

## What changed in 0.2.2

**8 September 2026. The library moved onto SOM 1.0.** Three things, none of which changes what any skill does.

**`som_schema_version` moves to `["1.0.0"]`.** SOM 1.0 withdrew three fields that v0.3.2 carried but never ratified — `assets[].voice_count`, the `FINALIZING` member of `assets[].status`, and `transforms[].transform_id`. No skill in this library referenced any of them, so every advert path and every worked configuration resolves against the 1.0 schema unchanged. `check_paths.py` passes 11/11 with zero non-resolving paths.

**The path check reads the published schema, not a copy.** `check_paths.py` previously resolved against a `som.v0.3.2.story-context.schema.json` vendored into this pack. It now reads `../schema/story-context.schema.json` — the schema this repository publishes. There is one copy and it is the source of truth, which removes the drift this library's own audits exist to catch.

**`gate-by-scope` worked configurations use neutral path identifiers.** The `path_map` examples named a specific vendor's destination paths. They now read `cms-web-path` and `cms-web-live`. Configuration shape and behaviour are unchanged.

## What changed in 0.2.1

A review of the shipped 0.2.0 pack found nine things. Read `SCHEMA-CONFORMANCE.md` for the evidence. The short version:

**Four of the nine were one failure, repeated.** A summary layer moved and the files underneath it did not, and every validator passed the whole time. The walkthrough claimed v0.1.0 in four places while a paragraph on the same page claimed 0.2.0. Three documents claimed three different build stamps. Eight of the ten skill files still printed proposal A as an open question ten days after it was decided. A document had its opening patched and its closing section left describing a contradiction that had already been removed.

**So `check_cover_claims.py` is new.** It reads the claims a document makes about other files, the declared library version, the declared build stamp, the withdrawn vocabulary, and fails when they disagree with what the files actually say. Every existing validator passed all four of those findings, because each file was individually legal.

**Six more field paths did not resolve, and the 0.2.0 audit could not have seen them.** They lived in prose, worked examples and eval-set rows, which `check_paths.py` never read. It now sweeps prose too, and it fails when a path-shaped token's root is neither a `story.context` property nor a declared out-of-schema namespace. It also fails when it finds no files, which it used to treat as a pass.

**`gate-by-scope` was describing a status vocabulary that does not exist.** Gates moved between `clear` and `held` through five sections; the schema's enum is `PENDING`, `APPROVED`, `REJECTED`. The file also indexed the gate array by path name. The permit test is now fixed and described rather than configured, and `publish_when` is removed.

**The lookup table is in the pack.** It was step 1 of both on-ramps and shipped in neither. `build_lookup_table.py` generates it from the skill frontmatter, so it is a projection of the adverts and cannot drift from them.

---

## What changed in 0.2.0

**Every path was checked against the real schema for the first time.** Six literal advert paths and nineteen worked-configuration paths did not resolve. All are corrected. A skill advertising a path that does not exist cannot be recalled correctly by anything, and a worked configuration pointing at a field that does not exist reads as usable and is not.

**All ten moved from `0.3.1` to `0.3.2`.** They had been on `0.3.1` the whole time, including after a conventions document said otherwise. That row is corrected too.

**Proposal A settles the registration question.** The house registers one advert row per configured instance.

**`tags[]` landed on `story.context`**, so a configured instance can be scoped by what a story is *about* for the first time. `select-provider-by-context` and `surface-on-context-match` both use it now.

---

## The check that is still missing

`check_paths.py` resolves **literal** paths. Every `{{ config.* }}` is unknowable until a house configures it, so a house can still configure a path that does not resolve and nothing will tell it.

Closing that is registration-time work, and under proposal A it belongs to the house registration layer A now owes. It is about forty lines.

---

## The demo configuration

`skills/` is generic and shared. A house loads its own values against it. The IBC walkthrough's configured runs — `demo-skills/` and `configured-instances/` in the working library — are **not part of SOM 1.0**: they name individual vendors and their products, and are held pending sign-off from each. They are expected in a later point release.

---

## Where to start

1. `docs/LOOKUP-TABLE.md`, and find the row for your tool type.
2. The skill files that row names, in `skills/`.
3. Configure an instance of that skill for your own house, following `docs/CONVENTIONS.md`.

---

## Running the checks

```
python3 scripts/validate_som_skill.py skills/<name>.md
python3 scripts/check_library.py skills/
python3 scripts/check_paths.py skills/
python3 scripts/build_lookup_table.py . --check
python3 scripts/check_cover_claims.py .
```

The path check reads `../schema/story-context.schema.json` — this repository's published schema, not a vendored copy. There is one copy of each schema in this repository and it is the source of truth.
