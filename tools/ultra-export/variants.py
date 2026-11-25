"""Quantization variants of the Ultra encoder, one change each, for the 11-24 study.

Every variant starts from build/encoder-model.onnx (fp32) and keeps the same int8 joint, so
only the encoder changes. Scored against fp32 on Record-2 and the decision-test takes.

  nix develop -c .venv/bin/python variants.py --build build --out variants
"""
import argparse
import collections
import shutil
from pathlib import Path

import onnx
from onnxruntime.quantization.matmul_nbits_quantizer import (
    HQQWeightOnlyQuantConfig,
    MatMulNBitsQuantizer,
)

BLOCK = 128


def save(model, path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    onnx.save(model, str(path))
    ops = collections.Counter(n.op_type for n in onnx.load(str(path), load_external_data=False).graph.node)
    print(f"{path.parent.name}: {path.stat().st_size / 1e6:.0f} MB, MatMulNBits {ops['MatMulNBits']}, MatMul {ops['MatMul']}")


def set_accuracy_level(model, level: int):
    for node in model.graph.node:
        if node.op_type == "MatMulNBits":
            for a in list(node.attribute):
                if a.name == "accuracy_level":
                    node.attribute.remove(a)
            node.attribute.append(onnx.helper.make_attribute("accuracy_level", level))
    return model


def rtn(src: Path, exclude=()):
    q = MatMulNBitsQuantizer(onnx.load(str(src)), bits=4, block_size=BLOCK, is_symmetric=False,
                             nodes_to_exclude=list(exclude))
    q.process()
    return q.model.model


def hqq(src: Path):
    q = MatMulNBitsQuantizer(onnx.load(str(src)), algo_config=HQQWeightOnlyQuantConfig(block_size=BLOCK, bits=4))
    q.process()
    return q.model.model


def edge_layers(src: Path) -> list[str]:
    """Weight MatMuls in the first and last Conformer layers, left unquantized for variant E."""
    g = onnx.load(str(src), load_external_data=False).graph
    weights = {i.name for i in g.initializer}
    layers = sorted({int(p.split(".")[1]) for n in g.node for p in n.name.replace("/", " ").split()
                     if p.startswith("layers.") and p.split(".")[1].isdigit()})
    if not layers:
        # The dynamo export names nodes node_linear_N in graph order: the subsampling
        # projection, then 24 layers of 8 weight MatMuls. Take the first 1 + 8 and the last 8.
        mm = [n.name for n in g.node if n.op_type == "MatMul" and any(i in weights for i in n.input)]
        per = (len(mm) - 1) // 24
        return mm[: 1 + per] + mm[-per:]
    edge = {f"layers.{layers[0]}", f"layers.{layers[-1]}"}
    return [n.name for n in g.node if n.op_type == "MatMul" and any(i in weights for i in n.input)
            and any(e + "/" in n.name or e + "." in n.name for e in edge)]


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("--build", default="build")
    p.add_argument("--out", default="variants")
    a = p.parse_args()
    build, out = Path(a.build), Path(a.out)
    src = build / "encoder-model.onnx"
    shared = ["decoder_joint-model.int8.onnx", "vocab.txt"]
    nemo128 = build / "install" / "parakeet-ultra-0.6b-int4" / "nemo128.onnx"

    def dir_for(name: str) -> Path:
        d = out / name
        d.mkdir(parents=True, exist_ok=True)
        for f in shared:
            shutil.copy(build / f, d / f)
        shutil.copy(nemo128, d / "nemo128.onnx")
        return d

    b = rtn(src)
    save(b, dir_for("B-rtn") / "encoder-model.int4.onnx")
    save(set_accuracy_level(rtn(src), 4), dir_for("C-rtn-w4a8") / "encoder-model.int4.onnx")
    save(hqq(src), dir_for("D-hqq") / "encoder-model.int4.onnx")
    edge = edge_layers(src)
    print(f"E keeps {len(edge)} edge-layer MatMuls at full precision")
    save(rtn(src, exclude=edge), dir_for("E-mixed") / "encoder-model.int4.onnx")


if __name__ == "__main__":
    main()
