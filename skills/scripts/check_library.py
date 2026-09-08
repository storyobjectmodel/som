#!/usr/bin/env python3
"""Library-level consistency checks across a folder of Smart Stories SOM skills.

The per-file validator in som-skill-author checks one file against the template.
This checks the things only visible across the whole set: shared constants that
have drifted, two files contradicting each other, a skill referenced by name that
does not exist, a hold advertised without a populated blocks array, and the
qualitative traps that let a single-use skill hide inside a general name.

Usage:
    python3 check_library.py <folder-of-skills>
    python3 check_library.py <folder> --json

Exit code 0 when there are no errors, 1 otherwise. Warnings never fail the run:
they are things a reviewer should look at, and the answer is often a sentence in
section 10 rather than a change.
"""

import argparse
import glob
import json
import os
import re
import sys

try:
    import yaml
except ImportError:
    sys.exit("pyyaml is required: pip install pyyaml --break-system-packages")

# Constants every file in this library shares. Drift here is the failure that
# a per-file validator cannot see, because each file is individually legal.
SHARED = {
    "publisher": "smart-stories",
    "origin": "reference",
    "skill_type": "REFERENCE",
    "migration_policy": "GATED",
    "disclosure_level": "L2",
    "auto_change_content": False,
    "fail_closed": True,
}

# Open items nobody has settled. A file that depends on one and does not name it
# reads to the next person as though the question were closed.
OPEN_ITEMS = {
    "clearance authority": r"authorit(y|ies)[^.]{0,80}(compar|undefined|not defined|open|unsettled)",
    "targeting binding vs advisory": r"(binding|advisory)[^.]{0,60}target|target[^.]{0,60}(binding|advisory)",
    "disclosure_level meaning": r"disclosure_level",
    "skill_uri requiredness": r"skill_uri",
    "skill_id format": r"skill_id[^.]{0,80}(format|four|incompatible|circulation)",
}

WITHDRAWN = [
    (r"most[- ]restrictive[- ]wins", "formally withdrawn 29 July 2026. Gates combine by conjunction."),
    (r"story[- ]owner precedence", "formally withdrawn 29 July 2026."),
    (r"\bfires_on\b", "renamed to recall_on under the passive convention."),
]

ORDERING_CLAIMS = [
    (r"sits above every", "an author may not claim what a skill outranks. State X, Y, Z unset."),
    (r"outranks (every|all)\b", "an author may not claim what a skill outranks."),
    (r"overrides (that|the other|another) skill", "the author does not know the house's overrides."),
    (r"\bskill_priority\b", "priority is a coordinate, not a word."),
]

ACTIVE_VOICE = [
    r"\bthe skill (fires|acts|subscribes|emits|runs itself|executes|blocks|holds|watches)\b",
    r"\bthis skill (fires|acts|subscribes|emits|blocks the story)\b",
    r"\bskill (subscribes to|listens on) the bus\b",
]

DASHES = "\u2014\u2013"  # em dash, en dash

# The library's settled position on its closest pair: hold-while-flagged is the one
# that may bind a whole story or one telling of it, and gate-by-scope is link scoped
# always, because a per-path publish permission has no meaning at story level. A file
# that lets gate-by-scope be story scoped, or that says "either" of the pair may sit at
# either scope, hands a vendor a configuration the other half of the library calls a
# configuration error and rejects. That contradiction shipped once already, and a vendor
# reads a green run as clearance to build against it, so this one is an error.
STORY_SCOPE = (r"(?:`?story:(?:<id>|\{\{[^}]{0,40}\}\}|[a-z0-9_.-]+)`?"
               r"|story[- ]scoped\b|story scope\b|story level\b)")
PERMISSIVE = (r"\b(?:may|might|can|could|either|both|permitted|allowed|optionally"
              r"|configurable)\b")
# Stay inside one sentence and one paragraph. Two claims that are correct apart, a
# prose colon between them and a paragraph break, would otherwise read as one wrong
# claim, and a checker that cries wolf on the corrected file is worth nothing.
GAP = r"(?:[^.?!\n]|\n(?!\s*\n))"

