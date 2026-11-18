"""Summarise Pocket result JSONs next to the frozen Ultra INT4 baseline (same rows, same labels).
usage: report.py OUT_DIR   (reads stripped.json win3.json win3frame.json frame.json full.json + ultra baseline)"""
import json, sys
d = sys.argv[1]
ULTRA = json.load(open('tools/pair-router-probe/vela3-onnx-2026-11-01/ultra-baseline.json'))['rows']
VELA = json.load(open('tools/pair-router-probe/vela3-onnx-2026-11-01/fast/expanded.json'))['rows']
def synth(rows):
    out = {}
    for r in rows:
        c = r['case']
        if c['source'] != 'synthetic': continue
        k = ('diag' if c['split'] == 'diagnostic' else 'ordinary', c['arm'])
        o = out.setdefault(k, dict(n=0, rep=0, rep_t=0, unw=0, pres_t=0, abst=0))
        o['n'] += 1; w = c['want'] == 'name'
        o['rep_t'] += w; o['pres_t'] += not w
        o['rep'] += w and r['choice'] == 'name'; o['unw'] += (not w) and r['choice'] == 'name'; o['abst'] += r['choice'] == 'abstain'
    return out
def line(name, rows):
    s = synth(rows); t = dict(rep=0, rep_t=0, unw=0, pres_t=0, abst=0, n=0)
    cells = []
    for arm in ('single', 'two'):
        o = s[('ordinary', arm)]; cells.append(f"{o['rep']}/{o['rep_t']} | {o['unw']}/{o['pres_t']} | {o['abst']}")
        for k in t: t[k] += o[k]
    both = f"{t['rep']}/{t['rep_t']} | {t['unw']}/{t['pres_t']} | {t['abst']}"
    dg = [s[('diag', a)] for a in ('single', 'two')]
    diag = f"{dg[0]['unw']+dg[0]['rep']}/{dg[0]['n']}, {dg[1]['unw']+dg[1]['rep']}/{dg[1]['n']}"
    print(f"| {name} | {cells[0]} | {cells[1]} | {both} | {diag} |")
print("| Model / input | single: repairs \\| unwanted \\| abstain | two: repairs \\| unwanted \\| abstain | both: repairs \\| unwanted \\| abstain | diag changed single, two |")
print("|---|---|---|---|---|")
line('Ultra INT4 stripped (frozen)', ULTRA); line('Vela fast stripped (frozen)', VELA)
files = [('Pocket stripped, native sentence/context', 'stripped.json'), ('Pocket local window +-3 words, generic question', 'win3.json'),
         ('Pocket window +-3, name-or-word question', 'win3frame.json'), ('Pocket full sentence/context, name-or-word question', 'frame.json')]
for n, f in files:
    try: line(n, json.load(open(f'{d}/{f}'))['rows'])
    except FileNotFoundError: pass
print()
print("Phone choices (heard | arm | Pocket (window3 / window3+frame) | Ultra | Vela):")
P = json.load(open(f'{d}/stripped.json'))['rows']; W = json.load(open(f'{d}/win3.json'))['rows']; F = json.load(open(f'{d}/win3frame.json'))['rows']
for i in range(32):
    c = P[i]['case']; print(f"{c['source']:8} {c['arm']:6} {c['candidate']['heard']:11} -> {c['candidate']['canonical']:10} pocket {P[i]['choice']:7} win3 {W[i]['choice']:7} win3frame {F[i]['choice']:7} ultra {ULTRA[i]['choice']:7} vela {VELA[i]['choice']}")
print()
bad = [(r['case']['id'], r['case']['arm'], r['case']['want'], r['choice'], ULTRA[i]['choice']) for i, r in enumerate(P) if r['case']['source'] == 'synthetic' and r['case']['split'] != 'diagnostic' and r['case']['want'] != 'name' and r['choice'] == 'name']
print("Pocket unwanted edits (id, arm, want, pocket, ultra):"); [print(' ', b) for b in bad]
miss = [(r['case']['id'], r['case']['arm'], r['choice'], ULTRA[i]['choice']) for i, r in enumerate(P) if r['case']['source'] == 'synthetic' and r['case']['want'] == 'name' and r['choice'] != 'name' and r['case']['split'] != 'diagnostic']
print("Pocket missed repairs (id, arm, pocket, ultra):"); [print(' ', b) for b in miss]
