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
