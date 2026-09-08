# Migrating to SOM 1.0 from v0.3.2

**Short version: stop emitting three fields, emit `som_version: "1.0.0"`, and turn on
format assertion. Nothing else in your payloads changes.**

SOM 1.0 is a clean break rather than a rename. v0.3.2 was the contract proven at IBC,
and it was explicitly provisional: it carried fields that had been proposed but never
ratified. 1.0 withdraws those, and from that point the compatibility policy applies —
nothing else comes out until 2.0.

That is the whole difference. Every other field, type, enum and constraint is
unchanged.

## 1. Three fields were withdrawn

| Withdrawn | Was | Why |
|---|---|---|
| `assets[].voice_count` | optional integer | Proposed §4 (#22). It carried a classification rule — `voice_count: 1` MAY classify a package SECONDARY — that was never settled |
| `assets[].status` → `FINALIZING` | enum member | Proposed for v0.3.2; named but not ratified |
| `transforms[].transform_id` | optional string | Proposed 9 July; a stable audit handle, still under discussion |

**You are almost certainly not affected.** None of the three appeared in any published
example, and nothing in the skill library referenced them.

If you did emit one, remove it. A 1.0 consumer rejects `voice_count` and
`transform_id` outright, because the objects that held them do not permit unknown
properties. `FINALIZING` fails enum validation.

All three remain candidates. Under the compatibility policy, re-adding an optional
field or an enum member is permitted in any 1.x release, so any of them can return in
1.1 the moment it is settled — additively, breaking nothing. They are recorded in
[`open-register.md`](open-register.md).

`assets[].authenticity_credential` was also marked proposed in v0.3.2. **It ships in
1.0** and is now normative: three skills in the library are built on it.

## 2. `som_version` — emit `"1.0.0"`

`"0.3.2"` is not SOM 1.0. A v0.3.2 payload may carry a withdrawn field, so the two
values do not denote the same contract and a consumer must not treat them as
interchangeable.

- Emit `"1.0.0"`.
- Do not branch on the field. It is informative; `message_type` is the only parsing
  discriminator.

## 3. Identifiers moved

The `$id` of every schema is now a resolvable, permanent URL on the project's own
domain. The old identifiers were never resolvable and carried `-proposed` in the
identity of a shipping standard.

| Was | Is |
|---|---|
| `https://som.spec/schema/v0.3/envelope` | `https://storyobjectmodel.com/schema/1.0/envelope.schema.json` |
| `https://som.spec/schema/v0.3.2-proposed/story-context` | `https://storyobjectmodel.com/schema/1.0/story-context.schema.json` |
| `https://som.spec/schema/v0.3.1-proposed/link-event` | `https://storyobjectmodel.com/schema/1.0/link-event.schema.json` |

These are stable for the life of 1.x — see [`compatibility-policy.md`](compatibility-policy.md).

If you resolve schemas by `$id`, update the base. If you load them from disk or
embedded a copy, nothing changes.

## 4. Layout is flat

The three version directories are gone. All seven families sit in `schema/`, one file
per family, no version in the filename.

| Was | Is |
|---|---|
| `som-v0.3-envelope.schema.json` | `envelope.schema.json` |
| `som-v0.3-skill-warning.schema.json` | `skill-warning.schema.json` |
| `v0.3.2-proposed/som-v0.3.2-story-context.schema.json` | `story-context.schema.json` |
| `v0.3.2-proposed/som-v0.3.2-telling-event.schema.json` | `telling-event.schema.json` |
| `v0.3.2-proposed/som-v0.3.2-delivery-media-available.schema.json` | `delivery-media-available.schema.json` |
| `v0.3.1-proposed/som-v0.3.1-link-event.schema.json` | `link-event.schema.json` |
| `v0.3.1-proposed/som-v0.3.1-system-audit.schema.json` | `system-audit.schema.json` |

The v0.3.2 "effective pack" spanned three directories, because v0.3.2 revised only
three families and the rest stayed authoritative at an earlier version. That
inheritance is now invisible, which is the point. For the record: apart from `$id`,
`title` and the withdrawals above, `link-event` and `system-audit` are byte-identical
to their v0.3.1 files, and `envelope` and `skill-warning` to their v0.3 files.

**If you dispatch on filename, this will break you.** Anything testing for `"v0.3.2"`
in a schema path must select on the family instead.

## 5. Turn on format assertion

1.0 asserts `format`. Your validator probably does not.

Most JSON Schema validators treat `format` as an annotation and ignore it unless
configured, so traffic that has been passing validation may contain a `message_id`
that is not a UUID or a `timestamp` that is not a date-time. That traffic was never
conformant; it was never checked.

Turn format assertion on and re-run your fixtures before claiming 1.0 conformance.
[`conformance.md`](conformance.md) §4 lists what this means per language.

---

## Coming from v0.3.1 or earlier

Everything above applies, plus one earlier retirement:

**`ai_enrichments` was removed from `story.context` in v0.3.2.** It is explicitly
disallowed, so a v0.3.1 payload carrying it fails validation rather than being quietly
ignored. Its replacement is `assets[]` carrying authorship provenance.

Everything else survives. The published v0.3.1 `link`, `system-audit`, `telling` and
`delivery` payloads all validate unchanged against the 1.0 schemas, and ship in
`examples/` for exactly that reason.
