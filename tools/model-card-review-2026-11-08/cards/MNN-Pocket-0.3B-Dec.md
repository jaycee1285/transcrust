---
license: apache-2.0
language:
- en
- zh
base_model: taobao-mnn/MNN-Pocket-0.3B
tags:
- mnn
- mnn-pocket
- on-device
- decision-model
- non-autoregressive
- lora
- peft
---

<div align="center">
  <img src="https://cdn-uploads.huggingface.co/production/uploads/6881be3a7d8f27f6508ef8de/hPDaFYzlZb3m07kc8a4ho.png" alt="MNN-Pocket" style="width: 100%; max-width: 100%; height: auto; display: inline-block; margin-bottom: 0.5em; margin-top: 0.5em;" />
  <div style="display: flex; justify-content: center; gap: 0.5em; margin-bottom: 1em;">
    <a href="https://huggingface.co/taobao-mnn/MNN-Pocket-0.3B"><strong>Main Model</strong></a> •
    <a href="https://huggingface.co/collections/taobao-mnn/mnn-pocket-03b"><strong>Collection</strong></a> •
    <a href="https://github.com/alibaba/MNN"><strong>GitHub (MNN)</strong></a> •
    <a href="https://huggingface.co/taobao-mnn/MNN-Pocket-0.3B/resolve/main/MNN-Pocket-0.3B-Technical-Report.pdf"><strong>Tech Report</strong></a>
  </div>
</div>

<p align="center">
English |
<a href="https://huggingface.co/taobao-mnn/MNN-Pocket-0.3B-Dec/blob/main/README-cn.md" target="_blank">中文</a>
</p>

# MNN-Pocket-0.3B-Dec

