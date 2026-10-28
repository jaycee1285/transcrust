#!/usr/bin/env python3
"""Validate and compare saved Q8 original/Bluey-expanded glossary runs."""
import json
from pathlib import Path
import statistics

from context_suite import HERE, jobs, output


def main():
    root = HERE
    data = []
    indices = []
    for directory, glossary in [('bluey-q8-original-glossary','glossary.md'),
                               ('bluey-q8-expanded-glossary','glossary-bluey.md')]:
        d = json.loads((root / directory / 'results.json').read_text())
        expected = {(r['arm'],r['case']['id']):r for r in jobs(root/'bluey-expanded-suite.json',root/glossary)}
        index = {(r['arm'],r['case']['id']):r for r in d['cases']}
        assert len(index)==len(d['cases'])==len(expected)==100
        assert index.keys()==expected.keys()
        for key,row in index.items():
            for field in ['case','state','questions','baseline_sha256','glossary']:
                assert row[field]==expected[key][field], (key,field)
        data.append(d)
        indices.append(index)
    assert data[0]['manifest']==data[1]['manifest']
    for key,a in indices[0].items():
        b=indices[1][key]
        for field in ['case','questions','temperature','baseline_sha256']:
            assert a[field]==b[field], (key,field)
    prior=json.loads((root/'two-sentence-glossary-q8/results.json').read_text())
    for a in prior['cases']:
        b=indices[0][a['arm'],a['case']['id']]
        for field in ['case','state','questions','input_ids','temperature']:
            assert a[field]==b[field], ('old suite drift',field,a['case']['id'])
    lines=['# Bluey glossary expansion — d1 Q8', '',
           '200 local inferences: 100 with the original five-entry glossary and 100 '
           'with four Bluey entries appended. Same pinned text-only graph, tokenizer, '
           'question, option descriptions, gold labels and native choice calibration. '
           'The 22 new scenarios each have a single- and two-sentence version; the '
           '28 original paired scenarios are retained as regression controls.', '',
           '## Added glossary', '', '```text']
    lines += (root/'glossary-bluey.md').read_text().splitlines()[-4:]
    lines += ['```', '',
              'Names and descriptions follow the supplied glossary, including Chili. '
              'The three direct common/proper splits are bingo/Bingo, chili/Chili and '
              'bandit/Bandit. Bluey adds character positives and a blue/Bluey near-match '
              'color control. Some positives require only capitalization; others use '
              'synthetic split/homophone forms bin go, chilly, band it and blue e. '
              'These are supplied-candidate semantic probes, not observed NVIDIA errors '
              'or end-to-end transcription accuracy.', '',
              'The original baseline still supplies candidate-specific descriptions in '
              'the choices; the change is the additional shared glossary. No gold answers '
              'enter model state. No API calls, audio replay, training or live changes.', '',
              '## New Bluey cases', '',
              '| Scope | Glossary | Character repairs | Name selections on common/ambiguous controls | Exact choices |',
              '|---|---|---:|---:|---:|']
    for scope in ['single','two']:
        for label,d in zip(['original','expanded'],data):
            rows=[r for r in d['cases'] if r['arm']==scope and r['case']['id'].startswith('bluey-')]
            pos=[r for r in rows if r['case']['want']=='name']
            neg=[r for r in rows if r['case']['want']!='name']
            lines.append(f"| {scope} | {label} | {sum(r['choice']=='name' for r in pos)}/{len(pos)} | "
                         f"{sum(r['choice']=='name' for r in neg)}/{len(neg)} | "
                         f"{sum(r['choice']==r['case']['want'] for r in rows)}/{len(rows)} |")
    lines += ['', '## Two-sentence results per character', '',
              '| Character | Glossary | Repairs | Name selections on common-word controls |',
              '|---|---|---:|---:|']
    for character in ['Bluey','Bingo','Chili','Bandit']:
        for label,d in zip(['original','expanded'],data):
            rows=[r for r in d['cases'] if r['arm']=='two' and r['case']['id'].startswith('bluey-') and r['case']['canonical']==character]
            pos=[r for r in rows if r['case']['want']=='name']
            neg=[r for r in rows if r['case']['want']!='name']
            lines.append(f"| {character} | {label} | {sum(r['choice']=='name' for r in pos)}/{len(pos)} | "
                         f"{sum(r['choice']=='name' for r in neg)}/{len(neg)} |")
    lines += ['', '## Existing cases with expanded glossary', '',
              'Excludes deliberate initials and quoted-spelling stress controls from this '
              'primary table. Those remain in the full receipts. Referent identification '
              'and edit permission are distinct; name selections below are proposed '
              'replacements by the harness, not an implemented edit policy.', '',
              '| Scope | Glossary | Repairs | Name selections on non-repairs | Exact choices |',
              '|---|---|---:|---:|---:|']
    for scope in ['single','two']:
        for label,d in zip(['original','expanded'],data):
            rows=[r for r in d['cases'] if r['arm']==scope and not r['case']['id'].startswith('bluey-')
                  and r['case']['category'] not in ('privacy-initial','intentional-spelling')]
            pos=[r for r in rows if r['case']['want']=='name']
            neg=[r for r in rows if r['case']['want']!='name']
            lines.append(f"| {scope} | {label} | {sum(r['choice']=='name' for r in pos)}/{len(pos)} | "
                         f"{sum(r['choice']=='name' for r in neg)}/{len(neg)} | "
                         f"{sum(r['choice']==r['case']['want'] for r in rows)}/{len(rows)} |")
    lines += ['', 'KEEP and abstention both preserve text; exact choices distinguish them. '
              'No confidence floor. Cases are small and deliberately include ambiguity and '
              'nearby-name distractions; their error fraction is not everyday dictation WER.', '',
              '## Resources', '',
              '| Glossary | Input tokens | Median two-sentence inference (ms) | End RSS (MiB) |',
              '|---|---|---:|---:|']
    for label,d in zip(['original','expanded'],data):
        sizes=[r['token_count'] for r in d['cases']]
        ms=statistics.median(r['latency_ms'] for r in d['cases'] if r['arm']=='two')
        lines.append(f"| {label} | {min(sizes)}–{max(sizes)} | {ms:.0f} | {d['rss_end_mib']:.0f} |")
    deltas=[indices[1][key]['token_count']-a['token_count'] for key,a in indices[0].items()]
    lines += ['', f'The expansion adds {min(deltas)} actual input tokens. '
              'Zero truncation. CPU, two inference threads, one warm-up. Timings are separate '
              'laptop runs under varying load, not an isolated benchmark or mobile measurement.', '',
              '## Every Bluey decision and proposed output', '',
              '| Scope / case | Expected | Original → expanded choice | P(name) original → expanded | Dictation | Original-glossary output | Expanded-glossary output |',
              '|---|---|---|---:|---|---|---|']
    for key,a in indices[0].items():
        if not a['case']['id'].startswith('bluey-'):
            continue
        b=indices[1][key]; c=a['case']
        lines.append(f"| {a['arm']} / {c['id']} | {c['want']} | {a['choice']} → {b['choice']} | "
                     f"{a['probabilities']['name']:.3f} → {b['probabilities']['name']:.3f} | {c['context']} | "
                     f"{output(a)} | {output(b)} |")
    lines += ['', '## Changes to existing cases', '',
              '| Scope / case | Expected | Original → expanded | P(name) original → expanded | Dictation |',
              '|---|---|---|---:|---|']
    for key,a in indices[0].items():
        b=indices[1][key]; c=a['case']
        if c['id'].startswith('bluey-') or a['choice']==b['choice']:
            continue
        lines.append(f"| {a['arm']} / {c['id']} | {c['want']} | {a['choice']} → {b['choice']} | "
                     f"{a['probabilities']['name']:.3f} → {b['probabilities']['name']:.3f} | {c['context']} |")
    lines += ['', 'Regenerate the saved comparison:', '', '```sh',
              'nix develop .#pair-probe -c python3 tools/pair-router-probe/bluey-report.py', '```', '',
              'Re-run expanded-glossary Q8:', '', '```sh',
              'nix develop .#pair-probe -c python3 tools/pair-router-probe/d1.py --quantization q8 '
              '--cases tools/pair-router-probe/bluey-expanded-suite.json '
              '--glossary tools/pair-router-probe/glossary-bluey.md '
              '--output-dir tools/pair-router-probe/bluey-q8-expanded-glossary', '```']
    destination=root/'bluey-report.md'
    destination.write_text('\n'.join(lines)+'\n')
    print('\n'.join(lines[:90]))
    print(destination)


if __name__=='__main__':
    main()
