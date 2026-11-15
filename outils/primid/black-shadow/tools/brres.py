import struct
def dict_entries(d,o):
    total,n=struct.unpack_from('>II',d,o)
    out=[]
    for i in range(1,n+1):
        e=o+8+16*i
        _id,fl,l,r,no,do=struct.unpack_from('>HHHHII',d,e)
        name=d[o+no:d.index(b'\0',o+no)].decode('latin1') if no else ''
        out.append((name,o+do))
    return out
def brres_files(d):
    assert d[:4]==b'bres'
    root=struct.unpack_from('>H',d,0xC)[0]
    res={}
    for folder,fo in dict_entries(d,root+8):
        res[folder]=dict_entries(d,fo)
    return res
def chr0_info(d,o):
    tag,size,ver=struct.unpack_from('>4sII',d,o)
    if ver==5: nf,ne=struct.unpack_from('>HH',d,o+0x20)
    else: nf,ne=struct.unpack_from('>HH',d,o+0x1C)
    return ver,nf,ne