A decision model using Transformers and PEFT, built on [MNN-Pocket-0.3B](https://huggingface.co/taobao-mnn/MNN-Pocket-0.3B). It takes a `state` and `questions` and returns decisions with candidate probabilities in a single forward pass. The full backbone and `lm_head` are retained, with a separate LoRA adapter.


## Model Details

| Property          | Value                                                                                     |
|-------------------|-------------------------------------------------------------------------------------------|
| Role              | Non-autoregressive decision model (not a chat model)                                      |
| Backbone          | MNN-Pocket-0.3B, decoder-only transformer, 28 layers — bundled under `base/`              |
| Hidden size       | 896                                                                                       |
| Attention         | 14 heads, GQA with 2 KV heads, head_dim 64                                                |
| FFN intermediate  | 2560 (SiLU)                                                                               |
| Adapter           | LoRA r=16, alpha=32, dropout=0.05, targets q/k/v/o/gate/up/down_proj, **unmerged**         |
| Decision head     | Pointer head, projection width 256, FP32, scaled dot-product over candidate positions      |
| Request length    | 32,768 tokens; longer input raises instead of truncating                                   |
| Vocabulary        | 65,536                                                                                    |
| Precision         | bfloat16 weights; head kept in float32; inference dtype float32 / bfloat16 / float16       |
| Size              | ~300M backbone (723 MB) + 28.7 MB adapter + 1.8 MB head                                    |
| Languages         | Chinese, English                                                                          |

The bundled backbone is **byte-identical** to
[MNN-Pocket-0.3B](https://huggingface.co/taobao-mnn/MNN-Pocket-0.3B) (same `model.safetensors`
checksum), which is what the decision model was initialised from.

## Files

| Path                             | Size    | Purpose                                                     |
|----------------------------------|---------|-------------------------------------------------------------|
| `inference.py`                   | 17 KB   | Loader, `PocketDecisionModel` API and CLI                   |
| `decision_config.json`           | 690 B   | Layout, pointer width, request limit, control-token IDs     |
| `temperature.json`               | 418 B   | Calibrated temperature and fit statistics                   |
| `decision_head.safetensors`      | 1.8 MB  | Pointer head weights (FP32)                                 |
| `adapter/adapter_model.safetensors` | 28.7 MB | LoRA adapter weights                                     |
| `adapter/adapter_config.json`    | 1.2 KB  | PEFT LoRA configuration                                     |
| `base/`                          | 723 MB  | Full backbone, tokenizer and chat template                  |
| `requirements.txt`               | 99 B    | Pinned versions used for this release                       |

`base/` has to stay inside the model directory: the loader rejects any configured path that escapes it.

## 🏃 How to run

```python
from inference import PocketDecisionModel

model = PocketDecisionModel.from_pretrained(".", device="cpu", dtype="float32")
result = model.predict(
    state=(
        "The shoes arrived, but they are the wrong size. "
        "I want to send them back for an exchange. "
        "Payment and delivery were fine."
    ),
    questions={
        "department": {
            "type": "choice",
            "instructions": "Which team should handle this customer request?",
            "criteria": {
                "returns": "Returns, exchanges and refunds",
                "billing": "Charges, invoices and payment problems"
            }
        }
    }
)
print(result)
```

## Question Types

| Type     | `criteria`                            | Extra fields in the answer                                  |
|----------|---------------------------------------|-------------------------------------------------------------|
| `choice` | object mapping candidate → description | `choice`: the winning candidate key                          |
| `noul`   | optional object with `false` / `true`  | `decision`: bool, `noul`: probability of `true`              |
| `score`  | ordered list of scale labels           | `score`: probability-weighted index, `legend`: index → label |

Every answer carries `type` and `probabilities` (a dict over candidate keys). The top-level result also
reports `input_tokens` (length of the encoded request) and the `temperature` actually applied.

## Input Handling

- **No chat template.** The request is serialised by `inference.py` itself: the state comes first, then
  one branch per question with its instructions and candidates. Insertion order of `questions` is
  preserved, and each branch is attention-isolated from the others.
- **Control sequences in user text are escaped** before tokenization, so a `state` or `criteria` string
  cannot inject the reserved markers that delimit states, questions, candidates and decisions.
- **Control-token IDs are verified at load time**: the tokenizer must map the reserved markers to exactly
  the IDs recorded in `decision_config.json`, otherwise loading fails.
- **Strict validation**: unsupported JSON types, non-finite numbers, an empty question set, an unknown
  question type or more than 255 candidates all raise instead of being silently repaired.

## Probability Calibration

Decision logits are divided by a single calibrated temperature before the softmax. It was fitted on a
600-question calibration partition by bounded golden-section search on log temperature (search range
0.05–20.0), minimising pooled question NLL:

| Metric                | Value  |
|-----------------------|--------|
| Temperature           | 1.1112 |
| NLL before calibration | 0.8598 |
| NLL after calibration  | 0.8576 |

Pass `temperature=` to `predict()` (or `--temperature` on the CLI) to override it; the value used is
echoed back in the result.

## Other Formats

- [MNN-Pocket-0.3B](https://huggingface.co/taobao-mnn/MNN-Pocket-0.3B) — BF16 instruction-tuned backbone, full model card (723 MB)
- [MNN-Pocket-0.3B-MNN](https://huggingface.co/taobao-mnn/MNN-Pocket-0.3B-MNN) — INT4 W4A16 QAT build for on-device deployment (173 MB)
- [MNN-Pocket-0.3B-GGUF](https://huggingface.co/taobao-mnn/MNN-Pocket-0.3B-GGUF) — F16 / Q8_0 (BF16) and Q4_1 (QAT) for llama.cpp
- [MNN-Pocket-0.3B-DFlash2](https://huggingface.co/taobao-mnn/MNN-Pocket-0.3B-DFlash2) — speculative-decoding draft model
- [MNN-Pocket-0.3B-Base](https://huggingface.co/taobao-mnn/MNN-Pocket-0.3B-Base) / [MNN-Pocket-0.3B-SFT](https://huggingface.co/taobao-mnn/MNN-Pocket-0.3B-SFT) — pre-training / SFT checkpoints

## License

Apache License 2.0. See `LICENSE.txt`.

