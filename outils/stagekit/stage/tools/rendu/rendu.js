const {chromium}=require('playwright');
(async()=>{const [,,out,w,h,cx,cy,cz,tx,ty,tz,fov]=process.argv;
const b=await chromium.launch({args:['--use-gl=angle','--use-angle=swiftshader','--enable-unsafe-swiftshader']});
const p=await b.newPage({viewport:{width:+w,height:+h}});
p.on('console',m=>console.log('console',m.text()));p.on('pageerror',e=>console.log('err',e.message));
await p.goto(`http://127.0.0.1:8765/index.html?w=${w}&h=${h}&cx=${cx}&cy=${cy}&cz=${cz}&tx=${tx}&ty=${ty}&tz=${tz}&fov=${fov}&bg=${process.env.UI_FOND||'87a060'}`);
await p.waitForFunction('window.FINI===true',null,{timeout:180000});
await p.locator('canvas').screenshot({path:out});await b.close();})();
