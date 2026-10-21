# Tasks

## 1. MVP baseline and upstream integration evidence

- [x] 1.1 Reconcile PRODUCT and HANDOFF with the agreed MVP, predecessor reuse and deferred scope; verify no document claims unfinished setup or physical acceptance is complete.
- [x] 1.2 Inventory current upstream source, provider configuration interfaces, released APK artifacts and signing metadata; record exact revisions and missing gates in docs/UPSTREAM_SUPPORT.md without modifying launcher repositories.

## 2. Local photo editor and accessible reviews

- [x] 2.1 Fix rejection-code messages, crop-failure rollback, aggregate draft quotas, immutable review, baseline path and stored-photo preview labels; verify focused mocked Playwright tests plus pnpm check.
- [x] 2.2 Verify mixed imports, order/custom intervals, keyboard controls, narrow layout, reduced motion, unsupported launcher fallback and demo isolation in editor tests; document supported formats/limits and observed UI behavior.
- [x] 2.3 Verify transfer resume, cancellation, missing/hash-mismatched assets and insufficient storage with injected fake-device tests; document preservation of active/rollback collections.

## 3. Persistent operation history and recovery

- [x] 3.1 Add identity-scoped safe journal summaries and bounded tolerant enumeration; verify unrelated TV records and private fields are excluded and corrupt records do not hide valid history.
- [x] 3.2 Add history UI after reopening, reconcile/restore actions and unknown/conflict feedback; verify mocked restart, changed identity and restore-review flows and document recovery limits.
- [x] 3.3 Extend typed durable setup operations for installation, HOME and package state without breaking existing journals; verify fake-executor interruptions, no-op, stale plans and precise compensations before exposing writes.

## 4. Upstream wallpaper provider configuration

- [x] 4.1 Add capability-driven provider/configuration and safe-status types with upstream collection-ID normalization; verify existing-key keep, replace/remove intent, malformed/duplicate IDs and supported-source selection with fixtures.
- [x] 4.2 Add source selector and masked Unsplash Access Key/collection editor with explicit review, distinct validation/persistence/fetch outcomes and no secret persistence; verify offline mocked UI tests and document configuration semantics.
- [ ] 4.3 Integrate a verified upstream private credential/configuration bridge with atomic activation and opaque rollback references; verify argument/log/export redaction, interrupted key update, removal and restoration without shared-storage secret staging. Block this task if that upstream interface is absent.
- [ ] 4.4 Verify upstream-driven retrieval attribution, fallback, offline/rate-limit behavior and continued playback with companion closed on the pinned compatible launcher; physical testing requires the user's restriction to be lifted.

## 5. Launcher installation and HOME handover

- [ ] 5.1 Add pinned trusted artifact metadata/download and compatibility/signature verification against a usable upstream release; verify corrupted download, mismatched signing identity, SDK/ABI mismatch and already-installed no-op fixtures. Missing trusted artifact blocks installation acceptance.
- [ ] 5.2 Add reviewed install and HOME chooser/handover with stock-launcher fallback; verify fake-device partial install, declined chooser, interrupted handover and exact prior HOME restore, and document APK/data rollback limits.

## 6. Validated optional TCL cleanup

- [x] 6.1 Add versioned exact-model/firmware profile rules and protected dependency constraints; verify unknown firmware, pre-disabled packages, bundled casting/voice and unselected choices yield no writes, and document evidence requirements.
- [ ] 6.2 Deliver reviewed sequential cleanup/app-owned restore using validated TCL profile evidence; verify postconditions/conflicts and physical remote/HDMI/casting/update behavior before approving the profile. Physical checks remain blocked during no-live testing.

## 7. Integrated MVP acceptance

- [x] 7.1 Re-run frontend/Rust/packaged build checks and all mocked setup/provider/history suites after integration; record passing commands and pending external dependencies separately.
- [ ] 7.2 Exercise fresh/existing setup, source changes, local photos, cleanup and restart/restore on packaged Windows 11 x64 and Ubuntu 24.04 x64; record novice keyboard/narrow/reduced-motion acceptance and physical limitations before calling MVP complete.
