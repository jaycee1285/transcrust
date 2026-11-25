import onnxruntime as ort, numpy as np, sys
def run(p):
    s=ort.InferenceSession(p,providers=["CPUExecutionProvider"])
    rng=np.random.default_rng(0); feed={}
    for i in s.get_inputs():
        sh=[1 if not isinstance(d,int) else d for d in i.shape]
        if i.name=="audio_signal": sh=[1,128,121]
        if i.name=="cache_last_channel": sh=[1,24,70,1024]
        if i.name=="cache_last_time": sh=[1,24,1024,8]
        if "int64" in i.type: feed[i.name]=np.zeros(sh,np.int64)
        elif i.name=="audio_signal": feed[i.name]=rng.standard_normal(sh).astype(np.float32)
        elif i.name=="length": feed[i.name]=np.array([121],np.int64)
        else: feed[i.name]=np.zeros(sh,np.float32)
    print([(i.name,i.shape,i.type) for i in s.get_inputs()]) if p.endswith("encoder-nemotron.onnx") else None
    return dict(zip([o.name for o in s.get_outputs()],s.run(None,feed)))
a=run("build/r13/encoder-nemotron.onnx"); b=run("build/pw/enc_pw_fp32.onnx")
for k in a: print(k,a[k].shape,np.abs(a[k].astype(np.float64)-b[k]).max())
