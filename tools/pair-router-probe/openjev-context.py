#!/usr/bin/env python3
"""Local OpenJev/Verdict paired-context probe, with and without shared glossary."""
import argparse
import hashlib
import json
from pathlib import Path
import resource
import time

import numpy as np
import onnxruntime as ort
from tokenizers import Tokenizer

from context_suite import CHOICES, HERE, jobs, model_report


def digest(path):
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


def rss():
    for line in Path('/proc/self/status').read_text().splitlines():
        if line.startswith('VmRSS:'):
            return int(line.split()[1]) / 1024


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--model-dir', type=Path, default=Path.home() / 'repos/dz/jev-options/rlcd-151m')
    parser.add_argument('--cases', type=Path, default=HERE / 'two-sentence.json')
    parser.add_argument('--glossary', type=Path, default=HERE / 'glossary.md')
    parser.add_argument('--report-only', action='store_true')
    args = parser.parse_args()
    if args.report_only:
        from openjev_context_report import report
        report()
        return
    model = args.model_dir / 'model_fp16.onnx'
    calibrator = json.loads((args.model_dir / 'calibrator.json').read_text())
    temperature = float(calibrator.get('per_k', {}).get('3', calibrator['temperature']))
    tokenizer = Tokenizer.from_file(str(args.model_dir / 'tokenizer.json'))
    tokenizer.no_truncation()
    prepared = []
    for glossary in (None, args.glossary):
        arm = []
        for job in jobs(args.cases, glossary):
            decision = job['questions']['decision']
            labels = [decision['criteria'][c] for c in CHOICES]
            prompt = ''.join('<<LABEL>>' + label for label in labels) + \
                     '<<SEP>>Question: ' + decision['instructions'] + '\n\nContext:\n' + job['state']
            encoding = tokenizer.encode(prompt)
            if len(encoding.ids) > 512:
                raise ValueError(f"{job['case']['id']}: {len(encoding.ids)} tokens exceeds export context")
            inputs = {'input_ids': np.array([encoding.ids], dtype=np.int64),
                      'attention_mask': np.array([encoding.attention_mask], dtype=np.int64)}
            arm.append((job, prompt, inputs))
        prepared.append(arm)
    options = ort.SessionOptions()
    options.intra_op_num_threads = 2
    options.inter_op_num_threads = 1
    start = time.perf_counter()
    session = ort.InferenceSession(str(model), sess_options=options, providers=['CPUExecutionProvider'])
    load_secs = time.perf_counter() - start
    loaded_rss = rss()
    metadata = {'model': str(model), 'model_label': 'OpenJev/Verdict 151M fp16',
        'model_sha256': digest(model), 'model_bytes': model.stat().st_size,
        'tokenizer_sha256': digest(args.model_dir / 'tokenizer.json'),
        'calibrator_sha256': digest(args.model_dir / 'calibrator.json'),
        'temperature': temperature, 'onnxruntime': ort.__version__,
        'provider': 'CPUExecutionProvider', 'threads': 2, 'truncated_cases': 0,
        'load_secs': load_secs, 'rss_after_load_mib': loaded_rss,
        'scope': 'paired single/two sentences', 'context_limit': 512}
    print(f'OpenJev loaded in {load_secs:.2f}s; RSS {loaded_rss:.0f} MiB; T={temperature}', flush=True)
    for enabled, work in zip((False, True), prepared):
        session.run(None, work[0][2])
        results = []
        for job, prompt, inputs in work:
            start = time.perf_counter()
            logits = np.asarray(session.run(None, inputs)[0][0][:3], dtype=np.float64)
            latency = (time.perf_counter() - start) * 1000
            if logits.shape != (3,) or not np.isfinite(logits).all():
                raise ValueError('Unexpected logits')
            scaled = logits / temperature
            exp = np.exp(scaled - scaled.max())
            probabilities = exp / exp.sum()
            choice = CHOICES[int(probabilities.argmax())]
            results.append({**job, 'choice': choice, 'confidence': float(probabilities.max()),
                'probabilities': dict(zip(CHOICES, map(float, probabilities))),
                'logits': logits.tolist(), 'latency_ms': latency, 'prompt': prompt,
                'input_ids': inputs['input_ids'][0].tolist(), 'token_count': inputs['input_ids'].shape[1]})
            print(f"glossary={enabled} {job['arm']}/{job['case']['id']}: {choice} "
                  f"want={job['case']['want']} ({latency:.0f}ms)", flush=True)
        data = {**metadata, 'glossary': work[0][0].get('glossary'), 'cases': results,
                'rss_end_mib': rss(), 'peak_rss_mib': resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024}
        destination = HERE / ('two-sentence-glossary-openjev' if enabled else 'two-sentence-openjev')
        destination.mkdir(parents=True, exist_ok=True)
        (destination / 'results.json').write_text(json.dumps(data, indent=2, ensure_ascii=False)+'\n')
        model_report(data, destination)
    from openjev_context_report import report
    report()


if __name__ == '__main__':
    main()
