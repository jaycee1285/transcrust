#!/usr/bin/env python3
"""Paired one/two-sentence fixtures and saved-result reporting; entirely local."""
import argparse
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
CHOICES = ('keep', 'name', 'abstain')


def jobs(path, glossary_path=None):
    glossary = None
    if glossary_path:
        raw_glossary = glossary_path.read_bytes()
        glossary = {'text': raw_glossary.decode('utf-8'),
                    'sha256': hashlib.sha256(raw_glossary).hexdigest()}
    source = path.read_bytes()
    fixtures = json.loads(source)
    seen = set()
    result = []
    if not fixtures:
        raise ValueError('Empty context suite')
    for fixture in fixtures:
        if fixture['id'] in seen:
            raise ValueError('Duplicate scenario: ' + fixture['id'])
        seen.add(fixture['id'])
        if fixture['single_context'] not in fixture['two_context']:
            raise ValueError('Two-sentence context must retain the complete target sentence')
        for scope in ('single', 'two'):
            context, want = fixture[scope + '_context'], fixture[scope + '_want']
            if (context.count('«') != 1 or context.count('»') != 1
                    or '«' + fixture['heard'] + '»' not in context or want not in CHOICES):
                raise ValueError('Invalid marked span or expected choice: ' + fixture['id'])
            case = {key: fixture[key] for key in ('id', 'category', 'heard', 'canonical',
                                                  'keep_description', 'name_description')}
            case.update(context=context, want=want, source='synthetic-two-sentence')
            result.append({'arm': scope, 'case': case,
                'baseline_sha256': hashlib.sha256(source).hexdigest(), 'state': context,
                'questions': {'decision': {'type': 'choice',
                    # Keep the prior referent question, isolating the context change.
                    'instructions': 'What does the marked span refer to in this spoken sentence?',
                    'criteria': {'keep': 'It is ' + fixture['keep_description'],
                                 'name': 'It is ' + fixture['name_description'],
                                 'abstain': 'insufficient evidence'}}}})
    if glossary:
        for job in result:
            job['glossary'] = glossary
            job['state'] = 'Vocabulary glossary:\n' + glossary['text'].rstrip() + '\n\nDictation:\n' + job['state']
    return result


def original(case):
    return case['context'].replace('«', '').replace('»', '')


def output(row):
    case = row['case']
    return (case['context'].replace('«' + case['heard'] + '»', case['canonical'])
            if row['choice'] == 'name' else original(case))


def model_report(data, destination):
    label = data.get('model_label') or ('d1 ' + data['quantization'].upper())
    lines = ['# ' + label + ' paired context decisions', '',
             'Each scenario judges one marked occurrence, with the target sentence alone '
             'and then with one neighboring sentence. Expected answers are not model inputs.', '',
             ('Shared glossary SHA-256: ' + data['glossary']['sha256']
              if data.get('glossary') else 'No shared glossary.'), '',
             '| Scenario | Scope | Want | Choice | P(name) | Original | Proposed |',
             '|---|---|---|---|---:|---|---|']
    for row in data['cases']:
        c = row['case']
        lines.append(f"| {c['id']} | {row['arm']} | {c['want']} | {row['choice']} | "
                     f"{row['probabilities']['name']:.3f} | {original(c)} | {output(row)} |")
    (destination / 'report.md').write_text('\n'.join(lines) + '\n')
    print(destination / 'report.md')


