# Scope review and verification

Reviewed 2026-10-20. Blueprint adversarial review completed by a separate reviewer.

Findings resolved in PRODUCT, ARCHITECTURE, TESTING and the milestone amendments:
- Pairing creates persistent debug trust; inspection does not change TV package/HOME/settings.
- Settings restoration must detect later changes with compare-and-set conflict handling.
- APK/data rollback is not generally possible and must not be promised.
- Ambiguous identity (including identical TVs and reused IP) blocks stale plans.
- Cross-process locks do not prevent external ADB changes: recheck each operation.
- Full/writable backup storage, journal retention and external navigation require explicit policy.
- This iteration is an inspection/demo preview; setup/restore mutation milestones are deferred.
- Native CI artifact generation and physical OS launch testing are separate acceptance checks.

Local verification: Svelte check has zero errors/warnings; production frontend build and four
state/validation tests passed. Four headless Playwright interaction/layout/reduced-motion/desktop-IPC-contract tests passed;
desktop and narrow screenshots reviewed. Rust unit tests and native installer builds passed on Linux, Windows and macOS. The initial Linux
AppImage was launched and its actual Tauri window was captured successfully. The user launched the
initial Windows installer and reported the pairing-flow issue addressed below. Physical platform launch
checks are separate from browser verification.
No TV credentials or private backups are imported into this repository. The API keys used on the TCL
remain in the original local environment/TV configuration and are unrelated to companion runtime.

## Windows pairing follow-up · 0.1.1

User observed the TV accepted pairing but the desktop stayed on the connection screen. A confirmed
UX defect: success text was below the fold and no connection/discovery followed pairing. The fix
shows feedback at the top, displays a paired checkpoint, discovers the same IPv4 host, and connects
automatically only when exactly one connection endpoint matches. Other TVs and pairing ports are
excluded. Missing/ambiguous discovery leaves a focused manual connection field.

ADB daemon startup now uses null output handles before captured client commands, to avoid Windows
server handle inheritance prolonging output reads. Actual Windows retest remains required to establish
whether this was also involved in the reported symptom. All command waits remain bounded.

Verification for 0.1.1 at commit 863915c: Svelte check passed with no warnings, five state/validation
tests passed, six Playwright interaction tests passed, and Rust tests plus native packaging passed
on Windows, Linux and macOS in workflow run 39388773164. Updated Windows device-flow retest remains
pending user verification. A first-release Windows/macOS signing gate is still open.

## README, presets and Windows upgrades · 0.1.2

The README now has a shorter user guide and five current screenshots, generated from explicitly
labeled demo data with `pnpm screenshots`. Local links, screenshot paths and upstream source commit
were verified. The comparison catalog covers shared Android packages plus five brand families, with
model/firmware provenance and trade-offs. Missing observations remain unknown, earlier disabled apps
are separate, HOME-dependent entries are conditional, and a T-Cast/Alexa bundle requires a feature choice.

Local Svelte checks have zero errors/warnings, 12 state/catalog tests and 7 UI tests passed, and the
frontend production build passed. Workflow 39392267939 compiled native packages successfully for all
three platforms, including the NSIS hooks. No automated Windows upgrade test was added, as requested.
Actual DLL-lock upgrade behavior is for the owner to verify using the steps in TESTING.md.

The installer uses path-scoped Windows Restart Manager resources before interactive legacy removal,
installation/update and uninstallation. It does not run blanket ADB process-name kills or remove keys.

## Automatic discovery implementation · in progress

The OpenSpec change automatic-tv-discovery-and-wallpaper-personalization currently adds explicit
connection states, cancellation/stale-result guards, startup discovery, remembered-TV identity checks,
and connection feedback. pnpm check, pnpm test, pnpm build and 13 Playwright tests pass locally.
These browser checks exercise mocked native IPC, not a physical TV.

After the host development dependencies were installed, all six Rust tests passed and
`pnpm tauri build --debug --no-bundle` succeeded. A regression test covers one physical TV exposed
through both its IP endpoint and ADB mDNS alias; discovery canonicalizes those aliases before identity
matching. Missing/duplicate identities and unreadable candidates still require selection.

Native Linux verification used the actual Tauri/GTK window and an isolated XDG app profile under
/tmp. On first launch, automatic discovery found the existing TV; selecting Check TV inspected it
without changing its configuration and saved a five-field identity preference with mode 0600.
Closing and reopening the build reached the live dashboard automatically, without selecting a device
or re-pairing. The actual TV remained on its existing launcher with its two disabled packages intact.
This verifies the documented fresh-profile/returning flow on Linux with an already authorized TV;
it does not establish new-TV pairing, Windows/macOS behavior, or wallpaper-write correctness.
The companion still exposes no wallpaper Apply controls or TV scheduling controls.

## Launcher protocol candidate

The isolated owner-runtime branch delivers a version 1 provider and private transaction store.
69 launcher tests pass, including seven new transaction tests. The matching-signature local release
2.5-companion.2 was installed and launched successfully. A wrong initial source base omitted the
owner's existing Unsplash customization; it was corrected to the actual installed runtime source
before completing compatibility verification. The delivered candidate includes those existing
wallpaper and remote-key additions. Theme bytes, HOME selection and disabled packages are unchanged.

Device smoke tests verified capabilities/safe state, unsupported protocol rejection, private-setting
rejection, a truncated upload followed by a complete verified PNG, stale activation rejection and
idempotent discard. Staged media was never activated. A temporary Android test application was denied
both provider calls and asset streams and was removed afterward. These are actual Android Binder/pipe
checks, separate from browser IPC fixtures. The local source patch and nonsecret release evidence are
under integrations/. No signing key, private theme, provider token or TV backup is included there.

## Native review journal

18 Rust tests pass after adding typed protocol transport, collection-manifest validation, local plans,
private journal writes, exact override baselines and device locks. One ignored harness helper is run
explicitly in subprocesses by the cross-process lock test; it is not a skipped behavior check.
Interrupted journal replacement leaves either the previous complete record or the new complete
record. A plan for another identity, an expired review or a changed TV revision is rejected before
writing. Restoration refuses later changes to owned values and preserves unrelated settings.

## 2026-10-20 source checkpoint (offline only)

The user paused implementation and requested conventional commits/pushes plus [HANDOFF.md](../HANDOFF.md).
After the user prohibited live testing, no TV checks or installations were performed; the owned native
test window was terminated. Latest source has not been installed on the TV.

Checkpoint validation: Svelte check 0 errors/warnings; 21 JavaScript tests; 34 Rust tests passed
(one ignored subprocess helper is explicitly exercised by its parent); 13 existing mocked Playwright
tests; Vite production build; launcher offline `assembleRelease testReleaseUnitTest`; strict OpenSpec
validation. Editor-specific browser coverage and physical acceptance remain pending. Launcher scheduling
is committed as work in progress with `theme_schedules_v1` absent from capabilities. See the handoff
for incomplete integration and UI review findings before enabling or releasing write features.
