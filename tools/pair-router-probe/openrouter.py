#!/usr/bin/env python3
"""Compare saved local probes with typed Decisions API models (paid requests)."""
import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime, timezone
import hashlib
import json
import math
from pathlib import Path
import statistics
import time
import urllib.error
import urllib.request

HERE = Path(__file__).resolve().parent
URL = 'https://openrouter.ai/api/alpha/decisions'
MODELS = ['typesafe/jev-1.13', 'cloudflare/clef-flash']
CHOICES = ['keep', 'name', 'abstain']
BASELINES = ['sentence', 'suite-verdict', 'cars-verdict', 'suite-verdict-correction']


def jobs():
    """Reuse actual local prompts, including their sentence scope and descriptions."""
    result = []
    for directory in BASELINES:
        path = HERE / directory / 'results.json'
        data = json.loads(path.read_text())
        for case in data['cases']:
            labels, rest = case['prompt'].split('<<SEP>>Question: ', 1)
            question, context = rest.split('\n\nContext:\n', 1)
            descriptions = labels.split('<<LABEL>>')[1:]
            if len(descriptions) != 3:
                raise ValueError(f"{case['id']}: expected three local labels")
            result.append({
                'arm': directory, 'case': case,
                'baseline_sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
                'state': context,
                'questions': {'decision': {'type': 'choice',
                    'instructions': question,
                    'criteria': dict(zip(CHOICES, descriptions))}},
            })
    return result


def call(model, job, key):
    payload = {'model': model, 'state': job['state'], 'questions': job['questions']}
    started = time.perf_counter()
    row = {'model': model, **job, 'request': payload,
           'timestamp': datetime.now(timezone.utc).isoformat()}
    try:
        request = urllib.request.Request(URL, json.dumps(payload).encode(),
                    {'Authorization': 'Bearer ' + key, 'Content-Type': 'application/json'})
        with urllib.request.urlopen(request, timeout=45) as response:
            data = json.loads(response.read())
        # Only response bodies, never headers, are retained. Redact defensively.
        row['response'] = json.loads(json.dumps(data).replace(key, '[REDACTED]'))
        answer = data['answers']['decision']
        probabilities = answer['probabilities']
        if answer['choice'] not in CHOICES or set(probabilities) != set(CHOICES):
            raise ValueError('Unexpected choice/probability schema')
        if not all(isinstance(p, (float, int)) and math.isfinite(p) and 0 <= p <= 1
                   for p in probabilities.values()):
            raise ValueError('Invalid probabilities')
        if abs(sum(probabilities.values()) - 1) > .02:
            raise ValueError('Probabilities do not sum to one')
        row.update(choice=answer['choice'], probabilities=probabilities,
                   confidence=probabilities[answer['choice']],
                   api_confidence=answer.get('confidence'),
                   cost_usd=data.get('usage', {}).get('cost'),
                   resolved_model=data.get('model'), provider=data.get('provider'))
    except urllib.error.HTTPError as error:
        # Error bodies may contain request details; do not print or retain them.
        row['error'] = f'HTTP {error.code}'
    except Exception as error:
        row['error'] = f'{type(error).__name__}: {str(error).replace(key, "[REDACTED]")}'
    row['latency_ms'] = (time.perf_counter() - started) * 1000
    return row


