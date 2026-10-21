# Companion MVP implementation status — 2026-10-21

Active OpenSpec change: `companion-full-setup-mvp`. Progress: **12/18 tasks complete**. This is an
implementation checkpoint, not a completed modifying MVP. The predecessor change remains historical.

## Completed companion work

- Saved and strictly validated the agreed full-setup MVP scope and source/release integration gates.
- Fixed local editor rejection feedback, aggregate quotas, crop rollback, immutable reviews and baseline
  disclosure; tested mixed imports, custom intervals/order, keyboard, narrow layout and reduced motion.
- Added bounded TV-identity-scoped operation history, explicit read-only reconciliation, reviewed restore
  and cancellation, retaining valid records when other records are corrupt or oversized.
- Added provider configuration/status models and a demo source editor for local/Unsplash/automatic,
  masked memory-only keys, collection IDs and separate saved-vs-fetched status. The panel shows the
  current source, reported collections, credential presence, saved configuration and retrieval health
  independently, keeps Reddit explicitly unavailable, and native configuration does not collect keys or
  offer Apply without a verified private bridge. Offline UI tests cover keep/edit/local/remove intent.
- Added a fake-backend-tested durable setup kernel for install/HOME/package operations. It preserves
  existing journals and verifies identity, revisions, review expiry, intent durability, drift and restore.
  No native setup-write adapter or Tauri write command is registered for this kernel.
- Added an exact-firmware cleanup policy with protected dependencies and explicit selections. No
  approved profile is shipped; the kernel cannot authorize real cleanup from brand detection.

## Remaining gates and implementation

| Tasks | Needed before completion |
| --- | --- |
| 4.3 | Integrate a verified upstream private credential/configuration bridge. The companion UI/model and offline tests are complete; no real provider request, key transfer or private rollback exists without the upstream interface. |
| 4.4 | Verify upstream retrieval/attribution/fallback and TV-owned playback with companion closed, after integration and explicit permission for physical testing. |
| 5.1 | A trusted compatible upstream APK with pinned hash, package/version, compatibility and signing identity. The inspected release has no APK assets. |
| 5.2 | Wire the typed kernel into reviewed native installation and HOME handover after the artifact gate; verify chooser/fallback and actual restore behavior. |
| 6.2 | Wire reviewed cleanup/restore and supply approved exact TCL firmware evidence, including physical checks. Community comparison lists are not that approval. |
| 7.2 | Packaged Windows 11 x64 and Ubuntu 24.04 x64 novice acceptance for all integrated flows. |

The predecessor change is at 22/29 tracked tasks. Its new theme-schedule editor is a local, preview-only
retained implementation; it is not MVP schedule delivery and does not advertise launcher capability or
write to a TV. See docs/SCHEDULE.md.

The source and artifact facts are pinned in UPSTREAM_SUPPORT.md and integrations/upstream-support.json.
Recheck the latest upstream revision when integration resumes; do not infer a private companion API
merely from a provider implementation or replace the agreed upstream baseline with an owner fork.
All launcher repositories were left unchanged during this MVP work.

## Verification

- `pnpm check`: zero errors/warnings.
- `pnpm test`: all JavaScript test files pass (35 tests), including provider configuration and schedule preview cases.
- `pnpm exec playwright test`: 32 mocked browser tests pass.
- `cargo test --locked --offline --manifest-path src-tauri/Cargo.toml`: 61 pass; one ignored subprocess
  helper is deliberately invoked by its parent lock test.
- `pnpm tauri build --debug --bundles deb -- --offline` with offline Cargo: Linux amd64 package built;
  package metadata checked with dpkg-deb. The executable/package was not launched or installed.
- `openspec validate companion-full-setup-mvp --strict`: passes.

No live TV, ADB, provider API calls with real credentials, or native application launches were used.
The no-live-TV restriction persists until the user explicitly lifts it. A build is not physical
acceptance, and demo provider success is not a working Unsplash integration.
