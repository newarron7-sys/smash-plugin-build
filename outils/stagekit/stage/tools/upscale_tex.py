"""Agrandit des textures x4 avec Real-ESRGAN (RealESRGAN_x4plus, via spandrel, sur CPU) en gardant le raccord des
textures qui se répètent (bord recopié en boucle avant, recoupé après).
usage : python3 upscale_tex.py <poids .pth> <png entrée> <png sortie>"""
import sys, numpy as np, torch, spandrel
from PIL import Image
W, IN, OUT = sys.argv[1:4]
m = spandrel.ModelLoader().load_from_file(W).eval()
im = np.asarray(Image.open(IN).convert('RGBA')).astype(np.float32) / 255
P = 16
pad = np.pad(im, ((P, P), (P, P), (0, 0)), mode='wrap')
with torch.no_grad():
    rgb = torch.from_numpy(pad[:, :, :3]).permute(2, 0, 1)[None]
    y = m(rgb)[0].permute(1, 2, 0).clamp(0, 1).numpy()
s = y.shape[0] // pad.shape[0]
y = y[P * s:-P * s, P * s:-P * s]
a = np.asarray(Image.fromarray((im[:, :, 3] * 255).astype(np.uint8)).resize((y.shape[1], y.shape[0]), Image.LANCZOS)) / 255
out = np.dstack([y, a])
Image.fromarray((out * 255 + 0.5).astype(np.uint8), 'RGBA').save(OUT)
print(IN, im.shape, '->', out.shape)
