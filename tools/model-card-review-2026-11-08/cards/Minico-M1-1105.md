---
license: other
license_name: lfm1.0
license_link: LICENSE
base_model: LiquidAI/LFM2.5-350M
library_name: transformers
pipeline_tag: text-generation
language:
- en
tags:
- lfm2
- liquid
- reasoning
- reasoning-effort
- thinking
- tool-calling
- small-language-model
- on-device
- edge
---

# Minico M1 1105

**A 350M reasoning model you can run on a laptop, and you choose how hard it thinks.**

Minico M1 1105 is the full release of Minico M1, fine-tuned from [LiquidAI/LFM2.5-350M](https://huggingface.co/LiquidAI/LFM2.5-350M).
It fixes what was broken in [Minico M1 Preview](https://huggingface.co/Mincofficial/Minico-M1-Preview): the effort levels now work, the
reasoning is no longer repeated in the answer, and it no longer loops.

**[Try it in your browser](https://minico.mincoffical.codes/try/)** (runs on your GPU with WebGPU, nothing leaves your computer)

GGUF: [Mincofficial/Minico-M1-1105-GGUF](https://huggingface.co/Mincofficial/Minico-M1-1105-GGUF) · MLX: [Mincofficial/Minico-M1-1105-MLX](https://huggingface.co/Mincofficial/Minico-M1-1105-MLX) · ONNX: [Mincofficial/Minico-M1-1105-ONNX](https://huggingface.co/Mincofficial/Minico-M1-1105-ONNX)

## Highlights

- **Reasoning with five effort levels**: `off`, `low`, `medium`, `high`, `max`. One of the first sub-400M reasoning models, and the only one we know of with a five-step effort dial.
- **Thinks once, answers once**: reasoning closes with `</think>` and one answer follows. Loop rate 0% (Preview: 18.8%).
- **Leads the sub-400M models we tested**: beats each of them on at least 4 of 5 benchmarks, with the highest average of every model we ran, Qwen2.5 0.5B included (49.8; next best LFM2.5 350M base at 47.0). Against Granite 4.0 350M: IFEval 72.5 vs 48.8, MMLU-Pro 18.8 vs 12.6, MATH-500 19 vs 17, multi-turn 99 vs 98; Granite is ahead on GSM8K (41.5 vs 39.8). Same harness, same questions; table below.
- **Above Qwen2.5 0.5B, a model 41% bigger, on IFEval, MMLU-Pro, Multi-turn** (IFEval 72.5 vs 28.8; MMLU-Pro 18.8 vs 14.6; Multi-turn 99 vs 81).
- **Tool calling**: 100% correct calls on our function-calling test (base LFM2.5-350M: 54.8%).
- **Runs anywhere**: ~229 MB as Q4_K_M, 371 tokens/s on a fanless MacBook Air with llama.cpp. GGUF and MLX builds included.

## Benchmarks

All models ran on the same harness, on the same held-out test questions, with their own chat templates and the same sampling
(temperature 0.1, top_k 50, repetition penalty 1.05). Minico M1 1105 is shown with thinking **off**, the way the other models answer.
Best sub-400M score in **bold**. Average = mean of the five scored benchmarks (loop rate not included). Qwen2.5 0.5B is included as a larger reference.
Every model used this same near-greedy sampling rather than its own recommended settings, so official numbers can differ:
Qwen2.5 0.5B's IFEval here matches its published score, while Gemma 3 270M scores lower here than in Google's report.

| Model | Params | GSM8K | MATH-500 | IFEval | MMLU-Pro | Multi-turn | Average | Loop rate ↓ |
|---|---|---|---|---|---|---|---|---|
| **Minico M1 1105** (effort off) | 350M | 39.8 | 19 | **72.5** | **18.8** | **99** | **49.8** | **0%** |
| Minico M1 Preview | 350M | 25.8 | 15.5 | 50.3 | 11.2 | 66 | 33.8 | 18.8% |
| Gemma 3 270M | 270M | 17.2 | 6 | 26.1 | 11.5 | 60 | 24.2 | 18.8% |
| Granite 4.0 350M | 350M | **41.5** | 17 | 48.8 | 12.6 | 98 | 43.6 | 1.2% |
| LFM2 350M | 350M | 26.8 | **22** | 55.3 | 17.1 | 71 | 38.4 | **0%** |
| LFM2.5 350M (base) | 350M | 35 | 19 | 71.7 | 18.4 | 91 | 47 | **0%** |
| SmolLM2 360M | 360M | 11.2 | 0 | 32.4 | 10.8 | 44 | 19.7 | 2.5% |
| Qwen2.5 0.5B | 494M | 47.8 | 36 | 28.8 | 14.6 | 81 | 41.6 | 7.5% |

GSM8K: 400 test questions. MATH-500: 200, scored only when the answer is in `\boxed{}` as the prompt asks (SmolLM2 rarely
does this, which is most of its 0). IFEval: 541 prompts, prompt-level strict. MMLU-Pro: 980 questions (70 per subject).
Multi-turn: 100 chats where the answer must address the latest message. Loop rate: share of 80 mixed prompts where the output
repeats until the token limit. Differences under about 3 points are within run-to-run noise.

### Versus the Preview

| | Preview | Minico M1 1105 |
|---|---|---|
| GSM8K, thinking on (best effort) | 6.5 | **37** |
| GSM8K, thinking off | 25.8 | **39.8** |
| MATH-500, thinking off | 15.5 | **19** |
| IFEval (prompt, strict) | 50.3 | **72.5** |
| MMLU-Pro | 11.2 | **18.8** |
| Multi-turn (answers the latest message) | 66 | **99** |
| Tool calls correct | 0 | **100** |
| Says who it is | 0 | **100** |
| Loop rate (greedy, lower is better) | 18.8% | **0%** |

### By effort

| Effort | GSM8K | MATH-500 | Median reasoning tokens | Unclosed `<think>` | Preview GSM8K | Preview unclosed `<think>` |
|---|---|---|---|---|---|---|
| off | 39.8 | 19 | 0 | 0% | 25.8 | 0% |
| low | 31.5 | – | 40 | 0% | 4.8 | 15% |
| medium | 34.8 | 12.5 | 54 | 0% | 4.5 | 16.2% |
| high | 37 | 18.5 | 66 | 0% | 5.5 | 13.8% |
| max | 34.8 | – | 78 | 0% | 6.5 | 13.8% |

Honest note: on GSM8K, thinking makes the answer easier to follow but not more often right; `off` scores about the same as `high`.
The effort levels change how long it thinks (median tokens above), and every level closes its reasoning and answers.

### Example: "What is 5 + 5 x 5?"

Asked 20 times at each effort (temperature 0.1), correct answers out of 20. With thinking on, the Preview said 50 in
80 of 80 tries.

| Effort | Minico M1 Preview | Minico M1 1105 |
|---|---|---|
| off | 16/20 | **7/20** |
| low | 0/20 | **20/20** |
| medium | 0/20 | **20/20** |
| high | 0/20 | **20/20** |
| max | 0/20 | **20/20** |

Thinking is what gets this one right: with `off`, Minico M1 1105 added first and said 35 in 12 of 20 tries. Written as `5 + 5 × 5`, it answered
30 in 99 of 100 tries across all five efforts. One run of each effort:

| Effort | Reasoning (start) | Final answer |
|---|---|---|
| off | (no thinking) | 35 |
| low | The user wants the result of 5 + 5 x 5. Let's break it down step by step. | 30 |
| medium | Okay, let's see. The user asked for 5 + 5 times 5. Hmm, first, I need to follow the order of operations. The numbers are in parentheses, so maybe they want me t | 30 |
| high | Okay, let's see. The user asked for 5 + 5 times 5. Hmm, first, I need to figure out what the expression is being asked about. The numbers in parentheses are 5 a | 30 |
| max | Okay, let's see. The user asked for 5 + 5 times 5. Hmm, first, I need to understand the expression inside the parentheses. It looks like a multiplication and ad | 30 |

## Quickstart

### Transformers

```python
from transformers import AutoModelForCausalLM, AutoTokenizer

model_id = "Mincofficial/Minico-M1-1105"
tok = AutoTokenizer.from_pretrained(model_id)
model = AutoModelForCausalLM.from_pretrained(model_id, dtype="auto")

messages = [{"role": "user", "content": "A shirt costs $25 and is 20% off. How much do I pay?"}]
inputs = tok.apply_chat_template(messages, add_generation_prompt=True, reasoning_effort="medium",
                                 return_tensors="pt", return_dict=True)
out = model.generate(**inputs, max_new_tokens=1152, do_sample=True, temperature=0.3, top_k=50, repetition_penalty=1.05)
print(tok.decode(out[0][inputs["input_ids"].shape[1]:], skip_special_tokens=False))
```

### llama.cpp

```bash
llama-server -hf Mincofficial/Minico-M1-1105-GGUF:Q8_0 --jinja --chat-template-kwargs '{"reasoning_effort":"high"}'
```

### MLX (Apple silicon)

```bash
mlx_lm.generate --model Mincofficial/Minico-M1-1105-MLX --prompt "What is 5 + 5 x 5?" --chat-template-config '{"reasoning_effort":"low"}'
```

### Effort levels

| Effort | What it does | Max new tokens we use |
|---|---|---|
| `off` | No thinking. Answers right away (an empty think block is prefilled). | 1024 |
| `low` | A short thought, then the answer. | 640 |
| `medium` (default) | A normal thought. | 1152 |
| `high` | Longer, more careful reasoning. | 2368 |
| `max` | The longest reasoning. | 4608 |

The chat template also accepts aliases (`none`, `minimal`, `xhigh`, and others) and `enable_thinking=False`, which means `off`.
Unknown values fall back to `medium`. Recommended sampling: temperature 0.1–0.3, top_k 50, repetition penalty 1.05.

Pick one effort per conversation. Switching the effort, or turning thinking on or off, partway through a chat is not
recommended: the model was trained on chats that keep one setting from start to finish. Start a new chat to change it.

### Tool calling

Pass tools with `tools=[...]` in `apply_chat_template`. The model answers with LFM-style calls:
`<|tool_call_start|>[set_volume(level=30)]<|tool_call_end|>`. Return results with the `tool` role.

## Training

1. **SFT 1**: reasoning traces at every effort level, distilled from LiquidAI/LFM2.5-1.2B-Thinking (thinking efforts) and
   LFM2.5-1.2B-Instruct (`off`), with length budgets per effort.
2. **SFT 2**: on-policy rejection sampling (correct, closed, non-looping answers only), verified instruction-following data,
   greetings and identity.
3. **DPO**: preferred vs rejected pairs for loops, reasoning repeated in the answer, and identity.
4. **GRPO**: verifiable rewards for math, instruction following, tool calls and multi-turn; penalties for loops, unclosed
   `<think>`, reasoning that contradicts the answer, calling tools when none are needed, and claiming another identity; effort
   length floors so higher efforts think longer.
5. **Merge**: extrapolation (ExPO) from the GRPO checkpoints, kept only because it scored higher on a separate dev split.

All training ran locally on a 32 GB MacBook Air with MLX.

## Limitations

- It is a 350M model. It gets facts and harder math wrong. Do not use it for medical, legal or financial advice.
- With tools available, it sometimes calls a tool when it should just answer (it skipped the tool correctly
  75% of the time on our test).
- Thinking does not raise GSM8K accuracy over `off` yet (37 at best vs 39.8 off).
- English only was tested.

## License

Minico M1 1105 is a modified version of [LiquidAI/LFM2.5-350M](https://huggingface.co/LiquidAI/LFM2.5-350M) and is released under the
**LFM Open License v1.0** (see `LICENSE`). The weights were changed by fine-tuning (SFT, DPO, GRPO) and weight merging, as described
above. Commercial use is allowed for organisations under the revenue threshold set in the license; read it before using the model
commercially.

Built on LFM2.5-350M by Liquid AI. Attribution notices from the base model are kept in `LICENSE`.
