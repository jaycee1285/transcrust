#!/usr/bin/env python3
"""Read saved local receipts only: no model loading, downloads, keys or API calls."""
import json
from pathlib import Path
import statistics

HERE = Path(__file__).resolve().parent
ARMS = [('sentence', 'Original 19'), ('suite-verdict', 'Main 60'),
        ('cars-verdict', 'Cars 18'), ('suite-verdict-correction', 'Correction question 60')]


def main():
    d1 = json.loads((HERE / 'd1-q8/results.json').read_text())
    needle = [json.loads(line) for line in (HERE / 'needle3/receipts.jsonl').read_text().splitlines()]
    expected = {(r['arm'], r['case']['id']) for r in d1['cases']}
    if len(expected) != len(d1['cases']):
        raise ValueError('Duplicate d1 cases')
    for mode in ['default', 'forced']:
        matching = [r for r in needle if r['mode'] == mode]
        if len(matching) != len(expected) or {(r['arm'], r['case']['id']) for r in matching} != expected:
            raise ValueError(f'Needle {mode} results are incomplete or duplicated')
    baseline_hashes = {(r['arm'], r['case']['id']): r['baseline_sha256'] for r in d1['cases']}
    if any(r['baseline_sha256'] != baseline_hashes[(r['arm'], r['case']['id'])] for r in needle):
        raise ValueError('The candidates were run on different baseline revisions')
    lines = ['# Offline correction candidates', '',
        'Deployment comparison: local d1 Q8 versus local Needle 3 alongside NVIDIA ASR. '
        'Previously authorized API calls are quality references only, not a runtime option '
        'or fallback for the family application. This report makes no network requests.', '',
        '| Surface | Candidate | Repairs | Edits on keep | Edits on ambiguous | Invalid |',
        '|---|---|---:|---:|---:|---:|']
    for arm, title in ARMS:
        groups = [('d1 Q8', [r for r in d1['cases'] if r['arm'] == arm])]
        for mode in ['default', 'forced']:
            groups.append(('Needle 3 ' + mode, [r for r in needle if r['arm'] == arm and r['mode'] == mode]))
        for model, rows in groups:
            counts = []
            for want in ['name', 'keep', 'abstain']:
                group = [r for r in rows if r['case']['want'] == want]
                edits = sum(r['choice'] == 'name' and r['case']['heard'] != r['case']['canonical'] for r in group)
                counts.append(f'{edits}/{len(group)}')
            invalid = sum(r['choice'] == 'invalid' for r in rows)
            lines.append(f'| {title} | {model} | ' + ' | '.join(counts) + f' | {invalid} |')
    lines += ['', 'No confidence floor is applied. Invalid or multiple tool calls are counted '
              'separately and never interpreted as permission to edit. Native no-call results '
              'leave text intact and are recorded as runtime abstention in Needle receipts.', '',
              '## Interpretation', '',
              'These are untuned, supplied-candidate adapters. d1 uses its upstream masked-option '
              'formatter and shipped choice temperature; Needle uses one classification tool '
              'with a three-value enum. Default Needle gating and forced-call mode are shown '
              'separately. Forced-call mode still offers KEEP and abstain; it is not a forced edit. '
              'This is not a verdict on tuned Needle or on its best possible task representation.', '',
              'Sentence scope, question and option meanings match the saved probe inputs. '
              'The original sentence splitter’s abbreviation behavior is inherited by both. '
              'd1 uses per_option=64 with verified zero truncation to retain all descriptions. '
              'Needle refuses context overflow; complete raw tool calls, suppressed calls and '
              'confidence scores are retained. The candidates’ confidence statistics are not '
              'interchangeable, so this report does not apply a shared numerical floor.', '',
              '## Memory and time', '',
              f"d1 text bundle: {sum(v['bytes'] for v in d1['manifest']['files'].values()) / 1e6:.1f} MB, "
              'including tokenizer and references. Only the text graph and its weight sidecar '
              'were downloaded. No vision/audio encoder is loaded.', '',
              f"d1 process peak RSS: {d1['peak_rss_mib']:.0f} MiB; end RSS: {d1['rss_end_mib']:.0f} MiB. "
              f"Load: {d1['load_secs']:.2f}s. Median inference: "
              f"{statistics.median(r['latency_ms'] for r in d1['cases']):.0f}ms. "
              'These include the Python/ORT process; there were competing Rust builds. '
              'They do not establish isolated speed or mobile memory/latency.', '']
    if needle:
        lines += [f"Needle model file: {needle[0]['model_bytes'] / 1e6:.1f} MB. "
                  f"Maximum runtime-reported peak RAM: {max(r.get('response', {}).get('peak_ram_mb', 0) for r in needle):.1f} MB. "
                  f"Median CLI round trip: {statistics.median(r['latency_ms'] for r in needle):.0f}ms. "
                  'Needle CLI timing includes loading the model for every case. Its runtime RAM '
                  'counter has different accounting from process RSS, so those figures are '
                  'not equivalent memory measurements. Needle telemetry was explicitly disabled.', '']
    lines += ['Detailed reports: [d1](d1-q8/report.md), [Needle 3](needle3/report.md). '
              'd1’s stock billing/technical control questions both selected the expected answer '
              'above 0.99; see `d1-q8/contract-smoke.json`.', '',
              'Rerun this saved-result comparison:', '', '```sh',
              'nix develop .#pair-probe -c python3 tools/pair-router-probe/offline-report.py', '```', '',
              'No live dictation, vocabulary, or Apple review-before-paste behavior was changed.']
    path = HERE / 'offline-comparison.md'
    path.write_text('\n'.join(lines) + '\n')
    print('\n'.join(lines[:len(ARMS) * 3 + 7]))
    print(path)


if __name__ == '__main__':
    main()
