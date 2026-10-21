<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  type Operation = {
    id:string;
    phase:'prepared'|'staged'|'activating'|'verified'|'needs_review'|'cancelled'|'restored'|string;
    owned_keys:string[];
    collection_count:number;
    resulting_revision?:string|null;
    restore_of?:string|null;
  };
  type History = { operations:Operation[]; unreadable_records:number; truncated:boolean };
  type Change = {key:string;before:unknown;after:unknown};
  type Review = {id:string|null;changes:Change[];collections:number;baseline_location:string|null};
  type Props = {serial:string;demo:boolean};
  let {serial,demo}:Props=$props();

  const nativeRuntime=typeof window!=='undefined' && '__TAURI_INTERNALS__' in window;
  const demoHistory:History={operations:[
    {id:'c'.repeat(32),phase:'verified',owned_keys:['wallpaper','wallpaper_dim'],collection_count:1,resulting_revision:'b'.repeat(64),restore_of:null},
    {id:'a'.repeat(32),phase:'activating',owned_keys:['wallpaper'],collection_count:0,resulting_revision:null,restore_of:null}
  ],unreadable_records:0,truncated:false};

  let history=$state<History|null>(null);
  let loading=$state(false);
  let busyId=$state('');
  let error=$state('');
  let notice=$state('');
  let review=$state<Review|null>(null);
  let checkedUncertain=$state(new Set<string>());
  let generation=0;

  async function refresh() {
    const request=++generation;
    review=null; error=''; notice='';
    if(demo) { history=demoHistory; return; }
    history=null;
    if(!nativeRuntime || !serial) { history={operations:[],unreadable_records:0,truncated:false}; return; }
    loading=true;
    try {
      const result=await invoke<History>('list_personalization_history',{serial});
      if(request===generation) history=result;
    } catch {
      if(request===generation) error='Could not verify this TV and load its local operation history. Refresh the TV before continuing.';
    } finally {
      if(request===generation) loading=false;
    }
  }

  onMount(()=>{
    void refresh();
    return ()=>{generation++;};
  });

  async function reconcile(operation:Operation) {
    if(demo || busyId) return;
    busyId=operation.id; error=''; notice='';
    try {
      const result=await invoke<Operation>('reconcile_personalization',{serial,id:operation.id});
      const resultMessage=result.phase==='verified' ? 'The TV confirmed this operation is active.'
        : result.phase==='needs_review' ? 'The TV state changed. This operation needs review; no retry was sent.'
        : result.phase==='activating' ? 'The result is still uncertain. No retry was sent; reconnect and reconcile again.'
        : `Current operation status: ${result.phase.replaceAll('_',' ')}.`;
      if(result.phase==='activating') checkedUncertain=new Set([...checkedUncertain,operation.id]);
      await refresh(); notice=resultMessage;
    } catch {
      error='The TV identity or operation state could not be verified. No activation was retried.';
    } finally { busyId=''; }
  }

  async function reviewRestore(operation:Operation) {
    if(demo || busyId) return;
    busyId=operation.id; error=''; notice='';
    try {
      review=await invoke<Review>('prepare_restore',{serial,id:operation.id});
      if(!review.id) { notice='There is no companion-owned change to restore.'; review=null; }
    } catch {
      error='Restore could not be prepared. The TV may have changed; refresh and review its current state.';
    } finally { busyId=''; }
  }

  async function applyRestore() {
    if(demo || busyId || !review?.id) return;
    const id=review.id; busyId=id; error=''; notice='';
    try {
      const result=await invoke<Operation>('apply_personalization',{serial,id});
      const resultMessage=result.phase==='verified' ? 'Restore verified on the TV.'
        : result.phase==='needs_review' ? 'The TV changed during restore. The operation needs review; no success is claimed.'
        : `Restore status: ${result.phase.replaceAll('_',' ')}. Refresh the history before continuing.`;
      review=null; await refresh(); notice=resultMessage;
    } catch (cause) {
      console.error('restore failure',cause);
      error='Restore did not complete with a verified result. Refresh history before retrying.';
    } finally { busyId=''; }
  }

  async function resume(operation:Operation) {
    if(demo || busyId) return;
    busyId=operation.id; error=''; notice='';
    try {
      const result=await invoke<Operation>('apply_personalization',{serial,id:operation.id});
      const resultMessage=result.phase==='verified' ? 'The saved operation is verified on the TV.'
        : result.phase==='needs_review' ? 'The TV state changed. This operation needs review; no success is claimed.'
        : result.phase==='activating' ? 'The result remains uncertain. Activation was not confirmed; reconcile before any further retry.'
        : `Operation status: ${result.phase.replaceAll('_',' ')}.`;
      checkedUncertain=new Set([...checkedUncertain].filter(value=>value!==operation.id));
      await refresh(); notice=resultMessage;
    } catch {
      error='The saved operation could not be resumed safely. Refresh its status and review the TV state.';
    } finally { busyId=''; }
  }

  async function cancelReview() {
    if(demo || busyId || !review?.id) return;
    const id=review.id; busyId=id; error=''; notice='';
    try {
      const result=await invoke<Operation>('cancel_personalization',{serial,id});
      if(result.phase!=='cancelled') {
        error='The restore plan was not confirmed cancelled. Its review remains open; refresh the TV before continuing.';
        return;
      }
      review=null; await refresh(); notice='Restore review cancelled. The TV was not changed.';
    } catch {
      error='The restore plan could not be confirmed cancelled. Its review remains open; refresh the TV before continuing.';
    } finally { busyId=''; }
  }

  function titlePhase(phase:string) {
    return ({prepared:'Ready to resume',staged:'Assets staged',activating:'Outcome uncertain',verified:'Verified',needs_review:'Needs review',cancelled:'Cancelled',restored:'Restored'} as Record<string,string>)[phase] ?? 'Unknown status';
  }
