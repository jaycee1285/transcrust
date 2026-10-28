#!/usr/bin/env python3
"""Fetch only d1's quantized text bundle and prompt references at pinned HF revisions."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import urllib.request


def fetch(url, path):
    partial = path.with_suffix(path.suffix + '.partial')
    with urllib.request.urlopen(url, timeout=90) as response, partial.open('wb') as out:
        shutil.copyfileobj(response, out)
    partial.replace(path)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--quantization', choices=['q8', 'q4'], default='q8')
    parser.add_argument('--model-dir', type=Path)
    parser.add_argument('--reference-dir', type=Path, help='Reuse pinned revisions and shared files from a verified bundle')
    args = parser.parse_args()
    args.model_dir = args.model_dir or Path('/tmp/transcrust-d1-omni-600M-' + args.quantization)
    args.model_dir.mkdir(parents=True, exist_ok=True)
    manifest_path = args.model_dir / 'manifest.json'
    manifest = json.loads(manifest_path.read_text()) if manifest_path.exists() else {'repos': {}, 'files': {}}
    reference = json.loads((args.reference_dir / 'manifest.json').read_text()) if args.reference_dir else None
    if reference:
        if manifest['repos'] and manifest['repos'] != reference['repos']:
            raise ValueError('Cached and reference revisions differ')
        manifest['repos'] = reference['repos'].copy()
    graph = 'model_quantized.onnx' if args.quantization == 'q8' else 'model_q4.onnx'
    bundles = {
        'onnx-community/d1-omni-600M-ONNX': ['onnx/' + graph,
            'onnx/' + graph + '_data', 'tokenizer.json', 'tokenizer_config.json',
            'config.json', 'LICENSE', 'README.md'],
        'LiquidAI/d1-omni-600M': ['prompt.py', 'README.md'],
    }
    for repo, files in bundles.items():
        if repo not in manifest['repos']:
            with urllib.request.urlopen('https://huggingface.co/api/models/' + repo, timeout=30) as response:
                manifest['repos'][repo] = json.load(response)['sha']
            manifest_path.write_text(json.dumps(manifest, indent=2) + '\n')
        revision = manifest['repos'][repo]
        for name in files:
            local_name = 'original-README.md' if repo.startswith('LiquidAI/') and name == 'README.md' else name
            path = args.model_dir / local_name
            path.parent.mkdir(parents=True, exist_ok=True)
            if reference and local_name in reference['files'] and not path.exists():
                shutil.copyfile(args.reference_dir / local_name, path)
                manifest['files'][local_name] = reference['files'][local_name]
            if not path.exists() or local_name not in manifest['files']:
                print(f'Downloading {repo}@{revision[:8]}/{name}', flush=True)
                fetch(f'https://huggingface.co/{repo}/resolve/{revision}/{name}', path)
            with path.open('rb') as source:
                digest = hashlib.file_digest(source, 'sha256').hexdigest()
            previous = manifest['files'].get(local_name)
            if previous and previous['sha256'] != digest:
                raise ValueError(f'Cached file checksum changed: {local_name}')
            manifest['files'][local_name] = {'repo': repo, 'path': name,
                'bytes': path.stat().st_size, 'sha256': digest}
            manifest_path.write_text(json.dumps(manifest, indent=2) + '\n')
    print(f'Text-only bundle verified: {args.model_dir}', flush=True)


if __name__ == '__main__':
    main()
