# Contributing

Thank you for wanting to improve the Story Object Model. This page is the practical
guide; [`GOVERNANCE.md`](GOVERNANCE.md) says who decides and how.

## The short version

Open a pull request. Anyone may. Say what you are changing and why, run the checks,
and expect a review. There is no contributor licence agreement to sign: by
contributing you agree that your contribution is offered under the licences this
repository already uses, Apache 2.0 for schemas, examples, tools and skills, and
CC BY 4.0 for specification prose.

## Before you propose a change to the schema

`schema/` is the standard, and it carries a stability promise. Before opening a pull
request that touches it, read [`spec/compatibility-policy.md`](spec/compatibility-policy.md)
and say in your pull request which kind of change yours is: additive (permitted in a
1.x release) or breaking (requires the working group's agreement and a major
version). If you are not sure, open an issue first and ask; that is quicker for
everyone than a pull request that has to be reworked.

Check [`spec/open-register.md`](spec/open-register.md) too. If the question you are
answering is already there, say so; the register is where the working group keeps
the questions it has deliberately left open, and a proposal that resolves one is
welcome.

## Running the checks

Every pull request should pass these before review:

```
pip install jsonschema
python3 tools/validate.py                                  # every example against the schemas
python3 tools/som_lint.py schema                           # the schemas against their own claims
python3 tools/validate_sequence.py examples/hurricane-run  # a story over time
```

If your change touches `skills/`, run its own checks as well; they are listed in
[`skills/README.md`](skills/README.md).

A change to a schema that makes an existing example fail is a breaking change by
definition. A change that adds a field should add or extend an example that uses it.

## What a good pull request looks like

- One change per pull request. A schema change and a prose change are two pull
  requests.
- A title that says what changed, and a description that says why. If the change
  came out of a working group discussion, say which one.
- For prose: where prose and schema disagree, the schema is right. Fix the prose to
  match the schema, not the other way round.
- No version bumps. Release numbering is done by the maintainers at release time.

## Worked examples and negative cases

Examples are as valuable as schema changes. A new worked message that shows a family
being used well, or a negative case that shows a message the schema must reject, is
a contribution that needs no working group decision and helps every implementer.

## Credit

If your change is accepted into the schema, add yourself to
[`CONTRIBUTORS.md`](CONTRIBUTORS.md) in the same pull request or a following one.
That is the intended route and it does not need anyone's permission.

## Questions

Open an issue, or write to admin@storyobjectmodel.com.
