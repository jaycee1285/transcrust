# nemotron-export

NVIDIA's `nemotron-speech-streaming-en-0.6b` (`.nemo`, pinned `ebe59e5a`) → the ONNX layout transcrust
and mopvox load, by NeMo's own exporter. One checkpoint gives every chunk: right context 6 is 560 ms,
13 is 1120 ms (left context 70 in both).

1. `export_nemotron.py --nemo <.nemo> --right 13 --out build/r13`: fp32, I/O identical to the installed asset.
2. `pointwise_rewrite.py`: the 48 pointwise (1×1) Conv1d layers become Transpose→MatMul→Add→Transpose
   (fp32 max diff 3e-7, `pointwise_verify.py`). NeMo exports them as Conv, which the int4 quantizer skips:
   303 MB at fp32.
3. `pointwise_quant.py` / `quantize_nemotron.py`: MatMulNBits int4, block 128, asymmetric (the installed
   asset's recipe). Decoder fp16 with fp32 I/O.

Result (11-25): 371 MB vs 448 MB installed, words identical on a 68 s take but for one soft "of",
stream 6% faster. Study: `~/syncthing/transcrust-work/quant-study-11-24/`.
