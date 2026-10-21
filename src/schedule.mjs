/** Deterministic theme-schedule validation and local preview.
 * This module never writes to a TV and never calls a launcher. It exists so the
 * desktop editor can reject ambiguous rules and show what the TV would do.
 */

export const THEMES = Object.freeze([
  Object.freeze({ id: 'warm', label: 'Warm' }),
  Object.freeze({ id: 'calm', label: 'Calm' }),
  Object.freeze({ id: 'contrast', label: 'High contrast' })
]);
export const SOLAR_EVENTS = Object.freeze(['sunrise', 'sunset']);
const THEME_IDS = new Set(THEMES.map(item => item.id));
const MAX_RULES = 24;
const MINUTE = 60_000;
const DAY = 86_400_000;

export function isValidTimeZone(value) {
  if (typeof value !== 'string' || !value.trim() || value.length > 64) return false;
  try {
    new Intl.DateTimeFormat('en-US', { timeZone: value }).format(0);
    return true;
  } catch {
    return false;
  }
}

export function isValidTime(value) {
  return typeof value === 'string' && /^([01]\d|2[0-3]):[0-5]\d$/.test(value);
}

function isFiniteIn(value, min, max) {
  return typeof value === 'number' && Number.isFinite(value) && value >= min && value <= max;
}

export function validateSchedule(input) {
  /** @type {Record<string,string>} */
  const errors = {};
  const rules = Array.isArray(input?.rules) ? input.rules : [];
  const theme = input?.defaultTheme;
  if (!THEME_IDS.has(theme)) errors.defaultTheme = 'Choose an available default theme.';
  const location = input?.location;
  if (!location || typeof location !== 'object') {
    errors.location = 'Set an explicit location for solar rules.';
  } else {
    if (!isFiniteIn(location.latitude, -90, 90)) errors.latitude = 'Latitude must be between -90 and 90.';
    if (!isFiniteIn(location.longitude, -180, 180)) errors.longitude = 'Longitude must be between -180 and 180.';
    if (!isValidTimeZone(location.timezone)) errors.timezone = 'Choose a valid IANA timezone, such as Europe/London.';
  }
  if (rules.length > MAX_RULES) errors.rules = `Use at most ${MAX_RULES} daily rules.`;
  const ids = new Set();
  const priorities = new Set();
  const fixedTimes = new Map();
  for (const [index, rule] of rules.entries()) {
    const key = `rule-${index}`;
    const id = typeof rule?.id === 'string' ? rule.id : '';
    if (!id || id.length > 40 || ids.has(id)) errors[`${key}.id`] = 'Each rule needs a unique identifier.';
    ids.add(id);
    if (typeof rule?.name !== 'string' || !rule.name.trim() || rule.name.length > 60) {
      errors[`${key}.name`] = 'Give this rule a short name.';
    }
    if (!THEME_IDS.has(rule?.theme)) errors[`${key}.theme`] = 'Choose an available theme.';
    if (!Number.isInteger(rule?.priority) || rule.priority < 0 || rule.priority > 100) {
      errors[`${key}.priority`] = 'Priority must be a whole number from 0 to 100.';
    } else if (priorities.has(rule.priority)) {
      errors[`${key}.priority`] = 'Priorities must be unique so coincident rules are deterministic.';
    } else {
      priorities.add(rule.priority);
    }
    if (rule?.collectionId && (typeof rule.collectionId !== 'string' || !/^[0-9a-f]{32}$/.test(rule.collectionId))) {
      errors[`${key}.collectionId`] = 'Collection IDs use 32 lowercase hexadecimal characters, or leave this empty.';
    }
    if (rule?.kind === 'fixed') {
      if (!isValidTime(rule.time)) {
        errors[`${key}.time`] = 'Use a 24-hour time such as 07:30.';
      } else {
        const existing = fixedTimes.get(rule.time);
        if (existing !== undefined) {
          errors[`${key}.time`] = `Duplicate fixed-time boundary with rule ${existing + 1}.`;
          errors[`rule-${existing}.time`] = `Duplicate fixed-time boundary with rule ${index + 1}.`;
        } else {
          fixedTimes.set(rule.time, index);
        }
      }
    } else if (rule?.kind === 'solar') {
      if (!SOLAR_EVENTS.includes(rule.event)) errors[`${key}.event`] = 'Choose sunrise or sunset.';
      if (!Number.isInteger(rule.offsetMinutes) || rule.offsetMinutes < -720 || rule.offsetMinutes > 720) {
        errors[`${key}.offsetMinutes`] = 'Use a whole-minute offset from -720 to 720.';
      }
      if (!isValidTime(rule.fallbackTime)) {
        errors[`${key}.fallbackTime`] = 'A fixed-time fallback is required when the sun event does not occur.';
      }
    } else {
      errors[`${key}.kind`] = 'Choose a fixed-time or solar rule.';
    }
  }
  return { valid: Object.keys(errors).length === 0, errors };
}

export function formatLocal(ms, timeZone) {
  return new Intl.DateTimeFormat('en-GB', {
    timeZone, weekday: 'short', day: '2-digit', month: 'short',
    hour: '2-digit', minute: '2-digit', hour12: false
  }).format(new Date(ms));
}

function localParts(ms, timeZone) {
  const parts = new Intl.DateTimeFormat('en-US', {
    timeZone, year: 'numeric', month: '2-digit', day: '2-digit',
    hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false
  }).formatToParts(new Date(ms));
  const get = type => Number(parts.find(part => part.type === type)?.value);
  return { year: get('year'), month: get('month'), day: get('day'), hour: get('hour') % 24, minute: get('minute'), second: get('second') };
}

