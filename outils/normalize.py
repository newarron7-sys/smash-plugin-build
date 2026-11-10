"""Remonte le volume de ses voix (les sons de Brawl sont plus bas que ceux d'Ultimate) : pic à -1 dB.

usage : python normalize.py <dossier wav>   (réécrit les .wav 16 bits en place, ignore les ondes vides)
"""
import array
import glob
import os
import sys
import wave

for f in sorted(glob.glob(os.path.join(sys.argv[1], '*.wav'))):
    w = wave.open(f)
    p = w.getparams()
    raw = w.readframes(p.nframes)
    w.close()
    if p.sampwidth != 2 or p.nframes < 100:
        continue
    a = array.array('h', raw)
    peak = max(abs(x) for x in a) or 1
    g = min(32767 * 0.89 / peak, 4.0)
    if g <= 1.01:
        continue
    a = array.array('h', (max(-32768, min(32767, int(x * g))) for x in a))
    o = wave.open(f, 'wb')
    o.setparams(p)
    o.writeframes(a.tobytes())
    o.close()
    print('NORM %-40s x%.2f' % (os.path.basename(f), g))
