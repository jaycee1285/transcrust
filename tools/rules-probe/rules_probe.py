#!/usr/bin/env python3
"""Deterministic rules-first ablation for transcrust dictionary decisions.

Implements steps 1-5 of the homophone chain in design-vocab-gate.md section 5
(inflection, +-2-word grammar slots, bare singular count noun, animacy, fixed
phrases / franchise co-occurrence). Steps 6 (speaker prior) and 7 (model) are
NOT implemented: whatever the rules do not settle is returned as "pass" (residue).

Every rule returns None (no opinion) or ("name"|"keep", reason_sentence).
No rule looks at a case id, a fixture sentence, or a gold label.

Run (python3 std-only, run from anywhere):
    python3 -I ~/repos/transcrust/tools/rules-probe/rules_probe.py
"""
import json, os, re, sys, collections

HERE = os.path.dirname(os.path.abspath(__file__))
PROBE = os.path.join(HERE, "..", "pair-router-probe")
CASES = os.path.join(PROBE, "cross-encoder-q8", "cases.json")
D1_RESULTS = os.path.join(PROBE, "d1-native-context", "results.json")
ULTRA_Q4 = os.path.join(PROBE, "ultra-int4-2026-11-01", "q4", "full.json")
GOLD_HOUSEHOLD = os.path.join(HERE, "gold-household.json")

# ---------------------------------------------------------------- word lists
# Each list is general English or a listed franchise phrase list. They were
# written before the first run. They are deliberately small; a word missing
# here simply means the rule stays silent (the case passes to the residue).

# Words that can head a verb phrase as a bare base form (verb-capable). Used by
# the verb-slot rule so that "to <non-verb>" (a preposition: "talked to Nell")
# is NOT read as an infinitive marker.
VERB_CAPABLE = set("""
gamble flow run walk talk play work call help turn move live look see make take
give find think know want need like love hate try ask use feel leave keep let
begin seem show hear stand drink eat sleep write read sing dance jump swim fly
drive ride cook bake draw paint build break fix cut hit push pull throw catch
carry drop fall rise climb wait stop start finish open close watch wash clean
laugh smile cry shout whisper bark race rush drift glide float pour stream
vote bet wager shuffle deal win lose pay buy sell
""".split())

# Common nouns that need a determiner in the singular (countable). Used by
# the bare-singular-count-noun rule. Mass nouns (flow, chili, bingo, lightning,
# water) are deliberately absent: "Lightning lit the sky" is grammatical.
COUNT_NOUNS = set("""
gamble bandit dog cat horse bird car truck boat plane train bike book table chair
house door window phone tool toy ball box bag hat shoe coat cup plate bowl knife
robber thief pirate cowboy sheriff soldier teacher doctor farmer driver player
river hill tree flower stone rock stick kite
""".split())

# Determiners / quantifiers / possessives that license a singular count noun.
DETERMINERS = set("""
a an the this that these those some any each every another no one two three
many few several both either neither my your his her its our their
""".split())

# Articles and demonstratives only (no possessives): a following word is a
# common-noun slot. Possessives are excluded because "my dog Gambol" is a name.
ARTICLES = set("a an the this that these those some any each every another".split())

# Words after which an infinitive/bare verb follows.
VERB_PREV = set("""
to will would can could should may might must shall
won't can't don't didn't doesn't wouldn't couldn't shouldn't wont cant dont
didnt doesnt wouldnt couldnt shouldnt gonna
""".split())
# Adverbs that may sit between the verb cue and the verb ("to quickly gamble").
ADVERBS = set("not never always just also really quickly slowly often still even only".split())
# Subject pronouns directly before a finite verb ("I gamble").
SUBJECT_PRONOUNS = set("i we you they".split())
# Particles after a verb ("flow into", "gamble away").
VERB_NEXT = set("on away into".split())

# Name-slot cues directly before the span ("walk Flo", "called Gambol", "Hey X").
NAME_PREV = set("""
walk call called named meet hey hi hello dear mr mrs ms miss dr uncle aunt
""".split())
# Relational nouns that take an appositive name ("my dog X"). The rule only
# fires if the noun also appears in the glossary description of the canonical.
RELATIONAL = set("""
dog cat puppy pet son daughter brother sister wife husband mom dad mother father
friend colleague neighbor teacher cousin boss
""".split())

