---
model_name: GLiDE-0.4B
license: other
library_name: transformers
tags:
- decision-model
- classification
license_name: fastino-community
license_link: https://huggingface.co/fastino/GLiDE-0.4B/blob/main/LICENSE
---

<a href="https://fastino.ai" target="_blank" rel="noopener noreferrer">
  <img src="assets/fastino-glide-banner.svg" alt="Fastino Labs — GLiDE: decisions in a single forward pass" width="100%" />
</a>

# GLiDE 0.4B

**Inference provider: [Fastino](https://fastino.ai)** — use GLiDE through Fastino's hosted API (`POST https://api.fastino.ai/v1/systemone`, model `fastino/GLiDE`). [Get an API key](https://agent.fastino.ai) · [API documentation](https://docs.fastino.ai/inference/systemone)

GLiDE 0.4B is a GLiDE decision model with 0.40B parameters. It is an encoder model, served by the `glide` package included in
this repository (the same Python API and HTTP server as the other GLiDE models). It scores 51.10 on the Decision Index 0.3.1 public suite,
measured with the official kit on one RTX PRO 6000, at a median latency of 6.9 ms per request on that GPU. Weights and compute are bf16.

GLiDE is a decision model from <a href="https://fastino.ai" target="_blank" rel="noopener noreferrer">Fastino</a>. Given a state (text or JSON) and one or more typed questions, it returns a
probability for every option of every question.

## Fastino API

Use GLiDE through <a href="https://docs.fastino.ai/inference/systemone" target="_blank" rel="noopener noreferrer">Fastino’s hosted API</a> for classification, routing, and scoring.
Send decision requests to `POST https://api.fastino.ai/v1/systemone` with model `fastino/GLiDE`.
<a href="https://agent.fastino.ai" target="_blank" rel="noopener noreferrer">Create an account and get an API key</a>, or follow the <a href="https://docs.fastino.ai/quickstart" target="_blank" rel="noopener noreferrer">API quickstart</a>.
The instructions below cover running this repository locally.

## Overview

- **Typed decisions:** `choice` (one of up to 255 named options), `noul` (yes / no, returns P(yes)) and `score` (ordered levels).
- **Probabilities for every option,** read from a 255-way decision readout. Nothing is generated.
- **One forward pass per question.** All questions of a request (and of concurrent requests) are scored together in micro-batches.
- **Text and JSON:** the state can be a string or any JSON value. This model has no vision encoder; requests with images are rejected.
- **Context:** up to 7,999 tokens per question prompt (state, question and options). Longer prompts are shortened by keeping their
  first 1,024 tokens and their end (the question and options), so they are still answered.
- **Interfaces:** a Python API and an HTTP server speaking the System One format (`POST /v1/systemone`).

## Hardware and memory

| | |
|---|---|
| Parameters | 0.40 B (0.39 B encoder, of which 52 M token embeddings; 0.26 M decision readout) |
| Weights | bf16, 0.81 GB on disk; loaded and run in bf16 |
| GPU memory | 0.79 GB after loading; 1.77 GB peak over the mixed requests of the latency test below |
| Tested GPUs | NVIDIA RTX PRO 6000 Blackwell Server Edition (96 GB) |
| Load time | 16 s from network storage |

Weights are copied to the GPU tensor by tensor, so host memory use during loading stays low.

## Install

```bash
pip install huggingface_hub
hf download fastino/GLiDE-0.4B --local-dir GLiDE-0.4B
cd GLiDE-0.4B
pip install -r requirements.txt
```

- Tested with Python 3.12, `torch==2.14.0` (CUDA 13.0 wheels) and `transformers==5.17.0` (all pins are in `requirements.txt`; the
  `flash-linear-attention` pin is not used by this model and can be skipped).
- The `glide/` package lives at the repository root: run from the repository directory or add it to `PYTHONPATH`.

## Quickstart

### Python

```python
from glide import Glide

model = Glide.from_pretrained("GLiDE-0.4B")   # local directory, or the Hub repo id "fastino/GLiDE-0.4B"

result = model.decide(
    state={"ticket": "I was charged twice for order #4821 and the second charge is still pending.", "customer_tier": "gold"},
    questions={
        "intent": {"type": "choice", "instructions": "What does the customer want?",
                   "criteria": {"refund": "Get money back", "cancel": "Cancel the order",
                                "track": "Track a delivery", "other": "Something else"}},
        "urgent": {"type": "noul", "instructions": "Should this ticket be escalated today?"},
        "priority": {"type": "score", "instructions": "What support priority should this ticket get?",
                     "criteria": ["low: no impact on the customer", "medium: the customer is inconvenienced",
                                  "high: the customer has lost money or cannot use the service"]},
    },
)
```

Result on an RTX PRO 6000 (rounded):

```json
{
  "model": "glide-0.4b",
  "answers": {
    "intent": {
      "type": "choice",
      "choice": "refund",
      "confidence": 0.694,
      "probabilities": {
        "refund": 0.784,
        "cancel": 0.089,
        "track": 0.043,
        "other": 0.084
      }
    },
    "urgent": {
      "type": "noul",
      "noul": 0.595,
      "confidence": 0.189
    },
    "priority": {
      "type": "score",
      "score": 2,
      "expected_level": 1.624,
      "confidence": 0.345,
      "probabilities": {
        "0": 0.032,
        "1": 0.311,
        "2": 0.656
      }
    }
  },
  "usage": {
    "input_tokens": 395,
    "output_tokens": 0
  }
}
```

`model.decide_batch([{"state": ..., "questions": ...}, ...])` scores several requests together and returns one item per request: a
response dict, or a `glide.RequestError` for a request that cannot be served (`decide` raises it instead).

### HTTP server

```bash
GLIDE_MODEL=/models/GLiDE-0.4B uvicorn glide.server:app --host 0.0.0.0 --port 8000
# or
python -m glide.server --model /models/GLiDE-0.4B --port 8000
```

The model loads at startup; `GET /health` answers once it is ready.

```bash
curl -s localhost:8000/v1/systemone -H 'Content-Type: application/json' -d '{
  "model": "glide-0.4b",
  "state": "Order #4821 was delivered to the wrong address and the customer wants it re-sent today.",
  "questions": {
    "action": {"type": "choice", "instructions": "What should support do next?",
               "criteria": {"resend": "Re-ship the order", "refund": "Refund the order", "investigate": "Open a carrier investigation"}},
    "urgent": {"type": "noul", "instructions": "Is the request time-sensitive?"}
  }
}'
```

#### Request

| Field | Type | Description |
|---|---|---|
| `model` | string | Optional. Ignored; the response reports the served model name (`GLIDE_MODEL_NAME`, default `glide-0.4b`). |
| `state` | string or JSON value | The context to decide on. Strings are used as is; other JSON values are serialized. |
| `questions` | object | `{key: question}`. Every question is answered independently, one forward pass each. |

Question fields:

| Field | Description |
|---|---|
| `type` | `choice`, `noul` or `score`. |
| `instructions` | The question. A string; objects are rendered as `key: value` lines. |
| `criteria` | **choice:** object `{option_key: description}`, 1 to 255 options, shown in the given order; descriptions may be strings or JSON (an empty description shows only the key). **noul:** optional `{"true": "...", "false": "..."}` descriptions (default Yes / No). **score:** a list of level descriptions (level *i* is index *i*), or an object keyed by numeric level (sorted ascending and re-indexed from 0). |

#### Response

For the curl request above (RTX PRO 6000, rounded; the HTTP response also carries `latency_ms`):

```json
{
  "model": "glide-0.4b",
  "answers": {
    "action": {
      "type": "choice",
      "choice": "resend",
      "confidence": 0.732,
      "probabilities": {
        "resend": 0.85,
        "refund": 0.033,
        "investigate": 0.117
      }
    },
    "urgent": {
      "type": "noul",
      "noul": 0.978,
      "confidence": 0.956
    }
  },
  "usage": {
    "input_tokens": 227,
    "output_tokens": 0
  }
}
```

| Answer type | Fields |
|---|---|
| `choice` | `choice`: most probable key. `probabilities`: `{key: p}` over all options, summing to 1. `confidence`: top minus second probability. |
| `noul` | `noul`: P(true). `confidence`: \|2 P(true) - 1\|. |
| `score` | `score`: most probable level index. `expected_level`: sum of index x probability. `probabilities`: `{"0": p, "1": p, ...}`. `confidence`: top minus second probability. |

`usage.input_tokens` is the prompt length summed over the request's questions. `latency_ms` is the server-side time for the request
(queueing plus inference).

Errors return `{"error": "..."}`: 422 for requests the model cannot serve (see below), 401 for a missing or wrong API key, 503 when the
queue is full, 504 on timeout, 500 for an internal error.

## Production notes

**Process model.** Run one server process per GPU (`workers=1`) and scale out with replicas behind a load balancer. A single worker
thread owns the GPU; HTTP handler threads only queue requests and wait for results.

**Batching and concurrency** (environment variables):

| Variable | Default | Meaning |
|---|---|---|
| `GLIDE_MODEL` | required | Model directory or Hub repo id. |
| `GLIDE_MODEL_NAME` | `glide-0.4b` | `model` field in responses. |
| `GLIDE_DEVICE` | `cuda` | Torch device, e.g. `cuda:1`. |
| `GLIDE_MAX_BATCH` | 16 | Max requests scored together in one micro-batch. |
| `GLIDE_BATCH_WAIT_MS` | 5 | How long the worker waits to fill a micro-batch. |
| `GLIDE_TOKEN_BUDGET` | 16384 | Max padded tokens per forward pass; question prompts are length-sorted and packed under it (a longer prompt runs alone). |
| `GLIDE_MAX_LEN` | 40000 | Upper bound on prompt length accepted by the server; this model itself shortens prompts above 7,999 tokens (see Overview). |
| `GLIDE_MAX_QUEUE` | 1024 | Queued requests before new ones get 503. |
| `GLIDE_REQUEST_TIMEOUT_S` | 600 | Max wait for a result before 504. |
| `GLIDE_THREADS` | 1024 | HTTP handler threads (concurrent open requests). |
| `GLIDE_API_KEY` | unset | If set, require `Authorization: Bearer <key>`. |

**Max context.** 7,999 tokens per question prompt; longer prompts are shortened as described in the Overview. Attention cost grows with
length, so long prompts dominate a micro-batch; set client timeouts with headroom.

**Health.** `GET /health` returns `{"status": "ok", ...}` with the queue length and batch counters once the model is loaded.

**Latency** (one RTX PRO 6000 Blackwell, warm, one request at a time; 750 requests of mixed length with 2.3 questions per request on
average):

| Interface | Median | Mean | p90 | p99 |
|---|---|---|---|---|
| Python API (in-process) | 6.9 ms | 12.1 ms | 24.8 ms | 85.1 ms |
| HTTP server (client wall time) | 9.3 ms | 14.2 ms | 27.3 ms | 87.1 ms |

Latency grows with prompt length and the number of questions.

**Throughput.** Scoring 1,500 mixed requests (6,596 questions) in token-budget micro-batches (16,384 tokens) on one RTX PRO 6000:
188 questions/s. Run one process per GPU and add replicas for more throughput.

**Cold start.** The first requests of a fresh process are slower while kernels warm up; send a few representative requests before
putting a replica into rotation.

**Unsupported requests** get HTTP 422 with a message:

- more than 255 options: `options per choice exceeds the label set (N > 255)`
- images: `images are not supported by this deployment`
- an unknown question type, missing `state`, or malformed `questions` / `criteria`

A request with an empty `questions` object returns 200 with empty `answers`.

**Determinism.** There is no sampling. The same request scored alone gives identical results on the same GPU type and software
stack. The questions of one request always share forward passes; when concurrent requests are also micro-batched, padding changes the
shape of the computation and probabilities can move slightly. Set `GLIDE_MAX_BATCH=1` when results must not depend on other traffic.

**Precision.** Weights and compute are bf16 with PyTorch SDPA attention (`Glide.from_pretrained` disables the cuDNN SDPA backend
process-wide, so the memory-efficient kernel is used).

**Logging.** Request contents are not logged.

## Performance

Decision Index 0.3.1 public suite: 51.10, measured with the official kit on one RTX PRO 6000.

## Limitations

- It chooses among the options it is given; it does not generate text or explanations.
- Probabilities are model estimates; set decision thresholds on your own data.
- At most 255 options per question; prompts above 7,999 tokens are shortened (beginning and end kept), so content in the middle of a
  very long state is not seen.
- Text and JSON only: no images.
- Cost grows with the number of questions: each question is a separate forward pass over the full state.

## License

GLiDE’s Fastino contributions are licensed under the <a href="https://huggingface.co/fastino/GLiDE-0.4B/blob/main/LICENSE" target="_blank" rel="noopener noreferrer">Fastino Community License</a>. Third-party components retain their original licenses; see the <a href="https://huggingface.co/fastino/GLiDE-0.4B/blob/main/NOTICE" target="_blank" rel="noopener noreferrer">NOTICE</a> for attribution.
