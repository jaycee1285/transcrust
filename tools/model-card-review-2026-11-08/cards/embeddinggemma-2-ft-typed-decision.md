---
library_name: transformers
license: apache-2.0
base_model: google/embeddinggemma-2
datasets:
  - LocalLLaMA/typed-decisions
language:
  - en
  - multilingual
tags:
  - typed-decisions
  - structured-decisions
  - calibration
  - embeddinggemma
  - custom_code
  - decision-model
pipeline_tag: text-classification
thumbnail: https://huggingface.co/mrm8488/embeddinggemma-2-ft-typed-decision/resolve/main/eg2td.png
---

# EmbeddingGemma 2 · typed decisions

<p align="center">
  <img src="https://huggingface.co/mrm8488/embeddinggemma-2-ft-typed-decision/resolve/main/eg2td.png"
       alt="Typed Decision · EmbeddingGemma 2" width="100%">
</p>

**EmbeddingGemma 2 fine-tuned for typed decisions**

A **typed decision model**: given the *state* of a workflow (an agent trace, a support ticket, an invoice, a
security alert…) and a **schema** of questions, it returns a typed, calibrated answer to each question.

| Question type | Example | Python type of `value` |
|---|---|---|
| `choice` | What should we do with this trace? → `continue / observe / human_review / stop` | `str` |
| `score` | How risky is it? → level `0..3` | `int` |
| `noul` | Does it need human review? → probabilistic yes/no | `bool` |

The head scores the **text of each option** against the state and the question, so the schema is plain data:
you can rename options, change descriptions or add new questions without retraining (answers to questions far from
the training data are less reliable).

## Usage

```python
from transformers import AutoModel

model = AutoModel.from_pretrained("mrm8488/embeddinggemma-2-ft-typed-decision", trust_remote_code=True)

state = {
    "agent": {"autonomy": "unsupervised", "model": "internal-agent-v2"},
    "task": "Clean up stale feature branches merged more than 90 days ago.",
    "constraints": ["Never delete anything matching release/*", "Do not force-push to any branch"],
    "trace_summary": {"steps": 9, "tool_errors": 1, "constraint_violations": 1,
                      "irreversible_actions": 1, "duration_s": 312.4},
}

questions = {
    "action": {
        "type": "choice",
        "instructions": "What should the observability system do with this trace?",
        "criteria": {
            "continue": "Let the agent proceed without interruption.",
            "observe": "Keep running, but flag the trace for later sampling.",
            "human_review": "Queue this trace for a human to review.",
            "stop": "Halt the agent now.",
        },
    },
    "needs_review": {"type": "noul", "instructions": "Does a human need to review this trace?"},
    "risk": {
        "type": "score",
        "instructions": "How risky is what the agent did?",
        "criteria": ["no risk", "low risk", "significant risk", "severe risk"],
    },
}

answers = model.decide(state, questions, workflow="agent_trace_observability")

for qid, answer in answers.items():
    print(f"{qid:13s} → {answer.value!r:15} ({answer.confidence:.0%})  {answer.probabilities}")
```

Every answer is a `TypedAnswer`:

```python
TypedAnswer(
    kind="choice",                  # "choice" | "score" | "noul"
    label="human_review",           # the winning option key
    value="human_review",           # typed: str for choice, int for score, bool for noul
    confidence=0.61,                # calibrated probability of the winning option
    probabilities={"continue": 0.02, "observe": 0.11, "human_review": 0.61, "stop": 0.26},
    expected_level=None,            # score questions only: E[level]
)
```

Use `answer.to_dict()` for JSON. The tokenizer is loaded from the same repo on first use.

### Schema format

* `choice`: `"criteria": {option_key: description, ...}`
* `score`: `"criteria": [level_0_description, level_1_description, ...]` (lowest to highest)
* `noul`: only `"instructions"`; the answer is `True` / `False` with a probability

### It works in other languages

EmbeddingGemma 2 is multilingual, so states and schemas don't have to be in English:

```python
preguntas = {
    "accion": {
        "type": "choice",
        "instructions": "¿Qué debe hacer el sistema de observabilidad con esta traza?",
        "criteria": {
            "continuar": "Dejar que el agente siga sin interrupción.",
            "observar": "Seguir ejecutando, pero marcar la traza para revisarla más tarde.",
            "revision_humana": "Poner la traza en cola para que la revise una persona.",
            "detener": "Detener el agente inmediatamente.",
        },
    },
    "necesita_revision": {"type": "noul", "instructions": "¿Necesita una persona revisar lo que ha hecho el agente?"},
}
paquita = {
    "agente": {"autonomía": "sin_supervisión"},
    "tarea": "Actualizar los precios del Roscón de Reyes en la tienda online.",
    "resumen_traza": {"violaciones_de_restricciones": 2, "acciones_irreversibles": 1,
                      "notas": "Aplicó un 90 % de descuento a todo el catálogo y envió un correo a 12.000 clientes."},
}
model.decide(paquita, preguntas, workflow="agent_trace_observability")
```