# Verbs that need a living subject (past and 3sg forms).
ANIMATE_VERBS = set("""
said says barked barks won wins ate eats likes liked loves loved wants wanted
thinks thought asked asks told tells smiled laughed cried shouted whispered
knew knows felt feels decided decides wondered promised
""".split())
# Inanimate common nouns that cannot be the subject of an animate verb.
INANIMATE_NOUNS = set("""
flow lightning gamble chili bingo six water storm rain weather wind river label
meter sky thunder
""".split())

# Franchise co-occurrence. A franchise's name entries are named when a
# distinctive franchise marker appears elsewhere in the text. Markers are
# phrases that cannot be ordinary words (Bluey's cast names are ordinary words
# and are therefore NOT markers).
FRANCHISES = {
    "cars": {
        "description_keys": ["cars movie"],
        "markers": ["lightning mcqueen", "mcqueen", "mater", "v8 cafe",
                    "radiator springs", "piston cup", "doc hudson",
                    "cars characters", "cars character", "cars movie", "cars film"],
    },
    "bluey": {
        "description_keys": ["bluey"],
        "markers": ["bluey", "heeler"],
    },
}

# ------------------------------------------------------------- case plumbing
def glossary_description(state, canonical):
    for line in state.split("\n"):
        m = re.match(r"\|\s*(.+?)\s*\|\s*(.+?)\s*\|$", line)
        if m and m.group(1).lower() == canonical.lower() and m.group(1) != "Term":
            return m.group(2)
    return ""

def canonical_of(case, baseline):
    if case["candidate"]:
        return case["candidate"]["canonical"]
    return baseline[case["id"]]["case"]["canonical"]

class Ctx:
    """Parsed view of one marked occurrence."""
    def __init__(self, case, canonical):
        text = case["state"].split("Dictation:\n", 1)[1]
        self.full_text = text
        a, b = text.index("«"), text.index("»")
        self.heard = text[a + 1:b]
        self.hl = self.heard.lower()
        self.canonical = canonical
        self.desc = glossary_description(case["state"], canonical).lower()
        before, after = text[:a], text[b + 1:]
        # Current sentence only (a context sentence before it is not "nearby").
        sent_before = re.split(r"[.!?]\s+", before)[-1]
        sent_after = re.split(r"[.!?]", after)[0]
        # Current clause only: cut at comma/colon/semicolon.
        self.prev = re.findall(r"[A-Za-z0-9'’]+", re.split(r"[,;:]", sent_before)[-1].lower())
        self.next = re.findall(r"[A-Za-z0-9'’]+", re.split(r"[,;:]", sent_after)[0].lower())
        self.after_raw = after
        self.clause_start = len(self.prev) == 0
        self.text_without_span = (before + " \u0000 " + after).lower()

# ------------------------------------------------------------------- rules
def rule1_inflection(c):
    h = c.hl
    if " " in h:
        return None
    # Bug fix 2026-11-18 (found by fixtures-extra): fire only when stripping the ending leaves a
    # verb, so lexicalised -ing nouns (lightning, morning, king) never count as inflections.
    for suf in ("ing", "ed", "s"):
        if h.endswith(suf) and len(h) - len(suf) >= 3 and h != c.canonical.lower():
            stem = h[: -len(suf)]
            stems = {stem, stem + "e", stem[:-1] if len(stem) > 3 and stem[-1] == stem[-2] else stem}
            if suf == "ed":
                stems.add(h[:-1])  # gambled -> gamble
            if stems & VERB_CAPABLE:
                return ("keep", f'kept "{c.heard}": the ending "-{suf}" on the verb "{sorted(stems & VERB_CAPABLE)[0]}" marks an inflected common word, which cannot be a name')
    return None

def _prev_verb_cue(c):
    p = c.prev
    if p and p[-1] in VERB_PREV:
        return p[-1]
    if len(p) >= 2 and p[-1] in ADVERBS and p[-2] in VERB_PREV:
        return p[-2]
    return None

