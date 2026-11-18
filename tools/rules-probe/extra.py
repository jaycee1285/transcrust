"""Run v1 and v2 chains over fixtures-extra.json (inflection, animacy, controls). Labels set by
linguistics before running. Run: python3 -I extra.py"""
import json, os, sys
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
import rules_probe as R, variants as V
cases = json.load(open(os.path.join(HERE, "fixtures-extra.json")))
for vname in ("v1", "v2"):
    run = V.chain(V.VARIANTS[vname]); ok = 0
    print(f"\n### {vname}\n| id | want | got | rule | reason |\n|---|---|---|---|---|")
    for case in cases:
        c = R.Ctx(case, case["candidate"]["canonical"]); rule, label, reason = run(c)
        ok += label == case["want"]
        print(f"| {case['id']} | {case['want']} | {label} | {rule} | {reason} |")
    print(f"\n{vname}: {ok}/{len(cases)} match")
