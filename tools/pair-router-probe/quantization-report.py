#!/usr/bin/env python3
"""Compare saved d1 Q4/Q8 decisions; no inference or network calls."""
import hashlib
import json
from pathlib import Path
import statistics

HERE = Path(__file__).resolve().parent


def main():
    data = [json.loads((HERE / f'd1-{q}/results.json').read_text()) for q in ['q8', 'q4']]
    q8, q4 = data
    assert q8['manifest']['repos'] == q4['manifest']['repos']
    for name in ['prompt.py', 'config.json', 'tokenizer.json', 'tokenizer_config.json']:
        assert q8['manifest']['files'][name]['sha256'] == q4['manifest']['files'][name]['sha256']
    indexed = [{(r['arm'], r['case']['id']): r for r in d['cases']} for d in data]
    assert len(indexed[0]) == len(indexed[1]) == 157
    assert indexed[0].keys() == indexed[1].keys()
    changes = []
    for key, a in indexed[0].items():
        b = indexed[1][key]
        for field in ['baseline_sha256', 'case', 'state', 'questions', 'input_ids', 'marker_pos', 'temperature']:
            assert a[field] == b[field], (key, field)
        baseline = HERE / a['arm'] / 'results.json'
        assert hashlib.sha256(baseline.read_bytes()).hexdigest() == a['baseline_sha256']
        if a['choice'] != b['choice']:
            changes.append((a, b))
    for d in data:
        initials = [r for r in d['cases'] if r['arm'].startswith('suite-verdict') and r['case']['id'] in ['son-keep-1', 'son-keep-2']]
        assert len(initials) == 4 and all(r['choice'] != 'name' for r in initials)
    lines = ['# d1 Q4 versus Q8 — text-only, 2026-10-08', '',
             'Q4 retains comparable quality on this small suite, with mixed individual changes. '
             'It gains two main-suite repairs and avoids one ambiguous edit, while adding an '
             'ambiguous flow → Flo edit in Cars and losing one original-probe repair. '
             'Both privacy-initial controls remain unchanged in both prompt arms. '
             'The smaller footprint makes Q4 worth carrying forward as the local candidate; '
             'these untuned results do not establish safe automatic correction.', '',
             '157 paired local decisions. Pinned source revisions, tokenizer, native prompt, '
             'temperature, complete input IDs and saved baseline hashes match. '
             'Only the quantized graph/weights differ. No API requests or media encoders.', '',
             '| Arm | Quantization | Repairs | Edits on keep | Edits on ambiguous | Exact decisions |',
             '|---|---|---:|---:|---:|---:|']
    for arm in dict.fromkeys(r['arm'] for r in q8['cases']):
        for d in data:
            rows = [r for r in d['cases'] if r['arm'] == arm]
            counts = []
            for want in ['name', 'keep', 'abstain']:
                group = [r for r in rows if r['case']['want'] == want]
                n = sum(r['choice'] == 'name' and r['case']['heard'] != r['case']['canonical'] for r in group)
                counts.append(f'{n}/{len(group)}')
            exact = sum(r['choice'] == r['case']['want'] for r in rows)
            lines.append(f"| {arm} | {d['quantization'].upper()} | " + ' | '.join(counts) + f' | {exact}/{len(rows)} |')
    improved = sum(b['choice'] == b['case']['want'] and a['choice'] != a['case']['want'] for a, b in changes)
    worsened = sum(a['choice'] == a['case']['want'] and b['choice'] != b['case']['want'] for a, b in changes)
    probability_delta = [abs(a['probabilities'][c] - indexed[1][key]['probabilities'][c])
                         for key, a in indexed[0].items() for c in ['keep', 'name', 'abstain']]
    lines += ['', f'{len(changes)}/157 choices changed: {improved} became exact, {worsened} lost exactness, '
              f'{len(changes)-improved-worsened} changed between incorrect choices. '
              f'Mean absolute option probability difference {statistics.mean(probability_delta):.4f}; '
              f'maximum {max(probability_delta):.4f}.', '',
              'KEEP and abstain both leave the sentence unchanged. Exact decisions distinguish them. '
              'Counts above apply no confidence floor. The editing-prompt arm repeats the main 60 '
              'sentences with a different question; these are not 157 independent utterances.', '',
              '| Measurement | Q8 | Q4 |', '|---|---:|---:|']
    for title, values in [
        ('Text graph + weights (decimal MB)', [sum(v['bytes'] for k,v in d['manifest']['files'].items() if k.startswith('onnx/'))/1e6 for d in data]),
        ('Process RSS at end (MiB)', [d['rss_end_mib'] for d in data]),
        ('Process peak RSS (MiB)', [d['peak_rss_mib'] for d in data]),
        ('Median inference (ms)', [statistics.median(r['latency_ms'] for r in d['cases']) for d in data]),
        ('Load (s)', [d['load_secs'] for d in data])]:
        lines.append(f'| {title} | {values[0]:.2f} | {values[1]:.2f} |')
    lines += ['', 'RAM includes Python, tokenizer and ONNX Runtime. Timings are separate laptop '
              'runs under varying load, not an isolated speed benchmark or a mobile measurement.', '',
              '## Every changed choice', '',
              '| Arm / case | Expected | Q8 → Q4 | P(name), Q8 → Q4 | Original | Q8 output | Q4 output |',
              '|---|---|---|---:|---|---|---|']
    for a, b in changes:
        case = a['case']
        original = case['context'].replace('«', '').replace('»', '')
        def output(row):
            return case['context'].replace('«'+case['heard']+'»', case['canonical']) if row['choice']=='name' else original
        lines.append(f"| {a['arm']} / {case['id']} | {case['want']} | {a['choice']} → {b['choice']} | "
                     f"{a['probabilities']['name']:.3f} → {b['probabilities']['name']:.3f} | {original} | {output(a)} | {output(b)} |")
    lines += ['', 'Re-run Q4 locally:', '', '```sh',
              'nix develop .#pair-probe -c python3 tools/pair-router-probe/d1.py --quantization q4',
              '```', '', 'Regenerate this paired report from saved results:', '', '```sh',
              'nix develop .#pair-probe -c python3 tools/pair-router-probe/quantization-report.py', '```']
    path = HERE / 'd1-q4/SUMMARY.md'
    path.write_text('\n'.join(lines)+'\n')
    print('\n'.join(lines[:35]))
    print(path)


if __name__ == '__main__':
    main()
