import test from 'node:test';
import assert from 'node:assert/strict';
import {assess, parseEndpoint, pairingCandidates, scenarios} from '../src/state.mjs';
test('existing setup is recognized without claiming private provider health', () => {
 const cards = assess(scenarios.existing); assert.equal(cards[0].state, 'done'); assert.equal(cards[3].state, 'unknown');
});
test('query failures remain unknown rather than becoming available cleanup', () => {
 assert.ok(assess(scenarios.interrupted).every(x => x.state === 'unknown'));
});
test('zero disabled packages is valid observed state', () => {
 assert.match(assess(scenarios.fresh)[1].detail, /^0 apps/);
});
test('rejects public endpoints, shell input, invalid ports and octets', () => {
 for (const value of ['8.8.8.8:5555','192.168.1.2:5555;id','192.168.1.2:0','192.168.1.2:65536','192.168.999.1:5555','localhost:5555']) assert.equal(parseEndpoint(value),false);
 assert.equal(parseEndpoint('192.168.1.50:46481'),true);
});

test('post-pair discovery excludes other TVs, pairing ports and legacy plaintext services', () => {
 const services=[
  {endpoint:'192.168.1.50:5555',status:'device',kind:'Legacy connection service'},
  {endpoint:'192.168.1.50:5555',status:'device',kind:'Connected device'},
  {endpoint:'192.168.1.99:46481',status:'device',kind:'Connected device'},
  {endpoint:'192.168.1.50:39123',status:'advertised',kind:'Pairing service'},
  {endpoint:'192.168.1.50:46481',status:'advertised',kind:'Connection service'},
  {endpoint:'192.168.1.50:46481',status:'device',kind:'Connected device'}
 ];
 assert.deepEqual(pairingCandidates(services,'192.168.1.50'),['192.168.1.50:46481']);
 services.push({endpoint:'192.168.1.50:42377',status:'advertised',kind:'Connection service'});
 assert.equal(pairingCandidates(services,'192.168.1.50').length,2);
});
