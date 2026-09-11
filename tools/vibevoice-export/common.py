"""Shared plumbing for the VibeVoice ASR ONNX export.

The upstream checkpoint is a 5.6 GB fp32 multi-modal training model. This host
has ~8 GB of RAM free, so nothing here ever instantiates the whole thing: each
graph is built in its own process from just the tensors it needs, read straight
out of the safetensors shards.
"""

import json
import os
import sys
from pathlib import Path

import torch
from safetensors import safe_open

HERE = Path(__file__).resolve().parent

REPO_ID = "microsoft/VibeVoice-ASR-Streaming-1.5B"
UPSTREAM_URL = "https://github.com/microsoft/VibeVoice.git"
UPSTREAM_COMMIT = (HERE / "VIBEVOICE_COMMIT").read_text().strip()


def upstream() -> Path:
    """Path to the upstream `vibevoice` package, cloned on first use.

    Pinned to a commit and fetched rather than copied into this repo: the
    checkpoint ships no modelling code, so the export needs upstream's classes,
    but a 400 KB transplant of someone else's tree does not belong in here.
    """
    root = HERE / "build" / "vibevoice-src"
    if not (root / "vibevoice" / "__init__.py").exists():
        import subprocess

        root.parent.mkdir(parents=True, exist_ok=True)
        if not (root / ".git").exists():
            subprocess.run(["git", "clone", "--filter=blob:none", UPSTREAM_URL, str(root)], check=True)
        subprocess.run(["git", "-C", str(root), "checkout", "--quiet", UPSTREAM_COMMIT], check=True)
    return root


sys.path.insert(0, str(upstream()))

# Only the ASR side of the checkpoint is exported. The acoustic *decoder* and
# the diffusion head are TTS machinery and are never loaded.
SPEECH_PREFIXES = (
    "model.acoustic_tokenizer.encoder.",
    "model.semantic_tokenizer.encoder.",
    "model.acoustic_connector.",
    "model.semantic_connector.",
)
LM_PREFIX = "model.language_model."


def checkpoint_dir() -> Path:
    override = os.environ.get("VIBEVOICE_CHECKPOINT")
    if override:
        return Path(override)
    from huggingface_hub import snapshot_download

    return Path(
        snapshot_download(
            REPO_ID, allow_patterns=["*.json", "*.txt", "*.safetensors"]
        )
    )


def load_config(ckpt: Path) -> dict:
    return json.loads((ckpt / "config.json").read_text())


def shard_index(ckpt: Path) -> dict:
    index = json.loads((ckpt / "model.safetensors.index.json").read_text())
    return index["weight_map"]


def load_tensors(ckpt: Path, predicate) -> dict:
    """Read the subset of checkpoint tensors matching `predicate`, shard by shard."""
    weight_map = shard_index(ckpt)
    wanted = [name for name in weight_map if predicate(name)]
    by_shard: dict[str, list[str]] = {}
    for name in wanted:
        by_shard.setdefault(weight_map[name], []).append(name)

    out: dict[str, torch.Tensor] = {}
    for shard, names in sorted(by_shard.items()):
        with safe_open(ckpt / shard, framework="pt") as handle:
            for name in names:
                out[name] = handle.get_tensor(name).to(torch.float32)
    return out


def strip_prefix(state: dict, prefix: str) -> dict:
    return {k[len(prefix):]: v for k, v in state.items() if k.startswith(prefix)}


def report(model_path: Path) -> None:
    total = model_path.stat().st_size
    data = model_path.with_suffix(model_path.suffix + ".data")
    if data.exists():
        total += data.stat().st_size
    print(f"  wrote {model_path.name}: {total / 1e6:.1f} MB")
