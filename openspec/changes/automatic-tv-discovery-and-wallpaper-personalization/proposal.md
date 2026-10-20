# Proposal

## Why

Opening the companion currently requires manual discovery and exposes connection details before helping users reach their TV. Personalization is still a roadmap item; users need a responsive desktop flow that configures launcher backgrounds and keeps their schedules working after the laptop closes.

## What Changes

- Start bounded local-network discovery on native app launch, with meaningful animation, recovery guidance, and identity-checked reconnection to a previously selected TV.
- Replace the connection-heavy experience with guided pairing, inspection, personalization, review, and verified feedback; returning users reach their TV dashboard.
- Add local file/folder selection and desktop drag-and-drop, preview, crop, dimming, collections, and configurable wallpaper intervals.
- Add fixed-time and sunrise/sunset theme schedules, with independent wallpaper rotation, executed and persisted on the TV.
- Add optional Unsplash and Reddit source adapters behind provider access/terms gates, with honest unavailable states and local-photo fallback.
- Investigate Google Photos import and continuous album access; document supported authenticated routes and avoid promising anonymous album sync.
- Scope all photo display to launcher backgrounds. Idle screensavers, mobile builds, package cleanup, and general launcher installation are outside this change.

## Capabilities

### New Capabilities

- `tv-discovery`: Automatic startup discovery, device selection, trusted reconnection, and recovery.
- `guided-tv-setup`: State-aware desktop setup with accessible microinteractions, review, and verified results.
- `wallpaper-collections`: Local imports, provider-backed collections, previews, rotation, and recoverable transfer.
- `theme-scheduling`: TV-owned fixed-time and solar schedules with offline and restart behavior.

### Modified Capabilities

None. No existing OpenSpec capabilities are registered.

## Impact

The Svelte app (`src/App.svelte`, `src/state.mjs`, `src/style.css`), Rust IPC (`src-tauri/src/lib.rs`), Tauri permissions, persistence, and tests require extension. The current backend only discovers/connects/pairs/inspects; modification requires the identity-bound baseline and journal described in `docs/ARCHITECTURE.md` and M3 of `plans/companion.md`.

A versioned launcher protocol and TV runtime changes are required; launcher source is not present in this repository. Unsupported launcher versions must remain read-only. External launcher work must land and be verified before shipping these controls. `docs/PRODUCT.md` currently requires explicit selection on discovery: update it to permit reconnection only to a previously selected, unambiguously verified TV. Provider downloads add optional Internet dependencies and revise the absolute “everything stays local” copy. No application account or subscription is required for core local features.
