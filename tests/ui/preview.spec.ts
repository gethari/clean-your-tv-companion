import {test,expect} from '@playwright/test';
test('existing setup, fresh TV and interrupted queries have honest distinct states',async({page})=>{
 await page.goto('/');
 await expect(page.getByText('Demo · no TV connected')).toBeVisible();
 await expect(page.getByRole('heading',{name:'Already feels like home.'})).toBeVisible();
 await expect(page.getByText('Private photo-provider settings aren’t visible here.')).toBeVisible();
 await page.selectOption('#scenario','fresh');
 await expect(page.getByRole('heading',{name:'A fresh start for your TV.'})).toBeVisible();
 await page.selectOption('#scenario','interrupted');
 await expect(page.getByText('Connection interrupted. Keep the TV awake')).toBeVisible();
 await page.getByRole('button',{name:'Connect a TV',exact:false}).click();
 await expect(page.getByRole('button',{name:'Connect and check'})).toBeDisabled();
 await page.getByRole('button',{name:'First time? Pair with a code'}).click();
 await expect(page.getByLabel('Six-digit pairing code')).toBeVisible();
 await page.screenshot({path:'test-results/connect.png',fullPage:true});
});
test('small layout and reduced motion remain usable',async({page})=>{
 await page.setViewportSize({width:680,height:900});
 await page.emulateMedia({reducedMotion:'reduce'});await page.goto('/');
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=window.innerWidth)).toBeTruthy();
 await page.screenshot({path:'test-results/overview-small.png',fullPage:true});
});
test('desktop overview',async({page})=>{await page.setViewportSize({width:1360,height:980});await page.goto('/');await page.screenshot({path:'test-results/overview.png',fullPage:true});});
test('desktop UI inspects an existing TV without exposing configuration actions',async({page})=>{
 await page.addInitScript(()=>{
   const state = {model:'Example TV',manufacturer:'Example',androidVersion:'12',home:'com.gathin.tvhome/.MainActivity',packages:['com.gathin.tvhome'],disabled:[],theme:{wallpaper:true,wallpaper_rotation_minutes:30},isTv:true,errors:[],collectedAt:'fixture'};
   (window as any).__TEST_CALLS__=[];
   (window as any).__TAURI_INTERNALS__={invoke:async(command:string)=>{
     (window as any).__TEST_CALLS__.push(command);
     if(command==='discover') return [];
     if(command==='remember_device') return null;
     if(command==='connect_device') return null;
     if(command==='inspect_device') return state;
     throw new Error('Unexpected command: '+command);
   }};
 });
 await page.goto('/');
 await page.getByRole('button',{name:'Find my TV'}).click();
 await page.getByLabel('Connection address',{exact:true}).fill('192.168.1.50:45737');
 await page.getByRole('button',{name:'Connect and check'}).click();
 await expect(page.getByText('● Live inspection')).toBeVisible();
 await expect(page.getByRole('heading',{name:'Already feels like home.'})).toBeVisible();
 await expect(page.getByText('Example TV',{exact:true})).toBeVisible();
 expect(await page.evaluate(()=>(window as any).__TEST_CALLS__)).toEqual(['discover','connect_device','inspect_device','remember_device']);
 await expect(page.getByRole('button',{name:/install|apply|disable|restore/i})).toHaveCount(0);
});
for (const autoConnect of [true,false]) {
 test(`pairing shows success and ${autoConnect ? 'continues to the matching TV' : 'prompts prominently for its connection port'}`,async({page})=>{
  await page.addInitScript(({autoConnect})=>{
   (window as any).__TEST_CALLS__=[];
   (window as any).__TAURI_INTERNALS__={invoke:async(command:string,args:any)=>{
    (window as any).__TEST_CALLS__.push({command,args});
    if(command==='remembered_device') return null;
    if(command==='remember_device') return null;
    if(command==='pair_device'||command==='connect_device') return null;
    if(command==='discover') return autoConnect ? [
      {endpoint:'192.168.1.99:12345',status:'device',kind:'Connected device'},
      {endpoint:'192.168.1.50:46481',status:'Advertised · not yet inspected',kind:'Connection service'}
    ] : [];
    if(command==='inspect_device') return {model:'Paired TV',manufacturer:'Example',androidVersion:'12',home:'com.gathin.tvhome/.MainActivity',packages:['com.gathin.tvhome'],disabled:[],theme:{wallpaper:true},isTv:true,errors:[],collectedAt:'fixture'};
    throw new Error('Unexpected command');
   }};
  },{autoConnect});
  await page.goto('/');await page.getByRole('button',{name:'Find my TV'}).click();
  await page.getByRole('button',{name:'First time? Pair with a code'}).click();
  await page.getByLabel('Pairing address',{exact:true}).fill('192.168.1.50:39123');
  await page.getByLabel('Six-digit pairing code').fill('123456');
  await page.getByRole('button',{name:'Pair computer',exact:true}).click();
  if(autoConnect){
   await expect(page.getByText('● Live inspection')).toBeVisible();
   await expect(page.getByText('Paired TV',{exact:true})).toBeVisible();
   const calls=await page.evaluate(()=>(window as any).__TEST_CALLS__);
   expect(calls.find((x:any)=>x.command==='connect_device').args.endpoint).toBe('192.168.1.50:46481');
   expect(calls.find((x:any)=>x.command==='inspect_device').args.serial).toBe('192.168.1.50:46481');
  }else{
   await expect(page.getByText('Computer paired with your TV',{exact:true})).toBeVisible();
   await expect(page.getByText('Pairing worked. Close the pairing-code popup')).toBeVisible();
   await expect(page.getByLabel('Connection address',{exact:true})).toHaveValue('192.168.1.50:');
   await expect(page.getByLabel('Connection address',{exact:true})).toBeFocused();
  }
 });
}
test('TV presets compare the existing setup and show protected features',async({page})=>{
 await page.goto('/');
 await page.getByRole('button',{name:'TV presets',exact:false}).click();
 await expect(page.getByRole('heading',{name:'A preset, with context.'})).toBeVisible();
 await expect(page.getByLabel('TV family')).toHaveValue('tcl');
 const logkit=page.locator('article.preset-row').filter({hasText:'com.tcl.logkit'});
 await expect(logkit.getByText('Already disabled',{exact:true})).toBeVisible();
 const tvplus=page.locator('article.preset-row').filter({hasText:'com.tcl.tv.plus'});
 await expect(tvplus.getByText('Available to review',{exact:true})).toBeVisible();
 await expect(page.getByRole('heading',{name:'Features we leave alone'})).toBeVisible();
 await expect(page.getByRole('button',{name:/disable|apply/i})).toHaveCount(0);
 await page.selectOption('#tv-preset','sony');
 await expect(page.getByText('com.sonyericsson.idd.agent',{exact:true})).toBeVisible();
});

