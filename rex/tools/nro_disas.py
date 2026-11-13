"""Désassemblage annoté de fonctions d'un plugin .nro (Skyline) : appels nommés (symboles importés via la PLT),
hash40 reconnus (effets, os), flottants chargés (fmov / ldr littéral), adresses calculées (adr / adrp+add).
Sert à relire les scripts d'effets d'un moveset pour reprendre ses effets (usage perso).

usage : python3 nro_disas.py <plugin.nro> <noms.txt (noms candidats pour les hash40)> <nom_de_script>...
        (cherche « nom_de_script » dans les chaînes du .nro, puis la fonction enregistrée juste après)
"""
import re, struct, sys, zlib
from capstone import Cs, CS_ARCH_ARM64, CS_MODE_ARM

NRO, NOMS = sys.argv[1:3]
SCRIPTS = sys.argv[3:]
d = open(NRO, 'rb').read()
tx_off, tx_sz = struct.unpack_from('<2I', d, 0x20)
md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)


def h40(s):
    b = s.encode()
    return zlib.crc32(b) | (len(b) << 32)


H = {}
for n in re.findall(r'[A-Za-z0-9_]+', open(NOMS).read()):
    H[h40(n.lower())] = n.lower()

# ---- symboles importés : MOD0 -> .dynamic -> dynsym / dynstr / jmprel ; stubs de PLT -> nom
mod0 = struct.unpack_from('<I', d, 4)[0]
dyn = mod0 + struct.unpack_from('<i', d, mod0 + 4)[0]
tags = {}
p = dyn
while True:
    t, v = struct.unpack_from('<qQ', d, p)
    if t == 0:
        break
    tags.setdefault(t, v)
    p += 16
symtab, strtab, jmprel, pltrelsz = tags.get(6), tags.get(5), tags.get(23), tags.get(2, 0)
got_name = {}
if jmprel:
    for i in range(0, pltrelsz, 24):
        off, info, add = struct.unpack_from('<QQq', d, jmprel + i)
        si = info >> 32
        nm = struct.unpack_from('<I', d, symtab + 24 * si)[0]
        s = d[strtab + nm:d.index(b'\0', strtab + nm)].decode()
        got_name[off] = s
stub_name = {}
for pc in range(tx_off, tx_off + tx_sz - 12, 4):
    i0, i1 = struct.unpack_from('<II', d, pc)
    if (i0 & 0x9F00001F) == 0x90000010 and (i1 & 0xFFC003FF) == 0xF9400211:  # adrp x16 ; ldr x17,[x16,#o]
        immlo, immhi = (i0 >> 29) & 3, (i0 >> 5) & 0x7FFFF
        imm = (immhi << 2) | immlo
        if imm & (1 << 20):
            imm -= 1 << 21
        page = (pc & ~0xFFF) + (imm << 12)
        got = page + ((i1 >> 10) & 0xFFF) * 8
        if got in got_name:
            stub_name[pc] = got_name[got]


def demangle(s):
    m = re.findall(r'\d+([A-Za-z_][A-Za-z0-9_]*)', s)
    return '::'.join(x for x in m if x not in ('app', 'lua2cpp', 'EP9lua_State')) or s


# ---- fonction enregistrée avec le nom du script : « adr x1, chaîne » puis « adr x3, fonction »
def adr_target(pc):
    i = struct.unpack_from('<I', d, pc)[0]
    if (i & 0x9F000000) != 0x10000000:
        return None
    immlo, immhi = (i >> 29) & 3, (i >> 5) & 0x7FFFF
    imm = (immhi << 2) | immlo
    if imm & (1 << 20):
        imm -= 1 << 21
    return pc + imm


ADR = {}
for pc in range(tx_off, tx_off + tx_sz, 4):
    t = adr_target(pc)
    if t is not None:
        ADR.setdefault(t, []).append(pc)


def fonctions(nom):
    out = []
    for loc in [m.start() for m in re.finditer(re.escape(nom.encode()), d)]:
        for pc in ADR.get(loc, []):
            for k in range(1, 5):
                t = adr_target(pc + 4 * k)
                if t is not None and tx_off <= t < tx_off + tx_sz:
                    out.append(t)
                    break
    return out


def lister(f):
    regs = {}
    pages = {}
    for ins in md.disasm(d[f:f + 0x4000], f):
        note = ''
        m = ins.mnemonic
        ops = ins.op_str
        if m in ('movz', 'mov', 'movk') and '#' in ops:
            r = ops.split(',')[0].strip().lstrip('wx')
            try:
                val = int(ops.split('#')[1].split(',')[0], 0)
            except ValueError:
                val = None
            sh = int(ops.split('lsl #')[1]) if 'lsl #' in ops else 0
            if val is not None:
                if m == 'movk':
                    v = regs.get(r, 0) & ~(0xFFFF << sh) | (val << sh)
                else:
                    v = val << sh
                regs[r] = v
                if v in H:
                    note = 'HASH ' + H[v]
        elif m == 'fmov' and '#' in ops:
            note = 'FLOAT ' + ops.split('#')[1]
        elif m == 'ldr' and ops.startswith('s') and '#0x' in ops and '[' not in ops:
            a = int(ops.split('#')[1], 16)
            note = 'FLOAT %g' % struct.unpack_from('<f', d, a)[0]
        elif m == 'adrp':
            pages[ops.split(',')[0].strip()] = int(ops.split('#')[1], 16)
        elif m == 'ldr' and ops.startswith(('s', 'd')) and '[x' in ops:
            rb = ops.split('[')[1].split(',')[0].strip(' ]')
            off = int(ops.split('#')[1].rstrip(']!'), 0) if '#' in ops else 0
            if rb in pages:
                a = pages[rb] + off
                if ops.startswith('s'):
                    note = 'FLOAT %g' % struct.unpack_from('<f', d, a)[0]
                else:
                    note = 'DOUBLE %g' % struct.unpack_from('<d', d, a)[0]
        elif m == 'adr':
            a = int(ops.split('#')[1], 16)
            if not (tx_off <= a < tx_off + tx_sz):
                s = d[a:a + 40].split(b'\0')[0]
                note = 'DATA %#x %r' % (a, s[:40])
        elif m == 'bl':
            a = int(ops.lstrip('#'), 16)
            note = 'APPEL ' + demangle(stub_name.get(a, '%#x' % a))
        print('  %#7x  %-8s %-40s %s' % (ins.address, m, ops, note))
        if m == 'ret':
            break


for s in SCRIPTS:
    if s.startswith('@'):
        print('=====', s, 'fonction a l\'adresse')
        lister(int(s[1:], 16))
        continue
    for f in fonctions(s):
        print('=====', s, 'fonction %#x' % f)
        lister(f)
