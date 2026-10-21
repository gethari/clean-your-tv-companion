# Upstream launcher support inventory

This inventory records the upstream launcher interface that can be verified from source. It distinguishes upstream support from changes in the companion's local integration patch and from a separately inspected fork branch. Checked 2026-10-21.

## Revisions inspected

| Reference | Immutable revision | Finding |
|---|---|---|
| `Gathin23/cleanyourtv` `main` | `9071ec8d2f41826778f6b4c7f5f42dadc22f5b22` (tree `f49d596674bc42ffc568d0369d0cb3453af0c242`) | Current upstream source inspected. Commit message: “Launcher 2.5 (versionCode 6)”. |
| Upstream tag `v2.5.0` | Annotated tag object `49c98c50f8e6d746e373bb14ac10b1bed1489214`, targeting the same `9071ec8d2f41826778f6b4c7f5f42dadc22f5b22` commit | Latest release inspected. Published 2026-10-21. Release API has no binary assets; release notes say to build with `launcher/build-and-install.sh`. |
| `gethari/cleanyourtv` branch `feat/unsplash-wallpaper-provider` | `716d8052cc9b832011ad5104abbf4347fda300a1` | Separate fork candidate, not upstream main. Comparison reports one commit ahead and 322 behind upstream at the inspected refs. |

The upstream release page lists no APK. GitHub's generated source archives are not installable launcher binaries. The release/build configuration does not provide a published signing certificate fingerprint; the Gradle release build points at the debug signing config. Consequently, there is no official APK artifact and signing identity here that the companion can verify for installation or signature-protected IPC.

## What upstream exposes

The upstream launcher supports local wallpaper selection and rotation. `Config.java` reads local wallpaper settings including `wallpaper`, `wallpaper_rotation_minutes`, and `wallpaper_dim`; `WallpaperRotation.java` and `WallpaperPlayer.java` implement local image rotation/display. The 2.5 release notes describe a local wallpaper picker and local theme wallpaper collection rotation. These facilities do not define remote-provider selection or credential configuration.

The exported `ThemeProvider` at authority `com.gathin.tvhome.theme` is guarded by `android.permission.WRITE_SECURE_SETTINGS`. Its implementation provides file-oriented operations over an allowlist of theme files, including config and wallpaper-collection files. It does not implement the companion's versioned capabilities/status protocol, private credential transaction, or a safe credential readback boundary. Its provider name and exported status must not be interpreted as evidence that those APIs exist.

At the inspected upstream SHA, the source tree contains `ThemeProvider.java`, `Config.java`, `WallpaperRotation.java`, and `WallpaperPlayer.java`; it does not contain `UnsplashWallpaper.java`, `WallpaperProvider.java`, or `WallpaperProviders.java`, and a source search finds no Unsplash provider implementation. The release notes also do not claim one.

## Unsplash claim and fork evidence

The Unsplash implementation the user referenced is present in the fork branch `gethari/cleanyourtv:feat/unsplash-wallpaper-provider` at `716d8052cc9b832011ad5104abbf4347fda300a1`. That branch adds provider classes, `tools/unsplash-setup.py`, and tests. It is a fork-only candidate at this inspection point, not evidence that the inspected implementation is available in `Gathin23/cleanyourtv` main or been shipped in its release. No private, app-to-app credential bridge or capability readback is established by the inspected upstream baseline. Provider enablement should remain gated on an approved credential flow and a verified launcher build rather than inferred from fork code or provider names.

## Integration decision

Treat the currently verifiable upstream contract as **local wallpaper configuration and file transfer only**. Keep remote providers unavailable in companion status until a supported launcher revision exposes an explicit, tested capability and safe credential write path. Do not send secrets through `ThemeProvider`'s generic theme-file operations. Do not claim secure credential storage or provider support from the local companion patch: `docs/LAUNCHER_PROTOCOL.md` and `integrations/launcher-protocol.patch` describe a separate owner-runtime integration, not upstream behavior.

Before enabling an upstream integration, require all of the following evidence:

- An upstream immutable commit or release containing the provider capability and private credential transaction protocol.
- A verified release APK and signing fingerprint matching the installed launcher, or an explicitly approved owner-built and signed artifact with its identity recorded.
- Runtime capability readback and negative tests showing that credentials are never returned in public status, logs, or generic theme files.

## Source references

- [Upstream main at inspected SHA](https://github.com/Gathin23/cleanyourtv/tree/9071ec8d2f41826778f6b4c7f5f42dadc22f5b22)
- [Upstream v2.5.0 release](https://github.com/Gathin23/cleanyourtv/releases/tag/v2.5.0)
- [Config.java](https://github.com/Gathin23/cleanyourtv/blob/9071ec8d2f41826778f6b4c7f5f42dadc22f5b22/launcher/app/src/main/java/com/gathin/tvhome/Config.java)
- [ThemeProvider.java](https://github.com/Gathin23/cleanyourtv/blob/9071ec8d2f41826778f6b4c7f5f42dadc22f5b22/launcher/app/src/main/java/com/gathin/tvhome/ThemeProvider.java)
- [AndroidManifest.xml](https://github.com/Gathin23/cleanyourtv/blob/9071ec8d2f41826778f6b4c7f5f42dadc22f5b22/launcher/app/src/main/AndroidManifest.xml)
- [Launcher build.gradle](https://github.com/Gathin23/cleanyourtv/blob/9071ec8d2f41826778f6b4c7f5f42dadc22f5b22/launcher/app/build.gradle)
- [WallpaperRotation.java](https://github.com/Gathin23/cleanyourtv/blob/9071ec8d2f41826778f6b4c7f5f42dadc22f5b22/launcher/app/src/main/java/com/gathin/tvhome/WallpaperRotation.java)
- [Fork branch with Unsplash candidate](https://github.com/gethari/cleanyourtv/tree/716d8052cc9b832011ad5104abbf4347fda300a1)
- [Fork UnsplashWallpaper.java](https://github.com/gethari/cleanyourtv/blob/716d8052cc9b832011ad5104abbf4347fda300a1/launcher/app/src/main/java/com/gathin/tvhome/UnsplashWallpaper.java)
- [Fork provider registry](https://github.com/gethari/cleanyourtv/blob/716d8052cc9b832011ad5104abbf4347fda300a1/launcher/app/src/main/java/com/gathin/tvhome/WallpaperProviders.java)
