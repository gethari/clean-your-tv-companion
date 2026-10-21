import test from 'node:test';
import assert from 'node:assert/strict';
import { normalizeCollectionIds, providerCapabilities, safeProviderStatus, reviewProviderDraft } from '../src/provider-configuration.mjs';
const capabilities = { providers: ['local', 'unsplash', 'auto'], configuration: true, privateCredentials: true };
const observed = { provider: 'unsplash', collectionIds: ['123', 'abc_9'], credentialPresent: true, health: 'unknown', configurationSaved: true };

test('collection IDs normalize commas/newlines and duplicates without accepting URLs', () => {
  assert.deepEqual(normalizeCollectionIds('123, abc_9\n123').ids, ['123', 'abc_9']);
  assert.ok(normalizeCollectionIds('https://unsplash.com/collections/123/photos').error);
  assert.ok(normalizeCollectionIds('123;cat').error);
  assert.ok(normalizeCollectionIds(Array(51).fill('a').join(',')).error);
});

test('version names and provider presence cannot grant editing or secret transport', () => {
  for (const input of [{version:'99',providers:['unsplash']},{...capabilities,privateCredentials:false},{...capabilities,configuration:false}]) {
    assert.equal(providerCapabilities(input).editable, false);
  }
  assert.deepEqual(providerCapabilities({...capabilities,providers:['unsplash','invented','local']}).providers,['local','unsplash']);
  assert.equal(providerCapabilities(capabilities).editable,true);
});

test('existing key is kept for collection edits, and exact current configuration is a no-op', () => {
  const same = reviewProviderDraft({provider:'unsplash',collectionIds:'123,abc_9'}, observed, capabilities);
  assert.equal(same.valid,true); assert.equal(same.review.changed,false); assert.equal(same.review.credentialAction,'keep');
  const edit=reviewProviderDraft({provider:'unsplash',collectionIds:'456'},observed,capabilities);
  assert.equal(edit.valid,true); assert.equal(edit.review.changed,true);
  assert.equal(reviewProviderDraft({provider:'unsplash',collectionIds:'456'},{...observed,credentialPresent:null},capabilities).valid,false);
});

test('replacement/removal are explicit and sensitive bytes never enter a review or error', () => {
  const key='fixture-AccessKey123';
  const result=reviewProviderDraft({provider:'unsplash',collectionIds:'456',credentialAction:'replace',accessKey:key},observed,capabilities);
  assert.equal(result.valid,true); assert.doesNotMatch(JSON.stringify(result),new RegExp(key));
  assert.equal(reviewProviderDraft({provider:'unsplash',collectionIds:'123',credentialAction:'remove'},observed,capabilities).valid,false);
  assert.equal(reviewProviderDraft({provider:'local',collectionIds:'123,abc_9',credentialAction:'keep'},observed,capabilities).valid,true);
  assert.equal(reviewProviderDraft({provider:'local',collectionIds:'',credentialAction:'remove'},observed,capabilities).valid,true);
  const bad=reviewProviderDraft({provider:'unsplash',collectionIds:'123',credentialAction:'replace',accessKey:'fixture/secret'},observed,capabilities);
  assert.equal(bad.valid,false); assert.doesNotMatch(JSON.stringify(bad),/fixture\/secret/);
});

test('safe status separates saved configuration from retrieval and excludes private details', () => {
  for (const health of ['offline','rate-limited','empty','invalid-credentials','collection-inaccessible']) {
    const status=safeProviderStatus({...observed,health,access_key:'fixture-secret',message:'fixture-secret',rollbackToken:'fixture-secret',retryAfterSeconds:60});
    assert.equal(status.configurationSaved,true); assert.equal(status.health,health);
    assert.doesNotMatch(JSON.stringify(status),/fixture-secret|rollbackToken|access_key/);
    assert.equal(status.retryAfterSeconds,60);
  }
  assert.equal(safeProviderStatus({}).credentialPresent,null);
  assert.equal(safeProviderStatus({health:'mystery'}).health,'unknown');
});

test('unavailable configuration never becomes a successful plan', () => {
  const result=reviewProviderDraft({provider:'unsplash',collectionIds:'123'},observed,{});
  assert.equal(result.valid,false); assert.ok(result.errors.support);
});

test('invalid collection syntax keeps the specific validation reason in the review', () => {
  const result=reviewProviderDraft({provider:'unsplash',collectionIds:'https://unsplash.com/collections/123'},observed,capabilities);
  assert.equal(result.valid,false);assert.match(result.errors.collectionIds,/without URLs/);
});
