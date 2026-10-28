#!/usr/bin/env python3
"""Compare saved d1 glossary ablations; no model loading or network calls."""
import hashlib
import json
from pathlib import Path

from context_suite import jobs, output

HERE = Path(__file__).resolve().parent


def main():
    path = HERE / 'two-sentence.json'
    glossary = HERE / 'glossary.md'
    expected = [{(r['arm'], r['case']['id']): r for r in jobs(path, g)}
                for g in (None, glossary)]
    datasets = {}
    changes = []
    deltas = []
    for quantization in ('q8', 'q4'):
        pair = []
        for prefix, want in zip(('two-sentence-', 'two-sentence-glossary-'), expected):
            data = json.loads((HERE / f'{prefix}{quantization}/results.json').read_text())
            index = {(r['arm'], r['case']['id']): r for r in data['cases']}
            assert len(index) == len(data['cases']) == len(want) == 56
            assert index.keys() == want.keys()
            for key, row in index.items():
                for field in ('case', 'state', 'questions', 'baseline_sha256'):
                    assert row[field] == want[key][field], (quantization, key, field)
                assert row.get('glossary') == want[key].get('glossary')
            pair.append((data, index))
        assert pair[0][0]['manifest'] == pair[1][0]['manifest']
        assert pair[1][0]['glossary']['sha256'] == hashlib.sha256(glossary.read_bytes()).hexdigest()
        for key, a in pair[0][1].items():
            b = pair[1][1][key]
            assert a['temperature'] == b['temperature']
            deltas.append(b['token_count'] - a['token_count'])
            if a['choice'] != b['choice']:
                changes.append((quantization, a, b))
        datasets[quantization] = pair
    for key in expected[1]:
        a = datasets['q8'][1][1][key]
        b = datasets['q4'][1][1][key]
        assert a['input_ids'] == b['input_ids'] and a['marker_pos'] == b['marker_pos']
    lines = ['# Five-entry glossary ablation — d1 Q8 / Q4', '',
             '56 local decisions per model with the glossary, compared with the 56 saved '
             'decisions without it. Same 28 paired scenarios, marked occurrences, questions, '
             'option descriptions, gold labels, model files and calibration. The only '
             'input change is the following shared glossary and section headings in state:', '',
             '```text', 'Vocabulary glossary:', glossary.read_text().rstrip(), '',
             'Dictation:', '<original one- or two-sentence marked text>', '```', '',
             'No instructions to favor names were added. The previous options already '
             'describe the candidate name; this tests an additional shared glossary, '
             'not first exposure to those relationships. Nell, Cigs and Mater are absent '
             'from the supplied glossary and remain in the suite as unlisted controls. '
             'Gold labels and scenario categories are never supplied as model inputs.', '',
             '| Model | Scope | Glossary | Name repairs | Edits on preserve/ambiguous | Exact choices |',
             '|---|---|---|---:|---:|---:|']
    for q, pair in datasets.items():
        for scope in ('single', 'two'):
            for enabled, (data, index) in zip(('no', 'yes'), pair):
                rows = [r for r in index.values() if r['arm'] == scope]
                positives = [r for r in rows if r['case']['want'] == 'name']
                preserve = [r for r in rows if r['case']['want'] != 'name']
                repair = sum(r['choice'] == 'name' for r in positives)
                bad = sum(r['choice'] == 'name' for r in preserve)
                exact = sum(r['choice'] == r['case']['want'] for r in rows)
                lines.append(f'| {q.upper()} | {scope} | {enabled} | {repair}/{len(positives)} | '
                             f'{bad}/{len(preserve)} | {exact}/{len(rows)} |')
    lines += ['', 'Single-sentence ambiguous cases have abstention gold labels: no '
              'single-sentence case requires a name repair. KEEP and abstain both preserve '
              'text, while exact-choice scoring distinguishes them. No confidence floor.', '',
              '## Two-sentence preservation detail', '',
              '| Model | Category | Without glossary | With glossary |', '|---|---|---:|---:|']
    for q, pair in datasets.items():
        for category in ('disambiguate-keep', 'name-distractor', 'privacy-initial', 'intentional-spelling'):
            cells = []
            for _, index in pair:
                rows = [r for r in index.values() if r['arm'] == 'two' and r['case']['category'] == category]
                cells.append(f"{sum(r['choice']=='name' for r in rows)}/{len(rows)}")
            lines.append(f'| {q.upper()} | {category} | ' + ' | '.join(cells) + ' |')
    lines += ['', 'Cells count unwanted edits, so smaller is better.', '']
    for q, pair in datasets.items():
        selected = [(a, b) for quant, a, b in changes if quant == q]
        gain = sum(b['choice'] == b['case']['want'] and a['choice'] != a['case']['want'] for a,b in selected)
        loss = sum(a['choice'] == a['case']['want'] and b['choice'] != b['case']['want'] for a,b in selected)
        sizes = [r['token_count'] for r in pair[1][1].values()]
        lines.append(f'{q.upper()}: {len(selected)}/56 choices change; {gain} become exact and '
                     f'{loss} lose exactness. Complete native inputs with glossary: {min(sizes)}–{max(sizes)} tokens.')
    lines += ['', f'The five-entry glossary plus headings adds {min(deltas)} '
              'tokens to the actual native inputs. Zero truncation in both runs. '
              'No API calls, ASR replay, model tuning or live dictation changes. '
              'Synthetic paired cases are correlated and are not production calibration. '
              'The glossary improves two-sentence repair recall, but the current referent '
              'prompt still fails the preservation contract; single-sentence unsupported '
              'edits increase. This does not establish a safe automatic-correction policy.', '',
              '## Every changed choice', '',
              '| Model | Scope / scenario | Expected | Choice without → with | P(name) without → with | Dictation | Output without | Output with |',
              '|---|---|---|---|---:|---|---|---|']
    for q,a,b in changes:
        c=a['case']
        lines.append(f"| {q.upper()} | {a['arm']} / {c['id']} | {c['want']} | {a['choice']} → {b['choice']} | "
                     f"{a['probabilities']['name']:.3f} → {b['probabilities']['name']:.3f} | "
                     f"{c['context']} | {output(a)} | {output(b)} |")
    lines += ['', 'Full glossary-run decisions: [Q4](two-sentence-glossary-q4/report.md), '
              '[Q8](two-sentence-glossary-q8/report.md). '
              '[Original paired-context report](two-sentence-report.md).', '',
              'Regenerate this saved comparison:', '', '```sh',
              'nix develop .#pair-probe -c python3 tools/pair-router-probe/glossary-report.py', '```', '',
              'Re-run Q4 locally:', '', '```sh',
              'nix develop .#pair-probe -c python3 tools/pair-router-probe/d1.py --quantization q4 '
              '--cases tools/pair-router-probe/two-sentence.json --glossary tools/pair-router-probe/glossary.md', '```']
    report = HERE / 'glossary-report.md'
    report.write_text('\n'.join(lines) + '\n')
    print('\n'.join(lines[:60]))
    print(report)


if __name__ == '__main__':
    main()
