// Legacy separate-adapter status model. MVP upstream configuration uses provider-configuration.mjs.
const freeze = (value) => Object.freeze(value);

export const SOURCE_STATES = freeze([
  'available',
  'empty',
  'unavailable',
  'authentication-required',
  'rate-limited',
  'offline'
]);

const sourceEntries = [
  {
    id: 'local',
    label: 'Local photos',
    type: 'local',
    availability: 'available',
    requiresNetwork: false,
    requiresCredentials: false,
    reason: null
  },
  {
    id: 'unsplash',
    label: 'Unsplash',
    type: 'online',
    availability: 'unavailable',
    requiresNetwork: true,
    requiresCredentials: true,
    reason: 'provider-approval-required'
  },
  {
    id: 'reddit',
    label: 'Reddit',
    type: 'online',
    availability: 'unavailable',
    requiresNetwork: true,
    requiresCredentials: true,
    reason: 'provider-approval-required'
  }
];

export const PHOTO_SOURCES = freeze(sourceEntries.map((source) => freeze(source)));

const sourceById = new Map(PHOTO_SOURCES.map((source) => [source.id, source]));

const stateMessages = freeze({
  available: 'Source is available.',
  empty: 'No photos are available from this source.',
  unavailable: 'This source is unavailable until provider access and terms are approved.',
  'authentication-required': 'Sign-in is required to use this source.',
  'rate-limited': 'This source has temporarily limited requests.',
  offline: 'This source needs an internet connection.'
});

export function getPhotoSource(sourceId) {
  return sourceById.get(sourceId) ?? null;
}

export function createSourceStatus(sourceId, state, options = {}) {
  if (!sourceById.has(sourceId)) throw new RangeError(`Unknown photo source: ${sourceId}`);
  if (!SOURCE_STATES.includes(state)) throw new RangeError(`Unknown source state: ${state}`);

  const status = {
    sourceId,
    state,
    message: options.message ?? stateMessages[state]
  };

  if (options.retryAfterMs !== undefined) {
    if (!Number.isFinite(options.retryAfterMs) || options.retryAfterMs < 0) {
      throw new RangeError('retryAfterMs must be a non-negative number');
    }
    status.retryAfterMs = options.retryAfterMs;
  }

  return freeze(status);
}

// Project runtime details onto a small UI-safe shape. Credentials, response bodies,
// raw errors, and arbitrary adapter fields never cross this boundary.
export function toPublicSourceStatus(status) {
  if (!status || !sourceById.has(status.sourceId) || !SOURCE_STATES.includes(status.state)) {
    throw new TypeError('Expected a known photo source and state');
  }

  const publicStatus = {
    sourceId: status.sourceId,
    state: status.state,
    message: stateMessages[status.state]
  };
  if (Number.isFinite(status.retryAfterMs) && status.retryAfterMs >= 0) {
    publicStatus.retryAfterMs = status.retryAfterMs;
  }
  return freeze(publicStatus);
}

// A failed optional source must not replace or clear the user's local collection.
export function preserveLocalCollectionOnFailure(collection, sourceStatus) {
  if (!collection || !Array.isArray(collection.localImages)) {
    throw new TypeError('Expected a collection with a localImages array');
  }
  if (!sourceStatus || !SOURCE_STATES.includes(sourceStatus.state)) {
    throw new TypeError('Expected a valid source status');
  }
  return sourceStatus.state === 'available' ? { ...collection } : collection;
}