def report():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cases', type=Path, default=HERE / 'two-sentence.json')
    args = parser.parse_args()
    expected = {(r['arm'], r['case']['id']): r for r in jobs(args.cases)}
    datasets = [json.loads((HERE / f'two-sentence-{q}/results.json').read_text()) for q in ('q8', 'q4')]
    indices = []
    for data in datasets:
        index = {(r['arm'], r['case']['id']): r for r in data['cases']}
        assert len(index) == len(data['cases']) == len(expected)
        assert index.keys() == expected.keys()
        for key, row in index.items():
            for field in ('case', 'state', 'questions', 'baseline_sha256'):
                assert row[field] == expected[key][field], (key, field)
        indices.append(index)
    assert datasets[0]['manifest']['repos'] == datasets[1]['manifest']['repos']
    for key in expected:
        for field in ('input_ids', 'marker_pos', 'temperature'):
            assert indices[0][key][field] == indices[1][key][field], (key, field)
    lines = ['# One versus two sentences — d1 Q8 / Q4', '',
             'Extra context helps some repairs but also causes nearby-name contamination '
             'under the existing referent prompt. Both quantizations repair 4/8 newly '
             'disambiguated names, but adding a neighboring name introduces edits in '
             '6/8 ordinary/other-person controls, 1/2 initial controls and both quoted '
             'spelling controls. These preservation failures were absent with the target '
             'sentence alone. The inputs are only 56–97 tokens: capacity is ample; '
             'respecting the marked occurrence and preservation contract is the issue.', '',
             '28 synthetic paired scenarios, 56 decisions per model. Eight names: Gambol, '
             'Rowan, Nell, transcrust, Cigs, Lightning, Mater and Flo. Both preceding and '
             'following context are included. Each inference judges exactly one marked '
             'occurrence; ordinary words and nearby names are never edited together.', '',
             'The eight ambiguous target sentences each appear with two opposing neighbors: '
             'one establishes the vocabulary referent and one establishes an ordinary/other '
             'referent. Eight further controls put vocabulary names near explicit ordinary '
             'uses; four preserve initials or explicitly quoted spellings. Gold labels were '
             'written before inference. No API calls, learned substitutions or live changes.', '',
             '| Category | Model | Scope | Exact decisions | Repairs | Unwanted edits |',
             '|---|---|---|---:|---:|---:|']
    for category in dict.fromkeys(r['case']['category'] for r in expected.values()):
        for data in datasets:
            for scope in ('single', 'two'):
                rows = [r for r in data['cases'] if r['case']['category'] == category and r['arm'] == scope]
                positives = [r for r in rows if r['case']['want'] == 'name']
                preserve = [r for r in rows if r['case']['want'] != 'name']
                exact = sum(r['choice'] == r['case']['want'] for r in rows)
                repairs = sum(r['choice'] == 'name' for r in positives)
                bad = sum(r['choice'] == 'name' and r['case']['heard'] != r['case']['canonical'] for r in preserve)
                lines.append(f"| {category} | {data['quantization'].upper()} | {scope} | {exact}/{len(rows)} | "
                             f'{repairs}/{len(positives)} | {bad}/{len(preserve)} |')
    lines += ['', 'Exact decisions distinguish KEEP from abstention; both preserve the text. '
              'An ambiguous single sentence and its disambiguated two-sentence version can '
              'have different expected answers. No confidence threshold is applied.', '']
    for data, index in zip(datasets, indices):
        good = bad = remain = 0
        for scope, ident in expected:
            if scope != 'two':
                continue
            a, b = index['single', ident], index['two', ident]
            was = a['choice'] == a['case']['want']
            now = b['choice'] == b['case']['want']
            good += not was and now
            bad += was and not now
            remain += now
        lengths = [r['token_count'] for r in data['cases']]
        lines.append(f"{data['quantization'].upper()}: two-sentence exact {remain}/28; "
                     f'{good} scenarios change from incorrect to correct relative to their scope-specific gold, '
                     f'{bad} from correct to incorrect. Complete native inputs use {min(lengths)}–{max(lengths)} tokens.')
    lines += ['', 'This is a small diagnostic, not a trained or calibrated production benchmark. '
              'Counterfactual pairs deliberately reuse target sentences, so decisions are correlated. '
              'No ASR audio is involved. This preserves the existing referent question rather '
              'than evaluating a newly tuned correction prompt; it is not a claim that '
              'every possible two-sentence interface fails. Review the individual '
              'sentences and gold interpretations.', '',
              '## All paired decisions', '',
              '| Scenario | Single sentence / expected | Added context / expected | Q8 single → two | Q4 single → two |',
              '|---|---|---|---|---|']
    for key, job in expected.items():
        scope, ident = key
        if scope != 'single':
            continue
        case = job['case']
        two = expected['two', ident]['case']
        added = two['context'].replace(case['context'], '').strip()
        decisions = []
        for index in indices:
            a, b = index[key], index['two', ident]
            decisions.append(f"{a['choice']} ({a['probabilities']['name']:.3f}) → "
                             f"{b['choice']} ({b['probabilities']['name']:.3f})")
        lines.append(f"| {ident} | {case['context']} / {case['want']} | {added} / {two['want']} | "
                     + ' | '.join(decisions) + ' |')
    lines += ['', 'Parenthesized probabilities are P(name), not P(chosen). '
              '[Q4 full inputs/outputs](two-sentence-q4/report.md) · '
              '[Q8 full inputs/outputs](two-sentence-q8/report.md)', '',
              'Regenerate the saved comparison:', '', '```sh',
              'nix develop .#pair-probe -c python3 tools/pair-router-probe/context_suite.py', '```']
    path = HERE / 'two-sentence-report.md'
    path.write_text('\n'.join(lines) + '\n')
    print('\n'.join(lines[:38]))
    print(path)


if __name__ == '__main__':
    report()
