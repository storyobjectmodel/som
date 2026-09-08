#!/usr/bin/env python3
"""Validate a SOM skill file against SOM_SKILL_TEMPLATE_v0.1, the X-Y-Z priority
model (Priority Specification v0.7) and the recall mechanism (Handover v4).

Usage:
    python3 validate_som_skill.py <skill.md> [more.md ...]
    python3 validate_som_skill.py --json <skill.md>

Exit code 0 when every file passes with no errors, 1 otherwise. Warnings never
fail the run: they are things a reviewer will ask about, and the right response
is usually a sentence in section 10 rather than a change.
"""

import argparse
import json
import re
import sys

try:
    import yaml
except ImportError:
    sys.exit("pyyaml is required: pip install pyyaml --break-system-packages")

CATEGORY_X = {
    "compliance": 1,
    "editorial": 2,
    "workflow": 3,
    "appearance": 4,
    "capability": 5,
}
ORIGIN_Y = {
    "government": 1,
    "legal": 2,
    "regulator": 3,
    "publishing-house": 4,
    "reference": 5,
    "vendor": 6,
}
SKILL_TYPES = {"NEWSROOM", "VENDOR", "REFERENCE"}
LIFECYCLES = {"draft", "candidate", "proposed", "ratified", "deprecated"}
SEVERITIES = ["hold", "flag", "inform"]
CONDITION_KINDS = {"field", "field_change"}
DEFERRED_KINDS = {"relationship"}
OPS = {"exists", "absent", "equals", "contains", "active"}
MIGRATION = {"HOT", "COLD", "GATED"}
DISCLOSURE = {"L1", "L2", "L3"}

REQUIRED = [
    "skill_id", "name", "skill_version", "som_schema_version",
    "description", "skill_type", "category", "lifecycle",
    "publisher", "origin", "owner_editorial", "owner_engineering", "licence",
    "skill_uri", "skill_content_sha", "level_2_uri", "level_3_uri",
    "recall", "output_messages", "severity_range",
    "auto_change_content", "fail_closed",
    "migration_policy", "disclosure_level", "depends", "supersedes", "superseded_by",
]

BANNED_FIELDS = {
    "skill_priority": "priority is a coordinate, not a word. Use category (X); Y follows from origin; Z is the publishing house's.",
    "tier": "not a SOM field. If this meant priority, see category and origin.",
    "z": "Z is set exclusively by the publishing house. A skill never carries it.",
    "priority": "priority is a coordinate. See category and origin.",
    "tools": "executor implementation belongs in section 11, vendor build notes, not in frontmatter.",
    "matches": "conditions belong under recall.conditions.",
    "reads": "the fields read belong in the advert and in section 2.",
    "produces": "output belongs in output_messages.",
}

SECTION_TITLES = [
    "What it is", "Recall advert", "Firing anchor", "Config surface",
    "Runtime loop", "Output contract", "Priority position", "Worked instances",
    "Eval set", "Open items", "Vendor build notes",
]

MANDATORY_EVAL_ROWS = [
    "declaring", "non declaring", "clearance", "wrong clearance",
    "idempotency", "fail closed", "never auto-change",
]

ACTIVE_VOICE = [
    (r"\bthe skill (fires|acts|subscribes|runs itself|executes)\b",
     "passive convention, 29 July 2026: skills declare and display, executors act."),
    (r"\bthis skill (fires|acts|subscribes|blocks the story)\b",
     "passive convention, 29 July 2026: skills declare and display, executors act."),
    (r"\bskill (subscribes to|listens on) the bus\b",
     "executors watch the bus. Skills are recalled from the lookup table."),
]

WITHDRAWN = [
    (r"most[- ]restrictive[- ]wins",
     "formally withdrawn 29 July 2026. Gates combine by conjunction: any one hold means held."),
    (r"story[- ]owner precedence",
     "formally withdrawn 29 July 2026 along with most-restrictive-wins."),
    (r"\bfires_on\b",
     "renamed to recall_on under the passive convention."),
    (r"\bskills_config\b.{0,60}\b(directive|instruct|tells? the executor to run)\b",
     "skills_config is the per-story record of what ran, not a directive of what to run."),
]