SCOPE_CLAIMS = [
    (r"either\s+(?:skill\s+|one\s+|of\s+(?:them|the\s+two|the\s+pair)\s+)?"
     r"(?:may|might|can|could)\s+be\s+(?:configured|set|declared|scoped|bound)",
     "puts the pair at either scope. Only hold-while-flagged may be story or link "
     "scoped; gate-by-scope is link:<id> always."),
    (r"both\s+(?:skills\s+|of\s+them\s+|of\s+the\s+pair\s+)?(?:may|might|can|could)\s+be\s+"
     r"(?:configured|set|declared|scoped|bound)" + GAP + r"{0,90}" + STORY_SCOPE,
     "puts both skills of the pair at story scope. Only hold-while-flagged may be "
     "story scoped."),
    (r"gate-by-scope" + GAP + r"{0,160}" + PERMISSIVE + GAP + r"{0,90}" + STORY_SCOPE,
     "reads gate-by-scope as permitted at story scope. It is link:<id> always, and an "
     "instance declaring one answer for the whole story is a configuration error that "
     "should be rejected rather than coerced."),
    (STORY_SCOPE + GAP + r"{0,120}" + PERMISSIVE + GAP + r"{0,80}gate-by-scope",
     "reads gate-by-scope as permitted at story scope. It is link:<id> always, and an "
     "instance declaring one answer for the whole story is a configuration error that "
     "should be rejected rather than coerced."),
]

# The clearance rule is the library's working position and nothing more. Calling it
# agreed invents an authority nobody granted, and a vendor who reads it as settled
# hard-codes enforcement against a comparison the group has not defined. The passive
# convention of 29 July 2026 and the withdrawals of the same date are genuinely agreed,
# so a status word sitting next to those is right and must not be flagged.
CLEARANCE_RULE = r"equal or higher authority"
CLEARANCE_STATUS = r"\b(agreed|ratified|settled|established)\b"
NEGATED = (r"\b(not|no|never|nobody|nothing|neither|nor|rather than|far from|without"
           r"|undefined|unsettled|unratified)\b|n't")
GENUINELY_AGREED = r"passive convention|passive[- ]skill convention|29 July 2026|withdraw|renamed"

# Documents that are not in the package. A vendor reading "see the handover" concludes
# that authoritative detail exists and is being withheld, and either stops or invents
# the missing rule. Everything a vendor needs has to be in the nine files and the
# conventions doc, or not be cited at all.
EXTERNAL_DOCS = [
    r"\bthe recall handover\b",
    r"\bthe handover\b",
    r"\bthe open-items reference\b",
    r"\bsource documents?\b",
    r"\breference tables?\b",
    r"\bthe library one-liner\b",
    r"\bopen item \d+\b",
]

# Forward references to work in another file go stale silently: the other file lands,
# nobody comes back to this sentence, and a vendor builds a workaround for something
# already delivered. State what is true now, or say nothing.
FORWARD_REFS = [
    r"is being updated in parallel",
    r"(?:is|are|was|were) being updated\b",
    r"will be added\b",
    r"\bto be added\b",
    r"\bpending\b",
    r"not yet carried\b",
    r"\bnot yet (?:added|present|written|available|reflected)\b",
]


class Findings:
    def __init__(self):
        self.errors = []
        self.warnings = []
        self.notes = []

    def error(self, where, msg):
        self.errors.append("%s: %s" % (where, msg))

    def warn(self, where, msg):
        self.warnings.append("%s: %s" % (where, msg))

    def note(self, msg):
        self.notes.append(msg)

    @property
    def ok(self):
        return not self.errors


def load(path, f):
    text = open(path, encoding="utf-8").read()
    m = re.match(r"^---\n(.*?)\n---\n(.*)$", text, re.S)
    if not m:
        f.error(os.path.basename(path), "no YAML frontmatter.")
        return None, ""
    try:
        return yaml.safe_load(m.group(1)), m.group(2)
    except yaml.YAMLError as exc:
        f.error(os.path.basename(path), "frontmatter does not parse: %s" % exc)
        return None, m.group(2)


