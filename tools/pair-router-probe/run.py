#!/usr/bin/env python3
"""Headless, read-only experiment: sentence + supplied pair -> keep/name/abstain.
No vocabulary, task files, daemon settings or destination editor are modified.
"""
import argparse
import hashlib
import json
import re
import statistics
import time
from pathlib import Path

import numpy as np
import onnxruntime as ort
from tokenizers import Tokenizer

HERE = Path(__file__).resolve().parent


def digest(path):
    with path.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--model-dir', type=Path,
                        default=Path.home() / 'repos/dz/jev-options/rlcd-151m')
    parser.add_argument('--output-dir', type=Path, default=HERE)
    parser.add_argument('--cases', type=Path, default=HERE / 'cases.json',
                        help='Model-independent marked-span evaluation cases')
    parser.add_argument('--decision-task', choices=['referent', 'correction'],
                        default='referent', help='Identify meaning or decide whether to edit')
    parser.add_argument('--sentence-only', action='store_true',
                        help='Only pass the sentence containing the marked occurrence')
    args = parser.parse_args()
    cases = json.loads(args.cases.read_text())
    if not cases:
        raise ValueError('The suite must contain at least one case')
    seen = set()
    for case in cases:
        if case['id'] in seen:
            raise ValueError(f"Duplicate case ID: {case['id']}")
        seen.add(case['id'])
        context = case['context']
        if context.count('«') != 1 or context.count('»') != 1:
            raise ValueError(f"{case['id']}: exactly one marked occurrence is required")
        if context.index('»') <= context.index('«') or \
                context.split('«')[1].split('»')[0] != case['heard']:
            raise ValueError(f"{case['id']}: marked span does not match heard text")
        if case['want'] not in ('name', 'keep', 'abstain'):
            raise ValueError(f"{case['id']}: invalid expected choice")
    model = args.model_dir / 'model_fp16.onnx'
    calibration = json.loads((args.model_dir / 'calibrator.json').read_text())
    temperature = float(calibration.get('per_k', {}).get('3', calibration['temperature']))
    tokenizer = Tokenizer.from_file(str(args.model_dir / 'tokenizer.json'))
    # Reject context overflow instead of silently losing the marked occurrence.
    tokenizer.no_truncation()
    options = ort.SessionOptions()
    options.intra_op_num_threads = 2
    options.inter_op_num_threads = 1
    start = time.perf_counter()
    session = ort.InferenceSession(str(model), sess_options=options,
                                   providers=['CPUExecutionProvider'])
    load_secs = time.perf_counter() - start
    print(f'Verdict loaded in {load_secs:.2f}s; 3 choices, T={temperature}', flush=True)
    results = []
    for case in cases:
        case = dict(case)
        if args.sentence_only:
            context = case['context']
            mark_start, mark_end = context.index('«'), context.index('»')
            # Keep the occurrence marker. Only sentence scope changes between
            # arms; the labels, question, model and temperature stay fixed.
            separators = list(re.finditer(r'[.!?]\s+', context))
            start = max((m.end() for m in separators if m.end() <= mark_start), default=0)
            end = min((m.start() + 1 for m in separators if m.start() > mark_end), default=len(context))
            case['context'] = context[start:end].strip()
        # Match training/export contract in Verdict core/formatting.py.
        labels = [f"It is {case['keep_description']}",
                  f"It is {case['name_description']}", 'insufficient evidence']
        question = 'What does the marked span refer to in this spoken sentence?'
        if args.decision_task == 'correction':
            labels = ['It is text that should be preserved exactly as written, including '
                      'intentional initials, quoted spellings, other people and ordinary words',
                      f"It is a transcription error that should be replaced with {case['canonical']}, "
                      f"meaning {case['name_description']}", 'insufficient evidence']
            question = ('Should the marked text be corrected to the supplied vocabulary name? '
                        'Preserve intentional initials and spellings. Correct only when the '
                        'sentence establishes a transcription error. If the intended meaning '
                        'is unclear, there is insufficient evidence. '
                        f"Vocabulary fact: {case['name_description']}. "
                        f"Alternative meaning: {case['keep_description']}.")
        prompt = ''.join('<<LABEL>>' + label for label in labels) + \
                 '<<SEP>>Question: ' + question + '\n\nContext:\n' + case['context']
        encoding = tokenizer.encode(prompt)
        if len(encoding.ids) > 512:
            raise ValueError(f"{case['id']}: {len(encoding.ids)} tokens exceeds model context")
        inputs = {'input_ids': np.array([encoding.ids], dtype=np.int64),
                  'attention_mask': np.array([encoding.attention_mask], dtype=np.int64)}
        start = time.perf_counter()
        logits = np.asarray(session.run(None, inputs)[0][0][:3], dtype=np.float64)
        elapsed = (time.perf_counter() - start) * 1000
        if not np.isfinite(logits).all():
            raise ValueError(f"{case['id']}: non-finite logits")
        scaled = logits / temperature
        exps = np.exp(scaled - scaled.max())
        probs = exps / exps.sum()
        choice = ['keep', 'name', 'abstain'][int(probs.argmax())]
        result = {**case, 'choice': choice, 'confidence': float(probs.max()),
                  'probabilities': dict(zip(['keep', 'name', 'abstain'], map(float, probs))),
                  'logits': list(map(float, logits)), 'latency_ms': elapsed,
                  'token_count': len(encoding.ids), 'prompt': prompt}
        results.append(result)
        print(f"{case['id']:25} {case['heard']:18} -> {choice:7} "
              f"p={probs.max():.3f} want={case['want']}", flush=True)
    args.output_dir.mkdir(parents=True, exist_ok=True)
    data = {'model': str(model), 'model_sha256': digest(model),
            'cases_file': str(args.cases), 'cases_sha256': digest(args.cases),
            'decision_task': args.decision_task,
            'tokenizer_sha256': digest(args.model_dir / 'tokenizer.json'),
            'calibrator_sha256': digest(args.model_dir / 'calibrator.json'),
            'temperature': temperature, 'onnxruntime': ort.__version__,
            'scope': 'sentence' if args.sentence_only else 'context-window',
            'provider': 'CPUExecutionProvider', 'threads': 2,
            'load_secs': load_secs, 'cases': results}
    (args.output_dir / 'results.json').write_text(json.dumps(data, indent=2) + '\n')
    lines = ['# Sentence-and-pair decision probe', '',
             'Model: local Verdict/OpenJev ModernBERT 151M, fp16 ONNX, CPU. '
             f'Per-3-choice temperature: {temperature}.', '',
             f"Context scope: {'marked sentence' if args.sentence_only else 'context window / supplied passage'}.", '',
             f'Decision task: **{args.decision_task}**. Referent identifies meaning; '
             'correction asks whether to replace text, preserving intentional spellings.', '',
             'Uses Verdict’s published label/SEP prompt contract. Each call has '
             'KEEP, the supplied canonical name, and insufficient evidence. '
             'Model probabilities are not validated calibration for this correction domain.', '',
             f'Cases: `{args.cases.name}`. Fixture SHA-256: `{digest(args.cases)}`. '
             'Synthetic sentences measure supplied-candidate decisions, not audio recognition '
             'or end-to-end WER. Record-2 rows, when included, are selected repairs from '
             'the unboosted delivered transcript, including its phonetic/casing post-pass.', '',
             '| Source | Confidence floor | Repairs made / wanted | Wrong edits on keep cases | Edits on ambiguous cases |',
             '|---|---:|---:|---:|---:|']
    for source in sorted({r['source'] for r in results}):
        rows = [r for r in results if r['source'] == source]
        positives = [r for r in rows if r['want'] == 'name']
        controls = [r for r in rows if r['want'] == 'keep']
        ambiguous = [r for r in rows if r['want'] == 'abstain']
        for floor in [0, .5, .7, .9]:
            apply = lambda r: r['choice'] == 'name' and r['confidence'] >= floor
            lines.append(f'| {source} | {floor:.1f} | '
                         f'{sum(map(apply, positives))}/{len(positives)} | '
                         f'{sum(map(apply, controls))}/{len(controls)} | '
                         f'{sum(map(apply, ambiguous))}/{len(ambiguous)} |')
    lines += ['', '## Decision accuracy by category', '',
              '| Category | Correct choice | Name edits on non-repair cases |',
              '|---|---:|---:|']
    for category in sorted({r.get('category', r['source']) for r in results}):
        rows = [r for r in results if r.get('category', r['source']) == category]
        negatives = [r for r in rows if r['want'] != 'name']
        lines.append(f'| {category} | '
                     f"{sum(r['choice'] == r['want'] for r in rows)}/{len(rows)} | "
                     f"{sum(r['choice'] == 'name' for r in negatives)}/{len(negatives)} |")
    lines += ['', f"Model load: {load_secs:.2f}s. Median inference: "
              f"{statistics.median(r['latency_ms'] for r in results):.0f}ms.", '',
              '| Case | Heard → candidate | Want | Chose | P(keep) | P(name) | P(abstain) |',
              '|---|---|---|---|---:|---:|---:|']
    for row in results:
        p = row['probabilities']
        lines.append(f"| {row['id']} | {row['heard']} → {row['canonical']} | {row['want']} | "
                     f"{row['choice']} | {p['keep']:.3f} | {p['name']:.3f} | {p['abstain']:.3f} |")
    lines += ['', '## Contexts', '']
    for row in results:
        lines += [f"**{row['id']}** ({row['source']}): {row['context']}", '']
        if row.get('reason'):
            lines += [f"Expected **{row['want']}**: {row['reason']}", '']
    (args.output_dir / 'report.md').write_text('\n'.join(lines) + '\n')
    # Inspection copy at a fixed illustrative floor, not a production policy.
    if not any(row['source'] == 'record-2' for row in results):
        print(args.output_dir / 'report.md')
        return
    baseline = (HERE / 'record-2-unboosted.md').read_text().split('\n# ', 1)[1].split('\n\n', 1)[1]
    edits = []
    for row in results:
        if row['source'] == 'record-2' and row['choice'] == 'name' and row['confidence'] >= .5:
            start, end = row['start'], row['end']
            if baseline[start:end] != row['heard']:
                raise ValueError(f"{row['id']}: baseline offset does not match")
            edits.append((start, end, row['canonical']))
    for start, end, replacement in sorted(edits, reverse=True):
        baseline = baseline[:start] + replacement + baseline[end:]
    (args.output_dir / 'record-2-proposed.md').write_text(
        '# Record-2 — proposed repairs, confidence floor 0.5\n\n'
        'Offline inspection artifact, not a human-approved reference.\n\n' + baseline)
    print(args.output_dir / 'report.md')


if __name__ == '__main__':
    main()