class Report:
    def __init__(self, path):
        self.path = path
        self.errors = []
        self.warnings = []
        self.notes = []

    def error(self, msg):
        self.errors.append(msg)

    def warn(self, msg):
        self.warnings.append(msg)

    def note(self, msg):
        self.notes.append(msg)

    @property
    def ok(self):
        return not self.errors


def split_frontmatter(text, rep):
    m = re.match(r"^---\n(.*?)\n---\n(.*)$", text, re.S)
    if not m:
        rep.error("no YAML frontmatter delimited by --- at the top of the file.")
        return None, text
    try:
        data = yaml.safe_load(m.group(1))
    except yaml.YAMLError as exc:
        rep.error("frontmatter does not parse as YAML: %s" % exc)
        return None, m.group(2)
    if not isinstance(data, dict):
        rep.error("frontmatter parsed but is not a mapping.")
        return None, m.group(2)
    return data, m.group(2)


def is_placeholder(value):
    return isinstance(value, str) and value.strip().startswith("<") and value.strip().endswith(">")


def check_identity(d, rep):
    name = d.get("name")
    if isinstance(name, str) and not is_placeholder(name):
        if not re.fullmatch(r"[a-z0-9]+(-[a-z0-9]+)*", name):
            rep.error("name must be kebab-case, lower case, no underscores: %r" % name)
        if "-and-" in name:
            rep.warn("name contains 'and'. A skill that does two things is two skills.")
        sid = d.get("skill_id")
        if isinstance(sid, str) and "/" in sid and not is_placeholder(sid):
            if sid.split("/", 1)[1] != name:
                rep.error("skill_id suffix %r does not match name %r."
                          % (sid.split("/", 1)[1], name))
    ver = d.get("skill_version")
    if not (isinstance(ver, str) and re.fullmatch(r"\d+\.\d+\.\d+", ver)):
        rep.error("skill_version must be three numbers with no suffix, got %r" % (ver,))
    pub, sid = d.get("publisher"), d.get("skill_id")
    if all(isinstance(v, str) for v in (pub, sid)) and "/" in str(sid):
        if not is_placeholder(pub) and not is_placeholder(sid):
            if sid.split("/", 1)[0] != pub:
                rep.error("publisher %r is not the namespace segment of skill_id %r." % (pub, sid))
    if not isinstance(d.get("som_schema_version"), list):
        rep.error("som_schema_version must be a list of schema versions.")


def check_axes(d, rep):
    cat = d.get("category")
    if isinstance(cat, str) and is_placeholder(cat):
        rep.error("category is still a placeholder. X cannot be resolved, so the skill cannot be positioned.")
    elif cat not in CATEGORY_X:
        rep.error("category must be one of %s, got %r" % (sorted(CATEGORY_X), cat))
    else:
        rep.note("X = %d (%s)" % (CATEGORY_X[cat], cat))

    origin = d.get("origin")
    if origin not in ORIGIN_Y:
        rep.error("origin must be one of %s, got %r" % (sorted(ORIGIN_Y), origin))
    else:
        rep.note("Y = %d (%s)" % (ORIGIN_Y[origin], origin))

    st = d.get("skill_type")
    if st not in SKILL_TYPES:
        rep.error("skill_type must be one of %s, got %r" % (sorted(SKILL_TYPES), st))
    elif origin in ORIGIN_Y:
        coarse = {"vendor": "VENDOR", "publishing-house": "NEWSROOM", "reference": "REFERENCE"}
        expected = coarse.get(origin)
        if expected and st != expected:
            rep.warn("skill_type %s sits oddly with origin %s. origin is the value the priority "
                     "model reads; keep them consistent or explain in section 10." % (st, origin))

    if d.get("lifecycle") not in LIFECYCLES:
        rep.error("lifecycle must be one of %s, got %r" % (sorted(LIFECYCLES), d.get("lifecycle")))

    for field, why in BANNED_FIELDS.items():
        if field in d:
            rep.error("frontmatter carries %r: %s" % (field, why))


