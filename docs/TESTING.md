# Test the companion

This iteration delivers a scoped repository and a real Tauri inspection/demo preview. Installation,
package disabling, theme writes and restore execution are later milestones. Do not expect an Apply
button in this build. The app can inspect an already configured TV without reinstalling its launcher.

## Linux laptop

Download `companion-linux` from a successful Desktop preview run. Extract it, mark the `.AppImage`
executable using its file properties, and open it when your desktop session is visible. A `.deb` is also
available for supported Debian/Ubuntu/Mint systems. AppImage may need FUSE; its `--appimage-extract`
fallback and `.deb` are alternatives. Native launch must still be tested in your actual desktop session;
a headless frontend screenshot alone does not establish WebKit/GTK behavior.

For contributors: `pnpm install --frozen-lockfile`, `pnpm tauri dev` with Rust + Tauri prerequisites.
Use `CYTV_ADB_PATH=/absolute/path/to/adb pnpm tauri dev` when developing without bundled tools.

## Windows

Download `companion-windows`, extract it and run the `*-setup.exe` NSIS installer. Official ADB and
its required DLLs are packaged. WebView2 must be present or installed by the bootstrapper; offline
machines may need the standalone WebView2 runtime beforehand. These preview installers are unsigned:
Windows may show a publisher warning. Verify the source/release hash; do not turn off antivirus or
system-wide security protections. Test on an actual Windows machine before calling it Windows-verified.

## macOS

Download the runner-produced DMG. Initial CI uses the runner's native architecture, not a universal
binary. A second architecture build is a release milestone. Preview is unsigned/not notarized, so
Gatekeeper can block it. Production signing/notarization remains a release gate.

## Try without a TV

Open Try the demo (desktop), or `pnpm dev` (browser). Switch between Already configured, Fresh TV,
and Connection interrupted. A demo badge must remain visible, unknown credentials stay unknown,
and connection controls must not execute from demo mode. No network image/font downloads are needed.

## Read-only live test

Use Connect a TV → discovery or enter the private IPv4 address and current debugging port. For first
pairing, enter the separate pairing address and six-digit code from the TV. Pairing adds trust on both
ends and ADB can create host keys/start a host server. This grants broad debug access; revoke the
computer in the TV's debugging settings when finished. The preview does not change packages, HOME,
wallpaper settings or launcher's private credentials. No request is made to a wallpaper API.

Validate against our TCL: Clean Your TV home recognized, disabled package names reflect the TV,
wallpaper rotation/dimming read correctly, online photo connection marked unknown, existing theme
unchanged. Keep a terminal-free usability checklist for a novice tester: discovery failure, pairing,
port change, multiple devices, refresh, unsupported device and no additional setup needed.

## Automated checks

`pnpm check`; `pnpm test`; `pnpm build`; `pnpm exec playwright test`;
`cargo test --manifest-path src-tauri/Cargo.toml` after preparing resources/icons. Native CI builds
Linux/Windows/macOS. Browser screenshots cover design behavior, not native IPC/packaging. See REVIEW
for actual test results; planned checks must not be represented as completed.
