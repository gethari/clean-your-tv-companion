<script lang="ts">
  import { THEMES, SOLAR_EVENTS, validateSchedule, upcomingTransitions, formatLocal } from './schedule.mjs';
  let { demo }: { demo: boolean } = $props();
  type Rule = {
    id: string; name: string; kind: 'fixed'|'solar'; time: string; event: 'sunrise'|'sunset';
    offsetMinutes: number; fallbackTime: string; theme: string; collectionId: string; priority: number;
  };
  type Schedule = {
    defaultTheme: string;
    location: { latitude: number; longitude: number; timezone: string };
    rules: Rule[];
  };
  let schedule = $state<Schedule>({
    defaultTheme: 'warm',
    location: { latitude: 51.5072, longitude: -0.1276, timezone: 'Europe/London' },
    rules: [
      { id: 'morning', name: 'Weekday morning', kind: 'fixed', time: '07:00', event: 'sunrise', offsetMinutes: 0, fallbackTime: '07:00', theme: 'warm', collectionId: '', priority: 10 },
      { id: 'evening', name: 'Evening', kind: 'solar', time: '18:00', event: 'sunset', offsetMinutes: 0, fallbackTime: '18:00', theme: 'calm', collectionId: '', priority: 20 }
    ]
  });
  let preview = $state<ReturnType<typeof upcomingTransitions>>([]);
  let previewedAt = $state('');
  let nextRule = 3;
  let validation = $derived(validateSchedule(schedule));
  let themeLabel = (id: string) => THEMES.find(item => item.id === id)?.label ?? id;

  function updateLocation(patch: Partial<Schedule['location']>) {
    schedule = { ...schedule, location: { ...schedule.location, ...patch } };
    preview = []; previewedAt = '';
  }
  function updateRule(index: number, patch: Partial<Rule>) {
    schedule = { ...schedule, rules: schedule.rules.map((rule, position) => position === index ? { ...rule, ...patch } : rule) };
    preview = []; previewedAt = '';
  }
  function addRule() {
    if (schedule.rules.length >= 24) return;
    const id = `rule-${nextRule++}`;
    schedule = { ...schedule, rules: [...schedule.rules, { id, name: 'New rule', kind: 'fixed', time: '12:00', event: 'sunrise', offsetMinutes: 0, fallbackTime: '12:00', theme: schedule.defaultTheme, collectionId: '', priority: 0 }] };
    preview = []; previewedAt = '';
  }
  function removeRule(index: number) {
    schedule = { ...schedule, rules: schedule.rules.filter((_, position) => position !== index) };
    preview = []; previewedAt = '';
  }
  function runPreview() {
    if (!validation.valid) return;
    preview = upcomingTransitions(schedule, Date.now(), 4);
    previewedAt = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }
</script>

