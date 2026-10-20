import test from 'node:test';
import assert from 'node:assert/strict';
import {presets,presetRows,assessPresetEntry,suggestedPreset,presetSources,sourceUrl} from '../src/presets.mjs';
import {scenarios} from '../src/state.mjs';
test('existing TV preset separates earlier disabled apps from optional review',()=>{
 const rows=presetRows('tcl',scenarios.existing);
 assert.equal(rows.find(x=>x.packageId==='com.tcl.logkit').state,'done');
 assert.equal(rows.find(x=>x.packageId==='com.tcl.tv.plus').state,'review');
 assert.equal(rows.find(x=>x.packageId==='com.tcl.ttvs').state,'absent');
});
test('partial audit cannot produce a review plan',()=>{
 assert.ok(presetRows('tcl',scenarios.interrupted).every(x=>x.state==='unknown'));
 assert.ok(presetRows('tcl',{...scenarios.existing,disabled:null}).every(x=>x.state==='unknown'||x.state==='absent'));
});
test('current HOME and protected packages never become review candidates',()=>{
 for(const packageId of ['org.example.stock','com.tcl.tv','com.google.android.tungsten.setupwraith','com.google.android.marvin.talkback','com.google.android.apps.mediashell']){
  assert.equal(assessPresetEntry({packageId},{...scenarios.fresh,packages:[packageId]}).state,'kept');
 }
});
test('home-dependent candidates stay conditional without a verified replacement',()=>{
 const tv={...scenarios.fresh,packages:['com.mitv.tvhome.atv'],disabled:[]};
 assert.equal(presetRows('xiaomi',tv).find(x=>x.packageId==='com.mitv.tvhome.atv').state,'conditional');
});
test('unsupported OS cannot become a review candidate',()=>{
 assert.ok(presetRows('tcl',{...scenarios.existing,isTv:false}).every(x=>x.state!=='review'));
});
test('catalog has sourced package IDs and excludes protected feature choices',()=>{
 for(const preset of presets)for(const item of presetRows(preset.id,null)){
  assert.match(item.packageId,/^[a-zA-Z_][a-zA-Z0-9_]*(\.[a-zA-Z0-9_]+)+$/);
  assert.ok(presetSources[item.source]);assert.match(sourceUrl(item.source),/\/blob\/[a-f0-9]{40}\/examples\//);
 }
 assert.equal(suggestedPreset({manufacturer:'TCL'}),'tcl');
 assert.equal(suggestedPreset({manufacturer:'MOKA'}),'common');
});

test('a bundle containing T-Cast and Alexa cannot silently become a cleanup suggestion',()=>{
 const tv={...scenarios.existing,packages:['com.tcl.ttvs'],disabled:[]};
 const bundle=presetRows('tcl',tv).find(row=>row.packageId==='com.tcl.ttvs');
 assert.equal(bundle.state,'conditional');assert.match(bundle.reason,/T-Cast and Alexa/);
});
