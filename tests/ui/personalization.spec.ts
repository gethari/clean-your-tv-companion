import {test,expect} from '@playwright/test';

test('demo photo draft supports keyboard interval validation and explicit review without native IPC',async({page})=>{
 await page.setViewportSize({width:390,height:844});
 await page.emulateMedia({reducedMotion:'reduce'});
 await page.goto('/');
 await page.getByRole('button',{name:/Backgrounds/i}).click();
 await expect(page.getByText(/Demo mode · changes use sample data/)).toBeVisible();
 await page.getByRole('button',{name:'Add sample photo'}).click();
 await expect(page.getByRole('button',{name:'Preview photo 1'})).toBeVisible();
 const interval=page.getByLabel('Change photo every');
 await interval.fill('0');
 await expect(page.getByRole('alert').filter({hasText:'Enter a whole number from 1 to 1440'})).toBeVisible();
 await expect(page.getByRole('button',{name:'Review changes'})).toBeDisabled();
 await interval.fill('15');
 await page.getByRole('button',{name:'Review changes'}).click();
 await expect(page.getByRole('heading',{name:'Change review'})).toBeVisible();
 await expect(page.getByText(/Recovery baseline saved locally/)).toBeVisible();
 await page.getByRole('button',{name:'Apply reviewed changes'}).click();
 await expect(page.getByRole('status').filter({hasText:'active and verified on the TV'})).toBeVisible();
 expect(await page.evaluate(()=>'__TAURI_INTERNALS__' in window)).toBe(false);
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=window.innerWidth)).toBeTruthy();
});

test('unsupported launcher remains read-only with no apply action',async({page})=>{
 await page.addInitScript(()=>{
  const state={settings:{},overrides:{},collections:[],revision:'f'.repeat(64)};
  (window as any).__TAURI_INTERNALS__={invoke:async(command:string)=>{
   if(command==='discover') return [];
   if(command==='connect_device'||command==='remember_device') return null;
   if(command==='inspect_device') return {model:'Test TV',manufacturer:'Test',androidVersion:'12',home:'launcher/.Main',packages:['launcher'],disabled:[],theme:{},isTv:true,errors:[],collectedAt:'test'};
   if(command==='personalization_state') return {editable:false,reason:'This launcher does not expose safe photo settings.',state};
   throw new Error('Unexpected command '+command);
  }};
 });
 await page.goto('/');
 await page.getByRole('button',{name:'Find my TV'}).click();
 await page.getByLabel('Connection address',{exact:true}).fill('192.168.1.50:45737');
 await page.getByRole('button',{name:'Connect and check'}).click();
 await page.getByRole('button',{name:/Backgrounds/i}).click();
 await expect(page.getByRole('heading',{name:'Personalization is read-only'})).toBeVisible();
 await expect(page.getByText('This launcher does not expose safe photo settings.')).toBeVisible();
 await expect(page.getByRole('button',{name:/Review changes|Apply/i})).toHaveCount(0);
});

