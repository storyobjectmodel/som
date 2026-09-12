# Governance

How the Story Object Model is looked after, who decides what changes, and how to
take part.

This document is deliberately short. SOM 1.0 was built by a small group working
quickly, and the governance it needs today is the governance of an open project in
its first weeks, not of a standards body. It will grow as the project does, and
changes to this file follow the same process as changes to anything else here.

## Open and unowned

The Story Object Model is an open standard. No organisation owns it, no membership is
required to implement it, and no permission or licence beyond the ones in this
repository is needed to use it, build on it, or ship products that speak it. The
licences are Apache 2.0 for the schemas, examples, tools and skills, and CC BY 4.0
for the specification prose, and they are the whole of the terms.

Where the model lives in the longer term, including whether it moves under the
umbrella of an existing standards organisation, is a decision for the working group,
taken openly and recorded here. Nothing in the current arrangement forecloses that.

## The working group

SOM was developed through the IBC 2026 Accelerator project SMART STORIES by a
consortium of broadcasters, news organisations and technology vendors. From 1.0 that
group continues as the SOM working group, and it is open. Anyone implementing the
model or affected by it is welcome to take part; the existing consortium is its
foundation, not its boundary.

The working group will meet online at a regular cadence. Its decisions are recorded in
this repository, in [`spec/version-history.md`](spec/version-history.md) for what
changed and in [`spec/open-register.md`](spec/open-register.md) for what is still
open, rather than in minutes kept elsewhere.

To take part, register interest at [storyobjectmodel.com](https://storyobjectmodel.com)
or write to admin@storyobjectmodel.com. Details of the meeting cadence and how
proposals are brought to the group will be published there and here as they are
settled.

## How changes are made

Every change to this repository, including to this file, arrives as a pull request.
Anyone may open one.

**Changes to `schema/`** are changes to the standard. They are reviewed by the
maintainer responsible for the schema and by at least one other maintainer, and they
are bound by [`spec/compatibility-policy.md`](spec/compatibility-policy.md): an
additive change may ship in a 1.x release, and a change that would break an existing
conformant implementation requires the working group's agreement and a major version.
A change is not merged while a working group member has an unanswered objection to
it.

**Changes to `examples/`, `tools/` and `skills/`** are reviewed by one maintainer.
The skill library versions separately from the standard and keeps its own changelog.

**Changes to `spec/`, `docs/` and the root documents** are reviewed by one
maintainer. Where prose and schema disagree, the schema is right and the prose is
fixed.

**`CONTRIBUTORS.md`** follows its own rule: anyone whose accepted change reached the
schema has earned a line, and the pull request is the way to claim it.

## Maintainers

Maintainers hold merge rights on the repository and are responsible for the review
process above. At 1.0 the maintainers are the four authors named in
[`CONTRIBUTORS.md`](CONTRIBUTORS.md); the maintainer responsible for the schema is
the specification author. Maintainers are added or replaced by agreement of the
working group and the change is recorded here.

The repository and the storyobjectmodel.com domain are held by the project on behalf
of the working group.

## Versioning, in one paragraph

Releases are numbered as set out in the compatibility policy. `1.0` is the first
stable release. Nothing is removed from `schema/` before `2.0`; withdrawn candidates
and open questions are carried in the open register so that nobody has to guess what
is settled. Each release is recorded in the version history with what changed and
why.

## Conduct

Participation is governed by [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md). Concerns go
to admin@storyobjectmodel.com.
