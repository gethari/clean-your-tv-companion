<script lang="ts">
  import { onDestroy } from 'svelte';
  import { PROVIDER_CHOICES, providerCapabilities, safeProviderStatus, reviewProviderDraft } from './provider-configuration.mjs';
  let { demo }: { demo: boolean } = $props();
  // Only the demo supplies a configuration bridge today. Upstream capability evidence is
  // recorded in docs/UPSTREAM_SUPPORT.md; do not infer a private bridge from a version name.
  const demoCaps = {providers:['local','unsplash','auto'],configuration:true,privateCredentials:true};
  let observed = $state({provider:'local',collectionIds:['123456'],credentialPresent:true,configurationSaved:true,health:'unknown'});
  let provider = $state('local');
  let collectionIds = $state('123456');
  let credentialAction = $state('keep');
  let accessKey = $state('');
  let errors = $state<Record<string,string>>({});
  let review = $state<{provider:string|null;collectionIds:readonly string[];credentialAction:string|null;changed:boolean}|null>(null);
  let notice = $state('');
  let support = $derived(providerCapabilities(demo ? demoCaps : {}));
  let status = $derived(safeProviderStatus(demo ? observed : {}));
  function prepare() {
    const result=reviewProviderDraft({provider,collectionIds,credentialAction,accessKey},observed,demoCaps);
    errors=result.errors;
    if (!result.valid) return;
    if (!result.review.changed) { notice='This configuration already matches the demo.'; return; }
    review=result.review; notice='';
  }
  function dismiss() { review=null;accessKey='';credentialAction='keep';notice='Review dismissed. The entered key was cleared.'; }
  function applyDemo() {
    if (!demo || !review) return;
    observed={provider:review.provider ?? 'local',collectionIds:[...review.collectionIds],credentialPresent:review.credentialAction==='remove'?false:review.credentialAction==='replace'?true:observed.credentialPresent,configurationSaved:true,health:'unknown'};
    review=null;accessKey='';credentialAction='keep';notice='Demo configuration saved. Image retrieval has not been checked. No TV or provider was contacted.';
  }
  onDestroy(()=>{accessKey='';});
</script>

<section class="provider-panel" aria-labelledby="provider-title">
  <p class="eyebrow">WALLPAPER SOURCE</p><h2 id="provider-title">Choose where photos come from</h2>
  {#if !support.editable}
    <p class="availability" role="status">Provider configuration needs a compatible upstream launcher with a verified private settings interface. Existing provider settings stay on the TV. Local photo editing is available when the launcher supports it.</p>
    <p>The MVP includes Unsplash Access Key and collection settings. This build cannot safely apply them to the current upstream release yet.</p>
  {:else}
    <p>Demo · configure an example source. Your Access Key stays in this form’s memory and is never sent.</p>
    <fieldset disabled={!!review}>
      <label for="wallpaper-provider">Wallpaper provider</label>
      <select id="wallpaper-provider" bind:value={provider}>{#each PROVIDER_CHOICES as item}<option value={item.id}>{item.label}</option>{/each}</select>
      <p>Automatic follows the launcher’s configured provider selection, with local fallback.</p>
      <label for="provider-collections">Unsplash collection IDs</label>
      <textarea id="provider-collections" bind:value={collectionIds} rows="2" placeholder="123456, abc_9" aria-invalid={!!errors.collectionIds}></textarea>
      <p>Separate collection IDs with commas or new lines. Changing to local photos keeps the saved provider configuration.</p>
      <label for="provider-key-action">Saved Unsplash Access Key</label>
      <select id="provider-key-action" bind:value={credentialAction}><option value="keep">Keep saved key</option><option value="replace">Replace key</option><option value="remove">Remove saved key</option></select>
      {#if credentialAction==='replace'}<label for="unsplash-access-key">New Unsplash Access Key</label><input id="unsplash-access-key" type="password" bind:value={accessKey} maxlength="200" autocomplete="off" spellcheck="false" aria-invalid={!!errors.accessKey}/><p>Use the Access Key. The Secret Key is not needed.</p>{/if}
    </fieldset>
    {#each Object.values(errors) as message}<p class="error" role="alert">{message}</p>{/each}
    <p class="safe-status">{status.credentialPresent ? 'A key is saved on the example TV.' : 'No key is saved.'} {status.configurationSaved ? 'Configuration saved.' : ''} {status.message}</p>
    {#if !review}<button type="button" onclick={prepare}>Review source changes</button>{:else}
      <div class="provider-review"><h3>Wallpaper source review</h3><dl><dt>Provider</dt><dd>{review.provider}</dd><dt>Collection IDs</dt><dd>{review.collectionIds.join(', ')||'None'}</dd><dt>Saved key</dt><dd>{review.credentialAction}</dd></dl><p>The key value is excluded from this review.</p><button type="button" onclick={applyDemo}>Apply demo source</button><button type="button" class="secondary" onclick={dismiss}>Dismiss source review</button></div>
    {/if}
    {#if notice}<p role="status">{notice}</p>{/if}
  {/if}
</section>

<style>
.provider-panel{padding:24px;border:1px solid #d9e4df;border-radius:18px;background:#fff;margin:20px 0;color:#183535}.eyebrow{font-size:11px;letter-spacing:.12em;color:#57817a;font-weight:700}h2{margin:8px 0 12px}p{font-size:13px;line-height:1.6;color:#617573}fieldset{border:0;padding:0;max-width:640px;min-width:0}label{display:block;font-weight:650;font-size:13px;margin:16px 0 7px}input,textarea,select{width:100%;box-sizing:border-box;padding:11px;border:1px solid #bdcec5;border-radius:9px;font:inherit;font-size:14px}button{border:1px solid #276b60;background:#276b60;color:white;border-radius:9px;padding:11px 15px;cursor:pointer;margin:8px 8px 0 0;font:inherit;font-size:13px}.secondary{background:white;color:#276b60}.availability{padding:14px;background:#edf5f0;border-radius:10px}.error{color:#8d3324}.provider-review{padding:16px;border:1px solid #bdcec5;border-radius:10px}.provider-review dt{font-weight:700;font-size:13px}.provider-review dd{margin:4px 0 14px;overflow-wrap:anywhere;font-size:13px}.safe-status{padding:10px 0}input:focus-visible,textarea:focus-visible,select:focus-visible,button:focus-visible{outline:3px solid #a6c9b8;outline-offset:3px}
</style>
