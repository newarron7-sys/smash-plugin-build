import struct,sys
def arc_entries(d):
    assert d[:4]==b'ARC\0', d[:4]
    n=struct.unpack_from('>H',d,6)[0]
    o=0x40; out=[]
    for i in range(n):
        typ,idx,size,grp,red=struct.unpack_from('>HHIBxh',d,o)
        data=d[o+0x20:o+0x20+size]
        out.append((typ,idx,size,red,data))
        o=o+0x20+((size+0x1f)&~0x1f)
    return out
TYPES={0:'None',1:'MiscData',2:'ModelData',3:'TextureData',4:'AnimationData',5:'SceneData',6:'Type6',7:'GroupedArchive',8:'EffectData'}
if __name__=='__main__':
    for p in sys.argv[1:]:
        d=open(p,'rb').read()
        print('==',p.split('/')[-1],'name',d[0x10:0x40].split(b'\0')[0])
        for typ,idx,size,red,data in arc_entries(d):
            print('  ',TYPES.get(typ,typ),idx,hex(size),red,data[:4])
