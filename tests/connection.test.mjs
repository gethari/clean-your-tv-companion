import test from 'node:test';
import assert from 'node:assert/strict';
import { createConnectionController } from '../src/connection.mjs';

function deferred() {
  let resolve;
  const promise = new Promise(done => { resolve = done; });
  return { promise, resolve };
}

test('prevents overlapping operations and publishes meaningful phases', async () => {
  const states = [], pending = deferred();
  const controller = createConnectionController(state => states.push(state));
  const first = controller.run('searching', 'Looking for TVs…', async context => {
    await pending.promise;
    context.transition('candidates', 'Choose a TV');
  });
  assert.equal(await controller.run('searching', 'Duplicate', () => assert.fail()), false);
  pending.resolve();
  assert.equal(await first, true);
  assert.deepEqual(states.map(s => s.phase), ['searching', 'candidates', 'ready']);
  assert.equal(states.at(-1).pending, false);
});

test('cancelled search cannot commit stale results after retry', async () => {
  const controller = createConnectionController(() => {}), pending = deferred();
  let result = '';
  const first = controller.run('searching', 'First', async context => {
    await pending.promise;
    context.commit(() => { result = 'stale'; });
  });
  await Promise.resolve();
  controller.cancel();
  assert.equal(await first, false);
  await controller.run('searching', 'Retry', async context => context.commit(() => { result = 'current'; }));
  pending.resolve();
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(result, 'current');
});

test('timeout ends pending state and ignores late completion', async () => {
  const states = [], pending = deferred();
  const controller = createConnectionController(state => states.push(state), { timeoutMs: 5 });
  let committed = false;
  assert.equal(await controller.run('connecting', 'Connecting', async context => {
    await pending.promise;
    context.commit(() => { committed = true; });
  }), false);
  assert.equal(states.at(-1).phase, 'recoverable-error');
  assert.equal(states.at(-1).pending, false);
  assert.match(states.at(-1).error, /try again/);
  pending.resolve();
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(committed, false);
  assert.equal(await controller.run('searching', 'Retry', async () => {}), true);
});

test('native errors become recoverable and permit retry', async () => {
  const states = [], controller = createConnectionController(state => states.push(state));
  await controller.run('authorizing', 'Pairing', async () => { throw new Error('Authorization removed'); });
  assert.equal(states.at(-1).error, 'Authorization removed');
  assert.equal(await controller.run('authorizing', 'Retry', async () => {}), true);
});
