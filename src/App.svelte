<script lang="ts">
  import { version as appVersion } from '../package.json';
  import { presets, presetRows, presetEvidence, keptFeatures, suggestedPreset } from './presets.mjs';
  import { invoke } from '@tauri-apps/api/core';
  import { assess, parseEndpoint, pairingCandidates, scenarios } from './state.mjs';
  const native = '__TAURI_INTERNALS__' in window;
  type Snapshot = {launcherVersion?: string|null; model: string|null; manufacturer: string|null; androidVersion: string|null; home: string|null; packages: string[]|null; disabled: string[]|null; theme: Record<string, any>|null; isTv: boolean|null; errors: string[]; collectedAt: string};
  type Device = {endpoint: string; status: string; kind: string};
  let demo = $state(!native);
  let scenario = $state('existing');
  let snapshot = $state<Snapshot|null>(!native ? scenarios.existing : null);
  let devices = $state<Device[]>([]);
  let endpoint = $state('');
  let pairEndpoint = $state('');
  let code = $state('');
  let pairing = $state(false);
  let paired = $state(false);
  let help = $state(false);
  let busy = $state('');
  let message = $state('');
  let error = $state('');
  let selected = $state('');
  let tab = $state('overview');
  let presetId = $state(native ? 'common' : 'tcl');
  let rows = $derived(presetRows(presetId, snapshot));
  let preset = $derived(presets.find(p => p.id === presetId)!);
  let cards = $derived(assess(snapshot));
  let active = $derived(snapshot?.home?.startsWith('com.gathin.tvhome/') ?? false);
  function chooseDemo(value: string) {
    demo = true; scenario = value; snapshot = scenarios[value as keyof typeof scenarios];
    error = ''; message = ''; tab = 'overview'; selected = ''; paired = false; presetId = suggestedPreset(snapshot);
  }
  async function task(label: string, work: () => Promise<void>) {
    if (busy) return;
    busy = label; error = ''; message = '';
    try { await work(); } catch (e) { error = String(e); } finally { busy = ''; }
  }
  function live() { paired = false; demo = false; snapshot = null; selected = ''; error = ''; message = ''; devices = []; }
  async function discover() {
    if (demo) { message = 'This is a demo. Choose “Use a real TV” in the desktop app to discover devices.'; return; }
    await task('Looking for TVs…', async () => {
      devices = await invoke<Device[]>('discover');
      message = devices.length ? 'Choose the TV you want to check.' : 'No advertised devices found. Keep the TV awake, enable debugging, or enter its address below.';
    });
  }
  async function inspect(serial: string) {
    const result = await invoke<Snapshot>('inspect_device', { serial }); result.collectedAt = 'Checked ' + new Date().toLocaleTimeString([], {hour: '2-digit', minute: '2-digit'}); snapshot = result; if (selected !== serial) presetId = suggestedPreset(result); selected = serial; tab = 'overview';
  }
  async function connect() {
    if (!parseEndpoint(endpoint)) { error = 'Enter a local IPv4 address and port, for example 192.168.1.50:45737.'; return; }
    await task('Connecting and checking your TV…', async () => {
      await invoke('connect_device', { endpoint: endpoint.trim() }); await inspect(endpoint.trim());
      message = 'Inspection complete. Your TV configuration has not been changed.';
    });
  }
  async function pair() {
    if (!parseEndpoint(pairEndpoint) || !/^\d{6}$/.test(code)) { error = 'Enter the pairing address and six-digit code currently shown on the TV.'; return; }
    await task('Pairing this computer with your TV…', async () => {
      const host = pairEndpoint.trim().split(':')[0];
      await invoke('pair_device', { endpoint: pairEndpoint.trim(), code });
      code = ''; paired = true; pairing = false;
      endpoint = host + ':';
      busy = 'Paired. Looking for this TV’s connection address…';
      try {
        devices = await invoke<Device[]>('discover');
      } catch {
        devices = [];
      }
      const matches = pairingCandidates(devices, host);
      if (matches.length === 1) {
        endpoint = matches[0];
        busy = 'Paired. Connecting and checking your TV…';
        await invoke('connect_device', { endpoint });
        await inspect(endpoint);
        message = 'Your TV is paired and connected. Inspection complete.';
      } else {
        message = 'Pairing worked. Close the pairing-code popup on the TV and enter the connection address from the main Wireless debugging screen.';
        setTimeout(() => document.getElementById('address')?.focus(), 0);
      }
    });
  }

