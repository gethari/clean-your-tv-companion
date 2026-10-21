# Development handoff — 2026-10-20

## Active MVP update — 2026-10-21

The user selected full setup, Windows + Linux acceptance, validated TCL cleanup first, and a pinned
upstream launcher release. The new active change is `companion-full-setup-mvp`; read its proposal,
design, specs and tasks. It includes local photos AND upstream provider switching with Unsplash
Access Key/collection IDs. Do not resume predecessor scheduling or third-party provider implementation
as MVP work. Preserve the historical evidence below without treating it as current acceptance.

All implementation is now companion-only. Do not edit original or temporary launcher checkouts.
Resolve the latest provider-capable upstream release and verify its actual configuration interfaces.
The no-live-TV restriction still applies. Follow the new tasks for current completion counts and
`docs/UPSTREAM_SUPPORT.md` for source/release/bridge evidence once the inventory is delivered.

## MVP implementation checkpoint

Current progress is **11/18**. See [MVP_STATUS.md](docs/MVP_STATUS.md) for completed work, all remaining
external/integration gates, and final verification (61 Rust tests, 26 mocked browser tests, and an
offline Linux deb build). No new native app, provider or TV test was performed.

Completed companion work now includes local editor correctness and focused tests, bounded identity-scoped
background-operation history with explicit reconcile/restore, a secret-free upstream provider draft/status
model and demo-only source editor, a fake-tested typed setup-operation kernel, and a deny-by-default cleanup
policy. Native provider configuration remains disabled: do not collect keys without a verified bridge.
The live installation/HOME/cleanup adapters are not wired and no cleanup profile is approved. Background
history currently covers v1 personalization; typed setup v2 records live separately pending live integration.

Read docs/LOCAL_PHOTOS.md, docs/RECOVERY.md, docs/PROVIDER_CONFIGURATION.md, docs/SETUP_OPERATIONS.md and
docs/CLEANUP.md for API/behavior boundaries. UI tests mock device/provider commands; no TV was used.
The upstream evidence inventory is pinned to an inspected source/release, not a claim about a later merge.
Recheck it when a provider-capable release/private bridge is supplied; do not substitute the owner's
experimental launcher branch without a new explicit integration decision.

## Resume here

The user requested a checkpoint, conventional commits and pushes, and this handoff in the repository. Implementation is paused at that request. Historical checkpoint for `automatic-tv-discovery-and-wallpaper-personalization`; read its [tasks](openspec/changes/automatic-tv-discovery-and-wallpaper-personalization/tasks.md), design and spec deltas before editing. Tracking currently records 14/29 tasks complete. Remaining unchecked tasks include partially implemented work; do not equate committed code with accepted features.

**Do not contact or test the real TV. Someone is watching it.** This restriction persists until the user explicitly lifts it. Do not launch the native companion (it automatically discovers/reconnects), run ADB, install APKs, query TV state, change Home, take TV screenshots, or run the live smoke scripts in `/tmp`. The owned native test process was terminated. Mocked browser tests, Rust unit tests and offline launcher Gradle tests are permitted.

The user requested parallel agents with `gpt-6-luna` (clarified from unavailable `5.6-luna`). No continuing agent work should be assumed after this checkpoint. Ask no redundant authorization questions for ordinary implementation, offline tests or the already requested Git pushes.

## Repositories and source preservation

- Companion: this repo, `main`, origin `https://github.com/gethari/clean-your-tv-companion.git`.
- Launcher: isolated worktree `/tmp/cytv-launcher-owner`, branch `feat/companion-owner-runtime`, fork remote `https://github.com/gethari/cleanyourtv.git`. Use this worktree, not the other older temporary worktrees.
- Original launcher checkout at the sibling `cleanyourtv` directory has the user's uncommitted owner customizations. **Do not reset or modify that checkout.** The isolated branch preserves its existing Unsplash and remote-key runtime additions.
- [integrations/README.md](integrations/README.md), [launcher-release.json](integrations/launcher-release.json) and the portable patch pin the source/base. The installed candidate evidence applies to an earlier source commit, not the latest playback/scheduler checkpoint. No new APK was installed after the no-live instruction.
- `.agents/` and `.claude/` were pre-existing untracked local tooling and are deliberately excluded from implementation commits. Do not indiscriminately `git add .`.
- Never commit APKs, signing keys, local provider credentials, caches or private TV baseline/config backups. Prior live evidence is documented in [REVIEW.md](docs/REVIEW.md); do not rerun its physical checks while the restriction applies.

## Implementation map and next work

