"""Export nvidia/nemotron-speech-streaming-en-0.6b to ONNX at one right context.

NeMo's own exporter, nothing custom: load the .nemo, pick the right context the
card offers ({0, 1, 6, 13} frames of 80 ms, left fixed at 70), turn on cache
support, export. One checkpoint gives every chunk size, so a 560 vs 1120 ms
comparison differs only in the chunk (11-24).

  nix develop -c .venv/bin/python export_nemotron.py --nemo <path.nemo> --right 6 --out build/r6
"""
import argparse
from pathlib import Path

import nemo.collections.asr as nemo_asr

p = argparse.ArgumentParser()
p.add_argument("--nemo", required=True)
p.add_argument("--right", type=int, choices=[0, 1, 6, 13], required=True)
p.add_argument("--out", required=True)
a = p.parse_args()
out = Path(a.out)
out.mkdir(parents=True, exist_ok=True)

model = nemo_asr.models.ASRModel.restore_from(a.nemo, map_location="cpu").eval()
model.encoder.set_default_att_context_size([70, a.right])
model.set_export_config({"cache_support": "True"})
model.export(str(out / "nemotron.onnx"))
print("chunk ms:", (a.right + 1) * 80, "->", sorted(f.name for f in out.iterdir()))
