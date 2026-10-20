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
state/validation tests passed. Three headless Playwright interaction/layout/reduced-motion tests passed;
desktop and narrow screenshots reviewed. Native CI builds are in progress; physical platform launch
checks are separate from browser verification.
No TV credentials or private backups are imported into this repository. The API keys used on the TCL
remain in the original local environment/TV configuration and are unrelated to companion runtime.