def rule2_slots(c):
    """+-2-word grammar slots. Verb/noun cues -> keep; name cues -> name.
    If both a keep cue and a name cue are present the rule abstains."""
    keep, name = [], []
    verb_capable = c.hl in VERB_CAPABLE
    cue = _prev_verb_cue(c)
    if cue and verb_capable:
        keep.append(f'verb slot after "{cue}"')
    if verb_capable and c.prev and c.prev[-1] in SUBJECT_PRONOUNS:
        keep.append(f'verb slot after subject pronoun "{c.prev[-1]}"')
    if verb_capable and c.next and c.next[0] in VERB_NEXT:
        keep.append(f'verb slot before "{c.next[0]}"')
    window = c.prev[-2:]
    for w in window:
        if w in ARTICLES:
            keep.append(f'common-noun slot after "{w}"')
            break
    if c.prev and c.prev[-1] in NAME_PREV:
        name.append(f'name slot after "{c.prev[-1]}"')
    if re.match(r"^['’]s\b", c.after_raw):
        name.append("possessive 's follows")
    if c.clause_start and re.match(r"^\s*(,|…|\.\.\.|!)", c.after_raw):
        name.append("clause-initial call followed by a pause mark (vocative)")
    if c.prev and c.prev[-1] in RELATIONAL and c.prev[-1] in re.findall(r"[a-z]+", c.desc):
        name.append(f'appositive after "{c.prev[-1]}", which the glossary gives for {c.canonical}')
    if keep and name:
        return None
    if keep:
        return ("keep", f'kept "{c.heard}": ' + "; ".join(keep))
    if name:
        return ("name", f'named "{c.canonical}": ' + "; ".join(name))
    return None

def rule3_bare_count_noun(c):
    if c.hl not in COUNT_NOUNS:
        return None
    if any(w in DETERMINERS or w.endswith(("'s", "’s")) for w in c.prev[-3:]):
        return None
    return ("name", f'named "{c.canonical}": "{c.heard}" is a singular count noun with no determiner, which a common noun cannot be here')

def rule4_animacy(c):
    if c.hl not in INANIMATE_NOUNS:
        return None
    n = c.next
    if n and n[0] in ADVERBS:
        n = n[1:]
    if n and n[0] in ANIMATE_VERBS:
        return ("name", f'named "{c.canonical}": "{n[0]}" needs a living subject and "{c.heard}" is not one')
    return None

def rule5_fixed_phrases(c):
    for fname, f in FRANCHISES.items():
        if not any(k in c.desc for k in f["description_keys"]):
            continue
        for m in f["markers"]:
            if re.search(r"\b" + re.escape(m) + r"\b", c.text_without_span):
                return ("name", f'named "{c.canonical}": {fname} marker phrase "{m}" appears in the same text')
    return None

RULES = [("1 inflection", rule1_inflection),
         ("2 grammar slots", rule2_slots),
         ("3 bare count noun", rule3_bare_count_noun),
         ("4 animacy", rule4_animacy),
         ("5 fixed phrases", rule5_fixed_phrases)]

def run_chain(c):
    for name, fn in RULES:
        r = fn(c)
        if r:
            return name, r[0], r[1]
    return None, "pass", "no rule was confident; passed to the residue"

