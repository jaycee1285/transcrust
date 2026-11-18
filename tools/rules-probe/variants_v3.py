"""v3 = v2 + fixed vocative cue (comma-list != vocative). Added alongside variants.py; v1 and v2 untouched.
Rule (stated before running): vocative needs clause-initial span + following pause mark AND the span must not be
list-signalled within its sentence: L1 `, and|or` follows; L2 `, <1-3 word item>(, <1-3 word item>)*, and|or` follows.
Run: python3 -I variants_v3.py   (writes nothing; prints markdown for the public sets: fixtures-vocative + extra)
"""
import copy, os, re, sys
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
import rules_probe as R, variants as V

_ITEM = r"[A-Za-z0-9'’-]+(?:\s+[A-Za-z0-9'’-]+){0,2}"
LIST_AFTER = re.compile(r"^\s*,\s*(?:(?:%s)\s*,\s*)*(?:and|or)\b" % _ITEM, re.I)

def list_signalled(c):
    sent_after = re.split(r"[.!?]", c.after_raw)[0]
    return bool(LIST_AFTER.match(sent_after))

def rule2_v3(c):
    if c.clause_start and list_signalled(c):
        c2 = copy.copy(c)
        c2.clause_start = False  # only the vocative cue reads clause_start
        return R.rule2_slots(c2)
    return R.rule2_slots(c)

V2 = V.VARIANTS["v2"]
V3 = V.swap(V2, 1, rule2_v3)
VARIANTS_V3 = {"v1": V.VARIANTS["v1"], "v2": V2, "v3": V3}

if __name__ == "__main__":
    import json
    for f in ("fixtures-vocative.json", "fixtures-extra.json"):
        cs = json.load(open(os.path.join(HERE, f)))
        for vn, rules in VARIANTS_V3.items():
            run = V.chain(rules); print(f"\n### {f} {vn}\n| id | want | got | rule |\n|---|---|---|---|")
            for c in cs:
                x = R.Ctx(c, c["candidate"]["canonical"]); rule, lab, _ = run(x)
                print(f"| {c['id']} | {c['want']} | {lab} | {rule} |")
