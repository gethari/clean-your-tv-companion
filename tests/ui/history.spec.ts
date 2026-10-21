import {test,expect} from '@playwright/test';

const SERIAL='192.168.1.50:45737';
const VERIFIED={id:'a'.repeat(32),phase:'verified',owned_keys:['wallpaper','wallpaper_dim'],collection_count:1,resulting_revision:'b'.repeat(64),restore_of:null};
const UNCERTAIN={id:'c'.repeat(32),phase:'activating',owned_keys:['wallpaper'],collection_count:0,resulting_revision:null,restore_of:null};

async function connect(page:any) {
 await page.getByRole('button',{name:'Find my TV'}).click();
 await page.getByLabel('Connection address',{exact:true}).fill(SERIAL);
 await page.getByRole('button',{name:'Connect and check'}).click();
 await expect(page.getByText('● Live inspection')).toBeVisible();
}

async function installMock(page:any, history= [VERIFIED]) {
 await page.addInitScript(({history,serial})=>{
  (window as any).__CALLS__=[];
  (window as any).__HISTORY__=history;
  (window as any).__IDENTITY_CHANGED__=false;
  (window as any).__CANCEL_FAILS__=false;
  (window as any).__TAURI_INTERNALS__={invoke:async(command:string,args:any={})=>{
   (window as any).__CALLS__.push({command,args});
   if(command==='discover') return [];
   if(command==='connect_device'||command==='remember_device') return null;
   if(command==='inspect_device') return {model:'Fixture TV',manufacturer:'Fixture',androidVersion:'12',home:'launcher/.Home',packages:[],disabled:[],theme:{},isTv:true,errors:[],collectedAt:'fixture'};
   if(command==='personalization_state') return {editable:true,state:{settings:{},overrides:{},collections:[],revision:'d'.repeat(64)}};
   if(command==='list_personalization_history') {
    if((window as any).__IDENTITY_CHANGED__) throw new Error('identity changed');
    if(args.serial!==serial) throw new Error('wrong selected TV');
    return {operations:(window as any).__HISTORY__,unreadable_records:0,truncated:false};
   }
   if(command==='reconcile_personalization') return {id:'c'.repeat(32),phase:'activating',owned_keys:['wallpaper'],collection_count:0,resulting_revision:null,restore_of:null};
   if(command==='prepare_restore') return {id:'e'.repeat(32),changes:[{key:'wallpaper',before:false,after:true}],collections:1,baseline_location:'/private/path/must-not-render'};
   if(command==='apply_personalization') return {id:'a'.repeat(32),phase:'verified',owned_keys:['wallpaper','wallpaper_dim'],collection_count:1,resulting_revision:'b'.repeat(64),restore_of:null};
   if(command==='cancel_personalization') {
    if((window as any).__CANCEL_FAILS__) throw new Error('cancellation uncertain');
    return {id:'a'.repeat(32),phase:'cancelled',owned_keys:['wallpaper','wallpaper_dim'],collection_count:1,resulting_revision:'b'.repeat(64),restore_of:null};
   }
   throw new Error('Unexpected command '+command);
  }};
 },{history,serial:SERIAL});
}

async function openHistory(page:any) {
 await page.getByRole('button',{name:/operation history|history/i}).click();
 await expect(page.getByRole('heading',{name:'Operation history'})).toBeVisible();
}

test('verified operation restore requires a separate review and explicit apply',async({page})=>{
 await installMock(page,[VERIFIED]); await page.goto('/'); await connect(page); await openHistory(page);
 await expect(page.getByText('Verified',{exact:true})).toBeVisible();
 await page.getByRole('button',{name:'Review restore'}).click();
 await expect(page.getByRole('heading',{name:'Restore review'})).toBeVisible();
 await expect(page.getByText('baseline_location')).toHaveCount(0);
 await expect(page.getByText('/private/path/must-not-render')).toHaveCount(0);
 let calls=await page.evaluate(()=>(window as any).__CALLS__);
 expect(calls.some((call:any)=>call.command==='apply_personalization')).toBe(false);
 await page.getByRole('button',{name:'Apply reviewed restore'}).click();
 await expect(page.getByText('Restore verified on the TV.')).toBeVisible();
 calls=await page.evaluate(()=>(window as any).__CALLS__);
 expect(calls.filter((call:any)=>call.command==='prepare_restore')).toHaveLength(1);
 expect(calls.filter((call:any)=>call.command==='apply_personalization')).toHaveLength(1);
});