# --------------------------------------------------------------- evaluation
def main():
    cases = json.load(open(CASES))
    baseline = {x["case"]["id"]: x for x in
                json.load(open(os.path.join(PROBE, "bluey-q8-expanded-glossary", "results.json")))["cases"]
                if x["arm"] == "single"}
    gold_h = json.load(open(GOLD_HOUSEHOLD))
    d1 = {(r["case"]["id"], r["case"]["arm"]): r["choice"] for r in json.load(open(D1_RESULTS))["records"]}
    ultra = {(r["case"]["id"], r["case"]["arm"]): r["choice"] for r in json.load(open(ULTRA_Q4))["rows"]}

    rows = []
    for case in cases:
        gold = case["want"] or gold_h[case["id"]]
        ctx = Ctx(case, canonical_of(case, baseline))
        rule, label, reason = run_chain(ctx)
        rows.append(dict(id=case["id"], arm=case["arm"], set="household" if case["source"] != "synthetic" else "synthetic",
                         gold=gold, rule=rule, label=label, reason=reason, heard=ctx.heard,
                         d1=d1.get((case["id"], case["arm"])), ultra=ultra.get((case["id"], case["arm"]))))

    out = []
    def P(s=""): out.append(s)

    def tally(rs):
        t = collections.Counter()
        for r in rs:
            if r["label"] == "pass":
                t["pass"] += 1
            elif r["label"] == r["gold"]:
                t["correct"] += 1
            elif r["label"] == "name":
                t["wrong_name"] += 1
            else:
                t["wrong_keep"] += 1
        t["n"] = len(rs)
        t["settled"] = t["n"] - t["pass"]
        return t

    P("## Settled / correct / wrong by set and arm")
    P("| set | arm | n | settled | correct | wrong-name | wrong-keep | passed |")
    P("|---|---|---:|---:|---:|---:|---:|---:|")
    for s in ("household", "synthetic"):
        for arm in ("single", "two", "both"):
            rs = [r for r in rows if r["set"] == s and (arm == "both" or r["arm"] == arm)]
            t = tally(rs)
            P(f"| {s} | {arm} | {t['n']} | {t['settled']} | {t['correct']} | {t['wrong_name']} | {t['wrong_keep']} | {t['pass']} |")
    P()
    P("## Per rule (both sets, both arms)")
    P("| rule | fires | correct | wrong-name | wrong-keep |")
    P("|---|---:|---:|---:|---:|")
    for name, _ in RULES:
        t = tally([r for r in rows if r["rule"] == name]) if any(r["rule"] == name for r in rows) else collections.Counter()
        P(f"| {name} | {t['n']} | {t['correct']} | {t['wrong_name']} | {t['wrong_keep']} |")
    P()
    P("## Per rule by set")
    P("| set | rule | fires | correct | wrong-name | wrong-keep |")
    P("|---|---|---:|---:|---:|---:|")
    for s in ("household", "synthetic"):
        for name, _ in RULES:
            rs = [r for r in rows if r["set"] == s and r["rule"] == name]
            t = tally(rs)
            P(f"| {s} | {name} | {t['n']} | {t['correct']} | {t['wrong_name']} | {t['wrong_keep']} |")
    P()
    P("## Gold-abstain rows (rules should pass; deciding is an overconfident error)")
    ga = [r for r in rows if r["gold"] == "abstain"]
    P(f"{len(ga)} rows; rules decided {sum(1 for r in ga if r['label']!='pass')}.")
    P()
    P("## Residue (ids as id/arm)")
    for s in ("household", "synthetic"):
        res = [f"{r['id']}/{r['arm']}" for r in rows if r["set"] == s and r["label"] == "pass"]
        P(f"{s} ({len(res)}): " + ", ".join(res))
    P()
    P("## Wrong decisions (ids, rule, label vs gold)")
    for r in rows:
        if r["label"] not in ("pass", r["gold"]):
            P(f"- {r['id']}/{r['arm']}: rule {r['rule']}: said {r['label']}, gold {r['gold']}")
    P()
    P("## Known cases")
    for r in rows:
        if r["id"] in ("parakeet-1", "phonon-1", "phonon-2", "parakeet-0", "phonon-0"):
            P(f"- {r['id']}/{r['arm']} gold={r['gold']} -> {r['label']} ({r['rule']}): {r['reason']}")
    P()
    P("## Model comparison (existing saved outputs)")
    def score(rs, pick):
        n = len(rs); rep = rep_t = unw = absn = acc = 0
        for r in rs:
            f = pick(r)
            rep_t += r["gold"] == "name"
            rep += r["gold"] == "name" and f == "name"
            unw += r["gold"] != "name" and f == "name"
            absn += f in ("abstain", "pass")
            acc += f == r["gold"]
        return f"n={n} repairs {rep}/{rep_t}, unwanted edits {unw}/{n-rep_t}, abstain-or-pass {absn}, exact-label {acc}"
    def rules_only(r): return r["label"]
    def hybrid(key): return lambda r: r["label"] if r["label"] != "pass" else r[key]
    def model(key): return lambda r: r[key]
    for s in ("household", "synthetic", "all"):
        rs = [r for r in rows if s == "all" or r["set"] == s]
        P(f"- {s}, rules alone (pass = no edit): {score(rs, rules_only)}")
        P(f"- {s}, D1 alone: {score(rs, model('d1'))}")
        P(f"- {s}, rules then D1: {score(rs, hybrid('d1'))}")
    hu = [r for r in rows if r["set"] == "household"]
    P(f"- household, UltraBERT INT4 full-glossary alone: {score(hu, model('ultra'))}")
    P(f"- household, rules then UltraBERT INT4: {score(hu, hybrid('ultra'))}")
    P("- synthetic UltraBERT per-case outputs: not saved in earlier runs; comparison pending.")
    P()
    P("## Full per-case log (id/arm | gold | rule | label | reason)")
    for r in rows:
        P(f"{r['id']}/{r['arm']} | {r['gold']} | {r['rule']} | {r['label']} | {r['reason']}")
    text = "\n".join(out)
    if "--no-log" in sys.argv:
        text = text.split("## Full per-case log")[0]
    print(text)

if __name__ == "__main__":
    main()
