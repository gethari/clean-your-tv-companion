# Proposal

## Why

The companion has a useful inspection preview and unfinished photo/recovery code, but cannot yet guide a novice through a complete, verified TV setup. The MVP must deliver reliable Windows/Linux setup, local wallpapers and upstream provider configuration with explicit recovery and a clear boundary around external launcher capabilities.

## What Changes

- Complete discovery, pairing, inspection and a state-aware setup dashboard for new and already customized TVs.
- Add reviewed installation and HOME handover using a pinned compatible upstream launcher artifact; preserve the original launcher fallback.
- Complete local photo import, crop/dim preview, intervals, transfers and durable history/restore.
- Configure the upstream wallpaper provider selection, including Unsplash Access Key and collection IDs, through a verified supported interface. Reuse launcher providers rather than implementing another image retrieval service.
- Add optional reversible cleanup only for validated TCL model/firmware profiles, with explicit selection and protected features.
- Require automated verification and packaged Windows/Linux user acceptance; no terminal or developer tools for normal use.
- Defer schedules, Google Photos, new provider implementations, mobile, DNS, screensavers and remote remapping.

## Capabilities

### New Capabilities

- `companion-setup`: Discovery, authorization, inspection, state-aware progression and desktop usability.
- `launcher-management`: Trusted launcher artifact installation, HOME handover and fallback preservation.
- `wallpaper-management`: Local collections and configuring existing upstream photo providers.
- `setup-recovery`: Durable reviewed operations, history, reconciliation and conflict-aware restore.
- `curated-cleanup`: Explicit package cleanup governed by tested firmware-specific rules.

### Modified Capabilities

None registered in main specs. Reuse implementation and evidence from `automatic-tv-discovery-and-wallpaper-personalization`; that unfinished change remains historical/future scope, not a second execution queue for this MVP. Its checked tasks do not establish MVP acceptance.

## Impact

Changes affect the Svelte setup/editor, narrowly scoped Rust IPC, existing journal/executor, local media cache, release metadata and offline tests. The maintained launcher repository is an external integration dependency, not an implementation target. Pin verified upstream source/artifacts after inspecting actual capabilities; do not infer a companion protocol merely from provider support. Physical TV acceptance remains blocked by the user's no-live-testing restriction until explicitly lifted.
