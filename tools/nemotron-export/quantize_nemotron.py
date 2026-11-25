"""NeMo export -> the layout transcrust and mopvox load, int4 encoder by the installed recipe.

The installed 1120 ms asset's own quantize.py: MatMulNBitsQuantizer bits=4 block_size=128, encoder
MatMuls only. The decoder stays as exported. filterbank.bin, preprocessor.config and tokens.txt
come from the installed asset, the same base model's frontend and tokenizer.

  nix develop -c .venv/bin/python quantize_nemotron.py --src build/r6 --dst build/install/...-560ms-int4
"""
import argparse
import shutil
from pathlib import Path

import onnx
from onnxruntime.quantization.matmul_nbits_quantizer import MatMulNBitsQuantizer

INSTALLED = Path.home() / "syncthing/models/nemotron-speech-en-0.6b-onnx-1120ms-int4"
p = argparse.ArgumentParser()
p.add_argument("--src", required=True)
p.add_argument("--dst", required=True)
p.add_argument("--symmetric", action="store_true")
a = p.parse_args()
src, dst = Path(a.src), Path(a.dst)
dst.mkdir(parents=True, exist_ok=True)

q = MatMulNBitsQuantizer(onnx.load(str(src / "encoder-nemotron.onnx")), bits=4, block_size=128,
                         is_symmetric=a.symmetric)
q.process()
onnx.save(q.model.model, str(dst / "encoder_model.onnx"), save_as_external_data=True,
          all_tensors_to_one_file=True, location="encoder_model.onnx.data")
onnx.save(onnx.load(str(src / "decoder_joint-nemotron.onnx")), str(dst / "decoder_model.onnx"),
          save_as_external_data=True, all_tensors_to_one_file=True, location="decoder_model.onnx.data")
for f in ("filterbank.bin", "preprocessor.config", "tokens.txt"):
    shutil.copy(INSTALLED / f, dst / f)
print(dst.name, sorted(f.name for f in dst.iterdir()))
