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
 expect(await page.evaluate(()=>(window as any).__TEST_CALLS__)).toEqual(['connect_device','inspect_device']);
 await expect(page.getByRole('button',{name:/install|apply|disable|restore/i})).toHaveCount(0);
});
