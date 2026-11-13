"""Télécharge le mod depuis GameBanana pendant la compilation (rien n'est stocké dans le dépôt)."""
import json, urllib.request, zipfile, io, sys, os
UA = {'User-Agent': 'Mozilla/5.0'}
def get(u, extra_headers=None):
    headers = dict(UA)
    headers.update(extra_headers or {})
    return urllib.request.urlopen(urllib.request.Request(u, headers=headers), timeout=300).read()
out = sys.argv[1]
p = json.loads(get('https://gamebanana.com/apiv11/Mod/591515/ProfilePage'))
f = p['_aFiles'][0]
z = zipfile.ZipFile(io.BytesIO(get(f['_sDownloadUrl'])))
want = ('/FitBlackShadow.pac', 'FitBlackShadowMotionEtc.pac', 'FitBlackShadow00.pac')
for n in z.namelist():
    if '__MACOSX' in n:
        continue
    if n.endswith('/FitBlackShadow.pac') and 'P+EX' not in n:
        continue
    if n.endswith(want):
        os.makedirs(out, exist_ok=True)
        open(os.path.join(out, n.rsplit('/', 1)[1]), 'wb').write(z.read(n))
# Inventaire des ressources audio de l'archive source.
# Les banques Brawl (.sawnd/.sbnk) ne sont pas directement utilisables dans Ultimate :
# on les identifie d'abord avant de tenter une conversion.
audio_ext = ('.sawnd', '.sbnk', '.nus3audio', '.idsp', '.wav', '.brstm', '.bcstm', '.mp3', '.ogg')
audio_candidates = [
    n for n in z.namelist()
    if not n.endswith('/') and (
        n.lower().endswith(audio_ext)
        or any(k in n.lower() for k in ('voice', 'sound', 'audio', 'sfx'))
    )
]
print('AUDIO_CANDIDATES_BEGIN')
for n in audio_candidates:
    info = z.getinfo(n)
    print(f'{n} ({info.file_size} bytes)')
print('AUDIO_CANDIDATES_END')

# Extraire les banques sonores originales de Black Shadow pour inspection.
sawnd_dir = os.path.join(out, 'sawnd')
os.makedirs(sawnd_dir, exist_ok=True)
for name in z.namelist():
    if name.lower().endswith('.sawnd') and not name.endswith('/'):
        target = os.path.join(sawnd_dir, os.path.basename(name))
        with open(target, 'wb') as bank:
            bank.write(z.read(name))
        print(f'SAWND_EXTRACTED {target} ({os.path.getsize(target)} bytes)')
print(sorted(os.listdir(out)))

# Source alternative F-Zero GX (facultative) : l'hébergeur peut refuser les téléchargements automatisés.
try:
    voice_url = 'https://sounds.spriters-resource.com/media/assets/416/419360.zip?updated=1755541621'
    voice_headers = {
        'Referer': 'https://sounds.spriters-resource.com/gamecube/fzerogx/asset/419360/',
        'Accept': 'application/zip,application/octet-stream,*/*',
    }
    voice_data = get(voice_url, voice_headers)
    voice_zip = zipfile.ZipFile(io.BytesIO(voice_data))
    voice_dir = os.path.join(out, 'voices')
    os.makedirs(voice_dir, exist_ok=True)
    for name in voice_zip.namelist():
        if name.lower().endswith('.mp3') and not name.endswith('/'):
            target = os.path.join(voice_dir, os.path.basename(name))
            with open(target, 'wb') as audio:
                audio.write(voice_zip.read(name))
    print('BLACK_SHADOW_GX_VOICES', sorted(os.listdir(voice_dir)))
except Exception as e:
    print('BLACK_SHADOW_GX_VOICES_UNAVAILABLE', repr(e))
