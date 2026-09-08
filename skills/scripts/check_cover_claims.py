#!/usr/bin/env python3
"""Fail when a cover claim disagrees with the files underneath it.

Four of the nine findings in the 0.2.0 review were one failure: a summary layer moved and
the files did not, and every validator passed the whole time. A validator that reads one
file at a time cannot see it, because each file is individually legal. This reads the
claims documents make ABOUT other files and checks them.

What it checks:

  library version   any "som-skill-library <v>" or "library v<v>" claim, against the
                    skill_version in skills/*.md frontmatter, which must all agree.
  build stamp       any "build <stamp>" or "Build <stamp>" claim, against the stamp the
                    walkthrough HTML declares in its masthead.
  decision          a file that names proposal A as still open, or prints both readings of
                    the registration question, after A was decided.
  generated files   docs/LOOKUP-TABLE.md and lookup-table.json against what the skill
                    frontmatter would generate right now.

Usage: check_cover_claims.py <library-root> [--walkthrough <path-to-html>]
"""
import glob, os, re, subprocess, sys

WITHDRAWN_CLAIMS = [
    (r"[Ee]ither the (?:global )?(?:lookup )?table resolves configuration",
     "proposal A settled this on 18 August. Say what A decided, not both readings."),
    (r"[Ww]hich of the two applies has not been settled",
     "proposal A settled it on 18 August."),
    (r"\bclear_state\b(?!`\s*\|\s*removed)",
     "removed in 0.2.0: editorial_gates[].status is a closed enum."),
    (r"gate\(path\)\s*==",
     "publish_when was removed in 0.2.1; the permit test is fixed, not an expression."),
    (r"editorial_gates\[[a-z_]+\]",
     "editorial_gates[] is not indexed by path name. Find the entry by gate_id."),
]

VERSION_CLAIM = re.compile(r"(?:som-skill-library|library)\s+v?(\d+\.\d+\.\d+)")
BUILD_CLAIM = re.compile(r"[Bb]uild(?:\s+stamp)?[^`\n]{0,12}[`*]*\s*(20\d\d-\d\d-\d\d[a-z]?)")


def frontmatter_versions(root):
    vs = {}
    for f in sorted(glob.glob(os.path.join(root, "skills", "*.md"))):
        m = re.search(r"^skill_version:\s*(\S+)", open(f).read(), re.M)
        if m:
            vs[os.path.basename(f)] = m.group(1)
    return vs


def walkthrough_stamp(path):
    if not path or not os.path.exists(path):
        return None
    m = re.search(r"BUILD (20\d\d-\d\d-\d\d[a-z]?)", open(path).read())
    return m.group(1) if m else None


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    root = args[0] if args else "."
    wt = sys.argv[sys.argv.index("--walkthrough") + 1] if "--walkthrough" in sys.argv else None
    if wt is None:
        for c in (os.path.join(root, "..", "IBC_SOM_Walkthrough_Skills.html"),
                  "IBC_SOM_Walkthrough_Skills.html"):
            if os.path.exists(c):
                wt = c
                break

    errors = []
    vs = frontmatter_versions(root)
    if not vs:
        errors.append(f"no skills found under {root}/skills. A check with nothing to check is not a pass.")
    if len(set(vs.values())) > 1:
        errors.append("skills disagree on skill_version: " +
                      ", ".join(f"{k}={v}" for k, v in sorted(vs.items())))
    lib_version = sorted(set(vs.values()))[0] if vs else None
    stamp = walkthrough_stamp(wt)

    targets = []
    for pat in ("*.md", "docs/*.md", "demo-skills/*.md", "skills/*.md",
                "configured-instances/*.yaml"):
        targets += sorted(glob.glob(os.path.join(root, pat)))
    if wt:
        targets.append(wt)

    HISTORY = {"SCHEMA-CONFORMANCE.md", "CHANGELOG.md"}
    for f in targets:
        rel = os.path.relpath(f, root)
        history = os.path.basename(f) in HISTORY
        txt = open(f).read()
        # strip the change-history notes: they quote old versions and stamps on purpose
        body = re.sub(r"\*\*(?:Changed|Removed|Updated) in [^*]*\*\*.*?(?=\n\n)", "", txt, flags=re.S)
        body = re.sub(r"##[^\n]*0\.2\.\d pass[\s\S]*?(?=\n## |\Z)", "", body)
        if lib_version and not history:
            for m in VERSION_CLAIM.finditer(body):
                if m.group(1) != lib_version:
                    line = body[:m.start()].count("\n") + 1
                    errors.append(f"{rel}:{line}: claims library {m.group(1)}; "
                                  f"skills/*.md declare {lib_version}")
        if stamp and not history:
            for m in BUILD_CLAIM.finditer(body):
                if m.group(1) != stamp:
                    line = body[:m.start()].count("\n") + 1
                    errors.append(f"{rel}:{line}: claims build {m.group(1)}; "
                                  f"the walkthrough declares {stamp}")
        if not history:
            for pat, why in WITHDRAWN_CLAIMS:
                for m in re.finditer(pat, body):
                    line = body[:m.start()].count("\n") + 1
                    errors.append(f"{rel}:{line}: `{m.group(0)}` is withdrawn: {why}")

    gen = os.path.join(root, "scripts", "build_lookup_table.py")
    if os.path.exists(gen):
        r = subprocess.run([sys.executable, gen, root, "--check"], capture_output=True, text=True)
        if r.returncode:
            errors.append("the generated lookup table is stale. Run build_lookup_table.py.")

    for e in errors:
        print(f"  ERROR    {e}")
    print(f"\n  {'FAIL' if errors else 'PASS'} ({len(errors)} cover claims disagree with the files"
          f"{'' if stamp else ', walkthrough not found so build stamps unchecked'})\n")
    sys.exit(1 if errors else 0)


if __name__ == "__main__":
    main()
