#!/usr/bin/env python3
"""
Prove that every message in examples/negative/ is REJECTED by the SOM 1.0 schemas.

Run:  python3 tools/validate_negative.py      (exits non-zero if any case is accepted)
Requires: pip install jsonschema rfc3339-validator

tools/validate.py proves the schemas accept good input. A schema that accepted
everything would pass all of that. This is the other half: a corpus of messages that
MUST fail, each with the reason stated, so that independent implementations can be
checked for rejecting the same things. See spec/open-register.md, item 4.

Each file in examples/negative/ has the shape:

  {
    "family": "story-context" | "telling" | "delivery" | "link" | "system-audit" | "skill-warning",
    "must_fail_because": "one sentence",
    "message": { ... }          a full envelope (has "payload") or a bare payload
  }

A case passes this check when validation produces at least one error. The first
error is printed so a reader can see the schema saying no for the stated reason.

One case is not a schema rejection. The envelope schema deliberately leaves som_version
as a pattern so that a 1.1 producer can emit "1.1.0" without a schema change; the rule
that "1.0.0" is the only SOM 1.0 wire version lives in spec/conformance.md section 3.
That case is labelled "conformance:" rather than "schema:" below, and an implementation
validating with the schemas alone is expected to accept it.
"""
import json, sys, glob, os

try:
    from jsonschema.validators import validator_for
except ImportError:
    sys.exit("pip install jsonschema first")

try:
    import rfc3339_validator  # noqa: F401 - jsonschema uses it for format: date-time
    HAS_RFC3339 = True
except ImportError:
    HAS_RFC3339 = False

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SCH  = os.path.join(ROOT, "schema")
WIRE_VERSIONS = {"1.0.0"}


def schema(name):
    return json.load(open(os.path.join(SCH, f"{name}.schema.json"), encoding="utf-8"))


ENV = schema("envelope")
FAMILY = {
    "story-context": schema("story-context"),
    "telling":       schema("telling-event"),
    "delivery":      schema("delivery-media-available"),
    "link":          schema("link-event"),
    "system-audit":  schema("system-audit"),
    "skill-warning": schema("skill-warning"),
}


def errs(sch, inst):
    V = validator_for(sch)
    return sorted(V(sch, format_checker=V.FORMAT_CHECKER).iter_errors(inst), key=str)


def failures(case):
    """Return (label, message) pairs; label is "schema" or "conformance"."""
    fam = FAMILY[case["family"]]
    msg = case["message"]
    out = []
    if isinstance(msg, dict) and "payload" in msg:
        out += [("schema", e.message) for e in errs(ENV, msg)]
        out += [("schema", e.message) for e in errs(fam, msg["payload"])]
        if msg.get("som_version") not in WIRE_VERSIONS:
            out.append(("conformance",
                        f"som_version {msg.get('som_version')!r} is not one of {sorted(WIRE_VERSIONS)} (spec/conformance.md section 3)"))
    else:
        out += [("schema", e.message) for e in errs(fam, msg)]
    return out


def main():
    files = sorted(glob.glob(os.path.join(ROOT, "examples", "negative", "*.json")))
    if len(files) < 10:
        print(f"FATAL: examples/negative/ holds {len(files)} file(s), expected at least 10")
        sys.exit(1)
    accepted = 0
    for path in files:
        rel = os.path.relpath(path, ROOT)
        case = json.load(open(path, encoding="utf-8"))
        fs = failures(case)
        if fs:
            label, first = fs[0]
            print(f"REJECTED  {rel}")
            print(f"            because: {case['must_fail_because']}")
            print(f"            {label + ':':<13}{first[:110]}")
        else:
            accepted += 1
            print(f"ACCEPTED  {rel}   <-- must not happen: {case['must_fail_because']}")
    total = len(files)
    print(f"\n{f'OK - {total}/{total} rejected' if not accepted else f'{accepted} of {total} case(s) were ACCEPTED'}")
    if accepted and not HAS_RFC3339:
        print("hint: rfc3339-validator is not installed, so format: date-time is not asserted. pip install rfc3339-validator")
    sys.exit(1 if accepted else 0)


if __name__ == "__main__":
    main()