test('native startup searches once and keeps manual connection available after no results',async({page})=>{
 await page.addInitScript(()=>{
  (window as any).__TEST_CALLS__=[];
  (window as any).__TAURI_INTERNALS__={invoke:async(command:string)=>{
   (window as any).__TEST_CALLS__.push(command);
   if(command==='discover') return [];
   throw new Error('Unexpected command');
  }};
 });
 await page.goto('/');
 await expect(page.getByText('No advertised devices found.',{exact:false})).toBeVisible();
 expect(await page.evaluate(()=>(window as any).__TEST_CALLS__)).toEqual(['discover']);
 await page.getByRole('button',{name:'Find my TV'}).click();
 await expect(page.getByLabel('Connection address',{exact:true})).toBeEnabled();
 await page.getByRole('button',{name:'Look for my TV'}).click();
 await expect.poll(()=>page.evaluate(()=>(window as any).__TEST_CALLS__.length)).toBe(2);
});

test('browser demo never invokes native discovery',async({page})=>{
 await page.goto('/');
 await expect(page.getByText('Demo · no TV connected')).toBeVisible();
 expect(await page.evaluate(()=>('__TAURI_INTERNALS__' in window))).toBe(false);
 await page.getByRole('button',{name:'Connect a TV',exact:false}).click();
 await expect(page.getByRole('button',{name:'Look for my TV'})).toBeDisabled();
});

