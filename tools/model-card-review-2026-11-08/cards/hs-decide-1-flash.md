---
license: apache-2.0
base_model: Qwen/Qwen3.5-0.8B
library_name: transformers
pipeline_tag: text-classification
language:
- en
- es
- fr
- de
- pt
- ja
- ko
- zh
tags:
- decision-model
- reranker
- classification
- agent-memory
- vllm
- hindsight
---

![Hindsight](assets/banner.png)

# hs-decide-1-flash

hs-decide-1-flash is a 0.8B decision model for [Hindsight](https://github.com/vectorize-io/hindsight).
You give it a state and one or more typed questions about it, and it returns
a probability for every answer option in a single forward pass, with no generated text.

## Highlights

- **Typed questions:** `choice` (pick one of 2 to 255 options), `score` (rate on 2 to 10
  ordered levels) and `noul` (yes or no).
- **Probabilities:** a probability for every option, plus a confidence.
- **Many questions per request:** all questions about one state are answered in the same pass.
- **Multilingual:** English, Spanish, French, German, Portuguese, Japanese, Korean and Chinese.
- **Long requests:** trained on requests up to 65,536 tokens.
- **Serves on vLLM** with the endpoint plugin in this repository's `serving/` folder.

## Model overview

| | |
|---|---|
| Model type | Decision model |
| Base model | `Qwen/Qwen3.5-0.8B` |
| Parameters | 0.8B |
| Languages | en, es, fr, de, pt, ja, ko, zh |
| Context length | 65,536 tokens per request (trained) |
| Question types | `choice`, `score`, `noul` |
| Precision | bfloat16 |

## Evaluation

### In Hindsight

![hs-decide-1-flash in Hindsight](assets/in-hindsight.png)

### Against other 0.8B decision models

![hs-decide-1-flash benchmarks](assets/benchmarks.png)

## Usage

### vLLM

The `serving/` folder is a vLLM plugin that adds a `POST /v1/systemone` endpoint to
`vllm serve`.

```bash
hf download hindsight-ai/hs-decide-1-flash --local-dir hs-decide-1-flash
pip install vllm==0.32.0 ./hs-decide-1-flash/serving
hindsight-decision-serve hs-decide-1-flash --port 8000
```

Flags after the model folder go to `vllm serve`, for example `--api-key` or
`--gpu-memory-utilization`.

### Request

```bash
curl 127.0.0.1:8000/v1/systemone \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "hs-decide-1-flash",
    "state": {
      "query": "When is the team offsite?",
      "documents": {
        "D00": "The offsite moved from April 10 to April 17.",
        "D01": "The team offsite is on April 10 in Denver."
      }
    },
    "questions": {
      "latest": {
        "type": "choice",
        "instructions": "Which document gives the current offsite date?",
        "criteria": {"D00": null, "D01": null}
      },
      "moved": {
        "type": "noul",
        "instructions": "Did the offsite date change?"
      },
      "D01": {
        "type": "score",
        "instructions": "How well does document `documents.D01` answer `query`?",
        "criteria": ["about something else", "same topic, no help", "helps", "answers"]
      }
    }
  }'
```

### Response

```json
{
  "model": "hs-decide-1-flash",
  "answers": {
    "latest": {
      "type": "choice",
      "choice": "D00",
      "confidence": 0.91,
      "probabilities": {"D00": 0.95, "D01": 0.05}
    },
    "moved": {"type": "noul", "noul": 0.97},
    "D01": {
      "type": "score",
      "score": 1.4,
      "confidence": 0.62,
      "legend": {"0": "about something else", "1": "same topic, no help", "2": "helps", "3": "answers"},
      "probabilities": {"0": 0.03, "1": 0.55, "2": 0.38, "3": 0.04}
    }
  }
}
```

A request longer than 65,536 tokens returns 400 with `error_type: max_tokens_exceeded`.

### Hindsight

Start the server with `--api-key`, then point Hindsight's decision model at it:

```bash
HINDSIGHT_API_DECISION_MODEL_PROVIDER=hindsight
HINDSIGHT_API_DECISION_MODEL_BASE_URL=http://localhost:8000
HINDSIGHT_API_DECISION_MODEL_API_KEY=<the server's --api-key>
```

### Hindsight Inference

hs-decide-1-flash is also served at `https://inference.hindsight.vectorize.io`. To use it from
Hindsight:

```bash
HINDSIGHT_API_DECISION_MODEL_PROVIDER=hindsight
HINDSIGHT_API_DECISION_MODEL_API_KEY=<your Hindsight Inference API key>
```

## License

Apache 2.0.

## Citation

```bibtex
@misc{hindsightai2026hsdecide1flash,
  title  = {hs-decide-1-flash},
  author = {{Hindsight AI}},
  year   = {2026},
  url    = {https://huggingface.co/hindsight-ai/hs-decide-1-flash}
}
```