</script>

<section class="history" aria-labelledby="history-title">
  <header class="history-head">
    <div><p class="eyebrow">PRIVATE TO THIS VERIFIED TV</p><h1 id="history-title">Operation history</h1>
      <p class="intro">Review saved background changes, reconcile uncertain results, or prepare a restore that checks for later changes first.</p></div>
    <button class="secondary" type="button" onclick={refresh} disabled={loading || !!busyId || demo}>Refresh history</button>
  </header>

  {#if demo}<p class="demo-note">Example only · no TV is connected and no commands are sent.</p>{/if}
  {#if error}<p class="message error" role="alert">{error}</p>{/if}
  {#if notice}<p class="message" role="status" aria-live="polite">{notice}</p>{/if}
  {#if loading}<p class="status" role="status">Verifying this TV and loading its history…</p>
  {:else if history}
    {#if history.unreadable_records}<p class="message warning" role="status">Some local history files could not be read ({history.unreadable_records}). Other valid history remains available.</p>{/if}
    {#if history.truncated}<p class="message warning" role="status">History reached the 1,000 record limit. Refresh after older records are archived to inspect more.</p>{/if}
    {#if history.operations.length}
      <div class="operation-list" aria-label="Saved operations">
        {#each history.operations as operation (operation.id)}
          <article class="operation-card">
            <div class="operation-top"><div><span class="phase" data-phase={operation.phase}>{titlePhase(operation.phase)}</span><h2>Change {operation.id.slice(0,8)}</h2></div>
              <p>{operation.owned_keys.length} setting {operation.owned_keys.length===1?'key':'keys'} · {operation.collection_count} collection {operation.collection_count===1?'update':'updates'}</p></div>
            {#if operation.phase==='activating'}<p class="detail">The desktop closed or lost connection during activation. Reconciliation reads the TV result and current state without retrying activation.</p>{/if}
            {#if operation.phase==='needs_review'}<p class="detail">The observed TV state differs from this operation. Refresh the selected TV and prepare a new review from its current state before making another change.</p>{/if}
            {#if operation.phase==='verified' && operation.restore_of}<p class="detail">This operation restores change {operation.restore_of.slice(0,8)}.</p>{/if}
            <div class="actions">
              {#if operation.phase==='activating'}<button class="secondary" type="button" onclick={()=>reconcile(operation)} disabled={demo || !!busyId}>{busyId===operation.id?'Checking…':'Reconcile with TV'}</button>{/if}
              {#if operation.phase==='prepared' || operation.phase==='staged'}<button class="secondary" type="button" onclick={()=>resume(operation)} disabled={demo || !!busyId}>{busyId===operation.id?'Resuming…':'Resume reviewed operation'}</button>{/if}
              {#if operation.phase==='activating' && checkedUncertain.has(operation.id)}<button class="secondary" type="button" onclick={()=>resume(operation)} disabled={demo || !!busyId}>{busyId===operation.id?'Checking and applying…':'Retry saved activation after check'}</button>{/if}
              {#if operation.phase==='verified'}<button class="secondary" type="button" onclick={()=>reviewRestore(operation)} disabled={demo || !!busyId}>{busyId===operation.id?'Checking current state…':'Review restore'}</button>{/if}
            </div>
          </article>
        {/each}
      </div>
    {:else}<p class="empty">No saved operations were found for this verified TV.</p>{/if}
  {/if}

  {#if review}<section class="restore-review" aria-labelledby="restore-title">
    <p class="eyebrow">CHECKED AGAINST CURRENT TV STATE</p><h2 id="restore-title">Restore review</h2>
    <p>{review.changes.length} setting {review.changes.length===1?'change':'changes'} and {review.collections} collection {review.collections===1?'update':'updates'} will be restored if the reviewed values still match.</p>
    {#if review.changes.length}<ul>{#each review.changes as change}<li>Restore companion-owned <strong>{change.key.replaceAll('_',' ')}</strong></li>{/each}</ul>{/if}
    <p class="detail">A later manual TV change causes a conflict and will be kept. The saved baseline remains private on this computer.</p>
    <div class="actions"><button class="primary" type="button" onclick={applyRestore} disabled={!!busyId}>{busyId?'Applying reviewed restore…':'Apply reviewed restore'}</button><button class="secondary" type="button" onclick={cancelReview} disabled={!!busyId}>{busyId?'Checking cancellation…':'Cancel review'}</button></div>
  </section>{/if}
</section>

<style>
  .history{--ink:#183535;--muted:#617573;--line:#d9e4df;--accent:#276b60;color:var(--ink);max-width:1000px;margin:0 auto;padding:30px 0 54px;font:inherit}
  .history *{box-sizing:border-box}.history-head{display:flex;justify-content:space-between;align-items:end;gap:20px;margin-bottom:22px}.history h1{font-size:clamp(30px,4vw,42px);line-height:1.08;letter-spacing:-.04em;margin:0 0 8px}.intro{max-width:690px;color:var(--muted);line-height:1.6;margin:0}.eyebrow{font-size:11px;letter-spacing:.14em;font-weight:800;color:#57817a;margin:0 0 9px}.secondary,.primary{min-height:42px;border-radius:11px;padding:0 15px;font:inherit;font-size:13px;font-weight:750;cursor:pointer}.secondary{background:#fff;border:1px solid #b9ccc5;color:var(--ink)}.primary{background:var(--accent);border:1px solid var(--accent);color:white}.secondary:disabled,.primary:disabled{opacity:.5;cursor:not-allowed}.demo-note,.message,.status,.empty{padding:13px 15px;border-radius:12px;background:#eaf4ef;color:#25564e;margin:0 0 15px}.message.error{background:#fff0ec;color:#8d3324}.message.warning{background:#fff8e8;color:#69501e}.operation-list{display:grid;gap:12px}.operation-card,.restore-review{padding:18px;border:1px solid var(--line);border-radius:16px;background:#fff;box-shadow:0 6px 20px #18353508}.operation-top{display:flex;justify-content:space-between;gap:14px;align-items:start}.operation-top h2,.restore-review h2{font-size:17px;margin:8px 0 0}.operation-top p{color:var(--muted);font-size:12px;margin:0;text-align:right}.phase{display:inline-flex;border-radius:99px;padding:5px 9px;background:#edf3f0;color:#416860;font-size:11px;font-weight:750}.phase[data-phase="activating"]{background:#fff1db;color:#74520f}.phase[data-phase="needs_review"]{background:#fff0ec;color:#8d3324}.detail{color:var(--muted);font-size:12px;line-height:1.55;margin:12px 0}.actions{display:flex;gap:9px;flex-wrap:wrap;margin-top:12px}.restore-review{margin-top:20px;border-color:#9ebcae;background:#f7fbf8}.restore-review>p:not(.eyebrow){color:var(--muted);font-size:13px;line-height:1.55}.restore-review ul{margin:12px 0;padding-left:20px}.restore-review li{font-size:13px;padding:5px 0}.empty{background:#f5f8f6;color:var(--muted)}
  @media(max-width:600px){.history{padding:18px 0 36px}.history-head{display:grid;align-items:start}.operation-top{display:grid}.operation-top p{text-align:left}.actions>*{flex:1}}
  @media(prefers-reduced-motion:reduce){*{scroll-behavior:auto}}
</style>
