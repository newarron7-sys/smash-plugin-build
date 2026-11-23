"""Agrandit des textures x4 avec Real-ESRGAN (RealESRGAN_x4plus, via spandrel, sur CPU) en gardant le raccord des
textures qui se répètent (bord recopié en boucle avant, recoupé après). Les grandes images sont traitées par tuiles
(chevauchement recoupé) pour tenir en mémoire.
usage : python3 upscale_tex.py <poids .pth> <png entrée> <png sortie>"""
import sys, numpy as np, torch, spandrel
from PIL import Image
W, IN, OUT = sys.argv[1:4]
m = spandrel.ModelLoader().load_from_file(W).eval()
im = np.asarray(Image.open(IN).convert('RGBA')).astype(np.float32) / 255
P = 16
TUILE = 256
pad = np.pad(im, ((P, P), (P, P), (0, 0)), mode='wrap')


def passe(x):
    with torch.no_grad():
        t = torch.from_numpy(np.ascontiguousarray(x[:, :, :3])).permute(2, 0, 1)[None]
        return m(t)[0].permute(1, 2, 0).clamp(0, 1).numpy()


H, Wd = pad.shape[:2]
if max(H, Wd) <= TUILE + 2 * P:
    y = passe(pad)
    s = y.shape[0] // H
else:
    s = 4
    y = np.zeros((H * s, Wd * s, 3), np.float32)
    for y0 in range(P, H - P, TUILE):
        for x0 in range(P, Wd - P, TUILE):
            y1, x1 = min(y0 + TUILE, H - P), min(x0 + TUILE, Wd - P)
            t = passe(pad[y0 - P:y1 + P, x0 - P:x1 + P])
            y[y0 * s:y1 * s, x0 * s:x1 * s] = t[P * s:P * s + (y1 - y0) * s, P * s:P * s + (x1 - x0) * s]
y = y[P * s:-P * s, P * s:-P * s]
a = np.asarray(Image.fromarray((im[:, :, 3] * 255).astype(np.uint8)).resize((y.shape[1], y.shape[0]), Image.LANCZOS)) / 255
out = np.dstack([y, a])
Image.fromarray((out * 255 + 0.5).astype(np.uint8), 'RGBA').save(OUT)
print(IN, im.shape, '->', out.shape)
