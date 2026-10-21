/** Safe configuration model for upstream provider integration. It does not fetch photos or store keys. */
export const PROVIDER_CHOICES = Object.freeze([
  Object.freeze({ id: 'local', label: 'Local photos' }),
  Object.freeze({ id: 'unsplash', label: 'Unsplash' }),
  Object.freeze({ id: 'auto', label: 'Automatic' })
]);
export const PROVIDER_HEALTH = Object.freeze([
  'unknown', 'not-configured', 'ready', 'invalid-credentials', 'collection-inaccessible',
  'empty', 'rate-limited', 'offline', 'unavailable'
]);
const KNOWN_IDS = new Set(PROVIDER_CHOICES.map(item => item.id));
const messages = Object.freeze({
  unknown: 'Provider health has not been checked.',
  'not-configured': 'This provider needs configuration.',
  ready: 'The launcher verified an image from this provider.',
  'invalid-credentials': 'The provider rejected the Access Key.',
  'collection-inaccessible': 'A collection could not be accessed.',
  empty: 'The selected collections have no usable photos.',
  'rate-limited': 'The provider has temporarily limited requests.',
  offline: 'The provider could not be reached. Your current background stays available.',
  unavailable: 'This launcher does not expose supported provider configuration.'
});

/** Upstream's collection tokens are IDs, not URLs/slugs; commas, lines and spaces delimit entries. */
export function normalizeCollectionIds(value) {
  if (typeof value !== 'string' || value.length > 4096) return { ids: [], error: 'Enter at most 50 collection IDs.' };
  const tokens = value.trim().split(/[\s,]+/).filter(Boolean);
  if (tokens.length > 50 || tokens.some(token => !/^[A-Za-z0-9_-]{1,128}$/.test(token))) {
    return { ids: [], error: 'Use collection IDs separated by commas or new lines, without URLs.' };
  }
  return { ids: [...new Set(tokens)], error: null };
}

/** Never derive capabilities from versionName or interpret an absent flag as support. */
export function providerCapabilities(raw = {}) {
  const candidates = Array.isArray(raw.providers) ? raw.providers : [];
  const providers = PROVIDER_CHOICES.filter(item => candidates.includes(item.id)).map(item => item.id);
  return Object.freeze({
    providers: Object.freeze(providers),
    configuration: raw.configuration === true,
    privateCredentials: raw.privateCredentials === true,
    // Retrieval is launcher-owned. Credential-presence/status readback never implies edit support.
    editable: raw.configuration === true && raw.privateCredentials === true && providers.length > 0
  });
}

/** Allowlist status fields; discard raw messages, URLs, credentials, tokens and future fields. */
export function safeProviderStatus(raw = {}) {
  const health = PROVIDER_HEALTH.includes(raw.health) ? raw.health : 'unknown';
  const ids = Array.isArray(raw.collectionIds) ? normalizeCollectionIds(raw.collectionIds.join(',')) : { ids: [], error: null };
  const result = {
    provider: KNOWN_IDS.has(raw.provider) ? raw.provider : null,
    credentialPresent: typeof raw.credentialPresent === 'boolean' ? raw.credentialPresent : null,
    collectionIds: Object.freeze(ids.error ? [] : ids.ids),
    configurationSaved: typeof raw.configurationSaved === 'boolean' ? raw.configurationSaved : null,
    health,
    message: messages[health]
  };
  if (Number.isSafeInteger(raw.retryAfterSeconds) && raw.retryAfterSeconds >= 0 && raw.retryAfterSeconds <= 86400) {
    result.retryAfterSeconds = raw.retryAfterSeconds;
  }
  return Object.freeze(result);
}

/**
 * Validate a memory-only draft and return a review projection that NEVER contains the Access Key.
 * A private native executor must handle sensitive bytes separately when a verified bridge exists.
 */
export function reviewProviderDraft(draft, observed, capabilityInput) {
  const capabilities = providerCapabilities(capabilityInput);
  const status = safeProviderStatus(observed);
  /** @type {Record<string, string>} */
  const errors = {};
  const provider = draft?.provider;
  const credentialAction = draft?.credentialAction ?? 'keep';
  const normalized = normalizeCollectionIds(draft?.collectionIds ?? '');
  if (!capabilities.editable) errors.support = messages.unavailable;
  if (!capabilities.providers.includes(provider)) errors.provider = 'Choose a provider supported by this launcher.';
  if (!['keep', 'replace', 'remove'].includes(credentialAction)) errors.credentialAction = 'Choose whether to keep, replace or remove the saved key.';
  if (normalized.error) errors.collectionIds = normalized.error;
  if (credentialAction === 'replace' && (typeof draft.accessKey !== 'string' || !/^[A-Za-z0-9_-]{10,200}$/.test(draft.accessKey))) {
    errors.accessKey = 'Enter a valid Unsplash Access Key. The Secret Key is not needed.';
  }
  if (provider === 'unsplash') {
    if (!normalized.error && !normalized.ids.length) errors.collectionIds = 'Add at least one collection ID.';
    if (credentialAction === 'remove') errors.credentialAction = 'Switch to Local photos before removing the saved key.';
    if (credentialAction === 'keep' && status.credentialPresent !== true) errors.accessKey = 'Add an Access Key before selecting Unsplash.';
  }
  const changed = provider !== status.provider || normalized.ids.join(',') !== status.collectionIds.join(',') || credentialAction !== 'keep';
  return {
    valid: Object.keys(errors).length === 0,
    errors,
    review: Object.freeze({ provider: KNOWN_IDS.has(provider) ? provider : null, collectionIds: Object.freeze(normalized.ids), credentialAction: ['keep', 'replace', 'remove'].includes(credentialAction) ? credentialAction : null, changed })
  };
}
