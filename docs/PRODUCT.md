# Product scope · companion MVP

## Promise

A person with a laptop and TV remote can understand what is installed, make a reviewed set of reversible
changes, and come back later to maintain or restore them. Runtime needs neither AI nor a paid service.
The default path must not require a terminal, Python, Java, Node, Rust, Android Studio or a subscription.
Developer tooling is needed only by contributors. Online wallpapers and app downloads need Internet;
local inspection, installed apps, offline wallpapers and restore do not need our servers.

## Audience and platform boundary

Windows 10/11 x64, modern macOS on Apple silicon/Intel, and supported Linux distributions. Exact minimum
OS versions follow the pinned Tauri/WebView toolchain and are tested before release. Bundle official
platform-tools with licenses. Windows WebView2 bootstrap/install detection must explain offline failure.
Linux AppImage is not a promise of zero host runtime dependencies. Publish a tested distro list.

Android TV/Google TV with authorized ADB are the initial targets. Brand is descriptive metadata, not
permission to apply a package list. Classify each feature as supported, conditional, unknown or blocked
by detected firmware capabilities. Tizen/webOS: explain incompatibility and exit without scanning harder.
Fire TV: future adapter only, because launcher restrictions and firmware behavior differ.

## End-to-end flow

1. Welcome: what you can do, what stays local, Try a demo, Find my TV.
2. Find: automatically discover already connected devices and mDNS ADB connect/pair advertisements on
   native startup. A discovery event is not proof of a TV. Require selection for new or ambiguous TVs.
   Reconnect only one previously selected TV after authorized identity verification; missing identity,
   unreadable candidates or duplicate matches require selection. Browser demo never discovers devices. Manual IP + port is always
   present; explain guest Wi-Fi, Ethernet versus Wi-Fi, VPN and local-network permission failures.
3. Authorize: illustrated remote instructions for developer mode. Explicit pairing and connection fields;
   never ask users to paste shell commands. Show the TV prompt and a waiting state. Modern TLS debugging
   preferred. Legacy port 5555 is an explicit optional path with a close-debugging finish step.
4. Inspect: bounded, read-only queries; identify Android TV feature, manufacturer, model, firmware, user,
   current HOME, launcher version, enabled/disabled packages and safe theme settings. Mark each source's
   timestamp and confidence. Query failure means unknown. Do not claim API credentials were verified.
5. Overview: existing setup is a success state. Show Done, Optional, Needs attention and Couldn't check.
   Clean launcher already active? Do not reinstall it. An installed APK can still be outdated, blocked,
   not selected as HOME, or signed incompatibly. A package already disabled gets no disable operation.
6. Personalize: dock order from installed apps; theme, columns, motion, local wallpaper folder, dimming;
   optional online photo provider after terms/credential review. Preserve unknown theme keys and user
   settings. Never flatten an existing custom setup into a starter preset.
7. Review: exact changes, reasons, benefits/tradeoffs, unavailable actions, baseline backup location,
   and a clear Apply button. Unselected optional changes are no-ops. No numeric health score that rewards
   disabling more packages. Critical functionality and updates are kept by default.
8. Apply: durable per-device journal, one verified operation at a time, observed progress rather than a
   fake countdown; pause for physical remote tests. Cancellation occurs at a safe operation boundary.
9. Finish: show verified changes, things skipped and things needing user confirmation. A friendly TV
   character rests when complete. No upgrade pitch, upsell or unnecessary task to reach 100%.
10. Return/restore: refresh before suggesting anything. Reconcile new observations with the journal;
    mark drift. Restore only this app's changes to recorded prior values, preserving pre-existing
    disabled packages and settings. On a different laptop without a baseline, explain that earlier
    history is unknown; offer current-state export, never a fictional original state.

## Release features

