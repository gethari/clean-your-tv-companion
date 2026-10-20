# Clean Your TV · Companion

A local desktop companion for Android TV and Google TV. No AI, account, subscription, or cloud backend.

**Status: scoped project + runnable inspection preview.** The preview discovers ADB services, connects/pairs,
reads the TV's current state, and distinguishes configured, available, unknown, and blocked features.
It deliberately does not install APKs, disable packages, change Home, or overwrite an existing setup.
Those operations have a concrete implementation plan and acceptance gates, not pretend working buttons.

Built with **Tauri 2 + Rust + Svelte 5 + TypeScript**. System webview, no Electron.

![Companion overview using clearly labeled example TV data](docs/preview.png)

- [Product scope](docs/PRODUCT.md): beginner flows, feature boundaries, completion states.
- [Caveat register](docs/CAVEATS.md): evidence from the upstream scripts and this TCL setup.
- [Architecture](docs/ARCHITECTURE.md): device identity, typed operations, recovery, secrets.
- [Construction plan](plans/companion.md): six independently verifiable milestones.
- [Design](docs/DESIGN.md): visual language, motion, accessibility and UX copy.

## Try it

Download the [0.1.1 inspection preview](https://github.com/gethari/clean-your-tv-companion/releases/tag/v0.1.1-preview.1) for the pairing-flow fix. Existing authorization can be reused; use the current connection port from the TV.

Download the latest successful **Desktop preview** workflow artifact from Actions: Windows provides an
NSIS setup `.exe`, Linux an AppImage and `.deb`, macOS a `.dmg`. These are unsigned testing builds.
A GitHub login may be needed to download CI artifacts; the installed app has no login requirement.
See [testing instructions](docs/TESTING.md). Artifacts are available only after the workflow passes.

Development: Node 24 and pnpm 12, Rust stable, and the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

```sh
pnpm install --frozen-lockfile
pnpm check
pnpm test
pnpm dev          # browser design preview; no real device commands
pnpm tauri dev    # desktop app; use bundled ADB or CYTV_ADB_PATH during development
```

`pnpm dev` always displays a visible **Demo** indicator and never touches a TV. Choose a scenario
in the app to inspect a fresh TV, an existing TCL setup, or an interrupted connection.
Desktop inspection requires enabling debugging and approving your computer on the TV. Pairing and
connection ports can differ and change after sleep or reboot. The app never silently enables plaintext
ADB or scans unrelated networks.

## Development rules

Use conventional commits (`docs(scope): …`, `feat(discovery): …`, `fix(audit): …`). Never commit TV dumps,
credentials, ADB private keys, personal IP addresses, downloaded binaries or signing keys. Keep inspection
read-only. Unknown observations are not evidence that a feature is absent. See [CONTRIBUTING.md](CONTRIBUTING.md).

Inspired by [Gathin23/cleanyourtv](https://github.com/Gathin23/cleanyourtv), MIT. This is a separate project;
it does not claim that every brand/model has been validated. Samsung Tizen, LG webOS and Fire TV are
outside the first release. The existing launcher remains a separate upstream project.
