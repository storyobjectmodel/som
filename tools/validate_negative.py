#!/usr/bin/env python3
"""
Prove that every message in examples/negative/ is REJECTED by the SOM 1.0 schemas.

Run:  python3 tools/validate_negative.py      (exits non-zero if any case is accepted)
Requires: pip install jsonschema

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
"""
import json, sys, glob, os

try:
    from jsonschema.validators import validator_for
except ImportError:
    sys.exit("pip install jsonschema first")

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
    fam = FAMILY[case["family"]]
    msg = case["message"]
    out = []
    if isinstance(msg, dict) and "payload" in msg:
        if msg.get("som_version") not in WIRE_VERSIONS:
            out.append(f"som_version {msg.get('som_version')!r} is not one of {sorted(WIRE_VERSIONS)}")
        out += [e.message for e in errs(ENV, msg)]
        out += [e.message for e in errs(fam, msg["payload"])]
    else:
        out += [e.message for e in errs(fam, msg)]
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
            print(f"REJECTED  {rel}")
            print(f"            because: {case['must_fail_because']}")
            print(f"            schema:  {fs[0][:110]}")
        else:
            accepted += 1
            print(f"ACCEPTED  {rel}   <-- must not happen: {case['must_fail_because']}")
    total = len(files)
    print(f"\n{f'OK - {total}/{total} rejected' if not accepted else f'{accepted} of {total} case(s) were ACCEPTED'}")
    sys.exit(1 if accepted else 0)


if __name__ == "__main__":
    main()
