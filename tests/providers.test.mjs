import test from 'node:test';
import assert from 'node:assert/strict';
import {
  PHOTO_SOURCES,
  SOURCE_STATES,
  createSourceStatus,
  getPhotoSource,
  preserveLocalCollectionOnFailure,
  toPublicSourceStatus
} from '../src/providers.mjs';

test('only local photos are available without network or credentials', () => {
  assert.deepEqual(PHOTO_SOURCES.map(({ id, availability }) => [id, availability]), [
    ['local', 'available'],
    ['unsplash', 'unavailable'],
    ['reddit', 'unavailable']
  ]);
  assert.equal(getPhotoSource('local').requiresNetwork, false);
  assert.equal(getPhotoSource('local').requiresCredentials, false);
  assert.equal(getPhotoSource('unsplash').requiresCredentials, true);
  assert.equal(getPhotoSource('reddit').requiresCredentials, true);
  assert.equal(getPhotoSource('google-photos'), null);
  assert.ok(Object.isFrozen(PHOTO_SOURCES));
  assert.ok(Object.isFrozen(PHOTO_SOURCES[0]));
});

test('provider approval gate is distinct from empty, auth, rate-limit, and offline states', () => {
  const unavailable = createSourceStatus('reddit', 'unavailable');
  assert.equal(unavailable.state, 'unavailable');
  assert.match(unavailable.message, /access and terms are approved/);

  for (const state of ['empty', 'authentication-required', 'rate-limited', 'offline']) {
    assert.ok(SOURCE_STATES.includes(state));
    assert.notEqual(createSourceStatus('reddit', state).state, unavailable.state);
  }
});

test('failed optional providers preserve cached local images by identity', () => {
  const collection = {
    id: 'home',
    localImages: [{ id: 'photo-1', path: 'imports/one.jpg' }],
    onlineImages: [{ id: 'cached-1' }]
  };

  for (const state of ['unavailable', 'empty', 'authentication-required', 'rate-limited', 'offline']) {
    const result = preserveLocalCollectionOnFailure(collection, createSourceStatus('unsplash', state));
    assert.strictEqual(result, collection);
    assert.strictEqual(result.localImages, collection.localImages);
  }
});

test('public status exposes only approved display fields and no secret or raw error values', () => {
  const status = toPublicSourceStatus({
    sourceId: 'unsplash',
    state: 'rate-limited',
    retryAfterMs: 30000,
    accessToken: 'do-not-expose-token',
    clientSecret: 'do-not-expose-secret',
    rawError: 'authorization: Bearer do-not-expose-token',
    responseBody: { secret: 'do-not-expose-secret' }
  });

  assert.deepEqual(Object.keys(status).sort(), ['message', 'retryAfterMs', 'sourceId', 'state']);
  assert.equal(status.retryAfterMs, 30000);
  assert.doesNotMatch(JSON.stringify(status), /do-not-expose|Bearer|clientSecret|rawError/);
  assert.ok(Object.isFrozen(status));
});

test('unknown sources and invalid retry hints are rejected', () => {
  assert.throws(() => createSourceStatus('google-photos', 'unavailable'), /Unknown photo source/);
  assert.throws(() => createSourceStatus('reddit', 'unknown'), /Unknown source state/);
  assert.throws(() => createSourceStatus('reddit', 'rate-limited', { retryAfterMs: -1 }), /retryAfterMs/);
});