test('uncertain history reconciles without retry and requires another explicit action',async({page})=>{
 await installMock(page,[UNCERTAIN]); await page.goto('/'); await connect(page); await openHistory(page);
 await page.getByRole('button',{name:'Reconcile with TV'}).click();
 await expect(page.getByText('The result is still uncertain. No retry was sent; reconnect and reconcile again.')).toBeVisible();
 let calls=await page.evaluate(()=>(window as any).__CALLS__);
 expect(calls.filter((call:any)=>call.command==='reconcile_personalization')).toHaveLength(1);
 expect(calls.some((call:any)=>call.command==='apply_personalization')).toBe(false);
 await page.getByRole('button',{name:'Retry saved activation after check'}).click();
 await expect(page.getByText('The saved operation is verified on the TV.')).toBeVisible();
 calls=await page.evaluate(()=>(window as any).__CALLS__);
 expect(calls.filter((call:any)=>call.command==='apply_personalization')).toHaveLength(1);
});

test('cancelling restore review confirms the saved plan is cancelled before clearing it',async({page})=>{
 await installMock(page,[VERIFIED]); await page.goto('/'); await connect(page); await openHistory(page);
 await page.getByRole('button',{name:'Review restore'}).click();
 await expect(page.getByRole('heading',{name:'Restore review'})).toBeVisible();
 await page.getByRole('button',{name:'Cancel review'}).click();
 await expect(page.getByText('Restore review cancelled. The TV was not changed.')).toBeVisible();
 await expect(page.getByRole('heading',{name:'Restore review'})).toHaveCount(0);
 const calls=await page.evaluate(()=>(window as any).__CALLS__);
 expect(calls.filter((call:any)=>call.command==='cancel_personalization')).toHaveLength(1);
 expect(calls.some((call:any)=>call.command==='apply_personalization')).toBe(false);
});

test('uncertain restore cancellation keeps its review open',async({page})=>{
 await installMock(page,[VERIFIED]); await page.goto('/'); await connect(page); await openHistory(page);
 await page.getByRole('button',{name:'Review restore'}).click();
 await page.evaluate(()=>{(window as any).__CANCEL_FAILS__=true;});
 await page.getByRole('button',{name:'Cancel review'}).click();
 await expect(page.getByText('The restore plan could not be confirmed cancelled. Its review remains open; refresh the TV before continuing.')).toBeVisible();
 await expect(page.getByRole('heading',{name:'Restore review'})).toBeVisible();
});

test('demo history never invokes native history commands',async({page})=>{
 await installMock(page); await page.goto('/');
 await page.getByRole('button',{name:'Try the demo'}).click();
 await page.getByRole('button',{name:/operation history|history/i}).click();
 await expect(page.getByText('Example only · no TV is connected and no commands are sent.')).toBeVisible();
 await expect(page.getByText('Change cccccccc')).toBeVisible();
 const calls=await page.evaluate(()=>(window as any).__CALLS__);
 expect(calls.some((call:any)=>call.command==='list_personalization_history')).toBe(false);
});

test('history is cleared when identity verification fails',async({page})=>{
 await installMock(page,[VERIFIED]); await page.goto('/'); await connect(page); await openHistory(page);
 await expect(page.getByText('Change aaaaaaaa')).toBeVisible();
 await page.evaluate(()=>{(window as any).__IDENTITY_CHANGED__=true;});
 await page.getByRole('button',{name:'Refresh history'}).click();
 await expect(page.getByText('Could not verify this TV and load its local operation history. Refresh the TV before continuing.')).toBeVisible();
 await expect(page.getByText('Change aaaaaaaa')).toHaveCount(0);
});