test('stopped startup search cannot replace a newer result',async({page})=>{
 await page.addInitScript(()=>{
  let calls=0;
  (window as any).__TAURI_INTERNALS__={invoke:async(command:string)=>{
   if(command==='remembered_device') return null;
   if(command!=='discover') throw new Error('Unexpected command');
   calls++;
   if(calls===1) return new Promise(resolve=>{(window as any).__FINISH_SEARCH__=resolve;});
   return [{endpoint:'192.168.1.80:43210',status:'advertised',kind:'Connection service'}];
  }};
 });
 await page.goto('/');
 await expect(page.getByRole('button',{name:'Stop search'})).toBeVisible();
 await page.getByRole('button',{name:'Stop search'}).click();
 await page.getByRole('button',{name:'Find my TV'}).click();
 await page.getByRole('button',{name:'Look for my TV'}).click();
 await expect(page.getByText('192.168.1.80:43210',{exact:true})).toBeVisible();
 await page.evaluate(()=>(window as any).__FINISH_SEARCH__([{endpoint:'192.168.1.99:5555',status:'device',kind:'Connected device'}]));
 await expect(page.getByText('192.168.1.99:5555',{exact:true})).toHaveCount(0);
});

for(const remembered of [true,false]) {
 test(`startup ${remembered ? 'opens a verified remembered TV at its new port' : 'requires selection for ambiguous or unreadable identity'}`,async({page})=>{
  await page.addInitScript(({remembered})=>{
   (window as any).__TEST_CALLS__=[];
   (window as any).__TAURI_INTERNALS__={invoke:async(command:string,args:any)=>{
    (window as any).__TEST_CALLS__.push({command,args});
    if(command==='discover') return [{endpoint:'192.168.1.80:43210',status:'advertised',kind:'Connection service'}];
    if(command==='remembered_device') return remembered ? '192.168.1.80:43210' : null;
    if(command==='inspect_device') return {model:'Remembered TV',manufacturer:'Example',androidVersion:'12',home:'com.gathin.tvhome/.MainActivity',packages:[],disabled:[],theme:null,isTv:true,errors:[],collectedAt:'fixture'};
    throw new Error('Unexpected command');
   }};
  },{remembered});
  await page.goto('/');
  if(remembered){
   await expect(page.getByText('Remembered TV',{exact:true})).toBeVisible();
   await expect(page.getByText('Reconnected to your TV.',{exact:false})).toBeVisible();
  }else{
   await expect(page.getByText('Choose the TV you want to check.')).toBeVisible();
   expect(await page.evaluate(()=>(window as any).__TEST_CALLS__.some((c:any)=>c.command==='inspect_device'))).toBe(false);
  }
 });
}

test('connection feedback supports keyboard recovery and reduced-motion discovery',async({page})=>{
 await page.emulateMedia({reducedMotion:'reduce'});
 await page.addInitScript(()=>{
  let calls=0;
  (window as any).__TAURI_INTERNALS__={invoke:async(command:string)=>{
   if(command==='remembered_device') return null;
   if(command==='discover') {
    calls++;
    if(calls===1) return new Promise(()=>{});
    return [{endpoint:'192.168.1.80:43210',status:'advertised',kind:'Pairing service'}];
   }
   throw new Error('Unexpected command');
  }};
 });
 await page.goto('/');
 const stop=page.getByRole('button',{name:'Stop search'});
 await expect(stop).toBeVisible();
 await expect(page.locator('.illustration')).toHaveClass(/searching/);
 expect(await page.locator('.orbit.one').evaluate(el=>getComputedStyle(el).animationName)).toBe('none');
 await stop.focus();await page.keyboard.press('Enter');
 await expect(page.getByText('Search stopped.',{exact:false})).toBeVisible();
 await page.getByRole('button',{name:'Find my TV'}).click();
 await expect(page.getByRole('list',{name:'Connection progress'})).toBeVisible();
 await page.getByRole('button',{name:'Look for my TV'}).click();
 const pair=page.getByRole('button',{name:'Pair',exact:true});
 await expect(pair).toBeVisible();await pair.focus();await page.keyboard.press('Enter');
 await expect(page.getByLabel('Six-digit pairing code')).toBeFocused();
 await page.getByLabel('Six-digit pairing code').fill('12');
 await page.getByRole('button',{name:'Pair computer',exact:true}).click();
 await expect(page.getByLabel('Six-digit pairing code')).toHaveAttribute('aria-invalid','true');
 await expect(page.getByRole('alert')).toContainText('six-digit code');
});
