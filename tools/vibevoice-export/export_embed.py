#!/usr/bin/env python
"""Export the token-embedding lookup as a small, int8-quantised ONNX graph.

    input_ids [1, S] int64  ->  inputs_embeds [1, S, 1536] float32

The decoder graph takes `inputs_embeds`, so the 151936 x 1536 table has to be
its own graph. At fp32 that table is 933 MB and at fp16 it is 466 MB, both of
which dominate the whole install; per-row symmetric int8 brings it to 233 MB
with negligible error, because each row gets its own scale.

`lm_head.weight` and `model.language_model.embed_tokens.weight` are tied in the
config and stored twice in the checkpoint. This asserts they really are equal
and keeps one copy.
"""

import argparse
from pathlib import Path

import numpy as np
import torch
from onnx import TensorProto, helper, numpy_helper, save_model

from common import checkpoint_dir, load_tensors, report


def quantize_rows(table: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    """Symmetric per-row int8. Rows of an embedding table have wildly different
    norms, so a single global scale would flatten the rare ones."""
    scale = np.abs(table).max(axis=1, keepdims=True) / 127.0
    scale[scale == 0] = 1.0
    quantized = np.rint(table / scale).clip(-127, 127).astype(np.int8)
    return quantized, scale.astype(np.float32)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--opset", type=int, default=17)
    args = parser.parse_args()

    ckpt = checkpoint_dir()
    state = load_tensors(
        ckpt,
        lambda name: name in ("lm_head.weight", "model.language_model.embed_tokens.weight"),
    )
    head = state["lm_head.weight"]
    embed = state["model.language_model.embed_tokens.weight"]
    assert torch.equal(head, embed), "lm_head and embed_tokens are not tied as the config claims"
    table = embed.numpy()
    vocab, hidden = table.shape
    print(f"embedding table: {vocab} x {hidden}")

    quantized, scale = quantize_rows(table)
    error = np.abs(quantized.astype(np.float32) * scale - table).max()
    print(f"  max absolute quantisation error: {error:.3e}")

    graph = helper.make_graph(
        nodes=[
            helper.make_node("Gather", ["embed_q", "input_ids"], ["rows_i8"], axis=0),
            helper.make_node("Cast", ["rows_i8"], ["rows_f32"], to=TensorProto.FLOAT),
            helper.make_node("Gather", ["embed_scale", "input_ids"], ["row_scale"], axis=0),
            helper.make_node("Mul", ["rows_f32", "row_scale"], ["inputs_embeds"]),
        ],
        name="embed_tokens",
        inputs=[
            helper.make_tensor_value_info(
                "input_ids", TensorProto.INT64, ["batch", "sequence"]
            )
        ],
        outputs=[
            helper.make_tensor_value_info(
                "inputs_embeds", TensorProto.FLOAT, ["batch", "sequence", hidden]
            )
        ],
        initializer=[
            numpy_helper.from_array(quantized, "embed_q"),
            numpy_helper.from_array(scale, "embed_scale"),
        ],
    )
    model = helper.make_model(
        graph, opset_imports=[helper.make_opsetid("", args.opset)], producer_name="transcrust"
    )
    model.ir_version = 10

    args.out.parent.mkdir(parents=True, exist_ok=True)
    save_model(
        model,
        str(args.out),
        save_as_external_data=True,
        location=args.out.name + ".data",
        all_tensors_to_one_file=True,
        size_threshold=1024,
    )
    report(args.out)


if __name__ == "__main__":
    main()
