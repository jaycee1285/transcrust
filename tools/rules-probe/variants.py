"""Rules-probe variants (v2, 2026-11-18). Post-hoc revisions, labelled as such.

Imports v1's rules unchanged and loops variants x set x arm over the same 132 frozen
decisions. Variants:
  v1           the original chain (control)
  v2-local     rule 5 looks only at the current sentence (design: fixed phrases are local;
               a neighbouring sentence is semantic context, which raised unwanted edits)
  v2-gloss     rule 3 (bare count noun) names only if the glossary description says the
               term is a person/animal/character; otherwise it passes
  v2           both
  v2+recency   v2, then discourse memory: a passed case becomes "name" only when the exact
               canonical spelling (capitalised) already appears in the preceding context
               sentence, i.e. the name was established earlier, not merely its franchise
Gate (stated before running): no variant may add wrong-names on household/single.
Run: python3 -I variants.py
"""
import collections, json, os, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import rules_probe as R  # noqa: E402

ANIMATE_DESC = re.compile(r"\b(dog|pet|cat|person|mom|dad|son|friend|character|kid|child|girl|boy|puppy)\b")

def rule3_gloss(c):
    r = R.rule3_bare_count_noun(c)
    if r and not ANIMATE_DESC.search(c.desc):
        return None
    return r

def current_sentence_text(c):
    text = c.full_text
    a, b = text.index("«"), text.index("»")
    before, after = text[:a], text[b + 1:]
    sb = re.split(r"[.!?]\s+", before)[-1]
    sa = re.split(r"[.!?]", after)[0]
    return (sb + " \u0000 " + sa).lower()

def rule5_local(c):
    saved = c.text_without_span
    c.text_without_span = current_sentence_text(c)
    try:
        return R.rule5_fixed_phrases(c)
    finally:
        c.text_without_span = saved

def recency(c):
    text = c.full_text
    a = text.index("«")
    before = text[:a]
    # Only a context sentence that precedes the current one counts as "earlier".
    parts = re.split(r"(?<=[.!?])\s+", before)
    earlier = " ".join(parts[:-1])
    if earlier and re.search(r"\b" + re.escape(c.canonical) + r"\b", earlier):
        return ("name", f'named "{c.canonical}": the name was already used earlier ("{c.canonical}" in the preceding sentence)')
    return None

def chain(rules):
    def run(c):
        for name, fn in rules:
            r = fn(c)
            if r:
                return name, r[0], r[1]
        return None, "pass", "passed"
    return run

V1 = list(R.RULES)
def swap(rules, idx, fn):
    out = list(rules); out[idx] = (out[idx][0] + "*", fn); return out

VARIANTS = {
    "v1": V1,
    "v2-local": swap(V1, 4, rule5_local),
    "v2-gloss": swap(V1, 2, rule3_gloss),
    "v2": swap(swap(V1, 4, rule5_local), 2, rule3_gloss),
}
VARIANTS["v2+recency"] = VARIANTS["v2"] + [("6 recency", recency)]

def main():
    cases = json.load(open(R.CASES))
    baseline = {x["case"]["id"]: x for x in
                json.load(open(os.path.join(R.PROBE, "bluey-q8-expanded-glossary", "results.json")))["cases"]
                if x["arm"] == "single"}
    gold_h = json.load(open(R.GOLD_HOUSEHOLD))
    d1 = {(r["case"]["id"], r["case"]["arm"]): r["choice"] for r in json.load(open(R.D1_RESULTS))["records"]}

    print("| variant | set | arm | settled | correct | wrong-name | wrong-keep | passed | hybrid exact | hybrid repairs | hybrid unwanted |")
    print("|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|")
    changes = collections.defaultdict(list)
    v1_labels = {}
    for vname, rules in VARIANTS.items():
        run = chain(rules)
        per = collections.defaultdict(list)
        for case in cases:
            gold = case["want"] or gold_h[case["id"]]
            c = R.Ctx(case, R.canonical_of(case, baseline))
            rule, label, reason = run(c)
            s = "household" if case["source"] != "synthetic" else "synthetic"
            final = label if label != "pass" else d1.get((case["id"], case["arm"]))
            per[(s, case["arm"])].append((gold, label, final, rule))
            key = (case["id"], case["arm"])
            if vname == "v1":
                v1_labels[key] = label
            elif label != v1_labels[key]:
                changes[vname].append((s, case["arm"], v1_labels[key], label, gold, rule))
        for s in ("household", "synthetic"):
            for arm in ("single", "two", "both"):
                rows = [x for a in (("single", "two") if arm == "both" else (arm,)) for x in per[(s, a)]]
                t = collections.Counter()
                for gold, label, final, _ in rows:
                    if label == "pass": t["pass"] += 1
                    elif label == gold: t["ok"] += 1
                    elif label == "name": t["wn"] += 1
                    else: t["wk"] += 1
                    if final == gold: t["hx"] += 1
                    if gold == "name": t["need"] += 1; t["rep"] += final == "name"
                    else: t["non"] += 1; t["unw"] += final == "name"
                print(f"| {vname} | {s} | {arm} | {len(rows)-t['pass']} | {t['ok']} | {t['wn']} | {t['wk']} | {t['pass']} | "
                      f"{t['hx']}/{len(rows)} | {t['rep']}/{t['need']} | {t['unw']}/{t['non']} |")
    print("\n## Label changes vs v1 (set, arm, v1 -> variant, gold, deciding rule)")
    for v, ch in changes.items():
        good = sum(1 for *_, new, gold, _r in [(x[0], x[1], x[2], x[3], x[4], x[5]) for x in ch] if new == gold)
        print(f"- **{v}**: {len(ch)} changed, {good} now match gold")
        for s, arm, old, new, gold, rule in ch:
            print(f"  - {s}/{arm}: {old} -> {new} (gold {gold}; {rule})")

if __name__ == "__main__":
    main()
