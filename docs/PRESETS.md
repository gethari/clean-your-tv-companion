# TV presets

Presets are comparison lists. Selecting one never executes commands or changes the TV. The preview
shows exact package IDs against a fresh inspection and explains each package's reported purpose and
trade-offs. Nothing is selected for disabling, and there is no bulk-disable action.

The catalog is in `src/presets.mjs`. It contains shared Android packages and a small set of brand-specific
entries. An unfamiliar manufacturer falls back to the shared list; a user can choose their TV family.
For example, a Philips TV may report MOKA as its manufacturer, so we do not infer Philips from that name.
A family suggestion is a convenience, not a firmware compatibility match.

## Source reports

Entries are adapted from the MIT-licensed upstream reports at commit
`e14d13ad096929ef033d3afd4e6c3fabca2953e8`, reviewed on 20 October 2026.
These reports describe individual devices and include owners' optional choices. The companion does
not import their whole disable lists or treat them as instructions.

| List | Source model and firmware | Report |
| --- | --- | --- |
| Shared Android packages | Chromecast with Google TV 4K, Android 14, UTTC.250917.004 | [Report](https://github.com/Gathin23/cleanyourtv/blob/e14d13ad096929ef033d3afd4e6c3fabca2953e8/examples/google-chromecast-gtv-4k/debloat.txt) |
| TCL | QM6K Pro G08, Android 14, AU16 | [Report](https://github.com/Gathin23/cleanyourtv/blob/e14d13ad096929ef033d3afd4e6c3fabca2953e8/examples/tcl-qm6k-google-tv/debloat.txt) |
| Xiaomi / Redmi | Mi TV 4A MOOQ1, Android 10, QTG3.200305.006.2591 | [Report](https://github.com/Gathin23/cleanyourtv/blob/e14d13ad096929ef033d3afd4e6c3fabca2953e8/examples/xiaomi-mi-tv-4a/debloat.txt) |
| Sony | KD-55X85J VH2, Android 12, South American firmware | [Report](https://github.com/Gathin23/cleanyourtv/blob/e14d13ad096929ef033d3afd4e6c3fabca2953e8/examples/sony-kd-55x85j/debloat.txt) |
| Hisense | HiSmartTV A4, Android TV 9, PTMR.190127.037 | [Report](https://github.com/Gathin23/cleanyourtv/blob/e14d13ad096929ef033d3afd4e6c3fabca2953e8/examples/hisense-hismarttv-a4/debloat.txt) |
| Philips | R8G whitefield, Google TV 14, UKRE.260524.058 | [Report](https://github.com/Gathin23/cleanyourtv/blob/e14d13ad096929ef033d3afd4e6c3fabca2953e8/examples/philips-r8g-google-tv/debloat.txt) |

## Interpretation

- **Already disabled:** observed in the disabled-package query. We do not know who disabled it.
- **Available to review:** observed installed and enabled. The owner still decides whether they use it.
- **Not installed:** absent from a successful installed-package query.
- **Not checked / State unknown:** an inspection source is missing or failed.
- **Check home screen first:** a home-dependent package needs a verified replacement before consideration.
- **Kept / Check compatibility:** protected functionality, the active HOME package, or unconfirmed TV support.

The preview's home-dependent check recognizes the Clean Your TV HOME component. This is sufficient
for organizing a review, not proof that a launcher handover or physical remote test has succeeded.
Execution remains a later milestone with its own capability checks, baseline, review, journal and restore.

Firmware updates, screen readers, remote/keyboard services, casting, Play services and live-TV/input
services are excluded from the candidate catalog. Even if an upstream list disables them by owner choice,
the companion does not turn that choice into a default. A package can bundle several features: the TCL
`com.tcl.ttvs` report includes T-Cast and Alexa alongside viewing-data services, so its description says so and its state stays conditional pending a separate feature choice.

## Adding a model

Add evidence for the exact model, Android version, firmware, region, package IDs and physical features
tested. Separate diagnostics, viewing recognition, optional apps and home dependencies. Include what is
lost by disabling each entry. Extend parser/state fixtures and inspect the result on the actual model.
Do not label a new model supported based on a brand or board-name match alone. Unknown packages stay kept.
