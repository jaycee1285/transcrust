import onnx, numpy as np, sys, collections
from onnx import helper, numpy_helper, TensorProto
m = onnx.load("build/r13/encoder-nemotron.onnx")  # loads external data
g = m.graph
init = {i.name: i for i in g.initializer}
nodes = []; n=0; shapes=collections.Counter(); uses=collections.Counter()
for nd in g.node:
    for i in nd.input: uses[i]+=1
for nd in g.node:
    if nd.op_type=="Conv" and nd.input[1] in init:
        at={a.name:helper.get_attribute_value(a) for a in nd.attribute}
        W=init[nd.input[1]]
        dims=list(W.dims)
        ok = len(dims)==3 and dims[2]==1 and at.get("group",1)==1 and all(x==1 for x in at.get("strides",[1])) \
          and all(x==0 for x in at.get("pads",[0,0])) and all(x==1 for x in at.get("dilations",[1])) \
          and at.get("auto_pad",b"NOTSET") in (b"NOTSET",)
        if ok:
            w=numpy_helper.to_array(W)[:,:,0].T.copy()  # [Cin,Cout]
            wn=nd.name+"_pwW"
            g.initializer.append(numpy_helper.from_array(w,wn))
            p=nd.name or f"pw{n}"; 
            t1=helper.make_node("Transpose",[nd.input[0]],[p+"_t1"],perm=[0,2,1],name=p+"_T1")
            mm=helper.make_node("MatMul",[p+"_t1",wn],[p+"_mm"],name=p+"_MM")
            nodes+= [t1,mm]; cur=p+"_mm"
            if len(nd.input)>2 and nd.input[2]:
                nodes.append(helper.make_node("Add",[cur,nd.input[2]],[p+"_ad"],name=p+"_Add")); cur=p+"_ad"
            nodes.append(helper.make_node("Transpose",[cur],[nd.output[0]],perm=[0,2,1],name=p+"_T2"))
            n+=1; shapes[(dims[1],dims[0])]+=1
            continue
    nodes.append(nd)
del g.node[:]; g.node.extend(nodes)
# drop unused old conv weight initializers
used={i for nd in g.node for i in nd.input}
keep=[i for i in g.initializer if i.name in used]
del g.initializer[:]; g.initializer.extend(keep)
print("rewritten",n,dict(shapes))
onnx.save(m,"build/pw/enc_pw_fp32.onnx",save_as_external_data=True,all_tensors_to_one_file=True,location="enc_pw_fp32.onnx.data")
