# Construction plan

Owner: gethari. Conventional commits. Separate repository; never copy private TV backup or credentials.
Plan status: scoped, inspection preview in progress; mutation features not implemented. This file is the
execution index for a fresh contributor. All steps read PRODUCT, CAVEATS, ARCHITECTURE and DESIGN first.

Dependency graph: M1 → M2 → M3 → M4 → M5 → M6. Profile research and installer QA can run alongside M3/M4,
but production package disable depends on both planner and approved profiles. Do not parallelize edits
to the device executor/journal. Each milestone is a branch/PR; no requirement for multiple agents.

## M1 · Inspection and interaction preview

Context: empty companion repository; upstream scripts are reference only. Build Tauri/Svelte scaffold,
read-only discovery/connect/pair/inspect IPC, demo scenarios for fresh/existing/disconnected TVs, and
native build matrix with bundled ADB. No TV package, HOME or settings changes (pairing persists trust). Use fixtures stripped of identifying details.
Verification: `pnpm install --frozen-lockfile && pnpm check && pnpm test && pnpm build`; `cargo test --manifest-path
src-tauri/Cargo.toml`; platform CI bundles. Real TV read-only smoke check and Linux/Windows launch.
Exit: observable failures stay unknown, demo clearly labeled, no package/settings mutation IPC, CI produces testable
installers. Rollback: revert preview commit; no TV changes to reverse. Commit: feat(preview).

## M2 · Trustworthy device audit and connection recovery

Context: M1 interface/API exists. Make each query typed, timestamped and output/timeout bounded. Detect
user, feature/firmware capabilities and HOME candidates; distinguish installed vs active vs blocked.
Add mDNS service changes, unauthorized/offline handling, locator changes and identity reconciliation.
Show current effective theme state without touching overrides or credentials. Add previewable sanitized
support export, interface selection and bounded optional legacy discovery. Use an actual launcher health
protocol only if implemented separately, never infer provider health from config.
Verification: parser fixture tests for multiple firmware outputs, partial/permission-denied queries,
changed port/IP, multiple TVs, no mDNS, network isolation and unsupported OS. Real TCL snapshot labels
wallpaper credential health unknown and does not reinstall anything.
Exit: identity/observation contract reviewed; every failure gives next step. Rollback: disable new
connection adapter and retain manual connect. Commit: feat(audit).

## M3 · Pure planner, baseline, journal, recovery

Context: M2 supplies trustworthy snapshots. Build typed target-state planner and durable journal before
adding writes. Plan only diffs. Refuse unknown preconditions, identity drift and stale snapshots.
Store exact previous settings and package states. Transaction log must survive process death. Recovery
re-queries ambiguous operations; no blind retry. Build diff/review/backup-location UX and cancellation.
Verification: fixture/property tests for idempotence and pre-existing disabled apps; injected crash
before/after each durable write and each simulated mutation; rollback failure, reset and different TV.
Exit: fake executor proves zero-op rerun and precise restore; no real package disable exposed yet.
Rollback: read-only mode with journals retained/exportable. Commit: feat(recovery).

## M4 · Launcher and personalization

Context: M3 enforces mutation lifecycle. Add integrity-checked, versioned APK catalog and trusted local
APK picker. Validate SDK/ABI/signature, install then launch then verify HOME then hand over; keep working
fallback. Merge dock/theme changes and unknown keys, persist settings precedence and remote helpers.
Local wallpapers first; optional Unsplash only after public-use terms/secret protocol resolved.
Physical remote, HDMI, casting and playback checks are explicit user checkpoints. Never claim an
injected key proves hardware support. Preserve/rebind helpers after restart.
Verification: actual TCL existing setup and a fresh emulator/device; signature mismatch, failed install,
chooser, HOME preference reset, denied settings, transfer disconnect and malformed theme tests.
Exit: verified no-op on existing desired state; recoverable launcher handover; novices complete flow.
Rollback: journal compensation restoring actual previous HOME/config. Commit: feat(setup).

## M5 · Curated cleanup and optional features

Context: M3 planner and M4 fallback are tested. Port reviewed profiles to schemas with provenance,
firmware limits and protected dependency groups. Unknown packages stay enabled. setupwraith ownership
unknown/present blocks disable. Explicit cast/voice/update choices, separate DNS and screensaver plans.
Never execute contributor lists as commands. Restore only app changes; preview change reasons.
Verification: one-command batches with postconditions; protected packages, missing Wi-Fi ownership,
community malicious names, DNS playback failure and disabled-before-baseline fixtures; real-device
physical input/remote tests after tiny batches.
Exit: approved profile per claimed device/firmware; no "universal safe" button. Rollback: exact journal
restore with reconnection instructions. Commit: feat(cleanup).

## M6 · Release readiness and usability

Context: M1–M5 complete; do not call early preview production-ready. Run novice sessions on Linux and
Windows; macOS native smoke test; keyboard/screen-reader/reduced-motion checks. Pin ADB artifacts and
validate redistribution licenses; sign Windows/macOS installers and notarize macOS; WebView2 offline
experience; Linux runtime matrix. Signed update channel only if separately scoped; runtime remains
free and local without accounts. Test final close-debugging checklist and reconnect for maintenance.
Verification: CI matrix, reproducible inventory/SBOM/hash manifest, real installer launch/uninstall,
interrupted install/reconnect/restore trials and review of sanitized exports.
Exit: honest compatibility list, no paid runtime dependency, no secrets in artifacts, all acceptance
gates met and release notes state limitations. Rollback: withdraw bad release, retain prior signed
installer and recovery docs. Commit: chore(release).

## Change protocol

Change scope/dependencies here and in PRODUCT together. Add evidence and new caveats as discovered;
never silently broaden supported devices. A failed gate blocks the related feature, not all read-only
value. Record plan review findings and disposition in docs/REVIEW.md.

## Adversarial review amendments

M2: test identical TVs and IP reuse; ambiguous identity requires reselection and blocks old plans.
M3: compare-and-set restore, cross-process locks, external ADB drift, disk-full/unwritable baseline and
journal retention/export tests are required. M4: APK/data rollback is not guaranteed; show limitations
before install and keep HOME/config compensation distinct. M6: native human launch verification is
separate from CI build completion. Scope review findings are recorded in docs/REVIEW.md.