function sameMinute(parts, wanted) {
  return parts.year === wanted.year && parts.month === wanted.month && parts.day === wanted.day &&
    parts.hour === wanted.hour && parts.minute === wanted.minute;
}

function compareLocal(parts, wanted) {
  return (parts.year - wanted.year) || (parts.month - wanted.month) || (parts.day - wanted.day) ||
    (parts.hour - wanted.hour) || (parts.minute - wanted.minute);
}

/** Convert a wall-clock time in an IANA zone to a UTC instant.
 * Repeated times resolve to the first occurrence; nonexistent times resolve to
 * the first valid instant afterward, matching the TV scheduler contract.
 */
export function zonedTimeToUtc(year, month, day, hour, minute, timeZone) {
  const wanted = { year, month, day, hour, minute };
  const target = Date.UTC(year, month - 1, day, hour, minute);
  const exact = [];
  for (let offset = -14 * 60; offset <= 14 * 60; offset++) {
    const candidate = target + offset * MINUTE;
    if (sameMinute(localParts(candidate, timeZone), wanted)) exact.push(candidate);
  }
  if (exact.length) return Math.min(...exact);
  // A DST gap: walk forward until the zone reaches the requested wall clock.
  for (let offset = -120; offset <= 180; offset++) {
    const candidate = target + offset * MINUTE;
    const parts = localParts(candidate, timeZone);
    if (compareLocal(parts, wanted) >= 0) return candidate;
  }
  return target;
}

function dayOfYear(year, month, day) {
  return Math.floor((Date.UTC(year, month - 1, day) - Date.UTC(year, 0, 1)) / DAY) + 1;
}

/** NOAA solar approximation. Returns null when the event does not occur. */
export function solarUtc(year, month, day, latitude, longitude, event) {
  const n = dayOfYear(year, month, day);
  const lngHour = longitude / 15;
  const t = n + ((event === 'sunrise' ? 6 : 18) - lngHour) / 24;
  const m = 0.9856 * t - 3.289;
  let l = m + 1.916 * Math.sin((m * Math.PI) / 180) + 0.020 * Math.sin((2 * m * Math.PI) / 180) + 282.634;
  l = ((l % 360) + 360) % 360;
  let ra = (Math.atan(0.91764 * Math.tan((l * Math.PI) / 180)) * 180) / Math.PI;
  ra = ((ra % 360) + 360) % 360;
  ra = (ra + (Math.floor(l / 90) * 90 - Math.floor(ra / 90) * 90)) / 15;
  const sinDec = 0.39782 * Math.sin((l * Math.PI) / 180);
  const cosDec = Math.cos(Math.asin(sinDec));
  const zenith = (90.833 * Math.PI) / 180;
  const cosH = (Math.cos(zenith) - sinDec * Math.sin((latitude * Math.PI) / 180)) /
    (cosDec * Math.cos((latitude * Math.PI) / 180));
  if (cosH > 1 || cosH < -1) return null;
  let h = (Math.acos(cosH) * 180) / Math.PI;
  if (event === 'sunrise') h = 360 - h;
  h /= 15;
  const localMean = h + ra - 0.06571 * t - 6.622;
  const ut = ((localMean - lngHour) % 24 + 24) % 24;
  return Date.UTC(year, month - 1, day) + Math.round(ut * 3_600_000);
}

function addDays(parts, days) {
  const date = new Date(Date.UTC(parts.year, parts.month - 1, parts.day) + days * DAY);
  return { year: date.getUTCFullYear(), month: date.getUTCMonth() + 1, day: date.getUTCDate() };
}

/** Return the next distinct transitions after `fromMs`, highest priority at a tie. */
export function upcomingTransitions(schedule, fromMs = Date.now(), limit = 4) {
  if (!validateSchedule(schedule).valid) return [];
  const { latitude, longitude, timezone } = schedule.location;
  const start = localParts(fromMs, timezone);
  const candidates = [];
  for (let offset = 0; offset < 8; offset++) {
    const date = addDays(start, offset);
    schedule.rules.forEach((rule, index) => {
      let at;
      let usedFallback = false;
      if (rule.kind === 'fixed') {
        const [hour, minute] = rule.time.split(':').map(Number);
        at = zonedTimeToUtc(date.year, date.month, date.day, hour, minute, timezone);
      } else {
        at = solarUtc(date.year, date.month, date.day, latitude, longitude, rule.event);
        if (at === null) {
          usedFallback = true;
          const [hour, minute] = rule.fallbackTime.split(':').map(Number);
          at = zonedTimeToUtc(date.year, date.month, date.day, hour, minute, timezone);
        } else {
          at += rule.offsetMinutes * MINUTE;
        }
      }
      if (at > fromMs) candidates.push({ at, rule, order: index, usedFallback });
    });
  }
  candidates.sort((a, b) => a.at - b.at || b.rule.priority - a.rule.priority || a.order - b.order);
  const transitions = [];
  for (const candidate of candidates) {
    if (transitions.length >= limit) break;
    if (transitions.at(-1)?.at === candidate.at) continue;
    transitions.push({
      at: candidate.at,
      label: formatLocal(candidate.at, timezone),
      timezone,
      ruleName: candidate.rule.name,
      theme: candidate.rule.theme,
      collectionId: candidate.rule.collectionId || '',
      priority: candidate.rule.priority,
      usedFallback: candidate.usedFallback
    });
  }
  return transitions;
}
