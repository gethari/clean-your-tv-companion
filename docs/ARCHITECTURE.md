# Architecture decision: Tauri 2 + Rust + Svelte

Accepted 2026-10-20. User explicitly rejected Electron. Tauri supplies a native shell using the system
webview; Svelte renders the UI. This is not a claim of native toolkit widgets. Rust owns all ADB,
identity, discovery, timeout, persistence and recovery behavior. Node is development tooling only.

## Boundaries

Svelte → small typed Tauri commands → Rust domain services → official bundled platform-tools.
The UI cannot send arbitrary shell commands, binary paths or filesystem paths to a general executor.
No shell plugin or remote navigation permissions. Enforce CSP, validated arguments, allowlisted commands,
per-device exclusive mutation lock, cancellation and time/output limits. Do not inherit arbitrary proxy
or ADB server overrides in production. Reuse an existing ADB server carefully; never kill a user's server
as discovery cleanup. Isolated app-owned key/server strategy is a later explicit decision with migration.

Preview IPC: `discover`, `connect_device(endpoint)`, `pair_device(endpoint, code)`, `inspect_device(serial)`.
Strict LAN endpoint syntax, separate pairing-code validation; code goes to ADB stdin, never logs/argv.
Preview has no package, HOME or settings mutation IPC. Pairing creates persistent trust and ADB can create host keys/start its server; revoking trust is a TV-side action. Discovery is connected-device + mDNS discovery; subnet sweep is deferred,
limited to user-selected private interface and explicit opt-in. mDNS does not identify manufacturer/OS.

## Observation versus intent

`DeviceSnapshot { identity, collectedAt, sources, capabilities, home, packages, theme }`.
Each observation has known/unknown/error semantics and per-user scope. An empty disabled-package list
is valid only after a successful query. Keep installed/enabled/default/verified separate. An inspection
failure never becomes an empty list. Snapshot properties are display data and never rendered as HTML.
Secrets are not included in snapshots. Private wallpaper state is unknown absent a launcher health API.

`ChangePlan { deviceIdentity, snapshotHash, profileVersion, operations[], conflicts[], expiresAt }`.
An operation holds typed intent, exact prior state, precondition, expected postcondition, compensation
and user-facing reason. A package already disabled produces zero operations. The planner is pure and
unit-testable. Execution refreshes preconditions, rejects stale/different identities, runs sequentially
and re-queries; exit code zero alone is not success. Unknown completion after disconnect means reconcile.

## Identity and restore

IP/port are locators, never identifiers. Serial/property identity can be missing or spoofed and is
combined with manufacturer/model/firmware, user's selected TV, and optional app-generated marker only
after explicit modification. Don't silently write a marker during read-only inspection. If identity
is ambiguous, require selection before any write. Firmware/reset drift invalidates pending plans.

Write baseline and operation intent durably before mutation, including fsync/atomic replacement; append
observed result afterwards. Store under OS app-data with restrictive permissions. Persist partial results
and resumable state. A restore reverses app-owned changes in dependency order to prior values only;
never enable every package in a list or assume default DNS/animation settings. Test rollback failures,
not just successful rollback. Baseline isn't a full firmware/data backup; explain its limits.

## Package and APK trust

Curated JSON profiles with schema/version/provenance, firmware constraints and protected dependencies.
Reject invalid package names, shell syntax, unknown actions and untrusted remote scripts. No execution
of upstream text files. APK catalog entries carry source URL, exact version, hash, package ID, min SDK,
ABI and signing identity. Verify bytes before install, then package identity/version/signature and launch.
Don't scrape/install arbitrary "latest" URLs without integrity metadata. Third-party apps are optional,
licensed/attributed, and user-provided APKs require explicit local selection and signature checks.

## Secrets and privacy

No telemetry or backend. Support export is explicit, previewable and redacted: network names, serials,
account IDs, tokens, ADB keys and property dumps are not exported by default. Future provider keys use
OS credential store; least privilege transfer to app-private TV storage via a narrow, versioned launcher
protocol. Never bundle this owner's shared env file, collections, device address, APK signing key or TV
backup in the companion. An API health response returns booleans/status only, never credentials.

## Packaging

Build natively on GitHub-hosted Windows, Linux and macOS runners; headless Linux checks frontend and
Rust tests, then build native bundles. Official platform-tools are downloaded for each target and
bundled as resources (Windows includes companion DLLs). Record version, source and SHA-256 in notices;
production gate pins a reviewed archive hash instead of mutable latest. Windows NSIS EXE, Linux
AppImage/deb and macOS DMG. App icons are original local vector artwork; no external image dependency.
Public stable requires native smoke tests, code signing/notarization and reviewed redistribution notices.
Unsigned preview downloads are labeled; do not tell users to disable antivirus or global OS protections.

## Recovery limits and conflicts

Restore compares current state with our last verified applied state before writing the recorded prior
value. Later manual/firmware changes create a conflict: Keep current or Restore recorded value, with
explicit review. No blind overwrite. APK/data rollback is not generally supported: installing updates
can migrate data and Android may refuse downgrades. Launcher rollback restores HOME/config where
possible, not an earlier APK/data snapshot. Explain this distinction before installation.

Use an OS-level per-device lock across companion instances; external ADB tools can still change a TV,
so recheck preconditions immediately before every operation and stop on unexpected postconditions.
Fail closed if identity is ambiguous, including identical TVs or reused IP addresses. Nicknames are not
identity proof. Verify baseline/journal storage is writable with enough space before a mutation; retain
journals on uninstall or offer explicit export/removal. Restrict in-app navigation to bundled content;
any future external links open in the system browser through a narrow allowlist.
