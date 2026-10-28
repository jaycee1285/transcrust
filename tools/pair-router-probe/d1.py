#!/usr/bin/env python3
"""Text-only d1 quantized decisions over the saved local/API probe inputs."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import resource
import statistics
import time

import numpy as np
import onnxruntime as ort
from tokenizers import Tokenizer

from openrouter import BASELINES, CHOICES, HERE, MODELS, jobs


class TokenizerAdapter:
    """Minimal HF-tokenizer interface for LiquidAI's inspected prompt.py."""
    def __init__(self, tokenizer, bos):
        self.tokenizer = tokenizer
        self.bos_token_id = bos

    def convert_tokens_to_ids(self, text):
        value = self.tokenizer.token_to_id(text)
        if value is None:
            raise ValueError(f'Missing special token: {text}')
        return value

    def __call__(self, text, add_special_tokens=False):
        return {'input_ids': self.tokenizer.encode(text, add_special_tokens=add_special_tokens).ids}


def rss_mib():
    for line in Path('/proc/self/status').read_text().splitlines():
        if line.startswith('VmRSS:'):
            return int(line.split()[1]) / 1024


def make_report(data, destination):
    if data.get('context_suite'):
        from context_suite import model_report
        model_report(data, destination)
        return
    api_path = HERE / 'openrouter-baseline/receipts.jsonl'
    api = [json.loads(line) for line in api_path.read_text().splitlines()] if api_path.exists() else []
    label = 'd1 ' + data['quantization'].upper()
    lines = [f'# {label} text comparison', '',
             'One CPU session, two inference threads, one warm-up call. Text graph only; '
             'no media encoders. Same saved questions, descriptions and sentence scopes '
             'as prior probes, using d1’s native formatter and choice calibration.', '',
             f"Load {data['load_secs']:.2f}s; median inference "
             f"{statistics.median(r['latency_ms'] for r in data['cases']):.0f}ms; "
             f"RSS after load {data['rss_after_load_mib']:.0f} MiB; "
             f"RSS at end {data['rss_end_mib']:.0f} MiB; "
             f"process peak RSS {data['peak_rss_mib']:.0f} MiB.", '',
             'RSS includes Python, tokenizer, ONNX Runtime and CPU allocator caches. '
             'These are laptop measurements, not mobile RAM/latency measurements. '
             'API timings include network overhead; local timings measure inference.', '',
             '| Arm | Model | Floor | Repairs | Edits on keep | Edits on ambiguous | Exact decisions |',
             '|---|---|---:|---:|---:|---:|---:|']
    for arm in BASELINES:
        path = HERE / arm / 'results.json'
        baseline_hash = hashlib.sha256(path.read_bytes()).hexdigest()
        local = json.loads(path.read_text())['cases']
        groups = [('Verdict', [{'case': c, **c} for c in local]),
                  (label, [r for r in data['cases'] if r['arm'] == arm])]
        for model in MODELS:
            groups.append((model, [r for r in api if r['arm'] == arm and r['model'] == model
                          and r['baseline_sha256'] == baseline_hash and 'error' not in r]))
        for model, rows in groups:
            if not rows:
                continue
            for floor in [0, .5, .7, .9]:
                counts = []
                for want in CHOICES:
                    group = [r for r in rows if r['case']['want'] == want]
                    count = sum(r['choice'] == 'name' and r['confidence'] >= floor
                                and r['case']['heard'] != r['case']['canonical'] for r in group)
                    counts.append(f'{count}/{len(group)}')
                # CHOICES is keep/name/abstain; display repairs first.
                counts[0], counts[1] = counts[1], counts[0]
                exact = sum(r['choice'] == r['case']['want'] for r in rows)
                lines.append(f'| {arm} | {model} | {floor:.1f} | '
                             + ' | '.join(counts) + f' | {exact}/{len(rows)} |')
    lines += ['', 'Exact decisions are raw choices, independent of the edit floor. '
              'KEEP and abstain both preserve the text. Floors are illustrative, '
              'not fitted correction-domain calibration.', '',
              '## d1 decisions', '',
              '| Arm | Case | Want | Choice | P(keep) | P(name) | P(abstain) | Original → proposed |',
              '|---|---|---|---|---:|---:|---:|---|']
    for row in data['cases']:
        case, p = row['case'], row['probabilities']
        original = case['context'].replace('«', '').replace('»', '')
        proposed = (case['context'].replace('«' + case['heard'] + '»', case['canonical'])
                    if row['choice'] == 'name' else original)
        lines.append(f"| {row['arm']} | {case['id']} | {case['want']} | {row['choice']} | "
                     f"{p['keep']:.3f} | {p['name']:.3f} | {p['abstain']:.3f} | {original} → {proposed} |")
    (destination / 'report.md').write_text('\n'.join(lines) + '\n')
    for arm in BASELINES:
        rows = [r for r in data['cases'] if r['arm'] == arm]
        positives = [r for r in rows if r['case']['want'] == 'name']
        print(f"{arm}: repairs {sum(r['choice']=='name' for r in positives)}/{len(positives)}; "
              f"edits on keep {sum(r['choice']=='name' and r['case']['want']=='keep' for r in rows)}; "
              f"edits on ambiguous {sum(r['choice']=='name' and r['case']['want']=='abstain' for r in rows)}")
    print(destination / 'report.md')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--quantization', choices=['q8', 'q4'], default='q8')
    parser.add_argument('--model-dir', type=Path)
    parser.add_argument('--output-dir', type=Path)
    parser.add_argument('--report-only', action='store_true')
    parser.add_argument('--cases', type=Path, help='Paired one/two-sentence fixture JSON')
    parser.add_argument('--glossary', type=Path, help='Shared glossary added to context-suite state')
    args = parser.parse_args()
    if args.glossary and not args.cases:
        parser.error('--glossary requires --cases')
    args.model_dir = args.model_dir or Path('/tmp/transcrust-d1-omni-600M-' + args.quantization)
    args.output_dir = args.output_dir or HERE / (('two-sentence-glossary-' if args.glossary else 'two-sentence-' if args.cases else 'd1-') + args.quantization)
    args.output_dir.mkdir(parents=True, exist_ok=True)
    if args.report_only:
        make_report(json.loads((args.output_dir / 'results.json').read_text()), args.output_dir)
        return
    manifest = json.loads((args.model_dir / 'manifest.json').read_text())
    # Upstream prompt.py was inspected before import; it uses only the standard library.
    prompt_path = args.model_dir / 'prompt.py'
    if hashlib.sha256(prompt_path.read_bytes()).hexdigest() != manifest['files']['prompt.py']['sha256']:
        raise ValueError('Prompt reference checksum mismatch')
    import sys
    spec = importlib.util.spec_from_file_location('d1_reference_prompt', prompt_path)
    prompt = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = prompt
    spec.loader.exec_module(prompt)
    config = json.loads((args.model_dir / 'config.json').read_text())
    tokenizer = Tokenizer.from_file(str(args.model_dir / 'tokenizer.json'))
    tokenizer.no_truncation()
    tok = TokenizerAdapter(tokenizer, config['bos_token_id'])
    prepared = []
    if args.cases:
        from context_suite import jobs as context_jobs
        work = context_jobs(args.cases, args.glossary)
    else:
        work = jobs()
    for job in work:
        q = prompt.as_question(job['questions']['decision'])
        # Native configurable option budget: retain the original longer descriptions.
        ids, markers = prompt.encode(tok, job['state'], q, config['max_length'], per_option=64)
        state_ids = tok(prompt.escape(prompt.serialize(job['state'])))['input_ids']
        question_ids = tok(prompt.escape(q.instructions))['input_ids']
        option_ids = [tok(prompt.escape(' ' + text))['input_ids'] for text in prompt.render_options(q)]
        # Prove no state, instruction or option text was truncated by the native formatter.
        expected_length = 2 + len(state_ids) + 1 + len(question_ids) + sum(3 + len(x) for x in option_ids) + 1
        if len(ids) != expected_length:
            raise ValueError(f"{job['case']['id']}: native formatter truncated text")
        if any(ids[position] != tokenizer.token_to_id(prompt.MARKER) for position in markers):
            raise ValueError('Marker positions do not identify mask tokens')
        temperature = config['temperatures'][prompt.temperature_key(q)]
        inputs = {'input_ids': np.array([ids], dtype=np.int64),
                  'attention_mask': np.ones((1, len(ids)), dtype=np.int64),
                  'prefix_len': np.array([0], dtype=np.int64),
                  'marker_pos': np.array([markers], dtype=np.int64),
                  'marker_mask': np.ones((1, len(markers)), dtype=np.int64),
                  'qtype': np.array([prompt.QTYPES[q.type]], dtype=np.int64)}
        prepared.append((job, inputs, temperature))
    options = ort.SessionOptions()
    options.intra_op_num_threads = 2
    options.inter_op_num_threads = 1
    started = time.perf_counter()
    graph = 'model_quantized.onnx' if args.quantization == 'q8' else 'model_q4.onnx'
    session = ort.InferenceSession(str(args.model_dir / 'onnx' / graph),
                sess_options=options, providers=['CPUExecutionProvider'])
    load_secs = time.perf_counter() - started
    loaded_rss = rss_mib()
    print(f'd1 {args.quantization.upper()} loaded in {load_secs:.2f}s; RSS {loaded_rss:.0f} MiB; {len(prepared)} cases', flush=True)
    print('Graph inputs:', [(i.name, i.shape, i.type) for i in session.get_inputs()], flush=True)
    session.run(None, prepared[0][1])
    results = []
    for job, inputs, temperature in prepared:
        started = time.perf_counter()
        logits = np.asarray(session.run(None, inputs)[0], dtype=np.float64)
        elapsed = (time.perf_counter() - started) * 1000
        if logits.shape != (1, 3) or not np.isfinite(logits).all():
            raise ValueError(f'Unexpected logits: {logits.shape}')
        scores = logits[0] / temperature
        exp = np.exp(scores - scores.max())
        probabilities = exp / exp.sum()
        choice = CHOICES[int(probabilities.argmax())]
        results.append({**job, 'choice': choice, 'confidence': float(probabilities.max()),
            'probabilities': dict(zip(CHOICES, map(float, probabilities))),
            'logits': logits[0].tolist(), 'temperature': temperature, 'latency_ms': elapsed,
            'input_ids': inputs['input_ids'][0].tolist(), 'marker_pos': inputs['marker_pos'][0].tolist(),
            'token_count': inputs['input_ids'].shape[1]})
        print(f"{job['arm']}/{job['case']['id']}: {choice} p={probabilities.max():.3f} "
              f"want={job['case']['want']} ({elapsed:.0f}ms)", flush=True)
    data = {'model': 'LiquidAI/d1-omni-600M', 'export': 'onnx-community/d1-omni-600M-ONNX',
            'quantization': args.quantization, 'context_suite': str(args.cases) if args.cases else None,
            'glossary': work[0].get('glossary'),
            'modality': 'text-only', 'manifest': manifest,
            'onnxruntime': ort.__version__, 'provider': 'CPUExecutionProvider', 'threads': 2,
            'per_option': 64, 'truncated_cases': 0, 'load_secs': load_secs,
            'rss_after_load_mib': loaded_rss, 'rss_end_mib': rss_mib(),
            'peak_rss_mib': resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024,
            'cases': results}
    (args.output_dir / 'results.json').write_text(json.dumps(data, indent=2, ensure_ascii=False) + '\n')
    make_report(data, args.output_dir)


if __name__ == '__main__':
    main()
