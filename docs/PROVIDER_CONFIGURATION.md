# Configuring existing upstream wallpaper providers

The MVP requires source switching and Unsplash Access Key/collection configuration using the launcher's
existing provider implementation. It does not fetch wallpaper bytes in a separate desktop service.

The provider draft model supports Local photos, Unsplash and Automatic when advertised by a verified
configuration interface. Collection IDs accept comma, whitespace or newline separators and are deduplicated;
URLs and malformed tokens are rejected. An existing saved key can be kept without reading its value,
replaced explicitly, or removed after switching away from Unsplash. Local selection preserves the provider
configuration unless removal is requested. The Secret Key is never needed.

Reddit is not offered as a source. No Reddit API approval or permitted offline-retention decision exists
for this app, so the UI keeps it explicitly unavailable instead of exposing a control that cannot work.
Local photo import does not need Reddit, Unsplash or any account.

Reviews contain only provider, collection IDs and keep/replace/remove intent. They never contain the
Access Key. Provider status distinguishes persisted configuration from a confirmed image fetch, plus
unknown, invalid credentials, inaccessible/empty collections, rate limits and offline status. Failed
retrieval must preserve the usable background.

## Current implementation and dependency

`src/provider-configuration.mjs` contains the tested, secret-free review/status model. The Backgrounds
page has a demo editor with masked memory-only input, immutable review, and clearing on dismissal,
completion or unmount. It shows the current source, reported collection IDs, credential presence,
saved-configuration state and retrieval health as separate fields. Demo actions contact neither a TV nor
Unsplash and do not validate real credentials. Offline mocked UI tests cover keeping an existing key while
editing collections, switching to local photos, rejected removal order, URL validation, dismissal clearing,
masked input and the native no-bridge state.

Native mode does not collect a key or expose Apply until the pinned upstream release has a verified
private configuration/credential bridge. The source inventory in UPSTREAM_SUPPORT.md records that gap.
The live adapter, private rollback references, runtime verification and release acceptance remain pending;
a passing model/demo test is not a completed Unsplash integration. The old global "provider approval"
placeholder does not describe this new configuration boundary: this work configures a user-selected
upstream provider and must honor that provider's applicable requirements.

When the bridge exists, sensitive key bytes must travel separately from ordinary plans; public readback
contains only credential-present status. Never use generic theme files or shell arguments to stage keys,
and never include them in logs, snapshots, recovery exports or ordinary journals.