The head was trained on English schemas only, so expect softer probabilities in other languages.

### GPU and precision

```python
import torch

model = AutoModel.from_pretrained("mrm8488/embeddinggemma-2-ft-typed-decision", trust_remote_code=True,
                                  dtype=torch.bfloat16).to("cuda")
```

Use `bfloat16` (Ampere or newer) or `float32`. **Never `float16`**: EmbeddingGemma 2 overflows and returns NaNs
without raising an error.

### Lower-level API

| Method | What it does |
|---|---|
| `model.decide(state, questions, workflow="")` | typed answers for one state |
| `model.encode(texts)` | normalized sentence embeddings, exactly as in training |
| `model.collate(rows)` | turns dataset rows `{workflow, state, questions, gold}` into a training batch |
| `model(**batch)` | `GemmaDecisionOutput(loss, logits, probabilities, text_embeds)` |

Fine-tuning further with the 🤗 `Trainer`:

```python
from datasets import load_dataset
from transformers import Trainer, TrainingArguments

ds = load_dataset("LocalLLaMA/typed-decisions", "all")
trainer = Trainer(
    model=model,
    args=TrainingArguments("out", per_device_train_batch_size=16, learning_rate=2e-5,
                           num_train_epochs=2, remove_unused_columns=False),
    train_dataset=ds["train"],
    data_collator=model.collate,
)
trainer.train()
```

A single learning rate fine-tunes the encoder and the head together; for the two-learning-rate recipe used to
train this model, build the optimizer yourself.

## How it was built

```
state     ──► EmbeddingGemma 2 ─┐
question  ──► EmbeddingGemma 2 ─┼─► field(state, question, type) ─┐
option_i  ──► EmbeddingGemma 2 ─┴───────────────────────────────── ┴─► logit_i

logit_i = prior_i + σ(gate) · (s · cos(field, option_i) + residual_i)
```

* **Encoder:** `google/embeddinggemma-2`, text only, `Classification` task prompt, mean pooling, 768-d, L2-normalized.
* **Input rendering:** the state is flattened into readable `key: value` lines (`trace summary · constraint violations: 1`)
  instead of raw JSON, which embedding models read much better.
* **Head:** schema-conditioned scorer (~0.5M parameters). The `prior` term is zero-shot embedding similarity;
  the trained part is a gated correction on top.
* **Phase 1:** head only, on cached embeddings (encoder frozen).
* **Phase 2:** end-to-end fine-tuning of encoder + head, with two learning rates (encoder 2e-5, head 1e-4),
  frozen token embeddings, 5 epochs, 32 rows per batch.
* **Loss:** cross-entropy against the annotators' *soft* labels + Brier term + ordinal distance for `score` questions.
* **Calibration:** one temperature per question type, fitted on held-out training rows and stored in the weights.

## Results

`LocalLLaMA/typed-decisions`, subset `all`, test split (argmax vs gold label, all question types):

| Model | Test accuracy | Soft CE | Brier |
|---|---|---|---|
| Phase 1 (frozen encoder) | 67.7% | 0.968 | 0.113 |
| **Phase 2 (this model)** | **76.4%** | **0.887** | **0.065** |

Per question type (phase 1 → phase 2): `choice` 66.2% → 71.2% · `score` 61.8% → 74.0% · `noul` 77.0% → 84.8%.

Fine-tuning did not hurt the encoder's multilingual alignment: on an English/Spanish probe, the mean cosine of
translation pairs went from 0.869 to 0.901, with 100% retrieval@1 before and after.

Hyperparameters were chosen while looking at the test split, so treat these numbers as slightly optimistic.

## Limitations

* **Soft labels set a ceiling.** Many rows are near-ties between annotators; on those, the gold label is close to a
  coin flip.
* **Calibration means "matches annotator disagreement".** A 43% answer on a row where annotators split 43/28/25 is
  correct behaviour, but looks underconfident if you only compare confidence against the majority label.
* **Numbers inside the state are harder** for an embedding model than words; `score` questions are the weakest type.
* **Unseen schemas and languages work, with softer probabilities.** Validate on your own data before relying on them.
* **The rendering of state, questions and options is part of the model.** It lives in `modeling_gemmadecision.py`;
  changing it changes the predictions.
