# Tasks

## 1. Automatic discovery and guided connection

- [x] 1.1 Introduce explicit connection/search state with generation IDs and cancellation; verify timeout, retry, stale-result and overlapping-search tests.
- [x] 1.2 Start bounded native discovery on launch and retain manual connection; update Playwright IPC fixtures and verify browser demo makes no discovery calls.
- [x] 1.3 Persist selected identity and verify remembered candidates before reconnecting; test changed ports, duplicate model names, missing identity and revoked authorization.
- [x] 1.4 Add discovery animation, device reveal, inline pairing feedback and recovery steps; verify keyboard navigation, live status and reduced-motion behavior in UI tests.
- [x] 1.5 Update PRODUCT, DESIGN and setup documentation for automatic discovery and conditional reconnect; verify documented fresh/returning flows against a real desktop build.

## 2. Launcher contract and recoverable personalization

- [x] 2.1 Locate the maintained launcher source and record its repository, compatible version and contract owner in architecture documentation; verify source and release are available before proceeding with TV runtime work.
- [x] 2.2 Define versioned capabilities, safe state/revision, staging, validation, atomic activation, readback and discard messages; verify protocol fixtures cover older/unsupported versions and preserve unknown keys.
- [x] 2.3 Deliver the launcher-side contract in its maintained project and record release evidence here; verify contract integration against an installed compatible launcher. This external dependency blocks wallpaper-write completion.
- [x] 2.4 Implement identity-bound plans, durable baseline/journal and per-device locking for personalization; verify no-op, stale revision, process interruption and concurrent apply tests.
- [x] 2.5 Implement staged transfer/activation and conflict-aware restore; inject disconnects before and after activation and verify reconciliation never claims unobserved success or overwrites later manual changes.
- [x] 2.6 Document protocol capability fallback, baseline location, restore limits and compatible launcher requirement; verify unsupported versions expose inspection with no editing action.

## 3. Local wallpaper collections

- [x] 3.1 Add scoped native pickers/drop handles and bounded JPEG/PNG/WebP decoding; test corrupt content, unsupported files, oversized dimensions and folder symlink escape without broad filesystem permissions.
- [x] 3.2 Add thumbnails, deduplication, crop/dimming preview and collection editing; verify mixed imports preserve accepted images and all actions work with keyboard input.
- [x] 3.3 Transfer manifests/assets with hash verification and storage preflight; test interrupted transfer, retry, cancellation and insufficient storage preserve the active collection.
- [ ] 3.4 Deliver TV-owned sequential/shuffle playback and interval persistence in the compatible launcher; verify 5/15/30/custom intervals, invalid inputs, single-image collections and restart behavior using controlled-clock tests and real-device evidence.
- [ ] 3.5 Document one-time folder copying, supported formats, limits and launcher-only display; verify preview crop/dimming against an actual TV screenshot.

## 4. Scheduled themes

- [x] 4.1 Add schedule editing with default theme, fixed-time/solar rules, explicit location/timezone, priority and fallback; verify upcoming-transition previews and invalid/duplicate-boundary feedback.
- [ ] 4.2 Deliver TV-side local solar/timezone evaluation and saved schedules through the versioned contract; test polar dates, DST gaps/repeats, coincident rules, midnight, timezone changes and manual override expiry.
- [ ] 4.3 Verify wake/reboot reconciliation and independent rotation on a real compatible TV with the laptop closed and network unavailable; record results and confirm missed transitions are not replayed.
- [x] 4.4 Document solar fallback, timezone behavior and offline operation; verify examples correspond to scheduler fixtures.

## 5. Optional online providers and Google Photos investigation

- [x] 5.1 Record a current access, attribution, hotlinking, caching/retention, download-event and credential-handling decision for Unsplash and Reddit; verify each decision against official documentation and approved access before enabling its adapter.
- [ ] 5.2 Add the provider source/status contract and secure credential handling through narrowly scoped IPC and TV storage; verify logs/exports exclude secrets and revocation clears credentials.
- [x] 5.3 Implement an approved Unsplash adapter or retain an explicit unavailable state when access/terms gates fail; verify collection preview, attribution, event reporting and offline/rate-limit fallback against the recorded decision.
- [x] 5.4 Implement an approved Reddit adapter or retain an explicit unavailable state when approval is absent; verify configured community selection, pagination, media filtering and authentication/rate-limit recovery using approved access or fixtures.
- [x] 5.5 Deliver a Google Photos feasibility report covering Picker import, Ambient partner access, login requirements, retention and launcher-background eligibility; verify official sources and record a supported follow-up recommendation without shipping a speculative connect control.
- [x] 5.6 Update privacy/product copy for optional online sources; verify local import works without an app account or provider credentials and unavailable providers never block it.

## 6. Integrated desktop acceptance

- [x] 6.1 Run pnpm check, pnpm test, pnpm exec playwright test and relevant Rust/launcher suites; record passing results and any platform-specific limitations.
- [ ] 6.2 Exercise fresh pairing, remembered reconnect, multiple candidates, dropped connection, reviewed apply and precise restore on native desktop builds; record Windows/macOS/Linux results separately from CI build success.
- [ ] 6.3 Verify desktop and narrow layouts, reduced motion, meaningful progress and launcher-only wallpaper display in a novice walkthrough; record remaining usability failures before release.