<section class="schedule-panel" aria-labelledby="schedule-title">
  <p class="eyebrow">THEME SCHEDULE · PREVIEW</p>
  <h2 id="schedule-title">Let the TV change with the day</h2>
  <p>{demo ? 'Demo mode previews a schedule locally.' : 'This preview runs locally.'} It does not save to a TV until a compatible launcher advertises local schedule support and delivery is verified. Wallpaper rotation stays independent.</p>

  <div class="schedule-basics">
    <div><label for="schedule-default-theme">Default theme</label><select id="schedule-default-theme" value={schedule.defaultTheme} onchange={event => { schedule = { ...schedule, defaultTheme: event.currentTarget.value }; preview = []; }}>{#each THEMES as theme}<option value={theme.id}>{theme.label}</option>{/each}</select>{#if validation.errors.defaultTheme}<p class="field-error" role="alert">{validation.errors.defaultTheme}</p>{/if}</div>
    <div><label for="schedule-timezone">Timezone</label><input id="schedule-timezone" value={schedule.location.timezone} oninput={event => updateLocation({ timezone: event.currentTarget.value })} placeholder="Europe/London" spellcheck="false" aria-describedby="timezone-help" aria-invalid={!!validation.errors.timezone}/><p id="timezone-help">Use an IANA name, for example Europe/London or America/New_York.</p>{#if validation.errors.timezone}<p class="field-error" role="alert">{validation.errors.timezone}</p>{/if}</div>
    <div><label for="schedule-latitude">Latitude</label><input id="schedule-latitude" type="number" step="0.0001" value={schedule.location.latitude} oninput={event => updateLocation({ latitude: Number(event.currentTarget.value) })} aria-invalid={!!validation.errors.latitude}/>{#if validation.errors.latitude}<p class="field-error" role="alert">{validation.errors.latitude}</p>{/if}</div>
    <div><label for="schedule-longitude">Longitude</label><input id="schedule-longitude" type="number" step="0.0001" value={schedule.location.longitude} oninput={event => updateLocation({ longitude: Number(event.currentTarget.value) })} aria-invalid={!!validation.errors.longitude}/>{#if validation.errors.longitude}<p class="field-error" role="alert">{validation.errors.longitude}</p>{/if}</div>
  </div>
  <p class="location-note">Solar times are calculated from these coordinates and the chosen timezone. No device location permission is requested.</p>

  <div class="rule-heading"><h3>Daily rules</h3><span>{schedule.rules.length} of 24</span></div>
  {#if validation.errors.rules}<p class="field-error" role="alert">{validation.errors.rules}</p>{/if}
  {#each schedule.rules as rule, index}
    <article class="schedule-rule" aria-labelledby={`rule-${index}-title`}>
      <div class="rule-title"><h4 id={`rule-${index}-title`}>Rule {index + 1}</h4><button type="button" class="secondary" onclick={() => removeRule(index)} aria-label={`Remove rule ${index + 1}`}>Remove</button></div>
      <div class="rule-grid">
        <div><label for={`rule-${index}-name`}>Name</label><input id={`rule-${index}-name`} value={rule.name} oninput={event => updateRule(index, { name: event.currentTarget.value })} aria-invalid={!!validation.errors[`rule-${index}.name`]}/>{#if validation.errors[`rule-${index}.name`]}<p class="field-error" role="alert">{validation.errors[`rule-${index}.name`]}</p>{/if}</div>
        <div><label for={`rule-${index}-kind`}>When</label><select id={`rule-${index}-kind`} value={rule.kind} onchange={event => updateRule(index, { kind: event.currentTarget.value as Rule['kind'] })}><option value="fixed">Fixed time</option><option value="solar">Sunrise or sunset</option></select></div>
        {#if rule.kind === 'fixed'}
          <div><label for={`rule-${index}-time`}>Local time</label><input id={`rule-${index}-time`} type="time" value={rule.time} oninput={event => updateRule(index, { time: event.currentTarget.value })} aria-invalid={!!validation.errors[`rule-${index}.time`]}/>{#if validation.errors[`rule-${index}.time`]}<p class="field-error" role="alert">{validation.errors[`rule-${index}.time`]}</p>{/if}</div>
        {:else}
          <div><label for={`rule-${index}-event`}>Solar event</label><select id={`rule-${index}-event`} value={rule.event} onchange={event => updateRule(index, { event: event.currentTarget.value as Rule['event'] })}>{#each SOLAR_EVENTS as option}<option value={option}>{option === 'sunrise' ? 'Sunrise' : 'Sunset'}</option>{/each}</select></div>
          <div><label for={`rule-${index}-offset`}>Minutes after event</label><input id={`rule-${index}-offset`} type="number" min="-720" max="720" step="1" value={rule.offsetMinutes} oninput={event => updateRule(index, { offsetMinutes: Number(event.currentTarget.value) })} aria-invalid={!!validation.errors[`rule-${index}.offsetMinutes`]}/>{#if validation.errors[`rule-${index}.offsetMinutes`]}<p class="field-error" role="alert">{validation.errors[`rule-${index}.offsetMinutes`]}</p>{/if}</div>
          <div><label for={`rule-${index}-fallback`}>Fixed fallback</label><input id={`rule-${index}-fallback`} type="time" value={rule.fallbackTime} oninput={event => updateRule(index, { fallbackTime: event.currentTarget.value })} aria-invalid={!!validation.errors[`rule-${index}.fallbackTime`]}/><p>Used when the sun event does not occur, such as polar dates.</p>{#if validation.errors[`rule-${index}.fallbackTime`]}<p class="field-error" role="alert">{validation.errors[`rule-${index}.fallbackTime`]}</p>{/if}</div>
        {/if}
        <div><label for={`rule-${index}-theme`}>Theme</label><select id={`rule-${index}-theme`} value={rule.theme} onchange={event => updateRule(index, { theme: event.currentTarget.value })}>{#each THEMES as theme}<option value={theme.id}>{theme.label}</option>{/each}</select></div>
        <div><label for={`rule-${index}-priority`}>Priority</label><input id={`rule-${index}-priority`} type="number" min="0" max="100" step="1" value={rule.priority} oninput={event => updateRule(index, { priority: Number(event.currentTarget.value) })} aria-invalid={!!validation.errors[`rule-${index}.priority`]}/><p>At the same instant, the higher number wins once.</p>{#if validation.errors[`rule-${index}.priority`]}<p class="field-error" role="alert">{validation.errors[`rule-${index}.priority`]}</p>{/if}</div>
        <div><label for={`rule-${index}-collection`}>Collection ID (optional)</label><input id={`rule-${index}-collection`} value={rule.collectionId} oninput={event => updateRule(index, { collectionId: event.currentTarget.value.trim() })} placeholder="32 hexadecimal characters" spellcheck="false" aria-invalid={!!validation.errors[`rule-${index}.collectionId`]}/>{#if validation.errors[`rule-${index}.collectionId`]}<p class="field-error" role="alert">{validation.errors[`rule-${index}.collectionId`]}</p>{/if}</div>
      </div>
    </article>
  {/each}

  <div class="preview-actions"><button type="button" class="primary" onclick={addRule} disabled={schedule.rules.length >= 24}>Add daily rule</button><button type="button" class="primary" onclick={runPreview} disabled={!validation.valid || !schedule.rules.length}>Preview upcoming changes</button></div>
  {#if !validation.valid}<p class="field-error" role="alert">Fix the highlighted schedule fields before previewing.</p>{/if}
  {#if preview.length}
    <div class="schedule-preview" aria-live="polite"><h3>Next changes · checked at {previewedAt}</h3><ol>{#each preview as item}<li><div><strong>{item.label}</strong><span>{item.ruleName} · {themeLabel(item.theme)}{item.usedFallback ? ' · fixed fallback used' : ''}</span></div><span class="priority">Priority {item.priority}</span></li>{/each}</ol><p>All times use {schedule.location.timezone}. Missed transitions are not replayed.</p></div>
  {:else if previewedAt}
    <p role="status">No upcoming changes were found in the next eight days. The default theme stays active.</p>
  {/if}
  <p class="preview-note">Preview only · no TV write is available in this build. Schedule delivery requires a compatible launcher capability and separate physical acceptance.</p>
</section>

<style>
  .schedule-panel{margin:20px 0;padding:24px;border:1px solid #d9e4df;border-radius:18px;background:#fff;color:#183535}
  .eyebrow{font-size:11px;letter-spacing:.12em;color:#57817a;font-weight:700;margin:0 0 8px}
  .schedule-panel h2{margin:0 0 10px;font-size:22px}.schedule-panel h3{margin:20px 0 10px;font-size:17px}.schedule-panel h4{margin:0;font-size:15px}
  .schedule-panel p{font-size:13px;line-height:1.55;color:#617573}
  .schedule-basics,.rule-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(170px,1fr));gap:14px}
  .schedule-basics{margin-top:18px}
  label{display:block;font-weight:650;font-size:13px;margin:0 0 6px}
  input,select{width:100%;box-sizing:border-box;padding:10px;border:1px solid #bdcec5;border-radius:9px;font:inherit;font-size:14px;background:#fff;color:inherit}
  .location-note{margin:10px 0 0}
  .rule-heading{display:flex;align-items:center;justify-content:space-between;gap:12px;margin-top:22px;border-top:1px solid #e4ece8;padding-top:18px}
  .rule-heading h3{margin:0}.rule-heading span{font-size:12px;color:#617573}
  .schedule-rule{border:1px solid #e1eae6;border-radius:14px;padding:16px;margin:12px 0;background:#fbfdfb}
  .rule-title{display:flex;align-items:center;justify-content:space-between;gap:12px;margin-bottom:12px}
  .field-error{color:#8d3324;font-size:12px;margin:6px 0 0}
  .preview-actions{display:flex;flex-wrap:wrap;gap:10px;margin:16px 0}
  button{min-height:42px;border-radius:10px;padding:0 15px;font:inherit;font-size:13px;font-weight:700;cursor:pointer}
  .primary{background:#276b60;border:1px solid #276b60;color:#fff}.primary:disabled{opacity:.5;cursor:not-allowed}
  .secondary{background:#fff;border:1px solid #b9ccc5;color:#183535;min-height:34px;font-size:12px}
  .schedule-preview{border:1px solid #bdcec5;border-radius:12px;padding:14px 16px;margin-top:12px}
  .schedule-preview h3{margin:0 0 10px}.schedule-preview ol{list-style:none;margin:0;padding:0;display:grid;gap:8px}
  .schedule-preview li{display:flex;justify-content:space-between;gap:12px;padding:8px 0;border-bottom:1px solid #e4ece8}
  .schedule-preview li:last-child{border-bottom:0}.schedule-preview li div{display:grid;gap:2px}
  .schedule-preview li span{font-size:12px;color:#617573}.schedule-preview .priority{white-space:nowrap}
  .preview-note{font-style:italic}
  input:focus-visible,select:focus-visible,button:focus-visible{outline:3px solid #a6c9b8;outline-offset:3px}
  @media (max-width:640px){.schedule-panel{padding:18px}.schedule-basics,.rule-grid{grid-template-columns:1fr}.schedule-preview li{display:grid}}
  @media (prefers-reduced-motion:reduce){.schedule-panel *{scroll-behavior:auto}}
</style>
