"""Décompression LZ77 Nintendo (0x10) et LZ77 étendue (0x11), utilisées par les .pac compressés de Brawl."""


def decompress(d):
    t = d[0]
    if t not in (0x10, 0x11):
        return d
    size = d[1] | (d[2] << 8) | (d[3] << 16)
    p = 4
    if size == 0:
        size = int.from_bytes(d[4:8], 'little')
        p = 8
    out = bytearray()
    while len(out) < size:
        flags = d[p]
        p += 1
        for bit in range(8):
            if len(out) >= size:
                break
            if not flags & (0x80 >> bit):
                out.append(d[p])
                p += 1
                continue
            if t == 0x10:
                b1, b2 = d[p], d[p + 1]
                p += 2
                n = (b1 >> 4) + 3
                disp = ((b1 & 0xF) << 8 | b2) + 1
            else:
                b1 = d[p]
                ind = b1 >> 4
                if ind == 0:
                    b2, b3 = d[p + 1], d[p + 2]
                    p += 3
                    n = ((b1 & 0xF) << 4 | b2 >> 4) + 0x11
                    disp = ((b2 & 0xF) << 8 | b3) + 1
                elif ind == 1:
                    b2, b3, b4 = d[p + 1], d[p + 2], d[p + 3]
                    p += 4
                    n = ((b1 & 0xF) << 12 | b2 << 4 | b3 >> 4) + 0x111
                    disp = ((b3 & 0xF) << 8 | b4) + 1
                else:
                    b2 = d[p + 1]
                    p += 2
                    n = ind + 1
                    disp = ((b1 & 0xF) << 8 | b2) + 1
            s = len(out) - disp
            for i in range(n):
                out.append(out[s + i])
    return bytes(out)