def report(rows, destination):
    lines = ['# Local vs API bounded-choice comparison', '',
        'One request per case; same marked sentence, question and option descriptions '
        'as the saved local prompt. API models receive a typed choice schema; local '
        'Verdict uses its LABEL/SEP serialization. Abstention is an explicit third '
        'choice here. No labels or expected answers are sent in state.', '',
        'Results are on synthetic/selected-span probes, not end-to-end recognition. '
        'Confidence floors use the selected option probability for all models; '
        'the API’s separate confidence statistic is retained, not mixed with it.', '',
        '| Arm | Model | Floor | Repairs | Edits on keep | Edits on ambiguous | Exact decisions |',
        '|---|---|---:|---:|---:|---:|---:|']
    for arm in BASELINES:
        baseline_path = HERE / arm / 'results.json'
        local = json.loads(baseline_path.read_text())['cases']
        baseline_hash = hashlib.sha256(baseline_path.read_bytes()).hexdigest()
        for model in ['local Verdict'] + MODELS:
            selected = ([{'case': c, **c} for c in local] if model == 'local Verdict'
                        else [r for r in rows if r['arm'] == arm and r['model'] == model
                              and r['baseline_sha256'] == baseline_hash and 'error' not in r])
            if not selected:
                continue
            for floor in [0, .5, .7, .9]:
                counts = []
                for want in ['name', 'keep', 'abstain']:
                    group = [r for r in selected if r['case']['want'] == want]
                    count = sum(r['choice'] == 'name' and r['confidence'] >= floor
                                and r['case']['heard'] != r['case']['canonical'] for r in group)
                    counts.append(f'{count}/{len(group)}')
                exact = sum(r['choice'] == r['case']['want'] for r in selected)
                lines.append(f'| {arm} | {model} | {floor:.1f} | '
                             + ' | '.join(counts) + f' | {exact}/{len(selected)} |')
    lines += ['', 'Exact decisions use the raw selected option, independent of the edit floor. '
              'KEEP and abstain both leave text intact, but have different meanings. '
              'Metrics require the current saved local baseline hash; '
              'the audit and cost totals retain all attempts.', '',
              '## API usage', '', '| Model | Successful calls | Failed attempts | Reported USD | Median round trip |',
              '|---|---:|---:|---:|---:|']
    for model in MODELS:
        selected = [r for r in rows if r['model'] == model]
        good = [r for r in selected if 'error' not in r]
        cost = sum(r.get('response', {}).get('usage', {}).get('cost') or 0 for r in selected)
        latency = f"{statistics.median(r['latency_ms'] for r in good):.0f} ms" if good else '—'
        lines.append(f'| {model} | {len(good)} | {len(selected)-len(good)} | ${cost:.8f} | {latency} |')
    lines += ['', 'Local timings measure inference only; API timings include network/queue/processing. '
              'Initial DNS failures happened inside the sandbox before reaching the API; '
              'both were retried successfully with network permission. '
              'Zero-cost or missing usage is retained in receipts and must not be read as proof '
              'that an unreported request was free.', '', '## Per-case API decisions', '',
              '| Arm | Model | Case | Want | Chose | P(chosen) | Original → proposed |',
              '|---|---|---|---|---|---:|---|']
    for row in rows:
        case = row['case']
        original = case['context'].replace('«', '').replace('»', '')
        proposed = (case['context'].replace('«' + case['heard'] + '»', case['canonical'])
                    if row.get('choice') == 'name' else original)
        lines.append(f"| {row['arm']} | {row['model']} | {case['id']} | {case['want']} | "
                     f"{row.get('choice', row.get('error'))} | {row.get('confidence', 0):.3f} | "
                     f'{original} → {proposed} |')
    (destination / 'report.md').write_text('\n'.join(lines) + '\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--key-file', type=Path,
                        default=Path.home() / '.config/key/openrouter')
    parser.add_argument('--output-dir', type=Path, default=HERE / 'openrouter-baseline')
    parser.add_argument('--limit', type=int, help='Limit cases per model for a smoke call')
    parser.add_argument('--report-only', action='store_true', help='Read saved receipts; no API calls')
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)
    receipt = args.output_dir / 'receipts.jsonl'
    rows = [json.loads(line) for line in receipt.read_text().splitlines()] if receipt.exists() else []
    if not args.report_only:
        key = args.key_file.read_text().strip()
        if not key:
            raise ValueError('Empty API key file')
        work = jobs()
        if args.limit is not None:
            work = work[:args.limit]
        done = {(r['model'], r['arm'], r['case']['id'], r['baseline_sha256'])
                for r in rows if 'error' not in r}
        pending = [(model, job) for model in MODELS for job in work
                   if (model, job['arm'], job['case']['id'], job['baseline_sha256']) not in done]
        print(f'{len(pending)} paid API requests pending; saved receipts are reused.', flush=True)
        with ThreadPoolExecutor(max_workers=4) as pool, receipt.open('a') as out:
            futures = [pool.submit(call, model, job, key) for model, job in pending]
            for number, future in enumerate(as_completed(futures), 1):
                row = future.result()
                rows.append(row)
                out.write(json.dumps(row, ensure_ascii=False) + '\n')
                out.flush()
                print(f"{number}/{len(pending)} {row['model']} {row['arm']}/{row['case']['id']}: "
                      f"{row.get('choice', row.get('error'))}", flush=True)
    report(rows, args.output_dir)
    for model in MODELS:
        good = [r for r in rows if r['model'] == model and 'error' not in r]
        cost = sum(r.get('cost_usd') or 0 for r in good)
        print(f'{model}: {len(good)} successful calls; reported cost ${cost:.8f}')
    print(args.output_dir / 'report.md')


if __name__ == '__main__':
    main()
