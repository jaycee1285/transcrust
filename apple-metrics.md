# **Apple SpeechTranscriber: ASR Benchmark Handoff**

**Target:** iPad A14 / A16 Bionic, iOS 26+, 6–12-second utterances.

**API:** `SpeechAnalyzer` + `SpeechTranscriber` (on-device, streaming). Figures below come from independent benchmarks, not Apple performance guarantees.

## **1. Word Error Rate (WER)**

| Benchmark         | WER              | Hardware     | Notes                                                       |
| ----------------- | ---------------- | ------------ | ----------------------------------------------------------- |
| LibriSpeech clean | 1.82%            | M4 Pro Mac   | Clean adult speech                                          |
| Earnings-22       | 12.03%           | M4 Pro Mac   | Financial presentations, more challenging adult speech      |
| Combined WER      | **Not reported** | —            | Cannot establish a valid aggregate without corpus weighting |
| Child speech WER  | **Unknown**      | A14/A16 iPad | Must measure using project recordings                       |

**WER definition:** `(substitutions + deletions + insertions) / reference words × 100`.

Do not average 1.82% and 12.03% and call it combined WER. A valid pooled WER requires total errors and reference word counts, or an explicitly defined aggregation method.

## **2. Processing time / throughput**

| Device / chip              | Audio length | Processing time | Speed factor | Notes                        |
| -------------------------- | ------------ | --------------- | ------------ | ---------------------------- |
| iPhone, A14                | 46.8 sec     | 0.86 sec        | 54.4×        | Mandarin, single recording   |
| iPhone 16e, A18            | 50 min       | 56.7 sec        | 52.9×        | Long-form podcast            |
| iPhone 16 Pro Max, A18 Pro | 50 min       | 56.4 sec        | 53.2×        | Long-form podcast            |
| iPhone 17 Pro, A19 Pro     | 50 min       | 35.1 sec        | 85.5×        | Long-form podcast            |
| MacBook Pro, M4 Pro        | 50 min       | 30.2 sec        | 99.3×        | Long-form podcast            |
| M1 Pro MacBook             | 5.4 sec      | 0.16 sec        | 33.8×        | Batch processing, warm model |
| M1 Pro MacBook             | 29.4 sec     | 0.64 sec        | 45.9×        | Batch processing, warm model |
| M1 Pro MacBook             | 270 sec      | 4.3 sec         | 62.8×        | Batch processing, warm model |

**Important:** The throughput measurements use different recordings, languages, and devices. They are not controlled hardware comparisons. Batch processing time does not necessarily include initialization or model loading.

## **3. Latency / initialization**

| Device / chip          | Metric                                | Time       | Notes                      |
| ---------------------- | ------------------------------------- | ---------- | -------------------------- |
| iPhone 16e, A18        | First provisional transcript          | 300–500 ms | Warm streaming             |
| iPhone 15 Pro, A17 Pro | Reported first-token latency          | 95 ms      | Low-confidence, unverified |
| M1 Pro MacBook         | Finalization after 5.4 sec utterance  | 100 ms     | Warm streaming             |
| M1 Pro MacBook         | Finalization after 29.4 sec utterance | 100–230 ms | Warm streaming             |
| A14 iPhone             | Cold model load                       | Unknown    | No reliable measurement    |
| A16 iPad               | Cold model load                       | Unknown    | No reliable measurement    |
| A14/A16 iPad           | First provisional transcript          | Unknown    | Requires device benchmark  |
| A14/A16 iPad           | Finalization latency                  | Unknown    | Requires device benchmark  |

An early iOS 26 beta report described a delay exceeding 14 seconds before an initial result. It was not isolated to model loading and should not be treated as a cold-start benchmark.

## **4. Implementation notes**

| Property                        | Status                                           |
| ------------------------------- | ------------------------------------------------ |
| Framework                       | `Speech`                                         |
| Transcription API               | `SpeechAnalyzer` / `SpeechTranscriber`           |
| Execution                       | On-device                                        |
| Streaming partial results       | Supported                                        |
| Partial result revisions        | Supported                                        |
| Model preloading                | `prepareToAnalyze(in:withProgressReadyHandler:)` |
| Model retention                 | `.processLifetime`                               |
| Architecture / parameter count  | Undisclosed                                      |
| A14/A16 availability            | Check `isAvailable` at runtime                   |
| Model download                  | Managed by Apple                                 |
| Custom vocabulary / fine-tuning | No general model fine-tuning API                 |
| Network required for inference  | No, once required assets are available           |

## **5. Project benchmark targets**

These are **test requirements, not measured Apple performance**.

| Metric                   | Test condition                                           | Record |
| ------------------------ | -------------------------------------------------------- | ------ |
| Combined WER             | All project utterances pooled                            | %      |
| Split WER: search terms  | Short, isolated utterances                               | %      |
| Split WER: sentences     | 6–12-second utterances                                   | %      |
| First partial latency    | Speech start → first provisional text                    | ms     |
| Stable correct candidate | Speech start → first consistently correct glossary match | ms     |
| Finalization latency     | Speech end → finalized result                            | ms     |
| Warm processing time     | Recorded 6-second and 12-second inputs                   | ms     |
| Cold initialization      | Fresh launch → model ready                               | ms     |
| Warm initialization      | Existing process → analyzer ready                        | ms     |

**Testing constraints:** Keep language, model availability, sample rate, microphone conditions, and reference transcripts consistent. Run at least five trials per timing test and report median and p95. Measure WER separately from latency.

## **References**

- [Apple SpeechAnalyzer documentation](https://developer.apple.com/documentation/speech/speechanalyzer?utm_source=chatgpt.com)
- [Apple WWDC25 speech transcription session](https://developer.apple.com/videos/play/wwdc2025/277/?utm_source=chatgpt.com)
- [Murmur latency and transcription benchmarks](https://saymurmur.app/blog/apple-speech-vs-whisper-benchmark?utm_source=chatgpt.com)
- [Argmax Apple speech accuracy and performance analysis](https://www.argmaxinc.com/blog/apple-and-argmax?utm_source=chatgpt.com)

**Baseline conclusion:** Published Apple results establish high offline throughput and low warm-streaming latency, with adult-speech WER ranging from 1.82% to 12.03% on the two cited corpora. No validated A14/A16 iPad cold-start, short-utterance latency, or child-speech WER figure is currently established in this handoff.