def section(body, n):
    m = re.search(r"^##\s+%d\..*?(?=^##\s|\Z)" % n, body, re.M | re.S)
    return m.group(0) if m else ""


def sentence_at(text, start, end, following=0):
    """The sentence containing text[start:end], optionally plus the ones after it.

    A claim about a rule and the status claimed for it are often two sentences: the
    rule is stated, then "that is settled" points back at it. Reading one sentence
    alone would miss the pointer, and reading the whole file would find every
    unrelated use of the same word.
    """
    lo = text.rfind(".", 0, start) + 1
    hi = end
    for _ in range(following + 1):
        nxt = text.find(".", hi)
        hi = len(text) if nxt < 0 else nxt + 1
    return text[lo:hi]


def near(text, index, pattern, window=140):
    """Is pattern within window characters either side of index."""
    lo = max(0, index - window)
    return re.search(pattern, text[lo:index + window], re.I) is not None


def preceded_by(text, index, pattern, window=90):
    """Is pattern within window characters before index, and not after it.

    A denial has to come first to be a denial. "no vendor should read it as settled"
    is the correct sentence; "settled, and no vendor should read the comparison as
    defined" is not, and a window that reached forwards would clear them both.
    """
    lo = max(0, index - window)
    return re.search(pattern, text[lo:index], re.I) is not None


def check_assertions(name, body, known, f):
    """Cross-file assertions that must read the same way wherever they are made.

    These run over the skills and over the conventions doc alike, because the doc every
    skill points to is the one place a contradiction propagates from.
    """
    # Scope claims about the closest pair. See SCOPE_CLAIMS: this is the class of
    # contradiction that shipped, so it fails the run rather than asking for a look.
    for pattern, why in SCOPE_CLAIMS:
        for m in re.finditer(pattern, body, re.I):
            f.error(name, "%r: %s" % (re.sub(r"\s+", " ", m.group(0))[:90], why))

    # The clearance rule's epistemic status.
    for m in re.finditer(CLEARANCE_RULE, body, re.I):
        span = sentence_at(body, m.start(), m.end(), following=1)
        for s in re.finditer(CLEARANCE_STATUS, span, re.I):
            if preceded_by(span, s.start(), NEGATED, window=90):
                continue
            if near(span, s.start(), GENUINELY_AGREED, window=140):
                continue
            f.warn(name, "calls the clearance rule %r. That a declaration closes only "
                         "on an assertion of equal or higher authority on the same "
                         "scope is this library's working position, not a rule any "
                         "group has ratified, and a vendor who reads it as settled "
                         "hard-codes enforcement against a comparison nobody has "
                         "defined." % s.group(0))

    # Documents nobody in this package can open.
    for pattern in EXTERNAL_DOCS:
        for m in re.finditer(pattern, body, re.I):
            f.warn(name, "cites %r, which is not in this package. A vendor reads a "
                         "pointer to a document they cannot open as authoritative "
                         "detail being withheld, so carry the substance across or drop "
                         "the pointer." % m.group(0))

    # Forward references to another skill's unfinished work.
    others = {k for k in known if k != name}
    for pattern in FORWARD_REFS:
        for m in re.finditer(pattern, body, re.I):
            span = sentence_at(body, m.start(), m.end())
            named = sorted(o for o in others if o in span)
            if not named:
                continue
            f.warn(name, "%r about %s. A forward reference goes stale the moment the "
                         "other file lands and nobody comes back to this sentence, and "
                         "a vendor builds a workaround for work already delivered. Say "
                         "what is true now." % (m.group(0), ", ".join(named)))


