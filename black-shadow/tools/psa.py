import struct,sys
from arc import arc_entries
def load(p):
    d=open(p,'rb').read()
    ms=[e for e in arc_entries(d) if e[0]==1][0][4]
    return ms
def sections(m):
    size,lk,lkn,dtn,exn=struct.unpack_from('>5I',m,0)
    base=0x20
    o=base+lk+lkn*4
    ents=[struct.unpack_from('>II',m,o+8*i) for i in range(dtn)]
    o2=o+8*dtn
    exts=[struct.unpack_from('>II',m,o2+8*i) for i in range(exn)]
    strbase=o2+8*exn
    def s(off): e=m.index(b'\0',strbase+off); return m[strbase+off:e].decode()
    return base,{s(so):off for off,so in ents},[(off,s(so)) for off,so in exts]
if __name__=='__main__':
    m=load(sys.argv[1]); base,sec,ext=sections(m)
    print('sections',sec); print('externes',len(ext),ext[:10])

HDR=['subactionFlags','modelVis','attributes','sseAttributes','misc','commonActionFlags','actionFlags','unk7','actionInterrupts','entryActions','exitActions','actionPre','subMain','subGFX','subSFX','subOther']
def header(m):
    base,sec,ext=sections(m)
    h=struct.unpack_from('>16I',m,base+sec['data'])
    return base,dict(zip(HDR,h)),ext
def cstr(m,base,off):
    e=m.index(b'\0',base+off); return m[base+off:e].decode('latin1')
def events(m,base,off,maxn=400):
    out=[]; o=base+off
    for _ in range(maxn):
        ns,eid,np_,unk,po=struct.unpack_from('>BBBBI',m,o)
        if ns==0 and eid==0 and np_==0: break
        params=[struct.unpack_from('>Ii',m,base+po+8*i) for i in range(np_)] if np_ and po else []
        out.append(((ns<<24)|(eid<<16)|(np_<<8)|unk,params)); o+=8
    return out
def subactions(m):
    base,h,ext=header(m)
    n=(h['subGFX']-h['subMain'])//4
    subs=[]
    for i in range(n):
        it,fl,_,no=struct.unpack_from('>BBHI',m,base+h['subactionFlags']+8*i)
        name=cstr(m,base,no) if no else ''
        mo=struct.unpack_from('>I',m,base+h['subMain']+4*i)[0]
        subs.append((i,name,mo))
    return base,subs
