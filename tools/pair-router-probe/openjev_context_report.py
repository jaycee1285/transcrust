#!/usr/bin/env python3
"""Read and validate the saved OpenJev/d1 glossary comparison, without inference."""
import json
from pathlib import Path
import statistics

from context_suite import HERE, jobs, output


def report():
    expected = [{(r['arm'],r['case']['id']): r for r in jobs(HERE / 'two-sentence.json', glossary)}
                for glossary in (None, HERE / 'glossary.md')]
    datasets = {}
    for label, suffix in [('OpenJev 151M fp16','openjev'), ('d1 Q4','q4'), ('d1 Q8','q8')]:
        pair = []
        for prefix, want in zip(('two-sentence-', 'two-sentence-glossary-'), expected):
            data = json.loads((HERE / (prefix+suffix) / 'results.json').read_text())
            index = {(r['arm'],r['case']['id']): r for r in data['cases']}
            assert len(index) == len(data['cases']) == len(want) == 56
            assert index.keys() == want.keys()
            for key,row in index.items():
                for field in ('case','state','questions','baseline_sha256'):
                    assert row[field] == want[key][field], (label,key,field)
                assert row.get('glossary') == want[key].get('glossary')
            pair.append((data,index))
        if suffix == 'openjev':
            for field in ('model_sha256','tokenizer_sha256','calibrator_sha256','temperature'):
                assert pair[0][0][field] == pair[1][0][field]
        datasets[label] = pair
    lines = ['# OpenJev versus d1 — shared glossary context', '',
             '112 new local OpenJev/Verdict 151M fp16 inferences: 56 without glossary and '
             '56 with the exact five-entry glossary. Same states, marked occurrences, '
             'questions, option meanings and pre-existing gold labels as d1. '
             'OpenJev uses its native LABEL/SEP format and shipped three-choice temperature '
             '5.0069; d1 uses its own native mask format/calibration. '
             'No API calls, audio replay, training or live changes.', '',
             'Results count raw name selections, mapped to proposed replacements by the '
             'harness. They do not demonstrate an edit policy. Initial and quoted-spelling '
             'stress cases are shown separately; their expected edit behavior is not the '
             'same as referent identification. These are small, correlated diagnostic cases.', '',
             '| Model | Scope | Glossary | Repairs | Name selections on non-repairs | Exact choices |',
             '|---|---|---|---:|---:|---:|']
    practical = ['## Two-sentence comparison excluding initial and quoted-spelling stress cases', '',
                 '| Model | Glossary | Repairs | Name selections on ordinary/other-person controls |',
                 '|---|---|---:|---:|']
    for label, pair in datasets.items():
        for enabled, (_, index) in zip(('no', 'yes'), pair):
            rows = [r for r in index.values() if r['arm'] == 'two'
                    and r['case']['category'] not in ('privacy-initial', 'intentional-spelling')]
            pos = [r for r in rows if r['case']['want'] == 'name']
            neg = [r for r in rows if r['case']['want'] != 'name']
            practical.append(f"| {label} | {enabled} | {sum(r['choice']=='name' for r in pos)}/{len(pos)} | "
                             f"{sum(r['choice']=='name' for r in neg)}/{len(neg)} |")
    practical += ['', 'The remaining controls include deliberately difficult ambiguity and different-person '
                  'cases; their error count is not an everyday dictation error rate.', '', '## Full diagnostic counts', '']
    position = lines.index('| Model | Scope | Glossary | Repairs | Name selections on non-repairs | Exact choices |')
    lines[position:position] = practical
    for label,pair in datasets.items():
        for scope in ('single','two'):
            for enabled,(_,index) in zip(('no','yes'),pair):
                rows=[r for r in index.values() if r['arm']==scope]
                pos=[r for r in rows if r['case']['want']=='name']
                neg=[r for r in rows if r['case']['want']!='name']
                lines.append(f"| {label} | {scope} | {enabled} | {sum(r['choice']=='name' for r in pos)}/{len(pos)} | "
                             f"{sum(r['choice']=='name' for r in neg)}/{len(neg)} | "
                             f"{sum(r['choice']==r['case']['want'] for r in rows)}/{len(rows)} |")
    lines += ['', 'Single-sentence cases require no repairs: ambiguous targets have abstention gold. '
              'KEEP and abstention both preserve text. No confidence floor.', '',
              '## Two-sentence results by category', '',
              '| Model | Category | Without glossary: names selected | With glossary: names selected |',
              '|---|---|---:|---:|']
    for label,pair in datasets.items():
        for category in dict.fromkeys(r['case']['category'] for r in expected[0].values()):
            counts=[]
            for _,index in pair:
                rows=[r for r in index.values() if r['arm']=='two' and r['case']['category']==category]
                counts.append(f"{sum(r['choice']=='name' for r in rows)}/{len(rows)}")
            lines.append(f'| {label} | {category} | '+' | '.join(counts)+' |')
    lines += ['', '## Resources', '',
              '| Model | Glossary | Two-sentence median inference (ms) | End process RSS (MiB) | Input tokens |',
              '|---|---|---:|---:|---|']
    for label,pair in datasets.items():
        for enabled,(data,_) in zip(('no','yes'),pair):
            sizes=[r['token_count'] for r in data['cases']]
            elapsed=statistics.median(r['latency_ms'] for r in data['cases'] if r['arm']=='two')
            lines.append(f"| {label} | {enabled} | {elapsed:.0f} | {data['rss_end_mib']:.0f} | {min(sizes)}–{max(sizes)} |")
    data=datasets['OpenJev 151M fp16'][1][0]
    lines += ['', f"OpenJev graph: {data['model_bytes']/1e6:.1f} MB; load {data['load_secs']:.2f}s; "
              f"process peak RSS {data['peak_rss_mib']:.0f} MiB. "
              'Both OpenJev arms share one loaded session, run without glossary first, '
              'with one warm-up per arm. d1 timings are from earlier runs under different '
              'laptop load. These are not isolated speed comparisons or mobile measurements. '
              'RAM includes Python/ORT and allocator caches. Inputs are rejected above the '
              'existing 512-token export contract, with zero truncation.', '',
              '## Every OpenJev choice changed by the glossary', '',
              '| Scope / scenario | Expected | Without → with | P(name) without → with | Dictation | Output without | Output with |',
              '|---|---|---|---:|---|---|---|']
    before=datasets['OpenJev 151M fp16'][0][1]
    after=datasets['OpenJev 151M fp16'][1][1]
    for key,a in before.items():
        b=after[key]
        if a['choice']==b['choice']:
            continue
        c=a['case']
        lines.append(f"| {a['arm']} / {c['id']} | {c['want']} | {a['choice']} → {b['choice']} | "
                     f"{a['probabilities']['name']:.3f} → {b['probabilities']['name']:.3f} | "
                     f"{c['context']} | {output(a)} | {output(b)} |")
    lines += ['', 'Full OpenJev inputs and outputs: [without glossary](two-sentence-openjev/report.md), '
              '[with glossary](two-sentence-glossary-openjev/report.md).', '',
              'Re-run both OpenJev arms:', '', '```sh',
              'nix develop .#pair-probe -c python3 tools/pair-router-probe/openjev-context.py', '```', '',
              'Read/regenerate this saved report without model loading:', '', '```sh',
              'nix develop .#pair-probe -c python3 tools/pair-router-probe/openjev-context.py --report-only', '```']
    path=HERE/'openjev-glossary-report.md'
    path.write_text('\n'.join(lines)+'\n')
    print('\n'.join(lines[:72]))
    print(path)


if __name__=='__main__':
    report()