- Connection state/controller and startup discovery: `src/connection.mjs`, `src/App.svelte`, Rust identity/remembered-device commands, existing Playwright fixtures. Discovery is bounded and read-only; never reconnect by IP/model alone.
- Recoverable changes: `launcher_protocol.rs`, `launcher_transport.rs`, `journal.rs`, `transaction.rs`, `personalization.rs`. Plans save exact prior override presence, identity and revision; per-device OS locks; intent is durable before activation; reconciliation verifies result plus operation/session/revision/readback; restore creates a separate reviewed inverse plan. Offline disconnect/drift/cache/restore tests exist. Review [LAUNCHER_PROTOCOL.md](docs/LAUNCHER_PROTOCOL.md) for limits and recovery behavior.
- Local imports: `media.rs`, native dialog plugin and Rust drag/drop bridge in `lib.rs`. No frontend path command. JPEG/PNG/WebP decode limits, orientation normalization, content hashes, private cache, thumbnails, exact 16:9 crop transform. Limits are 20 MiB/image, 33,177,600 pixels, 200 assets / 256 MiB per batch; merged collections must also enforce limits. No automatic folder sync.
- UI checkpoint: `src/Personalization.svelte`, mounted by Backgrounds navigation. State/capability check happens when opening that tab; explicit review and Apply; demo must issue no native calls. Import, crop, dim, interval, draft collection, recovery and restore controls need the remaining Playwright/acceptance coverage. Known editor follow-ups: rejection copy still maps underscore codes instead of native hyphen codes; baseline path is not displayed; provider footer uses static unavailable copy; stored-TV photo preview copy needs refinement; a failed crop must reset its slider anchor. New editor-specific Playwright coverage has not been added. Restoring from a previous app session still needs durable operation discovery/history UI; do not assume `lastAppliedId` in component memory is sufficient.
- Launcher playback: `WallpaperPlayer.java`, `WallpaperRotation.java` and tests. Sequential/shuffle, validated custom/preset intervals, persisted selection/time, single-image behavior. Companion collections override Unsplash only when selected usable companion assets exist; legacy Unsplash remains available and its credentials/settings stay untouched. Actual TV playback/restart evidence is pending.
- Scheduler is **work in progress**: `ThemeSchedule.java` and tests plus partial persistence/provider/MainActivity wiring. Pure evaluator covers fixed/solar events, priorities, DST, polar fallback and manual override expiry. Latest integration builds and passes JVM tests but is not accepted; `theme_schedules_v1` was removed from capabilities at checkpoint. Rust schedule schema/preview IPC, editor, complete safe validation, runtime integration, and wake/offline device acceptance remain pending. Keep the capability disabled until end-to-end work is verified. Launcher preview method is `schedule_preview`; proposed manual key `theme_manual_override` differs from the Rust reserved `theme_schedule_enabled` design and needs reconciliation before enabling.
- Providers: [PROVIDER_ACCESS.md](docs/PROVIDER_ACCESS.md), [GOOGLE_PHOTOS.md](docs/GOOGLE_PHOTOS.md), `src/providers.mjs` and tests. Official-source gate decisions are complete. No approved API access is established. Unsplash/Reddit stay unavailable; no scraping fallback. Secure credential storage/revocation task remains unfinished. Google Photos is investigation only: OAuth Picker is bounded selection, Ambient requires partner access; no speculative connect control. Photos affect launcher backgrounds only, never screensavers.

## Validation and commands

All checks at this checkpoint are offline. See the checkpoint results appended below. An initial parallel Rust lock test exposed transient fork descriptor inheritance; the two lock tests now serialize against each other. The subprocess lock helper is intentionally marked ignored and is explicitly invoked by its parent test.

Rust is installed outside the default PATH:

```bash
PATH=/home/linuxbrew/.linuxbrew/Cellar/rustup/1.29.0_2/bin:$PATH cargo test --locked --offline --manifest-path src-tauri/Cargo.toml
```

Launcher offline build/test environment:

```bash
JAVA_HOME=/tmp/cleanyourtv-build/jdk ANDROID_HOME=/tmp/cleanyourtv-build/android-sdk ANDROID_USER_HOME=/tmp/cleanyourtv-build/android-user GRADLE_USER_HOME=/tmp/cleanyourtv-build/gradle PATH=/tmp/cleanyourtv-build/jdk/bin:$PATH /tmp/cytv-launcher-owner/launcher/gradlew -p /tmp/cytv-launcher-owner/launcher --offline --no-daemon assembleRelease testReleaseUnitTest --console=plain
```

Cargo home/toolchain and Gradle daemon local sockets may require sandbox escalation. This is unrelated to TV authorization. Do not use any TV-oriented scripts to verify these builds. Temporary launcher APKs use the temporary build key and must not be installed over the existing TV build.

## Suggested skills

- `openspec-apply-change` — resume the existing tracked change; only tick fully verified requirements.
- `handoff` — refresh this document when handing off again; user explicitly wants it in the repo.
- `browser-qa` or `e2e-testing` — offline mocked UI, keyboard, narrow layout and reduced-motion acceptance.
- `tauri-v2` / `tauri-security` — native picker/event/IPC review as needed.
- `openspec-archive-change` — only once all required implementation and acceptance work is actually complete.

## Checkpoint results

- Rust: 34 passed, 0 failed; one explicitly invoked subprocess helper marked ignored.
- JavaScript unit tests: 21 passed.
- Svelte check: 0 errors/warnings; production Vite build passed; existing mocked Playwright suite: 13 passed.
- Launcher checkpoint `6534c90`: offline `assembleRelease testReleaseUnitTest` passed (latest integrated scheduler code included, capability disabled). Physical behavior remains unverified.
- Physical Windows/macOS/Linux apply/restore, TV wake/offline rotation, actual crop screenshot and novice walkthrough remain unverified. Do not tick those tasks or describe this as a release-ready feature.
