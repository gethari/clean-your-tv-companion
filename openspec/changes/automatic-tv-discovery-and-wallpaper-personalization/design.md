# Design

## Context

See proposal.md for motivation and scope. The app uses Tauri 2, Rust, Svelte 5 and bundled desktop ADB. `src/App.svelte` holds navigation and a single busy string; discovery is button-driven. Rust exposes four inspection-oriented commands and returns endpoint/status/kind, not stable identity. Existing UI tests assume no launch-time IPC. There is no TV launcher source, media transfer command, journal, provider adapter, or scheduler in this repository.

## Goals / Non-Goals

**Goals:** Separate UI orchestration from device operations; keep schedule execution independent of the desktop; make unsupported capabilities explicit; preserve existing settings and establish recoverable writes.

**Non-Goals:** Screensaver changes, mobile packaging, arbitrary shell/filesystem execution, anonymous Google Photos scraping, general package management, or automatic installation of a compatible launcher.

## Decisions

### Discovery and connection are distinct states

Use explicit idle/searching/candidates/authorizing/connecting/inspecting/ready/recoverable-error states with generation IDs, cancellation and bounded timeouts. Start one native launch search using connected devices plus ADB mDNS; retain manual entry and retry. Do not silently perform a subnet sweep. Recheck on user retry or network change with bounded backoff rather than continuous animation and polling.

Persist the last explicitly chosen TV identity in app data, excluding secrets. Verify candidate identity with authorized read-only queries before auto-selection. Endpoint, model, and service name alone are insufficient; ambiguous/missing identity requires selection. No marker writes during inspection. This changes the explicit-selection-only rule in PRODUCT for previously selected verified TVs. Alternatives: always selecting is unnecessary friction; selecting the first advertisement can target the wrong device.

### Desktop controller and TV runtime use a versioned contract

Define a narrow launcher protocol: capabilities/version, safe current state and revision, stage collection assets, validate patch, activate expected revision, read back, and discard staged assets. Persist collection metadata, schedule, interval and active theme in TV-private storage. Use content hashes, quotas, atomic activation, and acknowledgments. Preserve unknown configuration keys. No generic shell command or unrestricted path IPC is added. The private companion settings form a narrow override layer above the existing external theme; unrelated legacy config and TV preferences remain untouched. Safe state returns effective values and explicit companion override presence separately. Revision checks cover both layers and TV preferences; restore removes a newly added override rather than deleting an existing theme value.

The launcher implementation is an external deliverable: first locate its maintained source and agree the contract with a compatible release. This repository can implement an adapter and contract fixtures, but cannot claim autonomous rotation or scheduling complete until a real compatible launcher passes integration checks. Older launchers stay inspectable with editing unavailable. A desktop-only scheduler was rejected because it stops when the laptop closes.

### Establish personalization-specific durable writes

Implement the M3 identity/baseline/journal guarantees required by this change without building package cleanup. A reviewed change plan records identity, observed revision, exact prior values, media manifest, and intended patch. Durably save intent before writes, acquire a per-device lock, stage verified assets, atomically activate, and query actual state. On disconnect, reconcile revisions and hashes before retry; on drift, request review. Restore only companion-owned settings when current values still match the last applied state. Retain old assets needed for rollback within a declared storage budget. Never mark an unacknowledged operation complete.

### Local imports use explicit desktop capabilities

Use narrow native file/folder pickers and drop events, returning scoped import handles rather than arbitrary frontend filesystem paths. Rust validates readable regular files, actual image format, decoded dimensions and resource limits; folder import does not follow symlinks outside the selected root. Start with JPEG, PNG and WebP; report unsupported formats individually. Normalize orientation, create bounded thumbnails, hash/deduplicate images and preview the same crop/dimming transform used by the TV. Set byte/pixel/count budgets before transfer and expose insufficient storage. Local files are copied to the TV; changes to the original folder do not auto-sync. Automatic folder watching is deferred.

### Schedules resolve locally and predictably

Store a default theme plus ordered daily rules with local time or solar anchor, explicit timezone, location coordinates, fixed-time fallback and priority. Compute solar events locally from saved coordinates. Preview effective transitions and recompute on TV wake, restart, date/time changes and timezone changes. Use the latest effective rule, never replay a backlog. Use a timezone-aware implementation with explicit DST handling per spec. Duplicate fixed-time boundaries are rejected; coincident solar/fixed rules use visible priority. Manual override lasts until the next transition or cancellation. Wallpaper order and interval belong to the collection playback configuration, independent of theme rules.

### Providers are optional adapters with release gates

Use a common source contract for collection metadata, candidates, attribution, paging, fetch status and retry hints. Ship local sources first. Unsplash and Reddit require a documented access/terms decision before activation; do not use scraping as fallback. Review how provider hotlinking/cache/download-event requirements fit TV storage and offline behavior before shipping each adapter. Store desktop credentials in the OS credential store; transfer any required TV credential only through the approved private launcher protocol. Never put credentials in snapshots, logs, URLs shown in UI, or exported reports. Provider revocation clears credentials while preserving local collections and any media retention allowed by provider terms.

Google Photos is a bounded investigation, not a promised integration in this change. Record Picker import requirements, Ambient API partner eligibility, authorization and permitted storage, and feasibility for launcher backgrounds. No Google Photos connect button ships until a subsequent supported integration is specified. Native Google TV screensaver configuration is outside the selected scope.

References checked during exploration:
- https://developers.google.com/photos/support/updates
- https://developers.google.com/photos/ambient/guides/configure-your-app
- https://developers.google.cn/photos/partner-program/overview?hl=en
- https://unsplash.com/api-terms
- https://support.reddithelp.com/hc/en-us/articles/42728983564564-Responsible-Builder-Policy

### Feedback is driven by operations

Keep the illustrated TV, using a subtle scan during actual discovery, brief candidate reveal, explicit pairing confirmation, thumbnail/drop feedback and verified completion. Use existing 160ms interaction / 220ms reveal guidance as initial motion tokens. Progress text reflects actual transfer and verification phases; unknown-duration work uses indeterminate status, never invented percentages. Provide keyboard file selection, live announcements, inline validation and reduced-motion equivalents. Returning users land on a dashboard; fresh users see only the next relevant step. Desktop uses preview beside controls; narrow layouts stack. Reuse state and protocol boundaries for a possible mobile client, without assuming bundled desktop ADB can run on mobile.

## Risks / Trade-offs

- Launcher source absent → external source/release checkpoint blocks write-feature completion, with read-only capability fallback.
- mDNS unavailable or debugging off → manual entry and illustrated TV-side instructions; no promise of finding every powered-off TV.
- Weak TV identifiers → selection on ambiguity and revision/identity rechecks before writes.
- Large collections exhaust storage → preflight budgets, staged activation and preservation of the last working collection.
- Provider approval or retention restrictions → adapters remain unavailable until reviewed; local operation ships independently.
- New write surface → narrow IPC, secrets handling, journal, drift detection and injected-disconnect tests before physical apply.

## Migration Plan

1. Deliver automatic discovery and feedback independently, updating native IPC fixtures and product selection rules.
2. Agree and validate launcher contract; introduce versioned local state with additive migration, leaving old config intact.
3. Land durable personalization writes and local imports behind capability checks; test on an actual compatible TV before enabling.
4. Enable scheduling only after autonomous wake/restart tests pass with the companion closed.
5. Enable each approved provider independently. Publish Google Photos investigation separately from shipped capabilities.
6. Roll back companion UI to inspection if needed; retain journals and restore recorded personalization only after conflict checks. Do not downgrade launcher APK/data automatically.