def check_advert_against_build_notes(name, d, body, f):
    """Section 11 addresses exactly the system types the advert names.

    Under the binding reading of targeting that this library adopts, an executor is
    recalled only for a type the advert names, so a build note addressed to any other
    type describes work nobody could ever be recalled to do, and a named type with no
    build note leaves an addressee nothing to build.
    """
    types = ((d.get("recall") or {}).get("target_system_type")) or []
    if not isinstance(types, list) or not types:
        return
    s11 = section(body, 11)
    if not s11:
        return
    # Tolerant of table formatting: a row is a line that leads with a pipe, minus the
    # separator rows and the header, so an extra alignment colon or a stray space in a
    # rule does not change the count.
    rows = [ln.strip() for ln in s11.splitlines() if ln.strip().startswith("|")]
    rows = [ln for ln in rows if not re.match(r"^\|[\s:|-]+\|?$", ln)]
    if len(rows) < 2:
        return
    labels = [r.strip("|").split("|")[0].strip() for r in rows[1:]]
    if len(labels) == len(types):
        return
    unnamed = [lb for lb in labels
               if not any(t.replace("_", " ").lower() in lb.lower() for t in types)]
    detail = ("rows addressed to a type the advert does not name: %s"
              % ", ".join("%r" % lb for lb in unnamed)) if unnamed else \
             ("every row resolves to an advertised type, so a type is addressed more "
              "than once or not at all")
    f.warn(name, "section 11 has %d build note row(s) but the advert names %d target "
                 "system type(s) (%s); %s. Under the binding reading of targeting, a "
                 "build note addressed to a type the advert does not name describes "
                 "work no executor could be recalled to do."
           % (len(labels), len(types), ", ".join(types), detail))


def check_conventions(path, known, f):
    """The conventions doc, checked without demanding frontmatter.

    It is not a skill and has no advert, so the per-file skill checks do not apply and
    the loader's frontmatter requirement would only produce a false error. It is also
    the document every skill file points at, which makes it the worst place in the
    package for an unchecked assertion: the contradiction that shipped lived here.
    """
    name = os.path.basename(path)
    body = open(path, encoding="utf-8").read()
    check_assertions(name, body, known, f)

    # Withdrawn rules. The doc is the right place to record that a rule was withdrawn,
    # so a phrase marked as withdrawn or renamed is the correct text, not the failure.
    for pattern, why in WITHDRAWN:
        for m in re.finditer(pattern, body, re.I):
            if near(body, m.start(), r"withdraw|no longer|renamed|superseded", window=200):
                continue
            f.warn(name, "%r reads as current here: %s" % (m.group(0)[:50], why))


