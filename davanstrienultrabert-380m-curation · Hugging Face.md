[Edit model card](https://huggingface.co/davanstrien/ultrabert-380m-curation/edit/main/README.md)

## [](https://huggingface.co/davanstrien/ultrabert-380m-curation#ultrabert-curation-a-small-decision-model-for-data-curation)UltraBERT Curation: a small decision model for data curation

A 380M-parameter decision model for data curation, with **its training data and the recipe that made it, both open**. Give it a **state** (a web page, a prompt, a maths problem, a library record) and a set of named, typed **questions**; it returns a probability for every option of every question, with no generated text. It speaks the Jev / System One decision API (`choice`, `noul` yes/no, `score` on ordered levels) and also runs as a plain GLiClass zero-shot classifier.

Curation is the worked example. The main point of this release is to show how the training data for a small decision model was made: which sources, which labellers, which licences, and which data choices moved the scores. More people should share how they made the data for their decision models!

-   Training data, with source, labeller and licence on every row: [davanstrien/curation-decision-mix](https://huggingface.co/datasets/davanstrien/curation-decision-mix)
-   Recipe (data building, training, evaluation, inference): [github.com/davanstrien/curation-decision-recipe](https://github.com/davanstrien/curation-decision-recipe)
-   Example output: 1,020,000 FineWeb pages scored with this model, and the 63,788-page subset: [davanstrien/fineweb-curated-1m](https://huggingface.co/datasets/davanstrien/fineweb-curated-1m)

|  |  |
| --- | --- |
| Size | 380M parameters (UltraBERT-380M encoder + GLiClass scoring head) |
| Good at | curation questions on web pages, prompts, maths problems and library records: topic, format, educational value, difficulty, task type, validity, catalogue class |
| Not for | general decisions such as judging response quality, predicting pass rates, or safety (see Limitations) |
| Curation score | 0.705 (mean of 16 curation test units; GLiClass Edge 0.500, d1-2B 0.582; see Evaluation) |
| Compared with | [GLiClass Edge](https://huggingface.co/knowledgator/gliclass-v4-edge) (Knowledgator, ~440M parameters) and [d1-2B](https://huggingface.co/LiquidAI/d1-2B) (Liquid AI, 2B): small general decision models, run on the same inputs. See [why these models](https://huggingface.co/davanstrien/ultrabert-380m-curation#why-compare-with-small-models) |
| General decisions | Decision Index 0.3: 22.1 (GLiClass Edge reports 47.5, d1-2B 51.1). A specialist, not a general decision model |
| Interfaces | Jev / System One API (`jev_api.py`), GLiClass pipeline, packed GPU batch scoring (`fast_inference.py`), one-command Jobs script (`curate.py`) |
| Speed | ~530 question-rows/s on one RTX PRO 6000 with the packed path (see Speed for how the comparison was run) |
| Training data | 535,730 rows; 516,964 of them are released (18,766 rows without a verified open licence are left out) |
| Compute | everything ran on Hugging Face Jobs; one training run is about an hour on one GPU (about $3) |

## [](https://huggingface.co/davanstrien/ultrabert-380m-curation#limitations-read-first)Limitations (read first)

-   **It is a curation specialist.** On general decision tasks it is much weaker than general decision models: Decision Index 22.1 against 47.5 (Edge) and 51.1 (d1-2B). On tasks it was not trained for (this model vs Edge vs d1-2B): [HelpSteer2](https://huggingface.co/datasets/nvidia/HelpSteer2) response quality, Spearman 0.00–0.28 vs 0.13–0.48 vs 0.17–0.48; Polaris problem pass rates, Spearman 0.01 vs 0.24 vs 0.25; [Aegis](https://huggingface.co/datasets/nvidia/Aegis-AI-Content-Safety-Dataset-2.0) prompt safety, AUROC 0.82 vs 0.92 vs 0.91.
-   **Much of its lead comes from matching its teachers.** Several training labels were made by LLMs (Llama-3.1-405B, Llama-3-70B, GPT-4o and others; see Provenance). On test sets labelled by the same LLMs, it leads Edge by 30 points; on independent gold the lead is 8 points.
-   **The curation test sets were used during development** to choose between data recipes. They are held out from training, but they are not an untouched final test.
-   **Tested in English only.** The training data includes 21 other languages (from DecisionBank); they are untested.
-   **Length.** The API refuses inputs over 4,096 tokens (the training length); the fast path truncates at 2,048 tokens. Web pages were trained cut at 3,000 characters.
-   **Thresholds shift slightly.** The averaged encoder and the kept head are only approximately matched, so yes/no thresholds can move (−1 to −2 points on one validity question vs a single run). Set thresholds on your own data.
-   **Treat outputs as sorting signals,** not ground truth. The subset rule in the examples is a rule of thumb; this release does not show that it makes better training data.

## [](https://huggingface.co/davanstrien/ultrabert-380m-curation#try-it-on-your-data-in-one-command)Try it on your data in one command

[`curate.py`](https://huggingface.co/davanstrien/ultrabert-380m-curation/blob/main/curate.py) runs on [Hugging Face Jobs](https://huggingface.co/docs/huggingface_hub/guides/jobs). It reads a text column from any Hub dataset, asks four curation questions about every row (topic, format, educational value, reasoning depth), and pushes a dataset repo with a `scored` config (every row and its answers) and a `subset` config (educational value "useful for education" or higher; not spam, a listing, a product page or incomplete).

```bash
hf download davanstrien/ultrabert-380m-curation curate.py --local-dir .
hf jobs uv run curate.py -- --dataset HuggingFaceFW/fineweb-edu --config sample-10BT --text-column text \
    --limit 10000 --out-repo YOUR_NAME/fineweb-edu-curated
```

The script's PEP 723 header carries the Job settings (`[tool.hf-jobs]`: `a10g-small` GPU, 1-hour timeout, your `HF_TOKEN` to push the result), so the command needs only the script's own arguments. Flags such as `--flavor rtx-pro-6000` override the header.

Replace `YOUR_NAME` with your Hub username. You need a Hugging Face account with Jobs enabled (a paid plan or credits) and the `hf` CLI logged in. The output repo is private unless you pass `--public`. It needs a GPU from the Ampere generation or newer (`l4x1`, `a10g-small`, `a100-large`, `rtx-pro-6000`; not `t4`).

Measured on `l4x1` with FineWeb-Edu pages: 2,000 rows took 2.5 minutes in total (most of it loading and compiling the model) and 20,000 rows took 17 minutes, about 20 rows/s after start-up (about $0.25 at $0.80/hour). One `rtx-pro-6000` GPU scores about 150 rows/s on the same four questions, so use it for 100,000 rows or more.

For millions of pages, use the datatrove pipeline below, which splits the work over several GPU Jobs.

## [](https://huggingface.co/davanstrien/ultrabert-380m-curation#usage)Usage

### [](https://huggingface.co/davanstrien/ultrabert-380m-curation#decision-api-jev--system-one)Decision API (Jev / System One)

`jev_api.py` in this repo serves `POST /v1/systemone` and `GET /v1/models`, so clients written for System One can point at this model. It runs on CPU or GPU.

```bash
hf download davanstrien/ultrabert-380m-curation jev_api.py --local-dir .
uv run jev_api.py          # serves on port 8765; or: pip install "gliclass>=0.1.21" "transformers>=5.20" torch fastapi uvicorn
```

```bash
curl -s localhost:8765/v1/systemone -H 'Content-Type: application/json' -d '{
  "state": "Photosynthesis is how plants make food. Leaves take in carbon dioxide and water and, using light energy captured by chlorophyll, turn them into glucose and oxygen. Try it: put a pond-weed sprig in water under a lamp and count the oxygen bubbles per minute at different distances from the light.",
  "questions": {
    "edu": {"type": "score", "instructions": "Classify the educational value of this web page.",
            "criteria": ["not educational", "minimal educational value", "some educational value",
                         "useful for education", "highly educational", "outstanding educational value"]},
    "format": {"type": "choice", "instructions": "Classify the format of this web page.",
               "criteria": {"news": "News Article", "tutorial": "Tutorial / How-To Guide",
                            "knowledge": "Knowledge Article", "blog": "Personal Blog"}},
    "spam": {"type": "noul", "instructions": "Is this web page spam or an advertisement?"},
    "topics": {"type": "choice", "multiple": true, "instructions": "Which topics does this web page cover?",
               "criteria": {"science": "Science, Math & Technology", "health": "Health", "home": "Home & Hobbies"}}
  }}'
```

Response (rounded; `probabilities` per option shortened here):

```json
{"model": "davanstrien/ultrabert-380m-curation",
 "answers": {
   "edu": {"type": "score", "score": 3.38, "confidence": 0.52, "probabilities": {"3": 0.52, "4": 0.43, "...": "..."}},
   "format": {"type": "choice", "choice": "knowledge", "confidence": 0.81, "probabilities": {"knowledge": 0.81, "tutorial": 0.19, "...": "..."}},
   "spam": {"type": "noul", "noul": 0.002, "probabilities": {"true": 0.002, "false": 0.998}},
   "topics": {"type": "choice", "choices": ["science"], "probabilities": {"science": 0.98, "home": 0.16, "health": 0.01}}},
 "usage": {"input_tokens": 344, "output_tokens": 0}}
```

In Python, without the server: `from jev_api import system_one; system_one(state, questions)`.

| Type | Request | Answer |
| --- | --- | --- |
| `choice` | `instructions`, `criteria: {key: description}` | `choice`, `confidence`, `probabilities` per key |
| `noul` (yes / no) | `instructions`, optional `criteria: {"true": ..., "false": ...}` | `noul` = P(true) |
| `score` (ordered levels) | `instructions`, `criteria: [level 0, level 1, ...]` | `score` = expected level, `confidence`, `probabilities` |
| `choice` + `"multiple": true` | as `choice`; optional `"threshold"` (default 0.5) | `choices`, independent `probabilities` per key |

Each question is one pass through the model: the state is the text (a JSON record becomes `key: value` lines), `instructions` is the prompt and each option description is a label. This is the mapping used in training (about half the training rows are decision rows of this form; graded scales such as educational value were trained as ordered options, so `score` questions get the same input) and for the Decision Index result below. Other question types (text, number, date, box) are refused with HTTP 422, and so are inputs longer than 4,096 tokens (set `MAX_LENGTH`). Put the decision rule in `instructions`; the model was trained with up to 25 options per question.

### [](https://huggingface.co/davanstrien/ultrabert-380m-curation#gliclass-pipeline)GLiClass pipeline

```python
import torch
from gliclass import GLiClassModel, ZeroShotClassificationPipeline
from transformers import AutoTokenizer

model = GLiClassModel.from_pretrained("davanstrien/ultrabert-380m-curation", dtype=torch.bfloat16)
tokenizer = AutoTokenizer.from_pretrained("davanstrien/ultrabert-380m-curation")
pipe = ZeroShotClassificationPipeline(model, tokenizer, classification_type="single-label",
                                      max_length=2048, device="cuda:0")
pipe("…page text…", ["News", "Tutorial", "Product page", "Forum thread"],
     prompt="What is the format of this web page?")
# -> [[{'label': 'Tutorial', 'score': 0.9997}]] for a how-to page
```

Install with `pip install "gliclass>=0.1.21" "transformers>=5.20" torch` (tested with gliclass 0.1.21, transformers 5.21). A GPU is much faster: on an 8-vCPU cloud CPU machine one call took over a minute. bf16 gives the same top answers as fp32. In single-label mode the pipeline returns only the top option; pass `return_hierarchical=True` for a probability per option (`{label: probability}`).

### [](https://huggingface.co/davanstrien/ultrabert-380m-curation#curate-millions-of-pages-with-datatrove-on-hugging-face-jobs)Curate millions of pages with datatrove on Hugging Face Jobs

[`examples/datatrove_curation.py`](https://huggingface.co/davanstrien/ultrabert-380m-curation/blob/main/examples/datatrove_curation.py) is a [datatrove](https://github.com/huggingface/datatrove) pipeline that runs on Hugging Face Jobs. It reads parquet files from a Hub dataset, asks the same four questions as `curate.py` with the fast path, writes every page with its answers, and writes the filtered subset. You run the script on your machine; it launches one GPU Job per task and writes to a [storage bucket](https://huggingface.co/docs/huggingface_hub/guides/buckets).

```bash
hf download davanstrien/ultrabert-380m-curation examples/datatrove_curation.py --local-dir .
OUT="hf://buckets/YOUR_NAME/YOUR_BUCKET/run1"      # an existing bucket you can write to
uv run --python 3.12 --with "datatrove[io,processing]==0.10.1" --with "huggingface_hub>=1.32" \
    examples/datatrove_curation.py --out "$OUT" --limit 2000   # 2,000 pages per task, one task
```

On the first 1,020,000 pages of one [FineWeb](https://huggingface.co/datasets/HuggingFaceFW/fineweb) crawl (`CC-MAIN-2025-26`), six RTX PRO 6000 Jobs (`--tasks 6 --limit 170000`) scored every page in about 19 minutes, for about $5. The subset rule kept 63,788 pages (6.3 %). The result is [davanstrien/fineweb-curated-1m](https://huggingface.co/datasets/davanstrien/fineweb-curated-1m):

```bash
uv run --python 3.12 --with "datatrove[io,processing]==0.10.1" --with "huggingface_hub>=1.32" \
    examples/datatrove_curation.py --out "$OUT" --source hf://datasets/HuggingFaceFW/fineweb/data/CC-MAIN-2025-26 \
    --glob "000_0000*.parquet" --text-key text --id-key id --limit 170000 --tasks 6 --timeout 40m
```

The output keeps the top answer and its probability for each question (and, for educational value, the level and the expected level), not the full distribution. The subset rule uses the top educational-value level (3 or higher).

___

## [](https://huggingface.co/davanstrien/ultrabert-380m-curation#how-it-was-made)How it was made

The full recipe, with the scripts, is in [github.com/davanstrien/curation-decision-recipe](https://github.com/davanstrien/curation-decision-recipe); the data sources and the reasons for each are in the [dataset card](https://huggingface.co/datasets/davanstrien/curation-decision-mix).

-   **Model.** The encoder is [dmis-lab/UltraBERT-380M](https://huggingface.co/dmis-lab/UltraBERT-380M). On top sits a [GLiClass](https://github.com/Knowledgator/GLiClass) "uni-encoder" head: the options are written into the input next to the text, and a small feed-forward network scores each option against the text.
-   **Data.** A fixed mix of 535,730 rows from nine source groups (516,964 released), trained for 12,000 steps, which covers about two-thirds of the rows once.
-   **Loss.** Single-answer rows (453,751) use cross-entropy on soft targets (a probability per option); rows with several correct answers use a per-option yes/no loss (binary cross-entropy).
-   **Settings.** Learning rate 2e-5 for the encoder and 4e-5 for the head, 5 % warm-up, weight decay 0.01, batches of up to 32,768 tokens, training length 4,096 tokens.
-   **Weight averaging.** Three copies were trained with different random seeds. The release model averages the three **encoders** (a "weight soup", after [Wortsman et al., 2022](https://arxiv.org/abs/2203.05482)), keeps the head of the first copy, and then trains only the head for 500 steps with the encoder frozen. Averaging the whole model does not work: each copy's head starts from different random weights, and the averaged heads cancel out. The encoder-only average scores 0.9 points above the mean of the three single copies.

## [](https://huggingface.co/davanstrien/ultrabert-380m-curation#evaluation)Evaluation

**Curation score.** The mean over 16 curation test units. Each unit is one test set, except that a set asked with label descriptions is averaged with its plain twin, and the ten LMArena category questions count as one unit. Each set uses the metric that fits its question:

-   _top-1 accuracy_: is the most likely option the correct one;
-   _AUROC_ for yes/no questions: how well the scores separate yes from no (0.5 = chance, 1.0 = perfect);
-   _Spearman rank correlation_ for graded scales such as difficulty: does the model put items in the right order (0 = no relation).

Because the score mixes these metrics, 0.5 is not a chance level: Edge's 0.500 is a coincidence of its per-set scores, not random guessing.

**Why compare with small models.** [GLiClass Edge](https://huggingface.co/knowledgator/gliclass-v4-edge) (Knowledgator, ~440M parameters) and [d1-2B](https://huggingface.co/LiquidAI/d1-2B) (Liquid AI, 2B parameters) are general decision models: like this model, they answer typed questions about a text with a probability per option. They are the fair comparison because curation runs at scale: asking four questions about a billion web pages is four billion model calls. Large models (tens of billions of parameters, such as the LLMs that made some of the training labels) are more capable but cost far more per page; even a 4B LLM is about 5× slower than this model (see [Speed](https://huggingface.co/davanstrien/ultrabert-380m-curation#speed)). Edge and d1-2B were run without fine-tuning, on exactly the same inputs as our model, at 2,048 tokens.

**Gold made by the same labeller as some of our training rows** (9 units). Our model was trained on labels from these labellers, so these rows partly measure agreement with them:

| Test set (metric; who made the gold labels; rows) | This model | GLiClass Edge | d1-2B |
| --- | --- | --- | --- |
| [WebOrganizer](https://huggingface.co/WebOrganizer) topic (top-1; Llama-3.1-405B; 2,000 + 2,000 with descriptions) | 0.843 | 0.716 | 0.714 |
| WebOrganizer format (top-1; Llama-3.1-405B; 2,000 + 2,000) | 0.741 | 0.429 | 0.547 |
| [FineWeb-Edu](https://huggingface.co/datasets/HuggingFaceFW/fineweb-edu-llama3-annotations) educational value (Spearman; Llama-3-70B; 2,000 + 2,000) | 0.775 | 0.607 | 0.667 |
| [DeepMath](https://huggingface.co/datasets/zwhe99/DeepMath-103K) difficulty (Spearman; GPT-4o; 1,000) | 0.726 | 0.313 | 0.257 |
| [NuminaMath-1.5](https://huggingface.co/datasets/AI-MO/NuminaMath-1.5) question type (top-1; not documented; 1,000) | 0.976 | 0.621 | 0.871 |
| NuminaMath-1.5 problem validity (top-1; not documented; 819) | 0.835 | 0.538 | 0.669 |
| [WebInstruct-verified](https://huggingface.co/datasets/TIGER-Lab/WebInstruct-verified) answer type (top-1; not documented; 1,000) | 0.658 | 0.375 | 0.482 |
| WebInstruct-verified difficulty (Spearman; not documented; 1,000) | 0.774 | 0.151 | 0.657 |
| LMArena prompt categories, 10 yes/no questions (AUROC; LMArena classifiers; 2,000 each) | 0.855 | 0.750 | 0.787 |
| **Mean, 9 units** | **0.798** | 0.500 | 0.628 |

**Independent gold** (7 units: labels from humans or from sources that labelled none of our training rows; the fairer comparison). "Majority" is the score of always picking the most common answer:

| Test set (metric; majority; rows) | This model | GLiClass Edge | d1-2B |
| --- | --- | --- | --- |
| [Omni-MATH](https://huggingface.co/datasets/KbsdJames/Omni-MATH) difficulty, human ratings (Spearman; 3,202) | **0.683** | 0.444 | 0.525 |
| Omni-MATH maths domain (top-1; 0.36; 3,201) | **0.721** | 0.489 | 0.617 |
| [Essential-Web](https://huggingface.co/datasets/EssentialAI/essential-web-v1.0) document type (top-1; 0.31; 2,000) | **0.636** | 0.516 | 0.572 |
| Essential-Web subject, top level of its library-style subject scheme (top-1; 0.34; 2,000) | **0.578** | 0.500 | 0.528 |
| Essential-Web reasoning depth (Spearman; 2,000) | 0.619 | **0.622** | 0.534 |
| Essential-Web education level (Spearman; 2,000) | 0.423 | 0.474 | **0.489** |
| Essential-Web extraction artifacts (top-1; 0.575; 2,000; all three models below majority) | 0.428 | **0.452** | 0.390 |
| **Mean, 7 units** | **0.584** | 0.500 | 0.522 |
| **Curation score, all 16 units** | **0.705** | 0.500 | 0.582 |

Essential-Web labels were also generated by a model (Essential AI's), but it didn't label any of our training rows.

**Outside the curation score.** Library item form, a cataloguing scheme not seen in training (top-1; 3,250 Harvard records): 0.845 vs 0.819 (Edge) vs 0.854 (d1-2B). This set was used during development to choose between recipes, so it is not a clean test.

**Seed spread.** Single training runs of this recipe differ by up to ~8 points on single test sets (reasoning depth 0.55–0.63). The three runs average 0.695; the released weight average is 0.9 points above that.

**Agreement with established curation classifiers.** On 20,000 web pages from [IFM/TxT360-v2](https://huggingface.co/datasets/IFM/TxT360-v2) (`web-high-medium`), which ship with WebOrganizer topic and format classifier labels and a FineWeb-Edu classifier score, this model agrees on topic for 84.4 % of pages and on format for 66.1 %, and ranks educational value with a Spearman correlation of 0.86 against the FineWeb-Edu score. Those classifiers were trained on the same LLM labels as our model, so this is a consistency check, not independent gold. Most disagreements are between neighbouring labels (Software vs Software Development, Personal Blog vs Nonfiction Writing). The model's confidence is informative: keeping only answers with probability ≥ 0.7 raises agreement to 93 % on topic (77 % of pages kept) and 84 % on format (52 % kept).

**General decision benchmark.** On the [Decision Index](https://github.com/apolinario/decision-index) 0.3 ([leaderboard](https://huggingface.co/spaces/multimodalart/jev-decision-index)), a public suite of general decision tasks (our own run of the public suite), this model scores **22.08**. For comparison, our first baseline model scored 14.26; GLiClass Edge reports 47.5 and d1-2B 51.1 on their cards (not rerun by us). The curation data trades general decision skill for curation skill.

## [](https://huggingface.co/davanstrien/ultrabert-380m-curation#speed)Speed

Same 4,000 curation test rows for every model (Essential-Web document type, FineWeb-Edu, Omni-MATH difficulty, WebOrganizer topic; 1,000 each), one RTX PRO 6000 GPU on Hugging Face Jobs, inputs up to 2,048 tokens, tokenisation included, best of batch sizes 16 and 64 (2026-10-28). A row is one question about one text; asking four questions about a page costs four rows.

| Model | Rows per second | GPU cost per million rows (at $2.75/hour) |
| --- | --- | --- |
| **This model, packed fast path** (`fast_inference.py`, below) | **531** | **~$1.44** |
| GLiClass Edge, its standard path (not packed; an equivalent packing recipe was not tried) | 268 | ~$2.85 |
| This model, standard path (padded batches) | 221 | ~$3.46 |
| d1-2B (its batch API) | 102 | ~$7.49 |
| [Qwen3-4B-Instruct-2507](https://huggingface.co/Qwen/Qwen3-4B-Instruct-2507) served with vLLM (one answer letter; speed only, accuracy not measured) | 95 | ~$8.04 |

On their standard paths, this model and Edge run at similar speeds (221 vs 268 rows/s); the packed fast path makes this model about 2× faster than Edge's standard path.

### [](https://huggingface.co/davanstrien/ultrabert-380m-curation#fast-batch-scoring)Fast batch scoring

For large batches (curating a dataset), use `fast_inference.py` from this repo. It needs one CUDA GPU (Ampere or newer, for the flash-attention kernel). Its probabilities match the pipeline up to bf16 noise: on 500 FineWeb-Edu pages, the top answer agreed on 99.6 % and no probability differed by more than 0.017, at 660 pages/s against 210 pages/s for the pipeline (RTX PRO 6000, peak GPU memory 2.4 GB).

```bash
pip install "gliclass>=0.1.21" "transformers>=5.20" torch numpy "kernels>=0.17.2,<0.18"
hf download davanstrien/ultrabert-380m-curation fast_inference.py --local-dir .
uv run fast_inference.py --n 4000     # optional self-check: agreement with the pipeline, and speed
```

```python
from fast_inference import FastScorer

scorer = FastScorer("davanstrien/ultrabert-380m-curation")      # loads, fuses and compiles (about a minute)

page_texts = ["How to repot a houseplant. Step 1: ...", "Breaking: council approves new bus lanes ..."]  # your pages
texts = [t[:3000] for t in page_texts]                            # the training web text was cut at 3,000 characters
fmt = scorer.score(texts, ["News Article", "Tutorial / How-To Guide", "Product Page", "Personal Blog"],
                   prompt="Classify the format of this web page.")
# -> one {label: probability} dict per text (None if truncation cut off some labels: very long label lists only)

# Several questions in one packed run: one (text, labels, prompt) per request
levels = ["not educational", "minimal educational value", "some educational value", "useful for education",
          "highly educational", "outstanding educational value"]
topics = ["Health", "Science, Math & Technology", "Home & Hobbies", "Politics"]   # your options
requests = [(t, levels, "Classify the educational value of this web page.") for t in texts]
requests += [(t, topics, "Classify the topic of this web page.") for t in texts]
results = scorer.score_requests(requests)
edu, topic = results[: len(texts)], results[len(texts):]
```

**Decision requests in bulk.** `scorer.system_one(state, questions)` and `scorer.system_one_batch([(state, questions), ...])` take the same requests as `jev_api.py` (see [Decision API](https://huggingface.co/davanstrien/ultrabert-380m-curation#decision-api-jev--system-one)) and return the same per-question answer schema, packed into one run. Differences: the fast path truncates at 2,048 tokens instead of refusing long inputs, returns `None` for a question whose labels did not fit, and returns only `answers` (no `model` or `usage`).

```python
out = scorer.system_one(page_texts[0], {
    "edu": {"type": "score", "instructions": "Classify the educational value of this web page.", "criteria": levels},
    "tutorial": {"type": "noul", "instructions": "Is this web page a tutorial or how-to guide?"},
    "format": {"type": "choice", "instructions": "Classify the format of this web page.",
               "criteria": {"news": "News Article", "tutorial": "Tutorial / How-To Guide", "blog": "Personal Blog"}},
    "topics": {"type": "choice", "multiple": True, "instructions": "Which topics does this web page cover?",
               "criteria": {"health": "Health", "science": "Science, Math & Technology", "home": "Home & Hobbies"}},
})
a = out["answers"]
a["edu"]["score"], a["tutorial"]["noul"], a["format"]["choice"], a["topics"]["choices"]   # each also has "probabilities"
```

A state can be a string, a list or a record (a dict is written as `key: value` lines, as in training). Pass a few thousand requests or states per call: packing needs many documents to fill each 64K-token row. On smaller GPUs pass `FastScorer(..., step_tokens=32768)`.

The fast path:

-   packs many documents end to end into one long row of up to 64K tokens, so no compute is spent on padding;
-   merges the attention and feed-forward weight matrices so the GPU runs fewer, larger multiplications;
-   runs in 16-bit (bfloat16) precision and compiles the encoder with `torch.compile`.

It agrees with the standard path on the top answer for 99.0–99.8 % of rows, and accuracy against gold moves by at most 0.2 points per set. The 531 rows/s include tokenisation done before the GPU work; the GPU alone runs 836 rows/s (379K tokens/s).

## [](https://huggingface.co/davanstrien/ultrabert-380m-curation#provenance-of-the-training-labels)Provenance of the training labels

Per source and per row in the [dataset card](https://huggingface.co/datasets/davanstrien/curation-decision-mix).

-   **Labels made by models:** Llama-3.1-405B (WebOrganizer topic and format; the annotation datasets state no licence), Llama-3-70B (FineWeb-Edu), GPT-4o (DeepMath difficulty and topic), Qwen3.8-27B mixed with source labels or with the Jeff 1.0 teacher models Qwen3.6-27B / Qwen3.5-122B-A10B (DecisionBank; the latter on 4,188 rows), LMArena's category classifiers, and labellers the sources do not document (NuminaMath-1.5 and WebInstruct-verified columns).
-   **Labels made by people or programs:** National Library of Scotland and Harvard Library cataloguers, the original task datasets collected in tasksource, program-computed answers, and author-chosen arXiv categories.

## [](https://huggingface.co/davanstrien/ultrabert-380m-curation#compute)Compute

Everything ran on [Hugging Face Jobs](https://huggingface.co/docs/huggingface_hub/guides/jobs): data building, training, evaluation and labelling experiments. Data, checkpoints and results lived in Hugging Face storage buckets mounted into each job, so every run wrote its outputs to one place and could be compared or rerun later.

One training run takes about an hour on a single RTX PRO 6000 GPU (about $3; peak GPU memory about 55 GB). That made it practical to test every data choice with three seeds before adopting it. The release model is three such runs, averaged, plus a short head training run.

## [](https://huggingface.co/davanstrien/ultrabert-380m-curation#credits)Credits

**Built on:** [UltraBERT-380M](https://huggingface.co/dmis-lab/UltraBERT-380M) (DMIS Lab), the [GLiClass](https://github.com/Knowledgator/GLiClass) library and its v4 training recipe (Knowledgator), [DecisionBank](https://huggingface.co/datasets/jgeuter/DecisionBank), the [Decision Index](https://github.com/apolinario/decision-index), [datatrove](https://github.com/huggingface/datatrove), and the open datasets and labels listed, with links, in the [dataset card](https://huggingface.co/datasets/davanstrien/curation-decision-mix#sources).

**Ideas drawn on:**

-   weight averaging of fine-tuned models: _Model soups_ ([Wortsman et al., 2022](https://arxiv.org/abs/2203.05482));
-   curation labels and taxonomies: _Organize the Web_ / WebOrganizer ([Wettig et al., 2025](https://arxiv.org/abs/2502.10341)), FineWeb-Edu ([Penedo et al., 2024](https://arxiv.org/abs/2406.17557)) and Essential-Web, used as independent gold ([Essential AI, 2025](https://arxiv.org/abs/2506.14111));
-   text-blind baselines to catch shortcuts in training sources: hypothesis-only and annotation-artifact baselines ([Poliak et al., 2018](https://arxiv.org/abs/1805.01042); [Gururangan et al., 2018](https://arxiv.org/abs/1803.02324)).

The [recipe repo](https://github.com/davanstrien/curation-decision-recipe) maps each idea to the script that applies it. Made by [Daniel van Strien](https://huggingface.co/davanstrien).

## Model tree for davanstrien/ultrabert-380m-curation[](https://huggingface.co/docs/hub/model-cards#specifying-a-base-model)

Finetuned

([20](https://huggingface.co/models?other=base_model:finetune:dmis-lab/UltraBERT-380M))

this model

## Dataset used to train davanstrien/ultrabert-380m-curation

[

#### davanstrien/curation-decision-mix

Viewer •

Updated 2 days ago

•

1.03M

•

36

•

1





](https://huggingface.co/datasets/davanstrien/curation-decision-mix)

## Space using davanstrien/ultrabert-380m-curation 1

## Collection including davanstrien/ultrabert-380m-curation

## Papers for davanstrien/ultrabert-380m-curation