| Feature | Inspection preview | First modifying release |
|---|---|---|
| Demo scenarios and friendly state-aware dashboard | Yes | Yes |
| Devices, mDNS discovery, manual connect and pairing | Yes | Yes, recovery refinement |
| Current HOME, packages, launcher/theme observations | Yes | Yes, per-user enrichment |
| Conservative package profiles | Six sourced family comparison lists; no changes | Reviewed, versioned capability rules |
| APK install and launcher handover | Planned, no action control | Verified source/hash/signature + rollback |
| App preferences / dock / local wallpapers | Inspect safe values | Merge patch and preview |
| Backup / resume / restore | Specification | Durable, identity-bound journal |
| Unsplash / upstream providers | Credential and fetch health unknown without safe readback | Provider selection, Access Key and collection IDs via verified private bridge |
| Private DNS, screensaver, remote helpers | Explain only | Separate explicit choices and physical checks |
| Export sanitized support report | Planned | Opt-in, user preview, secrets redacted |
| Automatic updates / shared remote catalogs | No | Later; signed and opt-in |

## Definition of done for existing TCL

An inspection of our already configured TV shows current HOME and launcher version, counts only packages
actually observed disabled, and preserves the existing dock and theme. Wallpaper enabled/rotation/dim
are read from config; inability to read private credentials is shown as unknown, not setup missing.
The preview does not nudge a reinstall or disabling further packages. Reconnecting after a changed port
refreshes device state. Demo data never masquerades as live results. First modifying release additionally
shows a verified empty change plan when the chosen target already matches the observed state.

## Non-goals

Universal safe debloating inferred from package names; root; bypassing firmware installation restrictions;
paid streaming credentials; automatic VPN/content add-ons; firmware modification; TV factory reset;
cloud remote control; promising ad blocking within streaming video; replacing every vendor's OS.

## Success measures

Complete setup with remote + laptop and zero terminal instructions. Reopening produces no duplicate
mutations. Injected interruption after every mutation can resume/reconcile without repeating an unknown
operation. No lost pre-existing customizations. Every failure provides a useful next step. Real usability
sessions with novice testers on Windows/Linux precede a public stable release.

## Active MVP and delivery boundaries

The active implementation queue is `openspec/changes/companion-full-setup-mvp`. Its flow is discover,
pair, inspect, install/select a compatible pinned upstream launcher, configure local photos or supported
upstream providers, review optional validated TCL cleanup, and maintain/restore later. The predecessor
personalization change preserves implementation history; its unfinished schedules/provider-adapter tasks
are not the MVP execution queue. Existing source is a checkpoint, not a completed modifying release.

The MVP includes upstream provider switching (local/Unsplash/automatic where supported), a masked
Unsplash Access Key, multiple collection IDs, keeping/replacing/removing a saved key, distinct validation
and retrieval status, and reviewed writes. Configure the launcher's existing providers; do not create
another retrieval service. Credentials must use a verified private bridge and remain absent from ordinary
journals, logs and exports. A provider merge alone does not prove that this bridge or a signed downloadable
APK exists. Missing external integration is reported as a blocked feature, not a fake connected source.
Local photo import requires no account, provider credential or network; an unavailable Unsplash or Reddit
source never disables local collections and never replaces a usable background.

Windows 11 x64 and Ubuntu 24.04 x64 require packaged hands-on acceptance. macOS is experimental until
separately verified. Cleanup requires approved exact model/firmware evidence for TCL; other TVs retain
capability-supported setup and inspection. All cleanup choices start off and essential functions stay.
Schedules, Google Photos, new providers, mobile, DNS, screensavers and remote remapping are deferred as
MVP delivery. A local, preview-only schedule editor retained from the predecessor change validates rules
and shows upcoming transitions but advertises no launcher capability and writes nothing to a TV.

No live TV testing or native companion launches are permitted until the user explicitly lifts the
current restriction. Automated/mock checks do not satisfy physical acceptance. Launcher runtime work
is a separately scoped external dependency; implement this change in the companion repository only.

A baseline is not a full APK/application-data/firmware backup. Restore settings and enablement only
while detecting conflicts with later changes. Earlier APK versions/data are not promised recoverable.
Installation review must state this limit. Public signing may cost the maintainer money; using the
installed companion has no account, AI or subscription requirement.
