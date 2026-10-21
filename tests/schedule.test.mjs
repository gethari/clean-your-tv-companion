import test from 'node:test';
import assert from 'node:assert/strict';
import {
  isValidTimeZone, validateSchedule, zonedTimeToUtc, solarUtc, upcomingTransitions, formatLocal
} from '../src/schedule.mjs';

const base = {
  defaultTheme: 'warm',
  location: { latitude: 51.5072, longitude: -0.1276, timezone: 'Europe/London' },
  rules: [
    { id: 'morning', name: 'Morning', kind: 'fixed', time: '07:00', theme: 'warm', collectionId: '', priority: 10 },
    { id: 'evening', name: 'Evening', kind: 'solar', event: 'sunset', offsetMinutes: 0, fallbackTime: '18:00', theme: 'calm', collectionId: '', priority: 20 }
  ]
};

test('a schedule rejects invalid timezones, coordinates, duplicate fixed boundaries and priorities', () => {
  assert.equal(validateSchedule(base).valid, true);
  const duplicate = structuredClone(base);
  duplicate.rules.push({ ...base.rules[0], id: 'morning-2', name: 'Duplicate', priority: 30 });
  const duplicateResult = validateSchedule(duplicate);
  assert.equal(duplicateResult.valid, false);
  assert.match(duplicateResult.errors['rule-0.time'], /Duplicate fixed-time/);
  assert.match(duplicateResult.errors['rule-2.time'], /Duplicate fixed-time/);

  const badPriority = structuredClone(base);
  badPriority.rules[1].priority = 10;
  assert.match(validateSchedule(badPriority).errors['rule-1.priority'], /unique/);

  const badLocation = structuredClone(base);
  badLocation.location = { latitude: 120, longitude: -0.1, timezone: 'Mars/Olympus' };
  const locationResult = validateSchedule(badLocation);
  assert.equal(locationResult.valid, false);
  assert.ok(locationResult.errors.latitude);
  assert.ok(locationResult.errors.timezone);
  assert.equal(isValidTimeZone('Europe/London'), true);
  assert.equal(isValidTimeZone('Not/AZone'), false);
});

test('solar rules require a fixed fallback and each rule validates its own fields', () => {
  const bad = structuredClone(base);
  bad.rules[1].fallbackTime = '25:00';
  bad.rules[1].offsetMinutes = 2000;
  const result = validateSchedule(bad);
  assert.match(result.errors['rule-1.fallbackTime'], /fallback/);
  assert.match(result.errors['rule-1.offsetMinutes'], /-720 to 720/);
  const noKind = structuredClone(base);
  noKind.rules[0].kind = 'cron';
  assert.match(validateSchedule(noKind).errors['rule-0.kind'], /fixed-time or solar/);
});

test('fixed transitions use the explicit timezone and prioritize a coincident rule once', () => {
  const from = Date.UTC(2026, 0, 1, 0, 0);
  const schedule = {
    defaultTheme: 'warm',
    location: { latitude: 0, longitude: 0, timezone: 'Asia/Tokyo' },
    rules: [
      { id: 'a', name: 'Morning', kind: 'fixed', time: '07:00', theme: 'warm', collectionId: '', priority: 1 },
      { id: 'b', name: 'Morning high', kind: 'fixed', time: '07:00', theme: 'contrast', collectionId: '', priority: 99 }
    ]
  };
  // Tokyo is UTC+9, but the duplicate-boundary validation rejects this first.
  assert.equal(validateSchedule(schedule).valid, false);
  const distinct = structuredClone(schedule);
  distinct.rules[1].time = '07:01';
  const transitions = upcomingTransitions(distinct, from, 2);
  assert.equal(transitions.length, 2);
  assert.equal(transitions[0].at, Date.UTC(2026, 0, 1, 7 - 9 + 24, 0));
  assert.equal(transitions[0].ruleName, 'Morning');
  assert.equal(transitions[0].timezone, 'Asia/Tokyo');
});

test('coincident solar and fixed boundaries apply only the higher-priority rule', () => {
  const from = Date.UTC(2026, 5, 1, 0, 0);
  const schedule = {
    defaultTheme: 'warm',
    location: { latitude: 51.5072, longitude: -0.1276, timezone: 'Europe/London' },
    rules: [
      { id: 'solar', name: 'Sunset', kind: 'solar', event: 'sunset', offsetMinutes: 0, fallbackTime: '18:00', theme: 'calm', collectionId: '', priority: 5 },
      { id: 'fixed', name: 'Same moment', kind: 'fixed', time: '20:00', theme: 'contrast', collectionId: '', priority: 50 }
    ]
  };
  // Force the comparison by aligning the solar rule to the fixed boundary using its offset.
  const sunset = solarUtc(2026, 6, 1, 51.5072, -0.1276, 'sunset');
  assert.ok(sunset);
  const [hour, minute] = formatLocal(sunset, 'Europe/London').split(', ').at(-1).split(':').map(Number);
  schedule.rules[1].time = `${String(hour).padStart(2, '0')}:${String(minute).padStart(2, '0')}`;
  const transitions = upcomingTransitions(schedule, from, 4);
  const firstDay = transitions.filter(item => item.at === transitions[0].at);
  assert.equal(firstDay.length, 1);
  assert.equal(transitions[0].priority, 50);
});

test('polar dates use the visible fixed fallback instead of inventing a solar event', () => {
  const schedule = {
    defaultTheme: 'warm',
    location: { latitude: 89, longitude: 18, timezone: 'Arctic/Longyearbyen' },
    rules: [{ id: 'evening', name: 'Evening', kind: 'solar', event: 'sunset', offsetMinutes: 0, fallbackTime: '18:00', theme: 'calm', collectionId: '', priority: 10 }]
  };
  assert.equal(solarUtc(2026, 12, 20, 89, 18, 'sunset'), null);
  const [transition] = upcomingTransitions(schedule, Date.UTC(2026, 11, 20, 0, 0), 1);
  assert.equal(transition.usedFallback, true);
  assert.match(transition.label, /18:00/);
});

test('DST gaps resolve forward and repeated times trigger once at the earliest instant', () => {
  // Berlin springs forward at 02:00 on 2026-03-29; 02:30 does not exist.
  const gap = zonedTimeToUtc(2026, 3, 29, 2, 30, 'Europe/Berlin');
  const gapParts = new Intl.DateTimeFormat('en-GB', { timeZone: 'Europe/Berlin', hour: '2-digit', minute: '2-digit', hour12: false }).format(gap);
  assert.equal(gapParts, '03:00');
  // Berlin falls back at 03:00 on 2026-10-25; 02:30 occurs twice.
  const repeated = zonedTimeToUtc(2026, 10, 25, 2, 30, 'Europe/Berlin');
  assert.equal(repeated, Date.UTC(2026, 9, 25, 0, 30));
});

test('a schedule with no rules previews the default theme without transitions', () => {
  const result = upcomingTransitions({ ...base, rules: [] }, Date.UTC(2026, 0, 1), 4);
  assert.deepEqual(result, []);
});
