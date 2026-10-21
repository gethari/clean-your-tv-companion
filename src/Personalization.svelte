<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';

  type Asset = { id: string; sha256: string; mime: string; bytes: number; width: number; height: number; thumbnailDataUrl: string; sourceSha256?:string; sourceMime?:string; sourceBytes?:number; sourceWidth?:number; sourceHeight?:number; sourceThumbnailDataUrl?:string; cropX?:number; cropY?:number; appliedCropX?:number; appliedCropY?:number };
  type TransformedAsset = { sha256:string; mime:string; bytes:number; width:number; height:number; thumbnailDataUrl:string };
  type Rejection = { name: string; code: string };
  type ImportBatch = { accepted: Asset[]; rejected: Rejection[] };
  type Collection = { id: string; assets: { sha256: string; bytes: number; mime: string }[]; order: 'sequential'|'shuffle'; interval_minutes: number };
  type PersonalizationSnapshot = { settings: Record<string, unknown>; overrides: Record<string, unknown>; collections: Collection[]; revision: string };
  type StateReply = { editable: boolean; reason?: string; state?: PersonalizationSnapshot };
  type Review = { id: string|null; changes: {key:string;before:unknown;after:unknown}[]; collections:number; baseline_location:string|null };
  type JournalRecord = { id:string; phase:string; resulting_revision?:string|null; restore_of?:string|null };
  type Progress = { operation_id:string; phase:string; asset?:string; bytes_sent?:number; total_bytes?:number };
  type Props = { serial:string; demo:boolean; initialState?:StateReply|null };
  let { serial, demo, initialState = null }: Props = $props();

  const nativeRuntime = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
  const MAX_ASSETS = 200;
  const MAX_ASSET_BYTES = 20 * 1024 * 1024;
  const MAX_COLLECTION_BYTES = 256 * 1024 * 1024;
  const demoThumb = 'data:image/svg+xml,%3Csvg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1600 1100"%3E%3Cdefs%3E%3ClinearGradient id="g" x2="0" y2="1"%3E%3Cstop stop-color="%2384b7bb"/%3E%3Cstop offset="1" stop-color="%23f1c890"/%3E%3C/linearGradient%3E%3C/defs%3E%3Crect width="1600" height="1100" fill="url(%23g)"/%3E%3Ccircle cx="1180" cy="250" r="105" fill="%23fff1c2"/%3E%3Cpath d="M0 680 420 360l310 320 300-250 570 470H0" fill="%233a7777"/%3E%3Cpath d="m0 790 480-260 340 270 380-230 400 210v320H0" fill="%2324494c"/%3E%3C/svg%3E';
  const demoAsset: Asset = { id:'demo-sample-a', sha256:'a'.repeat(64), mime:'image/png', bytes:1024, width:1600, height:1100, thumbnailDataUrl:demoThumb, sourceSha256:'a'.repeat(64), sourceMime:'image/png', sourceBytes:1024, sourceWidth:1600, sourceHeight:1100, sourceThumbnailDataUrl:demoThumb, cropX:.5, cropY:.5 };
  const demoCollection: Collection = { id:'d'.repeat(32), assets:[{sha256:demoAsset.sha256,bytes:demoAsset.bytes,mime:demoAsset.mime}], order:'sequential', interval_minutes:15 };
  const demoReply: StateReply = { editable:true, state:{settings:{wallpaper:true,wallpaper_dim:20,active_collection:demoCollection.id},overrides:{wallpaper:true,wallpaper_dim:20,active_collection:demoCollection.id},collections:[demoCollection],revision:'e'.repeat(64)} };

  let stateReply = $state<StateReply|null>(null);
  let loading = $state(false);
  let saving = $state(false);
  let assets = $state<Asset[]>([]);
  let selectedCollectionId = $state('');
  let collectionId = $state('');
  let order = $state<'sequential'|'shuffle'>('sequential');
  let intervalText = $state('15');
  let dim = $state(0);
  let previewIndex = $state(0);
  let rejected = $state<Rejection[]>([]);
  let notice = $state('');
  let error = $state('');
  let review = $state<Review|null>(null);
  let reviewKind = $state<'apply'|'restore'>('apply');
  let reviewSubmitted = $state(false);
  let lastAppliedId = $state('');
  let progressStatus = $state('');
  let cropTimer = 0;
  let cropGeneration = 0;
  let importGeneration = 0;
  let componentActive = false;
  let cropPending = $state(false);
  let unlistenMedia: (()=>void)|null = null;
  let unlistenProgress: (()=>void)|null = null;
  let collections = $derived(stateReply?.state?.collections ?? []);
  let editable = $derived(!!stateReply?.editable);
  let currentCollection = $derived(collections.find(item => item.id === selectedCollectionId) ?? null);
  let draftSelected = $derived(!!collectionId && selectedCollectionId === collectionId);
  let selectedAssets = $derived(draftSelected ? assets : []);
  let previewAsset = $derived(selectedAssets[previewIndex] ?? selectedAssets[0] ?? null);
  let validInterval = $derived(/^\d+$/.test(intervalText) && Number(intervalText) >= 1 && Number(intervalText) <= 1440);
  let activeCollectionId = $derived(String(stateReply?.state?.settings?.active_collection ?? ''));

  function makeId() {
    const bytes = new Uint8Array(16);
    crypto.getRandomValues(bytes);
    return [...bytes].map(value => value.toString(16).padStart(2,'0')).join('');
  }
  function announceBatch(batch: ImportBatch) {
    if (saving || review) { notice='Finish or cancel the reviewed operation before adding more photos.'; return; }
    const seen = new Set(assets.map(asset => asset.sourceSha256 ?? asset.sha256));
    const accepted:Asset[]=[];
    const invalid:Rejection[]=[];
    let total=assets.reduce((sum,item)=>sum+item.bytes,0);
    for (const asset of batch.accepted) {
      if (!/^[0-9a-f]{64}$/.test(asset.sha256) || !Number.isSafeInteger(asset.bytes) || asset.bytes<1 || asset.bytes>MAX_ASSET_BYTES || !['image/jpeg','image/png','image/webp'].includes(asset.mime) || !Number.isSafeInteger(asset.width) || !Number.isSafeInteger(asset.height) || asset.width<1 || asset.height<1 || asset.width>16384 || asset.height>16384 || asset.width*asset.height>33177600) {
        invalid.push({name:asset.id || 'Photo',code:'invalid-metadata'}); continue;
      }
      if (seen.has(asset.sha256)) continue;
      if (assets.length + accepted.length >= MAX_ASSETS || total + asset.bytes > MAX_COLLECTION_BYTES) {
        invalid.push({name:asset.id || 'Photo',code:'collection-limit'}); continue;
      }
      seen.add(asset.sha256); total+=asset.bytes;
      accepted.push({...asset,sourceSha256:asset.sha256,sourceMime:asset.mime,sourceBytes:asset.bytes,sourceWidth:asset.width,sourceHeight:asset.height,sourceThumbnailDataUrl:asset.thumbnailDataUrl,cropX:.5,cropY:.5,appliedCropX:.5,appliedCropY:.5});
    }
    const added=accepted;
    if (added.length) {
      if (!collectionId) collectionId = makeId();
      assets = [...assets, ...added];
      selectedCollectionId = collectionId;
      previewIndex = 0;
    }
    rejected = [...rejected, ...batch.rejected, ...invalid];
    const duplicateCount = batch.accepted.length - added.length - invalid.length;
    notice = added.length ? `${added.length} ${added.length === 1 ? 'photo is' : 'photos are'} ready to review${duplicateCount ? ` · ${duplicateCount} duplicate${duplicateCount === 1 ? '' : 's'} skipped` : ''}. Nothing has changed on the TV.`
      : duplicateCount ? `${duplicateCount} duplicate photo${duplicateCount === 1 ? '' : 's'} skipped.` : 'No supported photos were added.';
    error = '';
  }
  function fixtureImport() {
    if (!assets.some(asset => asset.sha256 === demoAsset.sha256)) announceBatch({accepted:[demoAsset],rejected:[]});
    else notice = 'The sample photo is already in this draft.';
  }
  async function loadState() {
    if (demo) { stateReply = demoReply; selectedCollectionId = demoCollection.id; dim = 20; return; }
    if (!nativeRuntime) { stateReply = {editable:false,reason:'Open the desktop companion to import local photos.',state:undefined}; return; }
    if (initialState) { stateReply = initialState; hydrate(initialState); return; }
    loading = true; error = '';
    try { const result = await invoke<StateReply>('personalization_state',{serial}); if (!componentActive) return; stateReply = result; hydrate(result); }
    catch { stateReply = {editable:false,reason:'Could not check this TV’s personalization support. Reconnect and refresh before editing.'}; error = 'The TV state could not be checked. No changes were made.'; }
    finally { loading = false; }
  }
  async function refreshState() {
    if (demo) { await loadState(); return; }
    if (!nativeRuntime) return;
    loading=true; error=''; notice='';
    try { const refreshed=await invoke<StateReply>('personalization_state',{serial}); if (!componentActive) return; stateReply=refreshed; hydrate(refreshed); notice='TV state refreshed.'; }
    catch { error='The TV state could not be refreshed. No new changes were made.'; }
    finally { loading=false; }
  }
  function hydrate(reply:StateReply) {
    const tvCollections = reply.state?.collections ?? [];
    const activeId = String(reply.state?.settings?.active_collection ?? '');
    selectedCollectionId = tvCollections.some(item => item.id === activeId) ? activeId : (tvCollections[0]?.id ?? '');
    const current = tvCollections.find(item => item.id === selectedCollectionId);
    order = current?.order ?? (reply.state?.settings?.wallpaper_order === 'shuffle' ? 'shuffle' : 'sequential');
    intervalText = String(current?.interval_minutes ?? reply.state?.settings?.wallpaper_rotation_minutes ?? 15);
    dim = Number(reply.state?.settings?.wallpaper_dim ?? 0);
  }
  onMount(() => {
    let mounted = true;
    componentActive=true;
    void loadState();
    if (!demo && nativeRuntime) {
      void listen<ImportBatch>('media-imported', event => announceBatch(event.payload)).then(unlisten => { if (mounted) unlistenMedia = unlisten; else unlisten(); });
      void listen<Progress>('personalization-progress', event => {
        const item = event.payload;
        if (!review?.id || item.operation_id !== review.id) return;
        if (item.phase === 'transferring' && Number.isFinite(item.bytes_sent) && Number.isFinite(item.total_bytes)) {
          progressStatus = `Sending image ${item.asset?.slice(0,8) ?? ''} · ${item.bytes_sent} of ${item.total_bytes} bytes`;
        } else progressStatus = item.phase === 'activating' ? 'Applying the reviewed changes…' : item.phase === 'verifying' ? 'Checking the TV’s result…' : 'Preparing the reviewed changes…';
      }).then(unlisten => { if (mounted) unlistenProgress = unlisten; else unlisten(); });
    }
    return () => { mounted=false; componentActive=false; importGeneration++; cropGeneration++; unlistenMedia?.(); unlistenProgress?.(); window.clearTimeout(cropTimer); };
  });

  async function pick(kind:'files'|'folder') {
    if (demo) { fixtureImport(); return; }
    if (!nativeRuntime || !editable) return;
    const generation=++importGeneration;
    error = ''; notice = kind === 'folder' ? 'Choose a folder of photos…' : 'Choose photos…';
    try {
      const result = await invoke<ImportBatch>(kind === 'folder' ? 'pick_photo_folder' : 'pick_photo_files');
      if (!componentActive || generation!==importGeneration) return;
      announceBatch(result);
    } catch { error = 'Photos could not be imported. Choose the files again; the TV has not changed.'; notice = ''; }
  }
  function removeAsset(id:string) {
    const oldCollectionId=collectionId;
    if (assets[previewIndex]?.id===id) { window.clearTimeout(cropTimer); cropGeneration++; cropPending=false; }
    assets = assets.filter(asset => asset.id !== id);
    if (!assets.length) { collectionId = ''; if (selectedCollectionId === oldCollectionId) selectedCollectionId = activeCollectionId; }
    previewIndex = Math.min(previewIndex, Math.max(0, assets.length - 1));
    notice = 'Photo removed from this draft. The TV has not changed.';
  }
  function adjustCrop(axis:'x'|'y', event:Event) {
    const asset=previewAsset;
    if (!asset) return;
    const value=Number((event.currentTarget as HTMLInputElement).value);
    const cropX=axis==='x'?value:(asset.cropX ?? .5), cropY=axis==='y'?value:(asset.cropY ?? .5);
    assets=assets.map(item=>item.id===asset.id?{...item,cropX,cropY}:item);
    if (demo || !nativeRuntime || !asset.sourceSha256) return;
    window.clearTimeout(cropTimer);
    const generation=++cropGeneration;
    cropPending=true;
    cropTimer=window.setTimeout(async()=>{
      try {
        if (Math.abs(cropX-.5)<.005 && Math.abs(cropY-.5)<.005) {
          if (!componentActive || generation!==cropGeneration) return;
          assets=assets.map(item=>item.id===asset.id?{...item,sha256:item.sourceSha256!,mime:item.sourceMime!,bytes:item.sourceBytes!,width:item.sourceWidth!,height:item.sourceHeight!,thumbnailDataUrl:item.sourceThumbnailDataUrl!,appliedCropX:.5,appliedCropY:.5}:item);
          if (generation===cropGeneration) cropPending=false;
          return;
        }
        const transformed=await invoke<TransformedAsset>('transform_image',{sha256:asset.sourceSha256,cropX,cropY});
        if (!componentActive || generation!==cropGeneration) return;
        assets=assets.map(item=>item.id===asset.id?{...item,...transformed,cropX,cropY,appliedCropX:cropX,appliedCropY:cropY,sourceSha256:asset.sourceSha256,sourceMime:asset.sourceMime,sourceBytes:asset.sourceBytes,sourceWidth:asset.sourceWidth,sourceHeight:asset.sourceHeight,sourceThumbnailDataUrl:asset.sourceThumbnailDataUrl}:item);
        cropPending=false;
      } catch { if (componentActive && generation===cropGeneration) { cropPending=false; assets=assets.map(item=>item.id===asset.id?{...item,cropX:item.appliedCropX??.5,cropY:item.appliedCropY??.5}:item); error='This crop could not be prepared. The last valid crop is still in your draft.'; } }
    },180);
  }
  function selectCollection(id:string) {
    window.clearTimeout(cropTimer); cropGeneration++; cropPending=false;
    selectedCollectionId = id;
    const collection = collections.find(item => item.id === id);
    order = collection?.order ?? 'sequential';
    intervalText = String(collection?.interval_minutes ?? 15);
    previewIndex = 0;
  }
  function drop(event:DragEvent) {
    event.preventDefault();
    if (demo) { fixtureImport(); return; }
    notice = 'The desktop companion is reading the dropped items. Accepted photos will appear here.';
  }
  function collectionForApply():Collection|null {
    if (draftSelected && assets.length && collectionId) return {id:collectionId,assets:assets.map(({sha256,bytes,mime})=>({sha256,bytes,mime})),order,interval_minutes:Number(intervalText)};
    if (currentCollection) return {...currentCollection,order,interval_minutes:Number(intervalText)};
    return null;
  }
  function patchForApply(collection:Collection|null) {
    const set:Record<string,unknown> = {wallpaper:true,wallpaper_dim:dim};
    if (collection) { set.active_collection=collection.id; set.wallpaper_order=order; set.wallpaper_rotation_minutes=Number(intervalText); }
    return {set,remove:[] as string[]};
  }
  async function prepareApply() {
    error=''; notice='';
    if (!validInterval) { error='Enter a whole-number interval from 1 to 1440 minutes.'; return; }
    const collection=collectionForApply();
    if (!demo && !editable) { error=stateReply?.reason ?? 'This launcher does not support personalization editing.'; return; }
    saving=true;
    try {
      if (demo) {
        reviewSubmitted=false; reviewKind='apply'; review={id:'f'.repeat(32),changes:[{key:'wallpaper_dim',before:20,after:dim}],collections:collection && draftSelected ? 1 : 0,baseline_location:'Saved locally in the demo journal'};
        return;
      }
      const result=await invoke<Review>('prepare_personalization',{serial,patch:patchForApply(collection),collections:collection && (draftSelected || JSON.stringify(collection)!==JSON.stringify(currentCollection)) ? [collection] : []});
      if (!result.id) { notice='These settings already match the TV.'; return; }
      reviewSubmitted=false; reviewKind='apply'; review=result;
    } catch { error='A review could not be prepared. Refresh the TV and try again.'; }
    finally { saving=false; }
  }
  async function prepareRestore() {
    if (!lastAppliedId) return;
    error=''; notice=''; saving=true;
    try {
      if (demo) { reviewSubmitted=false; reviewKind='restore'; review={id:'c'.repeat(32),changes:[{key:'wallpaper_dim',before:dim,after:20}],collections:0,baseline_location:'Saved locally in the demo journal'}; return; }
      const result=await invoke<Review>('prepare_restore',{serial,id:lastAppliedId});
      if (!result.id) { notice='There is nothing from this operation to restore.'; return; }
      reviewSubmitted=false; reviewKind='restore'; review=result;
    } catch { error='A restore review could not be prepared. The TV may have changed; refresh before trying again.'; }
    finally { saving=false; }
  }
  async function applyReview() {
    if (!review?.id) return;
    saving=true; error=''; notice=''; progressStatus=reviewKind==='restore' ? 'Restoring reviewed settings…' : 'Applying reviewed collection…';
    reviewSubmitted=true;
    const operationId=review.id;
    try {
      const result:JournalRecord = demo ? {id:operationId,phase:'verified'} : await invoke<JournalRecord>('apply_personalization',{serial,id:operationId});
      if (result.phase === 'verified') {
        if (reviewKind === 'apply') lastAppliedId=result.id;
        else if (result.restore_of) lastAppliedId='';
        notice=reviewKind==='restore' ? 'Restore verified on the TV.' : 'Your collection is active and verified on the TV.';
        review=null;
        reviewSubmitted=false;
        if (!demo) {
          try { const refreshed=await invoke<StateReply>('personalization_state',{serial}); stateReply=refreshed; hydrate(refreshed); }
          catch { /* Verification is already durable in the journal. */ }
        }
      } else if (result.phase === 'needs_review') {
        error='The TV’s result needs review. Refresh its state before applying again.'; review=null; reviewSubmitted=false;
      } else {
        error='The operation is still in progress. Reconnect and reconcile this saved plan before trying another change.';
      }
    } catch {
      error='The connection ended before the result was confirmed. Reconnect and retry this saved plan to reconcile it.';
    } finally { saving=false; progressStatus=''; }
  }
  async function cancelReview() {
    if (!review?.id || demo) { review=null; reviewSubmitted=false; return; }
    saving=true; error='';
    try { await invoke('cancel_personalization',{serial,id:review.id}); review=null; reviewSubmitted=false; notice='The staged operation was cancelled.'; }
    catch { error='Cancellation could not be confirmed. Reconnect and reconcile the saved operation.'; }
    finally { saving=false; }
  }
  function friendly(code:string) {
    const reasons:Record<string,string>={'unsupported-format':'This file format is not supported.','corrupt-image':'The image could not be decoded.','file-too-large':'This image exceeds the 20 MiB file limit.','image-dimensions-exceeded':'This image exceeds the pixel or dimension limit.','normalized-file-too-large':'The prepared image exceeds the size limit.','thumbnail-too-large':'The preview could not be prepared within the size limit.','cache-unavailable':'The local photo cache is unavailable.','unreadable-folder':'This folder could not be read.','directory-entry-limit':'This folder contains too many items to scan safely.','unreadable-item':'This item could not be read.','symlink-not-allowed':'Linked files are not followed when importing a folder.','outside-selected-folder':'This item is outside the selected folder.','not-a-regular-file':'This item is not a regular image file.','image-processor-unavailable':'The image processor is unavailable.','collection-item-limit':'This collection has reached its 200 photo limit.','invalid-metadata':'This photo’s checked metadata was incomplete.','collection-limit':'This collection has reached its 200 photo or 256 MiB limit.'};
    return reasons[code] ?? 'This item could not be imported.';
  }
