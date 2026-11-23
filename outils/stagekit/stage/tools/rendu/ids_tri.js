const {chromium}=require('playwright');
(async()=>{const cams=process.argv[2];
const b=await chromium.launch({args:['--use-gl=angle','--use-angle=swiftshader','--enable-unsafe-swiftshader']});
const p=await b.newPage({viewport:{width:960,height:540}});p.on('pageerror',e=>console.log('err',e.message));
await p.goto('http://127.0.0.1:8766/ids_tri.html?c='+encodeURIComponent(cams));
await p.waitForFunction('window.FINI===true',null,{timeout:600000});
console.log(JSON.stringify(await p.evaluate('window.RES')));await b.close();})();