</script>

<svelte:head><title>Clean Your TV · Companion</title></svelte:head>
<div class="shell">
  <aside>
    <a class="brand" href="#main" aria-label="Clean Your TV home"><span class="brand-icon">▣</span><span>clean<span class="brand-light">your</span>tv<span class="brand-dot">.</span></span></a>
    <p class="edition">YOUR TV. YOUR SPACE.</p>
    <nav aria-label="Main navigation">
      <button class:current={tab === 'overview'} onclick={() => tab = 'overview'}><span>◫</span> Overview</button>
      <button class:current={tab === 'connect'} onclick={() => tab = 'connect'}><span>⌁</span> Connect a TV</button>
      <button class:current={tab === 'presets'} onclick={() => tab = 'presets'}><span>▤</span> TV presets</button>
      <button class:current={tab === 'scope'} onclick={() => tab = 'scope'}><span>✦</span> What’s coming</button>
    </nav>
    <div class="rail-note"><span class="privacy-dot"></span> Everything stays local.<p>No account. No subscription.<br/>Just you and your TV.</p></div>
    <div class="version">COMPANION <span>{appVersion} · inspection preview</span></div>
  </aside>
  <main id="main">
    <header><div class="breadcrumb">Your space <span>/</span> {tab === 'connect' ? 'Connect a TV' : tab === 'scope' ? 'Project scope' : tab === 'presets' ? 'TV presets' : 'Overview'}</div><span class="mode">{demo ? '◌ Demo · no TV connected' : selected ? '● Live inspection' : '◌ Ready to connect'}</span></header>
    <div class="content">
      <div class="feedback top-feedback" aria-live="polite">
        {#if busy}<p class="notice"><span class="spinner"></span>{busy}</p>{/if}
        {#if message}<p class="notice">{message}</p>{/if}
      </div>
      {#if error}<p class="notice warning" role="alert">{error}</p>{/if}
      {#if demo}
        <div class="demo-bar"><div><strong>Take a look around.</strong> This is example data, not your TV.</div><div class="demo-actions"><label class="sr-only" for="scenario">Demo scenario</label><select id="scenario" disabled={!!busy} value={scenario} onchange={(e) => chooseDemo(e.currentTarget.value)}><option value="existing">Already configured</option><option value="fresh">Fresh TV</option><option value="interrupted">Connection interrupted</option></select>{#if native}<button class="text-button" disabled={!!busy} onclick={live}>Use a real TV →</button>{/if}</div></div>
      {/if}
      {#if tab === 'overview'}
        <section class="hero"><div><p class="eyebrow">A LITTLE LESS NOISE. A LOT MORE YOU.</p><h1>{active ? 'Already feels\nlike home.' : snapshot?.errors.length ? 'Let’s reconnect\nthe dots.' : snapshot ? 'A fresh start\nfor your TV.' : 'Make room for\nwhat you love.'}</h1><p class="intro">{active ? 'Your clean home screen is already in place. Here’s what we can see, and what you can explore next.' : snapshot?.errors.length ? 'We couldn’t finish checking this TV. Your existing setup stays in place while we find it again.' : snapshot ? 'We’ve checked what’s on your TV. You decide what stays and what changes.' : 'Find your TV, see what’s already set up, and take the next step at your own pace.'}</p><div class="hero-actions"><button class="primary" disabled={!!busy} onclick={() => {tab = 'connect';}}> {selected ? 'Connection details' : 'Find my TV'} <span>↗</span></button>{#if !demo}<button class="secondary" disabled={!!busy} onclick={() => chooseDemo('existing')}>Try the demo</button>{/if}</div></div><div class="illustration" aria-hidden="true"><div class="orbit one"></div><div class="orbit two"></div><div class="spark s1">✧</div><div class="spark s2">✦</div><div class:scanning={!!busy} class="tv"><div class="tv-screen"><div class="hill back"></div><div class="hill front"></div><div class="sun"></div><div class="eyes"><i></i><i></i></div><div class="smile"></div></div><div class="tv-led"></div></div><div class="tv-feet"></div><span class="sticker">{snapshot?.errors.length ? 'One step at a time' : active ? 'Looking good!' : 'Hello, possibility.'}</span></div></section>
        {#if snapshot}
          <section class="device-strip"><span class="device-icon">▣</span><div><strong>{snapshot.model || 'Your TV'}</strong><p>{snapshot.manufacturer || 'Manufacturer unknown'} · Android {snapshot.androidVersion || 'version unknown'}{snapshot.launcherVersion ? ` · Launcher ${snapshot.launcherVersion}` : ''}</p></div><span class="device-state">{snapshot.isTv === false ? 'Not an Android TV' : snapshot.isTv == null ? 'TV compatibility unknown' : 'Android TV detected'}</span>{#if selected && !demo}<button class="secondary compact" disabled={!!busy} onclick={() => task('Refreshing TV state…', () => inspect(selected))}>Refresh</button>{/if}</section>
          <button class="text-button" onclick={() => tab = 'presets'}>Review a TV preset →</button>
          <div class="section-heading"><div><p class="eyebrow">THE CURRENT PICTURE</p><h2>What’s already in place</h2></div><span>{snapshot.collectedAt}</span></div>
          <section class="cards" aria-label="Observed TV state">{#each cards as card}<article class="state-card"><span class="status-icon" class:done={card.state === 'done'}>{card.state === 'done' ? '✓' : card.state === 'unknown' ? '?' : '✧'}</span><div><div class="card-heading"><h3>{card.name}</h3><span class="status" class:done={card.state === 'done'}>{card.state === 'done' ? 'In place' : card.state === 'unknown' ? 'Couldn’t check' : 'Optional'}</span></div><p>{card.detail}</p></div></article>{/each}</section>
          {#if active}<div class="all-good"><span>✺</span><div><strong>No need to start over.</strong><p>Your existing home screen stays as it is. This preview only checks your TV.</p></div></div>{/if}
          {#each snapshot.errors as problem}<p class="notice warning">{problem}</p>{/each}
          {#if snapshot.disabled}<details class="package-details"><summary>See the {snapshot.disabled.length} disabled packages</summary><p>Observed now. We don’t know who disabled them or their original state.</p>{#each snapshot.disabled as name}<code>{name}</code>{/each}{#if !snapshot.disabled.length}<p>No disabled packages reported.</p>{/if}</details>{/if}
        {:else}<div class="empty-note"><span>⌁</span><div><strong>A quick check comes first.</strong><p>We’ll look at the home screen, installed apps and wallpaper preferences before suggesting any changes.</p></div></div>{/if}
      {:else if tab === 'connect'}
        <p class="eyebrow">LET’S FIND EACH OTHER</p><h1 class="page-title">Meet your TV.</h1><p class="intro">Keep your laptop and TV on the same home network. You’ll need the TV remote for a couple of steps.</p>
        {#if paired}<section class="paired-banner"><span>✓</span><div><strong>Computer paired with your TV</strong><p>{busy ? 'Finding the TV’s connection so we can continue.' : 'Next: connect using the address on the main Wireless debugging screen. The pairing port is different.'}</p></div></section>{/if}
        <section class="connection-grid"><article class="panel"><div class="section-heading"><h2>Nearby devices</h2><span>Local discovery</span></div><p>Look for connected devices and advertised debugging services. You choose which one to inspect.</p><button class="primary" disabled={!!busy || demo} onclick={discover}>{busy || 'Look for my TV'} <span>⌁</span></button>{#each devices as device}<div class="found-device"><div><strong>{device.endpoint}</strong><p>{device.kind} · {device.status}</p></div><button class="secondary compact" disabled={!!busy} onclick={() => { if (device.kind === 'Pairing service') {pairing = true; pairEndpoint = device.endpoint;} else {endpoint = device.endpoint; task('Checking TV…', async () => {if (device.endpoint.includes(':')) await invoke('connect_device', {endpoint: device.endpoint}); await inspect(device.endpoint);});} }}>{device.kind === 'Pairing service' ? 'Pair' : 'Check TV'}</button></div>{/each}</article>
        <article class="panel"><h2>Have the TV’s address?</h2><p>Use the IP address and <strong>connection port</strong> on the main Wireless debugging screen.</p><form onsubmit={(e) => {e.preventDefault(); connect();}}><label for="address">Connection address</label><input id="address" bind:value={endpoint} placeholder="192.168.1.50:45737" autocomplete="off" disabled={!!busy || demo}/><button class="primary" disabled={!!busy || demo}>Connect and check →</button></form><button class="text-button" onclick={() => pairing = !pairing}>First time? Pair with a code</button></article></section>
        {#if pairing}<section class="panel pairing"><h2>Pair your computer</h2><p>On the TV: Wireless debugging → Pair device with pairing code. This address is usually different from the connection address.</p><form onsubmit={(e) => {e.preventDefault(); pair();}}><div><label for="pair-address">Pairing address</label><input id="pair-address" bind:value={pairEndpoint} placeholder="192.168.1.50:39123" autocomplete="off" disabled={!!busy || demo}/></div><div><label for="code">Six-digit pairing code</label><input id="code" bind:value={code} inputmode="numeric" maxlength="6" type="password" autocomplete="off" disabled={!!busy || demo}/></div><button class="primary" disabled={!!busy || demo}>{busy ? 'Pairing…' : 'Pair computer'}</button></form></section>{/if}
        <section class="help"><button class="text-button" onclick={() => help = !help} aria-expanded={help}> {help ? '−' : '+'} Where do I find Wireless debugging?</button>{#if help}<ol><li>On the TV, open Settings → About. Select Build seven times to enable developer options. Menu names vary by brand.</li><li>Open Developer options → Wireless debugging and switch it on. Keep this screen open.</li><li>Pair this computer if needed, then use the connection address from the main debugging screen. Accept authorization on the TV.</li><li>If the TV sleeps or reconnects to Wi-Fi, the port may change. Enter the new one. Guest networks or VPNs may prevent discovery.</li></ol><p>Some older TVs only offer legacy network debugging. That guided path is planned. This preview accepts private IPv4 addresses only. Disable debugging on the TV after finishing if you don’t need it.</p>{/if}</section>
        {#if demo}<p class="notice">Connection controls work in the desktop app. This demo doesn’t discover or change devices.</p>{/if}
      {:else if tab === 'presets'}
        <p class="eyebrow">START WITH WHAT’S ON YOUR TV</p><h1 class="page-title">A preset, with context.</h1>
        <p class="intro">Community TV reports help us find packages worth a look. Your TV’s actual apps decide what appears here.</p>
        <section class="preset-picker panel"><div><label for="tv-preset">TV family</label><select id="tv-preset" bind:value={presetId}>{#each presets as family}<option value={family.id}>{family.name}</option>{/each}</select></div><div><strong>{preset.name}</strong><p>{preset.note}</p></div></section>
        <div class="preset-summary"><span><strong>{rows.filter(row => row.state === 'done').length}</strong> already disabled</span><span><strong>{rows.filter(row => row.state === 'review').length}</strong> to review</span><span><strong>{rows.filter(row => row.state === 'absent').length}</strong> not installed</span></div>
        <p class="preset-note">Nothing changes when you choose a preset. This preview compares package names; it does not certify your model or disable apps.</p>
        <section class="preset-list" aria-label="Preset package comparison">{#each rows as row}<article class="preset-row" class:absent={row.state === 'absent'}><div class="preset-row-heading"><div><span class="group-label">{row.group}</span><h2>{row.title}</h2></div><span class="status" class:done={row.state === 'done'}>{row.label}</span></div><code>{row.packageId}</code><p>{row.impact}</p><p class="row-reason">{row.reason}</p><details><summary>Community report</summary><p>{presetEvidence(row.source).model}<br/>{presetEvidence(row.source).firmware}<br/>examples/{presetEvidence(row.source).path}/debloat.txt</p></details></article>{/each}</section>
        <section class="panel keep-panel"><h2>Features we leave alone</h2><p>A cleanup list shouldn’t quietly turn off things you use.</p><div class="kept-features">{#each keptFeatures as feature}<span>✓ {feature}</span>{/each}</div></section>
      {:else}
        <p class="eyebrow">A THOUGHTFUL LITTLE COMPANION</p><h1 class="page-title">The next chapters.</h1><p class="intro">This is the inspection preview. These features are scoped for later milestones and won’t make changes today.</p>
        <section class="roadmap">{#each [{title:'A setup that knows your TV',text:'Capability checks, safe package profiles and clear choices. Existing customizations stay yours.'},{title:'Make it yours',text:'Dock order, themes, local wallpapers and optional photo collections, with a preview before applying.'},{title:'A real way back',text:'A saved baseline and a record of every change. Restore only what the companion changed.'},{title:'Made for real life',text:'Resume after a lost connection, recover from a failed step, and verify the actual remote and inputs.'}] as item}<article class="panel"><span class="planned">Planned</span><h2>{item.title}</h2><p>{item.text}</p></article>{/each}</section>
      {/if}

      <footer><span>Made for your living room, not the cloud.</span><span>Inspection preview · TV configuration stays unchanged</span></footer>
    </div>
  </main>
</div>
