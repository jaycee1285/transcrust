"""Independent numpy check of the Rust port: unmerged LoRA, bf16->f32, HF-style Qwen3.
usage: python3 -I parity.py MODEL_DIR OUT.json [N]   (re-tokenizes with python tokenizers too)"""
import json, struct, sys
import numpy as np
from tokenizers import Tokenizer
root, out = sys.argv[1], sys.argv[2]
N = int(sys.argv[3]) if len(sys.argv) > 3 else 6
def st(path):
    b = open(path, 'rb'); n = struct.unpack('<Q', b.read(8))[0]; h = json.loads(b.read(n)); base = 8 + n
    raw = open(path, 'rb').read()
    d = {}
    for k, v in h.items():
        if k == '__metadata__': continue
        s, e = v['data_offsets']; buf = raw[base + s:base + e]
        if v['dtype'] == 'BF16':
            a = (np.frombuffer(buf, dtype=np.uint16).astype(np.uint32) << 16).view(np.float32)
        else:
            a = np.frombuffer(buf, dtype=np.float32)
        d[k] = a.reshape(v['shape'])
    return d
W = st(f'{root}/base/model.safetensors'); L = st(f'{root}/adapter/adapter_model.safetensors'); H = st(f'{root}/decision_head.safetensors')
def lin(x, name):
    y = x @ W[f'model.layers.{name}.weight'].T
    a = L[f'base_model.model.model.layers.{name}.lora_A.weight']; b = L[f'base_model.model.model.layers.{name}.lora_B.weight']
    return y + (x @ a.T @ b.T) * 2.0
def rms(x, w): return x / np.sqrt((x * x).mean(-1, keepdims=True) + 1e-6) * w
def rot(x, cos, sin):
    x1, x2 = x[..., :32], x[..., 32:]
    return np.concatenate([x1 * cos - x2 * sin, x2 * cos + x1 * sin], -1)
def fwd(ids):
    T = len(ids); x = W['model.embed_tokens.weight'][ids]
    inv = 1.0 / 1e6 ** (np.arange(0, 64, 2) / 64); ang = np.outer(np.arange(T), inv)
    cos, sin = np.cos(ang).astype(np.float32), np.sin(ang).astype(np.float32)
    mask = np.triu(np.full((T, T), -1e30, np.float32), 1)
    for i in range(28):
        p = f'model.layers.{i}.'
        h = rms(x, W[p + 'input_layernorm.weight'])
        q = rms(lin(h, f'{i}.self_attn.q_proj').reshape(T, 14, 64), W[p + 'self_attn.q_norm.weight'])
        k = rms(lin(h, f'{i}.self_attn.k_proj').reshape(T, 2, 64), W[p + 'self_attn.k_norm.weight'])
        v = lin(h, f'{i}.self_attn.v_proj').reshape(T, 2, 64)
        q = rot(q.transpose(1, 0, 2), cos, sin); k = rot(k.transpose(1, 0, 2), cos, sin); v = v.transpose(1, 0, 2)
        k = np.repeat(k, 7, 0); v = np.repeat(v, 7, 0)
        s = q @ k.transpose(0, 2, 1) / 8 + mask
        s = np.exp(s - s.max(-1, keepdims=True)); s /= s.sum(-1, keepdims=True)
        a = (s @ v).transpose(1, 0, 2).reshape(T, 896)
        x = x + lin(a, f'{i}.self_attn.o_proj')
        h = rms(x, W[p + 'post_attention_layernorm.weight'])
        g = lin(h, f'{i}.mlp.gate_proj'); u = lin(h, f'{i}.mlp.up_proj')
        x = x + lin((g / (1 + np.exp(-g))) * u, f'{i}.mlp.down_proj')
    return rms(x, W['model.norm.weight'])
res = json.load(open(out))
tok = Tokenizer.from_file(f'{root}/base/tokenizer.json')
rows = res['rows']; idx = np.linspace(0, len(rows) - 1, N).astype(int)
worst = 0; tokbad = 0
for i in idx:
    r = rows[i]; ids = r['token_ids']
    h = fwd(np.array(ids))
    q = h[r['decide']] @ H['q.weight'].T + H['q.bias']
    k = h[r['option_ends']] @ H['k.weight'].T + H['k.bias']
    lg = (k * q).sum(-1) / 16
    d = np.abs(lg - np.array(r['logits'])).max(); worst = max(worst, d)
    print(i, 'numpy', lg.round(4), 'rust', np.round(r['logits'], 4), 'maxdiff', float(d), 'argmax same', int(lg.argmax()) == int(np.argmax(r['logits'])))
# tokenization parity on state text
for i in idx:
    r = rows[i]; t = tok.encode(r['state'], add_special_tokens=False).ids
    ids = r['token_ids']; assert ids[0] == 65526 and ids[1:1 + len(t)] == t, f'token mismatch row {i}'
print('tokenizer state prefix parity ok; worst logit diff', float(worst))