def check_recall(d, rep):
    recall = d.get("recall")
    if not isinstance(recall, dict):
        rep.error("recall must be a mapping with target_system_type, conditions and recall_on.")
        return
    for key in ("target_system_type", "target_capability", "conditions", "recall_on", "state_path"):
        if key not in recall:
            rep.error("recall is missing %r." % key)

    tst = recall.get("target_system_type")
    if not isinstance(tst, list) or not tst:
        rep.error("recall.target_system_type must be a non-empty list from the SOM system_type enum.")

    conds = recall.get("conditions")
    if not isinstance(conds, list) or not conds:
        rep.error("recall.conditions must be a non-empty list.")
    else:
        for i, c in enumerate(conds):
            if not isinstance(c, dict):
                rep.error("recall.conditions[%d] is not a mapping." % i)
                continue
            kind = c.get("kind")
            if kind in DEFERRED_KINDS:
                rep.warn("recall.conditions[%d].kind is %r, which is recorded but lifted out of the "
                         "current handover. Build on field or field_change." % (i, kind))
            elif kind not in CONDITION_KINDS:
                rep.error("recall.conditions[%d].kind must be field or field_change, got %r" % (i, kind))
            op = c.get("op")
            if op not in OPS:
                rep.error("recall.conditions[%d].op must be one of %s, got %r" % (i, sorted(OPS), op))
            path = c.get("path")
            if isinstance(path, str) and "{{" in path:
                rep.warn("recall.conditions[%d].path is config-templated. The lookup table cannot "
                         "index it as a single row. Say so in sections 2 and 10." % i)

    ro = recall.get("recall_on")
    if not isinstance(ro, list) or not ro:
        rep.error("recall.recall_on must be a non-empty list of topics.")


def check_output_and_guarantees(d, rep):
    om = d.get("output_messages")
    if not isinstance(om, list) or not om:
        rep.error("output_messages must be a non-empty list.")
    elif [m for m in om if m != "skill.warning.raised"]:
        rep.warn("output_messages beyond skill.warning.raised: only that one has a ratified "
                 "payload. Note it in section 10.")

    sr = d.get("severity_range")
    if not isinstance(sr, list) or not sr:
        rep.error("severity_range must be a non-empty list, lower case.")
    else:
        for s in sr:
            if not isinstance(s, str) or s != str(s).lower():
                rep.error("severity values are lower case, permanently: %r" % (s,))
            elif s not in SEVERITIES:
                rep.error("severity %r is not one of %s" % (s, SEVERITIES))

    if d.get("auto_change_content") is not False:
        rep.error("auto_change_content must be false. It is the only permitted value.")
    if d.get("fail_closed") is not True:
        rep.error("fail_closed must be true. It is the only permitted value.")
    if d.get("migration_policy") not in MIGRATION:
        rep.error("migration_policy must be one of %s, got %r" % (sorted(MIGRATION), d.get("migration_policy")))
    if d.get("disclosure_level") not in DISCLOSURE:
        rep.error("disclosure_level must be one of %s, got %r" % (sorted(DISCLOSURE), d.get("disclosure_level")))
    if not isinstance(d.get("depends"), list):
        rep.error("depends must be a list, empty if there are none.")

    desc = d.get("description")
    if not isinstance(desc, str) or len(desc.split()) < 15:
        rep.warn("description is thin. It needs what the skill does and what it refuses to do.")
    elif not re.search(r"\b(does not|never|refuses|only)\b", desc, re.I):
        rep.warn("description does not state what the skill refuses to do. The refusals are the "
                 "half a reviewer checks the body against.")


