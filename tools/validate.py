#!/usr/bin/env python3
"""
Validate every example in this repository against the published SOM 1.0 schemas.

Run:  python3 tools/validate.py        (exits non-zero on any failure)
Requires: pip install jsonschema

Three things this does that the pre-1.0 validator did not:

  * It dispatches on DIRECTORY, not on a substring of the filename. The old
    validator picked the payload schema by testing `"v0.3.2" in name`; once the
    schemas went flat and the version left the filename, that test would have
    silently routed every story example to the wrong schema and still printed
    PASS. A directory cannot go quietly false the way a substring can.

  * It ASSERTS formats. jsonschema treats `format` as an annotation unless a
    format checker is supplied, so the old validator accepted a message_id of
    "NOT-A-UUID" with zero errors. Every field the envelope declares as a uuid
    or a date-time is now actually checked. Implementations in other languages
    must do the same - see spec/conformance.md.

  * It has no reference-implementation pin. The wire version used to be tied to
    a C# constant in the .NET starter; the standard now stands on its own.
"""
import json, sys, glob, os

try:
    from jsonschema.validators import validator_for
except ImportError:
    sys.exit("pip install jsonschema first")

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SCH  = os.path.join(ROOT, "schema")

# 1.0 is a clean break: it is the only value on the wire. v0.3.2 traffic is not
# 1.0 traffic - three fields were withdrawn before ratification, so a 0.3.2 payload
# may carry a field 1.0 disallows. See spec/migration-from-v0.3.2.md.
WIRE_VERSIONS = {"1.0.0"}


def load(p):
    try:
        return json.load(open(p, encoding="utf-8"))
    except FileNotFoundError:
        sys.exit(f"FATAL: missing {os.path.relpath(p, ROOT)}")


def schema(name):
    return load(os.path.join(SCH, f"{name}.schema.json"))


ENV = schema("envelope")

# examples/<dir>/ -> the payload family it holds. The whole dispatch table.
FAMILY = {
    "hurricane-run":  schema("story-context"),
    "story-context":  schema("story-context"),
    "telling":        schema("telling-event"),
    "delivery":       schema("delivery-media-available"),
    "link":           schema("link-event"),
    "system-audit":   schema("system-audit"),
    "skill-warning":  schema("skill-warning"),
}

# A family whose directory has emptied is a broken glob, not a clean run.
MINIMUM = {"hurricane-run": 7, "story-context": 3, "telling": 2, "delivery": 2,
           "link": 1, "system-audit": 1, "skill-warning": 1}


def errs(sch, inst):
    V = validator_for(sch)
    return sorted(V(sch, format_checker=V.FORMAT_CHECKER).iter_errors(inst), key=str)


def note(msg):
    return type("E", (), {"message": msg})()


def check(path, sch):
    """A fixture is either a full envelope (envelope + payload) or a bare payload."""
    d = load(path)
    if isinstance(d, dict) and "payload" in d:
        out = []
        if d.get("som_version") not in WIRE_VERSIONS:
            out.append(note(f"som_version {d.get('som_version')!r} is not one of "
                            f"{sorted(WIRE_VERSIONS)}"))
        return out + errs(ENV, d) + errs(sch, d["payload"])
    return errs(sch, d)


def main():
    targets = []
    for d, sch in sorted(FAMILY.items()):
        files = sorted(glob.glob(os.path.join(ROOT, "examples", d, "*.json")))
        if len(files) < MINIMUM[d]:
            print(f"FATAL: examples/{d}/ holds {len(files)} file(s), expected at least "
                  f"{MINIMUM[d]} - glob broken or files deleted")
            sys.exit(1)
        targets += [(f, sch) for f in files]

    bad = 0
    for path, sch in targets:
        es = check(path, sch)
        rel = os.path.relpath(path, ROOT)
        if es:
            bad += 1
            print(f"FAIL  {rel}")
            for x in es[:4]:
                print("        -", x.message[:120])
        else:
            print(f"PASS  {rel}")

    total = len(targets)
    print(f"\n{f'OK - {total}/{total} valid' if not bad else f'{bad} of {total} file(s) FAILED'}")
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