def check_file(name, d, body, f):
    # Shared constants have not drifted.
    for key, want in SHARED.items():
        got = d.get(key)
        if got != want:
            f.error(name, "shared constant %s should be %r across the library, got %r"
                    % (key, want, got))

    # depends has stayed empty; chaining goes through the bus.
    if d.get("depends"):
        f.warn(name, "depends is non-empty. Chaining happens through the bus, and depending on "
                     "what reads you inverts the dependency. If this is deliberate, say so in "
                     "section 10.")

    # skill_id, publisher and name agree.
    sid, pub, nm = d.get("skill_id"), d.get("publisher"), d.get("name")
    if isinstance(sid, str) and "/" in sid:
        ns, suffix = sid.split("/", 1)
        if ns != pub:
            f.error(name, "publisher %r is not the namespace of skill_id %r" % (pub, sid))
        if suffix != nm:
            f.error(name, "skill_id suffix %r does not match name %r" % (suffix, nm))

    # A hold must be backed by a populated blocks array and non_overridable.
    sev = d.get("severity_range") or []
    contract = section(body, 6)
    if "hold" in sev:
        if not re.search(r'"blocks"\s*:\s*\[\s*[^\]\s]', contract):
            f.error(name, "severity_range includes hold but section 6 shows no populated blocks. "
                          "blocks is required for a hold.")
        if re.search(r'"non_overridable"\s*:\s*false', contract):
            f.error(name, "severity_range includes hold but non_overridable is false.")
    else:
        if re.search(r'"non_overridable"\s*:\s*true', contract):
            f.warn(name, "non_overridable is true but severity_range has no hold.")
        if re.search(r'"blocks"\s*:\s*\[\s*[^\]\s]', contract):
            f.warn(name, "blocks is populated but severity_range has no hold. A skill with no "
                         "hold authority should withhold nothing.")

    # Placeholders that are not ours to supply must stay markers.
    for key in ("skill_uri", "skill_content_sha", "owner_editorial", "owner_engineering", "licence"):
        v = d.get(key)
        if isinstance(v, str) and not (v.strip().startswith("<") and v.strip().endswith(">")):
            f.warn(name, "%s has a concrete value. If it was invented, replace it with the "
                         "angle-bracket marker: a fake value reads as a real one." % key)

    # Withdrawn rules and ordering claims.
    for pattern, why in WITHDRAWN + ORDERING_CLAIMS:
        for m in re.finditer(pattern, body, re.I):
            f.error(name, "%r: %s" % (m.group(0)[:50], why))

    # Passive convention.
    for pattern in ACTIVE_VOICE:
        for m in re.finditer(pattern, body, re.I):
            f.warn(name, "active voice %r: skills declare and display, executors act."
                   % m.group(0))
    desc = d.get("description") or ""
    if re.match(r"\s*(Holds|Watches|Resolves|Runs|Raises|Records|Matches|Surfaces|Applies|Sets)\b",
                desc):
        f.warn(name, "the description opens in active voice. It is the most-read line in the "
                     "file, so it is the worst place for it.")

    # Dashes the house style forbids.
    for ch in DASHES:
        if ch in body or ch in yaml.safe_dump(d, allow_unicode=True):
            f.error(name, "contains %r. Use commas, colons or parentheses." % ch)

    # Relative link from skills/ to the conventions doc.
    if re.search(r"(?<!\.\./)\bdocs/CONVENTIONS\.md", body):
        f.error(name, "links to docs/CONVENTIONS.md, which does not resolve from skills/. "
                      "Use ../docs/CONVENTIONS.md.")

    # Three reuse scenarios, and two genuinely worked instances.
    s1, s8 = section(body, 1), section(body, 8)
    n_reuse = len(re.findall(r"^\s*\d\.\s+\*\*", s1, re.M))
    if n_reuse < 3:
        f.error(name, "section 1 shows %d reuse scenarios, needs 3. The third should be a story "
                      "nobody has drawn yet." % n_reuse)
    n_inst = len(re.findall(r"^```yaml", s8, re.M))
    if n_inst < 2:
        f.error(name, "section 8 shows %d worked instances, needs 2 from different situations."
                % n_inst)

    # Section 8 instances that differ only cosmetically leave the reuse claim untested.
    blocks = re.findall(r"^```yaml\n(.*?)^```", s8, re.M | re.S)
    if len(blocks) >= 2:
        def keys(b):
            return {ln.split(":", 1)[0].strip() for ln in b.splitlines()
                    if ":" in ln and not ln.strip().startswith("#")}
        a, b = blocks[0], blocks[1]
        shared_keys = keys(a) & keys(b)
        differing = 0
        for k in shared_keys:
            va = re.search(r"^%s:\s*(.*)$" % re.escape(k), a, re.M)
            vb = re.search(r"^%s:\s*(.*)$" % re.escape(k), b, re.M)
            if va and vb and va.group(1).strip() != vb.group(1).strip():
                differing += 1
        if differing <= 1:
            f.warn(name, "section 8's two instances differ in %d configured value(s) beyond the "
                         "label. Two knobs turned on one story is not two situations, and it "
                         "leaves the section 1 reuse claim untested." % differing)

    # Section 10 has to name what it depends on that is unresolved.
    s10 = section(body, 10)
    if len(s10.split()) < 60:
        f.warn(name, "section 10 is thin. Several things in this model are genuinely unresolved; "
                     "if none touch this skill, say that explicitly.")
    missing = [label for label, pat in OPEN_ITEMS.items()
               if not re.search(pat, s10, re.I)]
    if missing:
        f.warn(name, "section 10 does not mention: %s. Quietly picking a side reads as though "
                     "the question were settled." % ", ".join(sorted(missing)))

    # No narrative scaffolding. These files are standalone library references, so a
    # beat number or a "what the room sees" framing means the file still depends on a
    # walkthrough the reader does not have, and a reuse claim anchored to one telling
    # of one story is a single-use skill wearing a general name.
    scaffolding = re.findall(r"\bBeat \d|\bthe demo\b|\bwhat the room sees\b",
                             body, re.IGNORECASE)
    if scaffolding:
        f.warn(name, "narrative scaffolding is still present (%s). These files are read "
                     "without the walkthrough, so anchor the claim to a situation, not "
                     "to a beat." % ", ".join(sorted(set(s.lower() for s in scaffolding))))

    # Inform-only skills must not describe themselves as holding. "withholds nothing"
    # and "holds nothing" are the correct disclaimers, not the failure, so a negated
    # object has to be excluded or the check fires on exactly the right answer.
    if "hold" not in sev:
        for m in re.finditer(r"\bthis skill (holds|withholds)\s+(\w+)", body, re.I):
            if m.group(2).lower() not in ("nothing", "no", "none", "neither"):
                f.error(name, "describes itself as holding (%r) but has no hold authority."
                        % m.group(0))

    # The advert and the build notes address the same audience.
    check_advert_against_build_notes(name, d, body, f)


