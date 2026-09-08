#!/usr/bin/env python3
"""Resolve field paths against the SOM story.context schema.

Two sweeps:

  adverts and worked configs   the literal `path:` lines in the frontmatter and the
                               literal values of known path-carrying config keys.
  prose                        every backticked token in the body that has the shape of
                               a field path. Six dead paths survived the 0.2.0 audit by
                               living in prose, worked examples and eval-set rows, which
                               the first sweep never reads. This is why the second exists.

A path whose first segment is a story.context property must resolve. A path whose first
segment is not is an error too, unless that root is declared in OUT_OF_SCHEMA below: a
token that looks like a schema path and is not one reads to a vendor as though it were.

Usage: check_paths.py <dir-of-skill-markdown> [--schema <file>] [--no-prose]
Exit 0 when nothing failed, 1 otherwise. Finding no files to check is a failure.
"""
import json, re, glob, os, sys

# Roots that are deliberately not story.context. Each is a real namespace, and each is
# listed here rather than skipped silently so that adding one is a decision.
OUT_OF_SCHEMA = {
    "config":     "the configured instance's own values, resolved by the house",
    "item":       "the monitored source item, which has no schema in SOM",
    "transcript": "a live transcript item, outside the story object",
    "filing":     "a wire filing item, outside the story object",
    "extensions": "the vendor namespace, patternProperties, unknowable here",
    "payload":    "the message envelope, not the story context",
    "originating_system": "the message envelope",
    "skill":      "the skill file's own frontmatter",
    "gate":       "prose shorthand, not a path",
    "telling":    "the Telling event family on the bus, not a story.context property",
}

# Bus topics have the same dotted shape as field paths and are not field paths. The set is
# built from every skill's own recall_on and emits lists rather than hard-coded, so a new
# family is exempt the moment a skill declares it and not before.
TOPIC_PREFIX = ("som.",)


def checker(SC):
    D = SC['$defs']

    def deref(n):
        while isinstance(n, dict) and '$ref' in n:
            n = D[n['$ref'].split('/')[-1]]
        return n

    def props(n):
        n = deref(n)
        if not isinstance(n, dict):
            return None
        if n.get('type') == 'array' or 'items' in n:
            return props(n.get('items', {}))
        return n.get('properties')

    top = props(SC) or {}

    def resolve(p):
        """True resolves, False does not, None not applicable."""
        if not p or '{{' in p or p in ('null', 'none', 'true', 'false'):
            return None
        p = re.sub(r'\[[^\]]*\]', '', p.strip().strip('"').strip("'"))
        segs = [s for s in p.split('.') if s]
        if segs and segs[0] in ('story', 'story_context'):
            segs = segs[1:]
        if not segs:
            return None
        if segs[0] in OUT_OF_SCHEMA:
            return None
        node = SC
        for s in segs:
            pr = props(node)
            if pr is None or s not in pr:
                return False
            node = pr[s]
        return True

    return resolve, set(top)


PATHKEYS = ('watched_field', 'condition_path', 'match_field', 'subject_path', 'record_path',
            'trigger_path', 'confidence_path', 'version_set_field', 'enrichment_record_path',
            'match_against', 'state_path')

# A backticked token worth testing: dotted, or an array marker, lower snake case throughout.
PROSE = re.compile(r'`([a-z][a-z0-9_]*(?:\[[^\]`]*\])?(?:\.[a-z][a-z0-9_]*(?:\[[^\]`]*\])?)+)`')


def main():
    args = [a for a in sys.argv[1:] if not a.startswith('--')]
    d = args[0] if args else 'skills'
    schema = sys.argv[sys.argv.index('--schema') + 1] if '--schema' in sys.argv \
        else os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(
                              os.path.abspath(__file__)))),
                          'schema', 'story-context.schema.json')
    resolve, top = checker(json.load(open(schema)))
    prose_on = '--no-prose' not in sys.argv
    files = sorted(glob.glob(os.path.join(d, '*.md')))
    files = [f for f in files if os.path.basename(f) not in
             ('README.md', 'CONVENTIONS.md', 'LOOKUP-TABLE.md', 'SCHEMA-CONFORMANCE.md',
              'SYSTEM-TYPE-VOCABULARIES.md')]
    if not files:
        print(f"  error  no skill markdown found in {d}. A check with nothing to check is not a pass.")
        print("\n  FAIL (0 files)")
        sys.exit(1)
    root_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    topic_srcs = sorted(glob.glob(os.path.join(root_dir, 'skills', '*.md'))) or files
    topics = set()
    for f in topic_srcs:
        fm = re.split(r'\n---\n', open(f).read(), 1)[0]
        for m in re.finditer(r'^\s*-\s*([a-z][a-z0-9_]*(?:\.[a-z][a-z0-9_]*)+)\s*$', fm, re.M):
            topics.add(m.group(1))
        for m in re.finditer(r'^\s*(?:recall_on|emits|topic):\s*\[?([^\]\n]*)\]?', fm, re.M):
            for t in re.findall(r'[a-z][a-z0-9_]*(?:\.[a-z][a-z0-9_]*)+', m.group(1)):
                topics.add(t)

    bad = 0
    for f in files:
        txt = open(f).read()
        name = os.path.basename(f)[:-3]
        fm = re.split(r'\n---\n', txt, 1)[0]
        for m in re.finditer(r'^\s+path:\s*(.+)$', fm, re.M):
            p = m.group(1).strip()
            if resolve(p) is False:
                print(f"  error  {name}: advert path does not resolve: {p}"); bad += 1
        for m in re.finditer(r'^\s*(' + '|'.join(PATHKEYS) + r'):\s*(.+)$', txt, re.M):
            p = m.group(2).strip().split('#')[0].strip()
            if not p or p.startswith('['):
                continue
            if resolve(p) is False:
                print(f"  error  {name}: configured path does not resolve: {p}"); bad += 1
        if not prose_on:
            continue
        seen = set()
        for m in PROSE.finditer(txt):
            p = m.group(1)
            if p in seen:
                continue
            seen.add(p)
            if p in topics or p.startswith(TOPIC_PREFIX):
                continue
            root = re.sub(r'\[[^\]]*\]', '', p.split('.')[0])
            if root in OUT_OF_SCHEMA or root in ('story', 'story_context'):
                continue
            if root not in top:
                line = txt[:m.start()].count('\n') + 1
                print(f"  error  {name}:{line}: `{p}` has the shape of a story.context path and "
                      f"`{root}` is not a story.context property. Either it resolves, or its root "
                      f"belongs in OUT_OF_SCHEMA with a reason.")
                bad += 1
                continue
            if resolve(p) is False:
                line = txt[:m.start()].count('\n') + 1
                print(f"  error  {name}:{line}: prose path does not resolve: `{p}`")
                bad += 1
    print(f"\n  {'FAIL' if bad else 'PASS'} ({bad} non-resolving paths, {len(files)} files"
          f"{'' if prose_on else ', prose sweep off'})")
    sys.exit(1 if bad else 0)


if __name__ == '__main__':
    main()
