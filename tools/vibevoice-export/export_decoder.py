#!/usr/bin/env python
"""Export the Qwen2 decoder half of VibeVoice ASR to a KV-cached ONNX graph.

    inputs_embeds  [1, S, 1536]
    attention_mask [1, 1, S, P + S]   additive float mask, 0 / -inf
    position_ids   [1, S]             int64
    past.<i>.key   [1, 2, P, 128]     i in 0..27
    past.<i>.value [1, 2, P, 128]
  ->
    logits         [1, 1, 151936]     last position only
    present.<i>.key / .value  [1, 2, P + S, 128]

Two deliberate interface choices:

* `inputs_embeds`, not `input_ids`. The ASR prompt splices continuous speech
  features into the token embedding sequence, so the embedding lookup has to
  happen outside this graph (see `export_embed.py`).
* A 4-D additive `attention_mask`. Transformers passes a 4-D mask straight
  through `_prepare_4d_causal_attention_mask_with_cache_position`, which keeps
  every mask decision out of the traced graph and in the caller's hands. A 2-D
  mask would bake the trace-time sequence length into the graph.

Only the last position's logits are returned: greedy decode never looks at the
others, and a full [1, S, 151936] prefill output is a 180 MB tensor.
"""

import argparse
import gc
from pathlib import Path

import torch
import torch.nn as nn

from common import LM_PREFIX, checkpoint_dir, load_config, load_tensors, report, strip_prefix

NUM_LAYERS = 28


class DecoderWrapper(nn.Module):
    def __init__(self, language_model, lm_head):
        super().__init__()
        self.language_model = language_model
        self.lm_head = lm_head

    def forward(self, inputs_embeds, attention_mask, position_ids, cache_position, *past):
        from transformers.cache_utils import DynamicCache

        legacy = tuple((past[2 * i], past[2 * i + 1]) for i in range(NUM_LAYERS))
        cache = DynamicCache.from_legacy_cache(legacy)
        out = self.language_model(
            inputs_embeds=inputs_embeds,
            attention_mask=attention_mask,
            position_ids=position_ids,
            past_key_values=cache,
            cache_position=cache_position,
            use_cache=True,
            return_dict=True,
        )
        logits = self.lm_head(out.last_hidden_state[:, -1:, :])
        present = out.past_key_values.to_legacy_cache()
        flat = []
        for key, value in present:
            flat.append(key)
            flat.append(value)
        return (logits, *flat)


def build(ckpt: Path):
    from transformers.models.qwen2.configuration_qwen2 import Qwen2Config
    from transformers.models.qwen2.modeling_qwen2 import Qwen2Model

    config = load_config(ckpt)
    lm_config = Qwen2Config(**config["decoder_config"])
    lm_config._attn_implementation = "eager"
    assert lm_config.num_hidden_layers == NUM_LAYERS, lm_config.num_hidden_layers

    with torch.device("meta"):
        language_model = Qwen2Model(lm_config)
        lm_head = nn.Linear(lm_config.hidden_size, lm_config.vocab_size, bias=False)

    state = load_tensors(
        ckpt, lambda name: name.startswith(LM_PREFIX) or name == "lm_head.weight"
    )
    head_weight = state.pop("lm_head.weight")
    lm_head.load_state_dict({"weight": head_weight}, assign=True)
    language_model.load_state_dict(strip_prefix(state, LM_PREFIX), assign=True)
    del state
    gc.collect()

    # `inv_freq` is a non-persistent buffer, so it is absent from the checkpoint
    # and stays on the meta device after `assign=True`. Rebuild it for real.
    from transformers.models.qwen2.modeling_qwen2 import Qwen2RotaryEmbedding

    language_model.rotary_emb = Qwen2RotaryEmbedding(config=lm_config)
    still_meta = [name for name, t in language_model.named_buffers() if t.is_meta]
    still_meta += [name for name, t in language_model.named_parameters() if t.is_meta]
    assert not still_meta, f"unmaterialised tensors: {still_meta}"

    model = DecoderWrapper(language_model, lm_head).eval()
    for parameter in model.parameters():
        parameter.requires_grad_(False)
    return model, lm_config


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--opset", type=int, default=17)
    args = parser.parse_args()

    ckpt = checkpoint_dir()
    print(f"checkpoint: {ckpt}")
    model, lm_config = build(ckpt)

    head_dim = lm_config.hidden_size // lm_config.num_attention_heads
    kv_heads = lm_config.num_key_value_heads
    # Trace with a non-empty past and a multi-token step so neither the prefill
    # nor the decode shape is special-cased into the graph.
    seq, past_len = 3, 2
    example_embeds = torch.zeros(1, seq, lm_config.hidden_size)
    example_mask = torch.zeros(1, 1, seq, past_len + seq)
    example_positions = torch.arange(past_len, past_len + seq).unsqueeze(0)
    example_cache_position = example_positions[0]
    example_past = []
    for _ in range(NUM_LAYERS):
        example_past.append(torch.zeros(1, kv_heads, past_len, head_dim))
        example_past.append(torch.zeros(1, kv_heads, past_len, head_dim))

    input_names = ["inputs_embeds", "attention_mask", "position_ids", "cache_position"]
    output_names = ["logits"]
    dynamic_axes = {
        "inputs_embeds": {1: "sequence"},
        "attention_mask": {2: "sequence", 3: "total"},
        "position_ids": {1: "sequence"},
        "cache_position": {0: "sequence"},
    }
    for layer in range(NUM_LAYERS):
        for kind in ("key", "value"):
            past_name = f"past.{layer}.{kind}"
            present_name = f"present.{layer}.{kind}"
            input_names.append(past_name)
            output_names.append(present_name)
            dynamic_axes[past_name] = {2: "past"}
            dynamic_axes[present_name] = {2: "total"}

    args.out.parent.mkdir(parents=True, exist_ok=True)
    with torch.no_grad():
        torch.onnx.export(
            model,
            (example_embeds, example_mask, example_positions, example_cache_position, *example_past),
            str(args.out),
            input_names=input_names,
            output_names=output_names,
            dynamic_axes=dynamic_axes,
            opset_version=args.opset,
            do_constant_folding=True,
            dynamo=False,
        )
    report(args.out)


if __name__ == "__main__":
    main()
