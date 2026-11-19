"""Export the v3 chain (variants_v3.V3) decisions over the PUBLIC cases as a JSON golden for the Rust port
(crates/decision/src/chain.rs). Sets: fixtures-extra.json, fixtures-vocative.json, and every row of
../pair-router-probe/cross-encoder-q8/cases.json (synthetic rows + the name-scrubbed household rows).
Canonical for cases without a candidate comes from the same baseline lookup variants.py uses.
Run: python3 -I export_chain_golden.py   (writes chain-v3-golden.json next to this file)
"""
import json, os, sys
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
import rules_probe as R, variants as V, variants_v3 as V3

baseline = {x["case"]["id"]: x for x in
            json.load(open(os.path.join(R.PROBE, "bluey-q8-expanded-glossary", "results.json")))["cases"]
            if x["arm"] == "single"}
run = V.chain(V3.V3)
files = [("fixtures-extra.json", os.path.join(HERE, "fixtures-extra.json")),
         ("fixtures-vocative.json", os.path.join(HERE, "fixtures-vocative.json")),
         ("cases.json", R.CASES)]
out = []
for fname, path in files:
    for case in json.load(open(path)):
        canonical = R.canonical_of(case, baseline)
        c = R.Ctx(case, canonical)
        rule, label, reason = run(c)
        out.append({"file": fname, "id": case["id"], "arm": case["arm"], "source": case["source"],
                    "text": c.full_text, "canonical": canonical,
                    "description": R.glossary_description(case["state"], canonical),
                    "label": label, "rule": rule, "reason": reason})
with open(os.path.join(HERE, "chain-v3-golden.json"), "w", encoding="utf-8") as f:
    json.dump(out, f, ensure_ascii=False, indent=1); f.write("\n")
print(len(out), "cases")