def check_body(body, d, rep):
    heads = re.findall(r"^##\s+(\d+)\.\s*(.+?)\s*$", body, re.M)
    numbers = [int(n) for n, _ in heads]
    for i, title in enumerate(SECTION_TITLES, start=1):
        if i not in numbers:
            rep.error("section %d (%s) is missing." % (i, title))
    if numbers and numbers != sorted(numbers):
        rep.error("sections are out of order: %s" % numbers)

    low = body.lower()
    evals = re.search(r"^##\s+9\..*?(?=^##\s|\Z)", body, re.M | re.S)
    if not evals:
        rep.error("no section 9, so the eval set cannot be checked.")
    else:
        labels = [m.strip().lower() for m in
                  re.findall(r"^\|\s*([^|]+?)\s*\|", evals.group(0), re.M)]
        labels = [l for l in labels if l and not set(l) <= set("-: ")]
        for row in MANDATORY_EVAL_ROWS:
            if not any(l == row or l.startswith(row + ",") or l.startswith(row + " ")
                       for l in labels):
                rep.error("eval set has no %r case. The skill is not finished without it." % row)

    if "hold" in (d.get("severity_range") or []):
        if not re.search(r'"blocks"\s*:\s*\[\s*[^\]\s]', body):
            rep.warn("severity_range includes hold but the output contract shows no populated "
                     "blocks. blocks is required for a hold.")
        if re.search(r'"non_overridable"\s*:\s*false', body):
            rep.warn("severity_range includes hold but non_overridable shows false. A hold is "
                     "non-overridable.")
    else:
        if re.search(r"severity[^\n]{0,40}\bhold\b", body, re.I) and "never" not in low:
            rep.warn("the body mentions hold severity but severity_range does not include it.")

    if "causation_id" not in body:
        rep.warn("the output contract does not mention causation_id, which carries the triggering snapshot.")

    for pattern, why in ACTIVE_VOICE:
        for m in re.finditer(pattern, body, re.I):
            rep.warn("active voice %r: %s" % (m.group(0), why))
    for pattern, why in WITHDRAWN:
        for m in re.finditer(pattern, body, re.I | re.S):
            rep.warn("%r: %s" % (m.group(0)[:60], why))

    open_items = re.search(r"^##\s+10\..*?(?=^##\s|\Z)", body, re.M | re.S)
    if open_items and len(open_items.group(0).split()) < 25:
        rep.warn("section 10 is nearly empty. Several things in this model are unresolved; if none "
                 "of them touch this skill, say that explicitly.")

    reuse = re.search(r"^##\s+1\..*?(?=^##\s|\Z)", body, re.M | re.S)
    if reuse and len(re.findall(r"^\s*\d\.\s+\*\*", reuse.group(0), re.M)) < 3:
        rep.warn("section 1 does not show three reuse scenarios. The third should be a story "
                 "nobody has drawn yet.")


def validate(path):
    rep = Report(path)
    try:
        text = open(path, encoding="utf-8").read()
    except OSError as exc:
        rep.error("cannot read file: %s" % exc)
        return rep
    d, body = split_frontmatter(text, rep)
    if d is None:
        return rep
    for field in REQUIRED:
        if field not in d:
            rep.error("required field %r is missing." % field)
    check_identity(d, rep)
    check_axes(d, rep)
    check_recall(d, rep)
    check_output_and_guarantees(d, rep)
    check_body(body, d, rep)
    placeholders = [k for k, v in d.items() if is_placeholder(v)]
    if placeholders:
        rep.note("still placeholder: %s" % ", ".join(sorted(placeholders)))
    return rep


def main():
    ap = argparse.ArgumentParser(description="Validate SOM skill files.")
    ap.add_argument("paths", nargs="+")
    ap.add_argument("--json", action="store_true", help="machine-readable output")
    args = ap.parse_args()

    reports = [validate(p) for p in args.paths]

    if args.json:
        print(json.dumps([{"path": r.path, "ok": r.ok, "errors": r.errors,
                           "warnings": r.warnings, "notes": r.notes} for r in reports], indent=2))
    else:
        for r in reports:
            print("\n%s" % r.path)
            print("-" * len(r.path))
            for e in r.errors:
                print("  ERROR    %s" % e)
            for w in r.warnings:
                print("  warning  %s" % w)
            for n in r.notes:
                print("  note     %s" % n)
            print("  %s (%d errors, %d warnings)"
                  % ("PASS" if r.ok else "FAIL", len(r.errors), len(r.warnings)))
    return 0 if all(r.ok for r in reports) else 1


if __name__ == "__main__":
    sys.exit(main())
