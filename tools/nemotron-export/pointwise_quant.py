import onnx, shutil
from pathlib import Path
from onnxruntime.quantization.matmul_nbits_quantizer import MatMulNBitsQuantizer
D=Path("build/install/nemotron-speech-en-0.6b-ebe59e5a-1120ms-int4pw"); D.mkdir(parents=True,exist_ok=True)
q=MatMulNBitsQuantizer(onnx.load("build/pw/enc_pw_fp32.onnx"),bits=4,block_size=128,is_symmetric=False)
q.process()
onnx.save(q.model.model,str(D/"encoder_model.onnx"),save_as_external_data=True,all_tensors_to_one_file=True,location="encoder_model.onnx.data")
S=Path("build/install/nemotron-speech-en-0.6b-ebe59e5a-1120ms-int4")
for f in S.iterdir():
    if not f.name.startswith(("encoder","decoder")): shutil.copy(f,D/f.name)
for f in ("decoder_model.onnx","decoder_model.onnx.data"): shutil.copy("build/dec16/"+f,D/f)
import collections
print(collections.Counter(n.op_type for n in q.model.model.graph.node).most_common(8))
