#!/usr/bin/env python
"""Quantise the exported graphs down to something a laptop will actually hold.

The exported fp32 graphs total ~8.5 GB. Two passes bring that down:

* `--mode int4` runs `MatMulNBitsQuantizer` (block-wise RTN, block 32) over
  every MatMul. That is where nearly all the weight lives: the Qwen2 projections
  and the two conv encoders' FFN blocks.
* `--mode int8` runs `quantize_dynamic` instead, for a size/accuracy trade in
  the other direction.

Conv weights are left alone in int4 mode on purpose. ORT's dynamic path turns
Conv into ConvInteger, which on CPU is routinely *slower* than the fp32 kernel,
and the convs are a minority of the encoder's parameters.
"""

import argparse
import shutil
from pathlib import Path

import onnx
from onnxruntime.quantization import quantize_dynamic, QuantType
from onnxruntime.quantization.matmul_nbits_quantizer import MatMulNBitsQuantizer


def size_of(path: Path) -> int:
    total = path.stat().st_size
    for sibling in path.parent.glob(path.name + ".data"):
        total += sibling.stat().st_size
    return total


def int4(src: Path, dst: Path, block_size: int, accuracy_level: int) -> None:
    model = onnx.load(str(src), load_external_data=True)
    # No `algo_config`: that route wants neural-compressor. The default
    # DefaultWeightOnlyQuantizer is ORT's own round-to-nearest and needs nothing
    # beyond onnxruntime itself.
    quantizer = MatMulNBitsQuantizer(
        model,
        block_size=block_size,
        is_symmetric=True,
        accuracy_level=accuracy_level,
    )
    quantizer.process()
    dst.parent.mkdir(parents=True, exist_ok=True)
    quantizer.model.save_model_to_file(str(dst), use_external_data_format=True)


def int8(src: Path, dst: Path) -> None:
    dst.parent.mkdir(parents=True, exist_ok=True)
    quantize_dynamic(
        str(src),
        str(dst),
        weight_type=QuantType.QInt8,
        op_types_to_quantize=["MatMul"],
        extra_options={"MatMulConstBOnly": True},
        use_external_data_format=True,
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--src", type=Path, required=True)
    parser.add_argument("--dst", type=Path, required=True)
    parser.add_argument("--mode", choices=["int4", "int8", "copy"], default="int4")
    parser.add_argument("--block-size", type=int, default=32)
    # accuracy_level 4 asks MatMulNBits to accumulate in int8, which is the
    # difference between a usable and an unusable decode rate on CPU.
    parser.add_argument("--accuracy-level", type=int, default=4)
    args = parser.parse_args()

    print(f"{args.mode}: {args.src.name} -> {args.dst.name}")
    before = size_of(args.src)
    if args.mode == "int4":
        int4(args.src, args.dst, args.block_size, args.accuracy_level)
    elif args.mode == "int8":
        int8(args.src, args.dst)
    else:
        args.dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy(args.src, args.dst)
        data = args.src.parent / (args.src.name + ".data")
        if data.exists():
            shutil.copy(data, args.dst.parent / (args.dst.name + ".data"))
    after = size_of(args.dst)
    print(f"  {before / 1e6:.0f} MB -> {after / 1e6:.0f} MB")


if __name__ == "__main__":
    main()