</script>

<section class="personalization" aria-labelledby="personalization-title" ondragover={(event)=>event.preventDefault()} ondrop={drop}>
  <header class="personalization-head"><div><p class="eyebrow">MAKE THE HOME SCREEN YOURS</p><h1 id="personalization-title">Your photo collection</h1><p class="intro">Choose local photos, preview them on a TV-shaped screen, and review every change before it reaches your TV.</p></div><div class="head-actions"><span class="privacy-label">Local photos stay on your devices</span>{#if !demo}<button type="button" class="refresh-button" onclick={refreshState} disabled={loading||saving||!!review}>Refresh TV state</button>{/if}</div></header>

  {#if demo}<p class="demo-note" role="status">Demo mode · changes use sample data and never contact a TV.</p>{/if}
  {#if loading}<p class="status-card" role="status">Checking this TV’s companion support…</p>{/if}
  {#if error}<p class="message error" role="alert">{error}</p>{/if}
  {#if notice}<p class="message" role="status">{notice}</p>{/if}

  {#if stateReply && !editable && !demo}
    <section class="unsupported" aria-label="Personalization is read only"><span class="lock">◌</span><div><h2>Personalization is read-only</h2><p>{stateReply.reason ?? 'This launcher does not support safe personalization yet.'}</p><p>Your TV remains unchanged. Use a compatible launcher to create and restore reviewed photo collections.</p></div></section>
  {:else}
    <div class="workspace">
      <div class="editor-column">
        <section class="panel import-panel" aria-labelledby="import-title">
          <div class="panel-heading"><div><span class="step">01</span><h2 id="import-title">Add local photos</h2></div><span class="format-note">JPEG · PNG · WebP</span></div>
          <p>Choose photos or a folder. We copy the images once so the TV can use them without your computer.</p>
          <div class="import-actions">
            <button type="button" class="primary" onclick={()=>pick('files')} disabled={saving || !!review || (!demo && !editable)}>{demo ? 'Add sample photo' : 'Choose photos'}</button>
            <button type="button" class="secondary" onclick={()=>pick('folder')} disabled={saving || !!review || (!demo && !editable)}>Choose folder</button>
          </div>
          <div class="drop-hint" aria-hidden="true"><span>＋</span> Or drop photos here</div>
          {#if rejected.length}<div class="rejections" aria-live="polite"><h3>Couldn’t add these items</h3>{#each rejected as item, index}<p><strong>{item.name || `Item ${index+1}`}</strong> · {friendly(item.code)}</p>{/each}</div>{/if}
        </section>

        <section class="panel collection-panel" aria-labelledby="collection-title">
          <div class="panel-heading"><div><span class="step">02</span><h2 id="collection-title">Collection settings</h2></div></div>
          {#if collections.length || assets.length}
            <label class="field-label" for="collection-select">Collection to use</label>
            <select id="collection-select" value={selectedCollectionId} onchange={event=>selectCollection(event.currentTarget.value)} disabled={saving||!!review}>
              {#each collections as collection}<option value={collection.id}>{collection.id===activeCollectionId ? 'Current TV collection' : 'Saved TV collection'} · {collection.assets.length} {collection.assets.length===1?'photo':'photos'}</option>{/each}
              {#if assets.length}<option value={collectionId}>New local collection · {assets.length} {assets.length===1?'photo':'photos'}</option>{/if}
            </select>
          {:else}<p class="empty-collection">Add at least one photo to create a collection.</p>{/if}
          <div class="setting-grid">
            <div><label class="field-label" for="rotation-order">Photo order</label><select id="rotation-order" bind:value={order} disabled={saving||!!review}><option value="sequential">In order</option><option value="shuffle">Shuffle</option></select></div>
            <div><label class="field-label" for="rotation-interval">Change photo every</label><div class="interval-control"><input id="rotation-interval" type="number" min="1" max="1440" step="1" bind:value={intervalText} aria-describedby="interval-help" disabled={saving||!!review}/><span>minutes</span></div></div>
          </div>
          <div class="interval-presets" aria-label="Common rotation intervals">{#each [5,15,30] as minutes}<button type="button" class:chosen={intervalText===String(minutes)} onclick={()=>intervalText=String(minutes)} disabled={saving||!!review}>{minutes} min</button>{/each}<span id="interval-help">Choose 1–1440 minutes.</span></div>
          {#if !validInterval}<p class="field-error" role="alert">Enter a whole number from 1 to 1440.</p>{/if}
          <div class="dim-control"><label class="field-label" for="wallpaper-dim">Background dimming <strong>{dim}%</strong></label><input id="wallpaper-dim" type="range" min="0" max="100" step="1" bind:value={dim} disabled={saving||!!review}/><p>Darkens the image behind TV menus.</p></div>
        </section>

        {#if selectedAssets.length}
          <section class="panel photo-panel" aria-labelledby="photos-title"><div class="panel-heading"><div><span class="step">03</span><h2 id="photos-title">Photos in this draft</h2></div><span class="count">{selectedAssets.length}</span></div>
            <ul class="photo-list">{#each selectedAssets as asset, index}<li class:active={index===previewIndex}><button class="photo-select" type="button" onclick={()=>{window.clearTimeout(cropTimer);cropGeneration++;cropPending=false;previewIndex=index;}} aria-label={`Preview photo ${index+1}`} aria-pressed={index===previewIndex} disabled={saving||!!review}><img src={asset.thumbnailDataUrl} alt=""/><span>Photo {index+1}</span></button><button class="remove-photo" type="button" aria-label={`Remove photo ${index+1}`} onclick={()=>removeAsset(asset.id)} disabled={saving||!!review}>Remove</button></li>{/each}</ul>
          </section>
        {/if}
      </div>

      <aside class="preview-column" aria-label="TV preview and review">
        <section class="panel preview-panel"><div class="panel-heading"><div><span class="step">PREVIEW</span><h2>On your TV</h2></div></div>
          <div class="tv-frame"><div class="tv-screen" aria-label="16 by 9 center-crop preview" role="img">
            {#if previewAsset}<img src={previewAsset.thumbnailDataUrl} alt="" style={`object-position:${demo?(previewAsset.cropX??.5)*100:50}% ${demo?(previewAsset.cropY??.5)*100:50}%;`}/><div class="dim-overlay" style={`background:rgba(0,0,0,${Number(dim)/100})`}></div>
            {:else}<div class="preview-placeholder"><span>▧</span><p>{currentCollection?.assets.length?'TV photos are stored on the TV':'Your photo will appear here'}</p></div>{/if}
            <div class="screen-menu" aria-hidden="true"><i></i><i></i><i></i></div>
          </div><div class="tv-neck"></div></div>
          <p class="preview-caption">16:9 TV crop · {selectedAssets.length || currentCollection?.assets.length || 0} {selectedAssets.length===1 || currentCollection?.assets.length===1?'photo':'photos'} · {order==='shuffle'?'shuffled':'in order'}</p>
          <p class="preview-note">{previewAsset?'The launcher fills the screen and places the selected crop behind its menus.':currentCollection?'Preview unavailable: saved photos stay on the TV.':'The launcher fills the screen and places the selected crop behind its menus.'}</p>
          {#if previewAsset && selectedAssets.length}
            <div class="crop-controls"><label for="crop-x">Horizontal crop <span>{Math.round((previewAsset.cropX??.5)*100)}%</span></label><input id="crop-x" type="range" min="0" max="1" step="0.01" value={previewAsset.cropX??.5} oninput={event=>adjustCrop('x',event)} aria-label="Horizontal crop" disabled={saving||!!review}/><label for="crop-y">Vertical crop <span>{Math.round((previewAsset.cropY??.5)*100)}%</span></label><input id="crop-y" type="range" min="0" max="1" step="0.01" value={previewAsset.cropY??.5} oninput={event=>adjustCrop('y',event)} aria-label="Vertical crop" disabled={saving||!!review}/>{#if cropPending}<p class="crop-pending" role="status">Preparing this crop for the TV…</p>{/if}</div>
          {/if}
        </section>

        <section class="panel apply-panel"><div class="panel-heading"><div><span class="step">04</span><h2>Review before applying</h2></div></div>
          <p>Nothing is sent to the TV until you review and apply. Your previous settings are saved for restore.</p>
          <button class="primary apply-button" type="button" onclick={prepareApply} disabled={saving || !!review || cropPending || !validInterval || (!assets.length && !currentCollection) || (!demo && !editable)}>{saving?'Preparing review…':cropPending?'Preparing photo crop…':review?'Review prepared':'Review changes'}</button>
          {#if lastAppliedId}<button class="text-action" type="button" onclick={prepareRestore} disabled={saving||!!review}>Review restore for last change</button>{/if}
          {#if progressStatus}<div class="transfer-status" role="status" aria-live="polite"><span class="spinner" aria-hidden="true"></span><span>{progressStatus}</span></div>{/if}
          {#if review}<div class="review-card" aria-labelledby="review-title"><h3 id="review-title">{reviewKind==='restore'?'Restore review':'Change review'}</h3><p>{review.changes.length} setting {review.changes.length===1?'change':'changes'} · {review.collections} collection {review.collections===1?'update':'updates'}</p>
            <ul>{#each review.changes as change}<li><span>{change.key.replaceAll('_',' ')}</span><strong>{change.before===null?'Not set':String(change.before)} → {change.after===null?'Not set':String(change.after)}</strong></li>{/each}</ul>
            {#if review.baseline_location}<p class="baseline-note">Recovery baseline saved locally at <code>{review.baseline_location}</code>.</p>{/if}
            <div class="review-actions"><button class="primary" type="button" onclick={applyReview} disabled={saving}>{saving?'Applying…':reviewSubmitted?'Reconcile saved plan':reviewKind==='restore'?'Apply restore':'Apply reviewed changes'}</button><button class="secondary" type="button" onclick={cancelReview} disabled={saving}>Cancel review</button></div>
          </div>{/if}
        </section>
      </aside>
    </div>
  {/if}
  <p class="provider-note">Local collection photos are copied to your computer’s app cache, then sent to the TV when you apply a reviewed change.</p>
</section>

<style>
  .personalization{--ink:#183535;--muted:#617573;--line:#d9e4df;--paper:#fbfdfb;--mint:#eaf3ee;--accent:#276b60;color:var(--ink);max-width:1120px;margin:0 auto;padding:30px 0 54px;font-family:inherit}
  .personalization *{box-sizing:border-box}.personalization-head{display:flex;align-items:flex-end;justify-content:space-between;gap:24px;margin-bottom:24px}.eyebrow{font-size:11px;letter-spacing:.15em;font-weight:800;color:#57817a;margin:0 0 9px}.personalization h1{font-size:clamp(30px,4vw,44px);line-height:1.08;letter-spacing:-.04em;margin:0 0 10px}.intro{color:var(--muted);max-width:650px;margin:0;line-height:1.6}.privacy-label{white-space:nowrap;color:#397065;background:var(--mint);padding:9px 13px;border-radius:99px;font-size:12px;font-weight:700}.demo-note,.message,.status-card{padding:12px 15px;border-radius:12px;background:#eaf4ef;color:#25564e;margin:0 0 16px}.message.error{background:#fff0ec;color:#8d3324}.workspace{display:grid;grid-template-columns:minmax(0,1.1fr) minmax(310px,.9fr);gap:20px;align-items:start}.editor-column,.preview-column{display:grid;gap:16px}.panel{background:#fff;border:1px solid var(--line);border-radius:18px;padding:20px;box-shadow:0 6px 20px #18353508}.panel-heading{display:flex;align-items:center;justify-content:space-between;gap:12px;margin-bottom:9px}.panel-heading>div{display:flex;align-items:center;gap:10px}.panel h2{font-size:18px;letter-spacing:-.02em;margin:0}.panel p{color:var(--muted);font-size:13px;line-height:1.55;margin:0 0 14px}.step{color:#669188;font-size:10px;letter-spacing:.12em;font-weight:800}.format-note,.count{font-size:11px;color:var(--muted)}.import-actions{display:flex;gap:10px;flex-wrap:wrap}.primary,.secondary{min-height:42px;border-radius:11px;padding:0 16px;font:inherit;font-size:13px;font-weight:750;cursor:pointer}.primary{background:var(--accent);border:1px solid var(--accent);color:white}.primary:hover:not(:disabled){background:#1c5149}.secondary{background:#fff;border:1px solid #b9ccc5;color:var(--ink)}.primary:disabled,.secondary:disabled{opacity:.5;cursor:not-allowed}.drop-hint{margin-top:15px;border:1px dashed #b8cbc3;border-radius:12px;padding:12px;color:#71827f;font-size:12px;text-align:center}.drop-hint span{font-size:19px;vertical-align:-2px;margin-right:5px}.rejections{margin-top:14px;padding-top:13px;border-top:1px solid var(--line)}.rejections h3{font-size:12px;margin:0 0 7px}.rejections p{font-size:11px;margin:5px 0}.field-label{display:flex;justify-content:space-between;gap:10px;font-size:12px;font-weight:750;margin:13px 0 7px}.field-label strong{color:var(--accent)}select,.interval-control input{width:100%;min-height:42px;border:1px solid #c8d6d0;border-radius:10px;padding:0 11px;background:white;color:var(--ink);font:inherit;font-size:13px}.setting-grid{display:grid;grid-template-columns:1fr 1fr;gap:13px;margin-top:5px}.setting-grid>div{min-width:0}.interval-control{position:relative}.interval-control input{padding-right:65px}.interval-control span{position:absolute;right:11px;top:13px;color:var(--muted);font-size:11px}.interval-presets{display:flex;align-items:center;gap:7px;margin-top:9px;flex-wrap:wrap}.interval-presets button{border:1px solid var(--line);background:#fff;border-radius:99px;padding:6px 10px;color:#48615e;font-size:11px;cursor:pointer}.interval-presets button.chosen{background:var(--mint);border-color:#8fb5a9;color:#20564d}.interval-presets span{font-size:10px;color:#758582;margin-left:auto}.field-error{color:#9e3727!important;margin:7px 0!important}.dim-control{margin-top:17px;padding-top:14px;border-top:1px solid var(--line)}.dim-control input{width:100%;accent-color:var(--accent)}.dim-control p{font-size:11px;margin:3px 0 0}.empty-collection{padding:11px;background:#f5f8f6;border-radius:10px}.photo-list{list-style:none;padding:0;margin:10px 0 0;display:grid;gap:7px}.photo-list li{display:flex;align-items:center;justify-content:space-between;padding:6px 8px;border:1px solid var(--line);border-radius:11px;gap:10px}.photo-list li.active{border-color:#6fa094;background:#f5faf7}.photo-select{display:flex;align-items:center;gap:10px;border:0;background:none;color:var(--ink);font:inherit;font-size:12px;cursor:pointer}.photo-select img{width:54px;height:34px;object-fit:cover;border-radius:6px}.remove-photo,.text-action{border:0;background:none;color:#8a473a;font:inherit;font-size:11px;text-decoration:underline;cursor:pointer;padding:8px}.tv-frame{padding:10px 16px 0}.tv-screen{position:relative;aspect-ratio:16/9;overflow:hidden;background:linear-gradient(135deg,#d1e7dd,#8eb4b1);border:7px solid #24403d;border-radius:10px;box-shadow:0 9px 17px #152e2b20}.tv-screen>img{width:100%;height:100%;object-fit:cover;object-position:center;display:block}.dim-overlay{position:absolute;inset:0;pointer-events:none}.preview-placeholder{height:100%;display:grid;align-content:center;justify-items:center;color:#42635e}.preview-placeholder span{font-size:32px}.preview-placeholder p{font-size:12px;margin:4px}.screen-menu{position:absolute;left:6%;right:6%;bottom:11%;display:flex;gap:9px}.screen-menu i{height:21px;width:54px;border-radius:4px;background:#f6f7ef;opacity:.88}.tv-neck{width:52px;height:12px;background:#29413e;margin:0 auto;position:relative}.tv-neck:after{content:"";position:absolute;width:90px;height:5px;background:#29413e;border-radius:9px;bottom:-4px;left:50%;transform:translateX(-50%)}.preview-caption{text-align:center;font-size:11px!important;margin:16px 0 7px!important;color:#415d59!important}.preview-note{font-size:11px!important;margin:0!important}.apply-panel>p{margin-bottom:13px}.apply-button{width:100%}.text-action{display:block;margin:5px auto 0;color:#28665c}.transfer-status{display:flex;align-items:center;gap:9px;background:#eff6f2;border-radius:10px;padding:10px;margin-top:13px;font-size:11px;color:#2b5c54}.spinner{width:13px;height:13px;border:2px solid #b9d1c8;border-top-color:#276b60;border-radius:50%;animation:spin .8s linear infinite;flex:0 0 auto}@keyframes spin{to{transform:rotate(360deg)}}.review-card{margin-top:15px;padding:14px;border:1px solid #a8c5b9;border-radius:13px;background:#f7fbf8}.review-card h3{font-size:15px;margin:0 0 6px}.review-card>p{font-size:11px;margin-bottom:9px}.review-card ul{list-style:none;margin:0;padding:0}.review-card li{display:flex;justify-content:space-between;gap:12px;padding:8px 0;border-top:1px solid #dfebe4;font-size:11px}.review-card li strong{text-align:right}.baseline-note{padding-top:8px;border-top:1px solid #dfebe4}.review-actions{display:grid;gap:7px}.baseline-note{margin:10px 0!important}.unsupported{display:flex;gap:16px;background:#f7faf8;border:1px solid var(--line);border-radius:16px;padding:24px}.unsupported h2{font-size:19px;margin:0 0 8px}.unsupported p{color:var(--muted);font-size:13px;line-height:1.55}.lock{font-size:25px;color:#6c9289}.provider-note{margin:18px 0 0;border-top:1px solid var(--line);padding-top:13px;font-size:11px;color:var(--muted)}
  @media(max-width:760px){.personalization{padding:18px 0 34px}.personalization-head{display:block}.privacy-label{display:inline-block;margin-top:15px}.workspace{grid-template-columns:1fr}.preview-column{grid-row:1}.setting-grid{grid-template-columns:1fr 1fr}.panel{padding:16px}.interval-presets span{width:100%;margin:2px 0 0}.tv-frame{padding:8px 4px 0}}
  @media(max-width:420px){.setting-grid{grid-template-columns:1fr}.import-actions>*{flex:1}.review-card li{display:grid;gap:3px}.review-card li strong{text-align:left}}
  @media(prefers-reduced-motion:reduce){.spinner{animation:none}}
</style>