def check_lock_agreement(fms, f):
    """Every file declares the same schema version, lifecycle and skill version.

    Each of these values is legal on its own, so the per-file validator reading one file
    cannot tell the set has split. Only a read across the library can. Divergence is an
    error rather than a warning because the lookup table and the README both speak for
    the set: one file that has moved makes every sentence either of them writes about
    the other eight untrue, which is a worse state than a set where nothing has moved.
    """
    for key in ("som_schema_version", "lifecycle", "skill_version"):
        groups = {}
        for name in sorted(fms):
            # Values are grouped by their printed form because som_schema_version is a
            # list, and a list cannot be a dict key.
            groups.setdefault(repr(fms[name].get(key)), []).append(name)
        if len(groups) < 2:
            continue
        # Largest group first: the odd file out is what a reader is looking for.
        detail = "; ".join("%s in %s" % (value, ", ".join(who)) for value, who in
                           sorted(groups.items(), key=lambda kv: (-len(kv[1]), kv[0])))
        f.error("library", "%s does not agree across the %d files: %s. The lookup table and the "
                           "README both speak for the set, so a library where one file has moved "
                           "is worse than one where none has." % (key, len(fms), detail))


def check_set(names, bodies, fms, f):
    # Duplicate names.
    seen = {}
    for n in names:
        seen[n] = seen.get(n, 0) + 1
    for n, c in seen.items():
        if c > 1:
            f.error("library", "skill name %r appears %d times." % (n, c))

    # A skill referenced by name in another file must exist.
    known = set(names)
    for name, body in bodies.items():
        # Section 10 legitimately names hypothetical conformant forms of a skill's own
        # name, so a name there is a discussion, not a dangling reference.
        outside_10 = body.replace(section(body, 10), "")
        for ref in set(re.findall(r"`([a-z][a-z0-9]*(?:-[a-z0-9]+){1,4})`", outside_10)):
            if ref in known or ref == name:
                continue
            # Only complain about things shaped like a skill name we half-recognise.
            if any(ref.startswith(k.split("-")[0] + "-") for k in known):
                f.warn(name, "references `%s`, which is not a skill in this library. If it is "
                             "planned, say so; if it is a typo, fix it." % ref)

    # Two files must not give contradictory accounts of the same pair.
    pair = ("hold-while-flagged", "gate-by-scope")
    if set(pair) <= known:
        for name in pair:
            body = bodies[name]
            other = pair[1] if name == pair[0] else pair[0]
            if re.search(r"%s[^.]{0,120}(always|only|stays)\s+`?story:" % re.escape(other), body):
                f.error(name, "claims %s is fixed at story scope. Only hold-while-flagged "
                              "exposes scope as config; gate-by-scope is link:<id> always. The "
                              "two are distinguished by what is read (a typed flag versus a "
                              "path's gate), not by scope." % other)

    # Assertions that have to read the same way in every file that makes them. The check
    # above catches one shape of one contradiction; these catch the claim wherever it is
    # made, in whichever file makes it.
    for name, body in bodies.items():
        check_assertions(name, body, known, f)

    # The lock, read across the set rather than file by file.
    check_lock_agreement(fms, f)

    # The flag-to-hold chain has a known gap; files that rely on it must name it.
    raisers = [n for n in names if n in ("flag-on-mismatch", "raise-flag-on-match")]
    for n in raisers + (["hold-while-flagged"] if "hold-while-flagged" in known else []):
        s10 = section(bodies[n], 10)
        if not re.search(r"materialis|carrier|editorial_gates\[\][^.]{0,80}(gap|nobody|assum)",
                         s10, re.I):
            f.warn(n, "relies on the flag-to-hold chain but section 10 does not name the gap: "
                      "nothing in the model says who carries a raised warning into "
                      "editorial_gates[].")

    f.note("%d skills checked: %s" % (len(names), ", ".join(sorted(names))))