test('native mixed import shows individual rejection, preserves review on unknown apply, and uses last valid crop',async({page})=>{
 await page.addInitScript(()=>{
  const callbacks:Record<number,(event:any)=>void>={};let next=1;
  const snapshot={settings:{wallpaper:true,wallpaper_dim:10,active_collection:'c'.repeat(32)},overrides:{wallpaper:true},collections:[{id:'c'.repeat(32),assets:[{sha256:'d'.repeat(64),bytes:900,mime:'image/jpeg'}],order:'sequential',interval_minutes:30}],revision:'e'.repeat(64)};
  const good={id:'asset-1',sha256:'a'.repeat(64),mime:'image/jpeg',bytes:1200,width:1600,height:900,thumbnailDataUrl:'data:image/png;base64,AA=='};
  (window as any).__TEST_CALLS__=[];
  (window as any).__TAURI_INTERNALS__={
   transformCallback:(callback:any)=>{const id=next++;callbacks[id]=callback;return id;},
   invoke:async(command:string,args:any)=>{
    (window as any).__TEST_CALLS__.push({command,args});
    if(command==='plugin:event|listen') return next++;
    if(command==='plugin:event|unlisten') return null;
    if(command==='discover') return [];
    if(command==='connect_device'||command==='remember_device') return null;
    if(command==='inspect_device') return {model:'Test TV',manufacturer:'Test',androidVersion:'12',home:'launcher/.Main',packages:['launcher'],disabled:[],theme:{},isTv:true,errors:[],collectedAt:'test'};
    if(command==='personalization_state') return {editable:true,state:snapshot};
    if(command==='pick_photo_files') return {accepted:[good],rejected:[{name:'notes.txt',code:'unsupported-format'}]};
    if(command==='transform_image') throw new Error('crop conversion failed');
    if(command==='prepare_personalization') return {id:'b'.repeat(32),changes:[{key:'wallpaper_dim',before:10,after:10}],collections:1,baseline_location:'/app-data/backups/tv.json'};
    if(command==='apply_personalization') throw new Error('connection lost');
    throw new Error('Unexpected command '+command);
   }
  };
  (window as any).__EVENT_CALLBACKS__=callbacks;
 });
 await page.goto('/');
 await page.getByRole('button',{name:'Find my TV'}).click();
 await page.getByLabel('Connection address',{exact:true}).fill('192.168.1.50:45737');
 await page.getByRole('button',{name:'Connect and check'}).click();
 await page.getByRole('button',{name:/Backgrounds/i}).click();
 await expect(page.getByText('Preview unavailable: saved photos stay on the TV.')).toBeVisible();
 await page.getByRole('button',{name:'Choose photos'}).click();
 await expect(page.getByText('This file format is not supported.')).toBeVisible();
 await expect(page.getByRole('button',{name:'Preview photo 1'})).toBeVisible();
 const crop=page.getByRole('slider',{name:'Horizontal crop'});
 await crop.focus();await page.keyboard.press('ArrowRight');
 await expect.poll(async()=>page.evaluate(()=>(window as any).__TEST_CALLS__.some((call:any)=>call.command==='transform_image'))).toBe(true);
 await expect(page.getByRole('alert').filter({hasText:'last valid crop'})).toBeVisible();
 await page.getByLabel('Photo order').selectOption('shuffle');
 const interval=page.getByLabel('Change photo every');
 await interval.fill('17');
 await page.getByRole('button',{name:'Review changes'}).click();
 await expect(page.getByText('/app-data/backups/tv.json')).toBeVisible();
 await expect(interval).toBeDisabled();
 await page.getByRole('button',{name:'Apply reviewed changes'}).click();
 await expect(page.getByRole('alert').filter({hasText:'retry this saved plan'})).toBeVisible();
 await expect(page.getByRole('button',{name:'Reconcile saved plan'})).toBeVisible();
 await expect(page.getByRole('button',{name:'Cancel review'})).toBeVisible();
 const manifest=(await page.evaluate(()=>(window as any).__TEST_CALLS__.find((call:any)=>call.command==='prepare_personalization').args.collections[0]));
 expect(manifest.order).toBe('shuffle');
 expect(manifest.interval_minutes).toBe(17);
});

test('two import batches deduplicate by content and enforce the aggregate collection quota',async({page})=>{
 await page.addInitScript(()=>{
  let batch=0;
  const callbacks:Record<number,(event:any)=>void>={};let next=1;
  (window as any).__TAURI_INTERNALS__={
   transformCallback:(callback:any)=>{const id=next++;callbacks[id]=callback;return id;},
   invoke:async(command:string)=>{
    if(command==='plugin:event|listen') return next++;
    if(command==='plugin:event|unlisten') return null;
    if(command==='discover') return [];
    if(command==='connect_device'||command==='remember_device') return null;
    if(command==='inspect_device') return {model:'Test TV',manufacturer:'Test',androidVersion:'12',home:'launcher/.Main',packages:['launcher'],disabled:[],theme:{},isTv:true,errors:[],collectedAt:'test'};
    if(command==='personalization_state') return {editable:true,state:{settings:{},overrides:{},collections:[],revision:'e'.repeat(64)}};
    if(command==='pick_photo_files') {
     batch++;
     const count=batch===1?1:200;
     const accepted=Array.from({length:count},(_,index)=>{
      const n=batch===1?0:index;
      return {id:`asset-${n}`,sha256:n.toString(16).padStart(64,'0'),mime:'image/jpeg',bytes:batch===1?100:1_600_000,width:1200,height:800,thumbnailDataUrl:'data:image/png;base64,AA=='};
     });
     return {accepted,rejected:[]};
    }
    throw new Error('Unexpected command '+command);
   }
  };
 });
 await page.goto('/');
 await page.getByRole('button',{name:'Find my TV'}).click();
 await page.getByLabel('Connection address',{exact:true}).fill('192.168.1.50:45737');
 await page.getByRole('button',{name:'Connect and check'}).click();
 await page.getByRole('button',{name:/Backgrounds/i}).click();
 await page.getByRole('button',{name:'Choose photos'}).click();
 await page.getByRole('button',{name:'Choose photos'}).click();
 await expect(page.locator('.photo-list li')).toHaveCount(168);
 await expect(page.locator('.rejections').getByText(/collection has reached its 200 photo or 256 MiB limit/).first()).toBeVisible();
});
