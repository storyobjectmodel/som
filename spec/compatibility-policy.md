# SOM 1.0 — Compatibility policy

What SOM may change without breaking you, and what would require a 2.0.

Calling a release 1.0 is a commitment to stability rather than a claim of completeness.
This document is that commitment, stated precisely enough to plan against.

## The promise

**An implementation built against SOM 1.0 will keep working against every 1.x release,
without code changes, for as long as 1.x is published.**

If you rely only on what §3 permits us to change, you never have to touch your
integration. If you rely on something in §2, you may.

## 1. Identifiers never move

Every schema is identified by a stable URL:

```
https://storyobjectmodel.com/schema/1.0/<family>.schema.json
```

The `$id` of a published schema MUST NOT change within 1.x. A new compatibility line
gets a new path segment (`/schema/2.0/`); it never re-points an existing one. Patch
releases republish in place at the same `$id`.

Note that the URL carries the compatibility line (`1.0`), while `som_version` on the
wire carries three components (`1.0.0`). This is deliberate and not a mismatch to be
tidied up.

## 2. Changes that are permitted in a 1.x release

All of these are additive. A consumer built for 1.0 continues to work unchanged.

- **Adding an optional field** to any payload family.
- **Adding a member to an open enum**, where the schema and this policy do not declare
  that enum closed.
- **Adding a new message family**, with its own `message_type`. Consumers ignore
  `message_type` values they do not handle (see conformance §5), so a new family cannot
  break an existing one.
- **Adding a new `$defs` definition.**
- **Relaxing a constraint** — widening a numeric range, removing a `maxLength`, making a
  required field optional.
- **Clarifying a description**, where the clarification does not change what validates.
- **Adding examples, tools or documentation.**

Producers MAY start emitting new optional fields at any 1.x release. Consumers MUST
tolerate them, which conformance §5 already requires.

## 3. Changes that require a 2.0

None of these will happen in a 1.x release.

- Removing or renaming a field.
- Adding a field to a `required` set.
- Narrowing a type, a pattern, or a numeric range.
- Removing a member from an enum, or closing an enum that was open.
- Changing the meaning of an existing field while keeping its name — the most damaging
  kind of break, because nothing fails validation.
- Changing an existing `$id`.
- Changing the envelope's parsing model, including making anything other than
  `message_type` a discriminator.

## 4. Deprecation

A field that is destined for removal is deprecated first, never removed inside 1.x.

1. The field is marked deprecated in its description, with the release that did so and
   what replaces it.
2. It keeps working, and stays valid, for the remainder of 1.x.
3. It may be removed only in 2.0.

Producers SHOULD stop emitting deprecated fields. Consumers MUST keep accepting them
until 2.0.

Two worked precedents, both from **before** 1.0, and both of which this policy would
now forbid inside 1.x:

- `ai_enrichments` was retired between v0.3.1 and v0.3.2; its replacement is `assets[]`
  with authorship provenance.
- `asset.voice_count`, the `FINALIZING` member of `asset.status`, and
  `transforms[].transform_id` were withdrawn at 1.0, having never been ratified.

Both removals happened while the specification was explicitly provisional. That window
is now closed. Anything in `schema/` as of 1.0 stays until 2.0.

## 5. What is not covered

This policy governs `schema/` — the normative directory — and the wire contract in
`spec/conformance.md`.

`skills/`, `examples/`, `tools/` and the rest of `spec/` are versioned independently
and may change more freely. In particular the skill library carries its own version
(`som-skill-library 0.2.2`) and its own compatibility story.

Items in the open register are unresolved questions, not commitments. Resolving one
produces a change governed by §2 or §3 like any other.