def find_conventions(folder):
    """Locate the conventions doc from the skills folder the caller pointed at.

    Every skill file links to it as ../docs/CONVENTIONS.md, so it is normally a sibling
    of the folder under check rather than in it. Looking in both places means a caller
    pointing at either layout gets the doc checked instead of silently skipped.
    """
    folder = os.path.abspath(folder)
    for candidate in (os.path.join(folder, "CONVENTIONS.md"),
                      os.path.join(folder, "docs", "CONVENTIONS.md"),
                      os.path.join(os.path.dirname(folder), "docs", "CONVENTIONS.md"),
                      os.path.join(os.path.dirname(folder), "CONVENTIONS.md")):
        if os.path.isfile(candidate):
            return candidate
    return None


def main():
    ap = argparse.ArgumentParser(description="Check a Smart Stories SOM skill library for "
                                             "cross-file consistency.")
    ap.add_argument("folder")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    # Skip the documents that are not skills. Pointed at the library root rather than
    # skills/, an earlier version globbed the README, found no frontmatter and FAILed on
    # it. A vendor's first contact with the tooling should not be a false failure, and
    # "run it against the right folder" is a worse answer than not tripping over.
    NOT_SKILLS = {"README.md", "CONVENTIONS.md", "LOOKUP-TABLE.md"}
    paths = [p for p in sorted(glob.glob(os.path.join(args.folder, "*.md")))
             if os.path.basename(p) not in NOT_SKILLS]
    if not paths:
        sys.exit("no skill files in %s" % args.folder)

    f = Findings()
    conventions = find_conventions(args.folder)
    names, bodies, fms = [], {}, {}
    for p in paths:
        # The conventions doc is prose, not a skill. It goes through its own pass below.
        if os.path.basename(p).lower() == "conventions.md":
            continue
        d, body = load(p, f)
        if d is None:
            continue
        name = d.get("name") or os.path.basename(p)
        names.append(name)
        bodies[name] = body
        fms[name] = d
        check_file(name, d, body, f)
    if names:
        check_set(names, bodies, fms, f)
    if conventions:
        check_conventions(conventions, set(names), f)
        f.note("conventions doc checked: %s" % conventions)
    else:
        f.note("no CONVENTIONS.md found beside %s, so its assertions were not checked."
               % args.folder)

    if args.json:
        print(json.dumps({"ok": f.ok, "errors": f.errors,
                          "warnings": f.warnings, "notes": f.notes}, indent=2))
    else:
        for e in f.errors:
            print("  ERROR    %s" % e)
        for w in f.warnings:
            print("  warning  %s" % w)
        for n in f.notes:
            print("  note     %s" % n)
        print("\n  %s (%d errors, %d warnings)"
              % ("PASS" if f.ok else "FAIL", len(f.errors), len(f.warnings)))
    return 0 if f.ok else 1


if __name__ == "__main__":
    sys.exit(main())
