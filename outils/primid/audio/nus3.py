"""Lecture d'un .nus3audio (liste des pistes : id, nom, taille, format)."""
import struct
import sys


def read(path):
    d = open(path, 'rb').read()
    assert d[:4] == b'NUS3', 'pas un nus3audio'
    o = 8
    sec = {}
    while o + 8 <= len(d):
        if d[o:o + 8] == b'AUDIINDX':
            size = struct.unpack_from('<I', d, o + 8)[0]
            sec['AUDIINDX'] = (o + 12, size)
            o += 12 + size
            continue
        tag = d[o:o + 4].decode('latin1')
        size = struct.unpack_from('<I', d, o + 4)[0]
        sec[tag] = (o + 8, size)
        o += 8 + size
    cnt = struct.unpack_from('<I', d, sec['AUDIINDX'][0])[0]
    ids = struct.unpack_from('<%dI' % cnt, d, sec['TNID'][0])
    nmof = struct.unpack_from('<%dI' % cnt, d, sec['NMOF'][0])
    adof = struct.unpack_from('<%dI' % (2 * cnt), d, sec['ADOF'][0])
    tracks = []
    for i in range(cnt):
        e = d.index(b'\0', nmof[i])
        name = d[nmof[i]:e].decode('latin1')
        off, size = adof[2 * i], adof[2 * i + 1]
        tracks.append((ids[i], name, size, d[off:off + 4]))
    return tracks


if __name__ == '__main__':
    for t in read(sys.argv[1]):
        print('TRACK %4d %-40s %8d %s' % t)


def read_full(path):
    """pistes complètes : [(id, nom, données)]"""
    d = open(path, 'rb').read()
    out = []
    o = 8
    sec = {}
    while o + 8 <= len(d):
        if d[o:o + 8] == b'AUDIINDX':
            size = struct.unpack_from('<I', d, o + 8)[0]
            sec['AUDIINDX'] = (o + 12, size)
            o += 12 + size
            continue
        tag = d[o:o + 4].decode('latin1')
        size = struct.unpack_from('<I', d, o + 4)[0]
        sec[tag] = (o + 8, size)
        o += 8 + size
    cnt = struct.unpack_from('<I', d, sec['AUDIINDX'][0])[0]
    ids = struct.unpack_from('<%dI' % cnt, d, sec['TNID'][0])
    nmof = struct.unpack_from('<%dI' % cnt, d, sec['NMOF'][0])
    adof = struct.unpack_from('<%dI' % (2 * cnt), d, sec['ADOF'][0])
    for i in range(cnt):
        e = d.index(b'\0', nmof[i])
        out.append((ids[i], d[nmof[i]:e].decode('latin1'), d[adof[2 * i]:adof[2 * i] + adof[2 * i + 1]]))
    return out


def write(path, tracks):
    """écrit un .nus3audio : tracks = [(id, nom, données)], mêmes sections que les fichiers du jeu"""
    n = len(tracks)
    names = b''.join(t[1].encode('latin1') + b'\0' for t in tracks)
    # positions : NUS3(8) AUDIINDX(16) TNID(8+4n) NMOF(8+4n) ADOF(8+8n) TNNM(8+len) JUNK PACK
    tnnm_start = 8 + 16 + (8 + 4 * n) * 2 + (8 + 8 * n) + 8
    after_tnnm = tnnm_start + len(names)
    junk = (-(after_tnnm + 8 + 8)) % 0x10 or 0x10   # données du PACK alignées sur 0x10
    pack_start = after_tnnm + 8 + junk + 8
    data = b''
    adof = []
    for t in tracks:
        pad = (-len(data)) % 0x10
        data += b'\0' * pad
        adof.append((pack_start + len(data), len(t[2])))
        data += t[2]
    data += b'\0' * ((-len(data)) % 0x10)
    name_off = []
    o = tnnm_start
    for t in tracks:
        name_off.append(o)
        o += len(t[1]) + 1
    b = b'AUDIINDX' + struct.pack('<II', 4, n)
    b += b'TNID' + struct.pack('<I', 4 * n) + struct.pack('<%dI' % n, *[t[0] for t in tracks])
    b += b'NMOF' + struct.pack('<I', 4 * n) + struct.pack('<%dI' % n, *name_off)
    b += b'ADOF' + struct.pack('<I', 8 * n) + b''.join(struct.pack('<II', *a) for a in adof)
    b += b'TNNM' + struct.pack('<I', len(names)) + names
    b += b'JUNK' + struct.pack('<I', junk) + b'\0' * junk
    b += b'PACK' + struct.pack('<I', len(data)) + data
    out = b'NUS3' + struct.pack('<I', len(b)) + b
    assert out.index(b'TNNM') + 8 == tnnm_start, 'positions'
    open(path, 'wb').write(out)
