import {test,expect} from '@playwright/test';

test('demo source review excludes the Access Key and distinguishes saved settings from retrieval',async({page})=>{
 await page.goto('/');
 await page.getByRole('button',{name:'Backgrounds'}).click();
 await page.getByLabel('Wallpaper provider',{exact:true}).selectOption('unsplash');
 await page.getByLabel('Unsplash collection IDs',{exact:true}).fill('123,abc_9\n123');
 await page.getByLabel('Saved Unsplash Access Key',{exact:true}).selectOption('replace');
 const key=page.getByLabel('New Unsplash Access Key',{exact:true});
 await key.fill('fixtureKey-123456');
 await expect(key).toHaveAttribute('type','password');
 await page.getByRole('button',{name:'Review source changes'}).click();
 const review=page.locator('.provider-review');
 await expect(review).toContainText('123, abc_9');
 await expect(review).not.toContainText('fixtureKey-123456');
 await expect(page.getByLabel('Wallpaper provider',{exact:true})).toBeDisabled();
 await page.getByRole('button',{name:'Apply demo source'}).click();
 await expect(page.getByRole('status').filter({hasText:'Image retrieval has not been checked'})).toBeVisible();
 await expect(key).toHaveCount(0);
 expect(await page.evaluate(()=>'__TAURI_INTERNALS__' in window)).toBe(false);
});

test('provider collection errors preserve the draft and a dismissed key is cleared',async({page})=>{
 await page.goto('/');await page.getByRole('button',{name:'Backgrounds'}).click();
 await page.getByLabel('Wallpaper provider',{exact:true}).selectOption('unsplash');
 const ids=page.getByLabel('Unsplash collection IDs',{exact:true});
 await ids.fill('https://unsplash.com/collections/123');
 await page.getByRole('button',{name:'Review source changes'}).click();
 await expect(page.getByRole('alert').filter({hasText:'without URLs'})).toBeVisible();
 await expect(ids).toHaveValue('https://unsplash.com/collections/123');
 await ids.fill('123');
 await page.getByLabel('Saved Unsplash Access Key',{exact:true}).selectOption('replace');
 await page.getByLabel('New Unsplash Access Key',{exact:true}).fill('fixtureKey-123456');
 await page.getByRole('button',{name:'Review source changes'}).click();
 await page.getByRole('button',{name:'Dismiss source review'}).click();
 await page.getByLabel('Saved Unsplash Access Key',{exact:true}).selectOption('replace');
 await expect(page.getByLabel('New Unsplash Access Key',{exact:true})).toHaveValue('');
});

test('collection-only edits keep the saved key, local photos preserve configuration, and Reddit stays unavailable',async({page})=>{
 await page.goto('/');await page.getByRole('button',{name:'Backgrounds'}).click();
 await expect(page.getByText('Reddit is unavailable.')).toBeVisible();
 await expect(page.getByRole('option',{name:'Reddit'})).toHaveCount(0);
 await page.getByLabel('Wallpaper provider',{exact:true}).selectOption('unsplash');
 await page.getByLabel('Unsplash collection IDs',{exact:true}).fill('456');
 await page.getByRole('button',{name:'Review source changes'}).click();
 const review=page.locator('.provider-review');
 await expect(review).toContainText('keep');
 await expect(review).not.toContainText('New Unsplash Access Key');
 await expect(page.getByLabel('New Unsplash Access Key',{exact:true})).toHaveCount(0);
 await page.getByRole('button',{name:'Dismiss source review'}).click();
 await page.getByLabel('Wallpaper provider',{exact:true}).selectOption('local');
 await page.getByRole('button',{name:'Review source changes'}).click();
 await expect(page.locator('.provider-review')).toContainText('local');
 await expect(page.locator('.provider-review')).toContainText('keep');
 await page.getByRole('button',{name:'Dismiss source review'}).click();
 await page.getByLabel('Wallpaper provider',{exact:true}).selectOption('unsplash');
 await page.getByLabel('Saved Unsplash Access Key',{exact:true}).selectOption('remove');
 await page.getByRole('button',{name:'Review source changes'}).click();
 await expect(page.getByRole('alert').filter({hasText:'Switch to Local photos before removing'})).toBeVisible();
});

test('provider status separates saved configuration from the retrieval check',async({page})=>{
 await page.goto('/');await page.getByRole('button',{name:'Backgrounds'}).click();
 const status=page.locator('.safe-status');
 await expect(status).toContainText('Current source');
 await expect(status).toContainText('Local photos');
 await expect(status).toContainText('Saved · value never read back');
 await expect(status).toContainText('Not checked');
});

test('native provider configuration has no credential input or apply without a verified bridge',async({page})=>{
 await page.addInitScript(()=>{
  (window as any).__TAURI_INTERNALS__={invoke:async(command:string)=>{
   if(command==='discover')return [];
   if(command==='connect_device'||command==='remember_device')return null;
   if(command==='inspect_device')return {model:'TV',manufacturer:'Fixture',androidVersion:'14',home:'stock/.Home',packages:[],disabled:[],theme:null,isTv:true,errors:[],collectedAt:'fixture'};
   if(command==='personalization_state')return {editable:false,reason:'Inspection only.'};
   throw new Error('Unexpected command');
  }};
 });
 await page.goto('/');await page.getByRole('button',{name:'Find my TV'}).click();
 await page.getByLabel('Connection address',{exact:true}).fill('192.168.1.50:45737');
 await page.getByRole('button',{name:'Connect and check'}).click();
 await page.getByRole('button',{name:'Backgrounds'}).click();
 await expect(page.getByRole('status').filter({hasText:'verified private settings interface'})).toBeVisible();
 await expect(page.getByLabel('New Unsplash Access Key',{exact:true})).toHaveCount(0);
 await expect(page.getByRole('button',{name:'Apply demo source'})).toHaveCount(0);
});
