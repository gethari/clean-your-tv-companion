# Design

## Context

See proposal.md. The predecessor change has 14/29 checked tasks and supplies connection, local media and a personalization-only journal. Its editor still lacks focused browser coverage and durable operation-history UI. The six preset families are comparisons, not approved execution profiles. Existing launcher patches are historical experiments and are not the MVP integration baseline.

## Goals / Non-Goals

**Goals:** Complete vertical setup flows using existing Rust/Tauri/Svelte boundaries; keep each operation verifiable and recoverable. Windows 11 x64 and Ubuntu 24.04 x64 are first physical acceptance targets; macOS remains experimental.

**Non-Goals:** Launcher runtime development, generic ADB command UI, a second wallpaper-provider service, scheduling, new provider implementations or broad cleanup support.

## Decisions

### Upstream evidence and capabilities gate

Resolve the latest provider-capable upstream source/release and pin verified artifacts, signing metadata and capabilities. Store a sanitized evidence inventory here. Provider playback and secure companion configuration IPC are different capabilities; check both. A source merge is not evidence of a published APK or private credential bridge. Missing interfaces/artifacts block the dependent acceptance tasks, not independent connection/editor fixes. Do not use the owner's unpublished custom runtime as a substitute or modify either launcher checkout.

### Reuse narrow native boundaries

Keep discovery and typed observation in Rust; refresh identity before mutations. Svelte orchestrates explicit reviewed operations. No generic path/shell IPC. Extend typed operations to installation/HOME/package state through a versioned setup journal, keeping existing personalization records readable. Reuse OS per-device locks, atomic writes, expected-state checks and exact inverse patches. Expose history summaries, not raw journals, scoped to the currently verified TV. Do not include private source metadata in exports.

### Provider configuration

Use a source contract driven by observed capabilities and safe status. Support upstream local/Unsplash/automatic semantics; unknown registered sources can be shown as unsupported without invented configuration controls. Normalize collection IDs using the upstream contract and report duplicate/invalid IDs before review. Existing keys remain private on the TV; replacing/removing keys is explicit. Never persist key values in ordinary journals or pass them in shell arguments. A verified TV-private configuration transaction must own credential activation and rollback; desktop state contains only safe status and an opaque reference. No shared-storage secret staging fallback. Network validation is explicit, bounded and optional when connectivity is unavailable; saving configuration and fetching a photo are separate outcomes.

### Editor and review correctness

Correct native rejection-code copy, baseline path disclosure, failed crop rollback and stored-photo preview labels. Enforce 200 assets/256 MiB over the entire draft and 20 MiB/33,177,600 pixels per image. Invalidate review on any changed draft; ignore stale asynchronous crop/IPC results after navigation. Add actual transfer phases and recovery actions without artificial timers or claimed success on unverified results.

### Installation and cleanup

Trusted artifact metadata precedes download/install; verify source/hash/signature/minSDK/ABI and re-inspect installed state. HOME handover requires observed verification and keeps stock HOME enabled. APK/data rollback is not promised. Cleanup profiles are exact tested model/firmware rules, all choices initially off; no profile is approved by copying community lists. Preserve casting/voice/update/input/network/accessibility dependencies and baseline disabled states. Missing validated profile means inspection-only cleanup.

## Risks / Trade-offs

- Missing upstream private configuration IPC or published artifact → record explicit dependency evidence and block its adapter/release acceptance; never present a fake working connection.
- Partial legacy observations → retain unknown state; no planning writes on guessed defaults.
- Older journals/corrupt records → retain existing format, validate before use, report failures without hiding valid records; never silently reset history.
- No live TV authorization → all physical checks remain pending until user lifts the restriction; mocked tests do not replace them.

## Migration Plan

1. Record agreed MVP scope and upstream capability evidence; stop executing predecessor deferred tasks.
2. Fix existing editor/review defects and add offline UI coverage.
3. Surface durable scoped history and reconcile/restore flows; extend setup journal and typed executor with fake-device tests.
4. Integrate upstream provider configuration and trusted launcher installation only when their external gates are satisfied.
5. Add validated TCL cleanup; test packaged Windows/Linux flows and record physical evidence before declaring MVP complete.
6. Commit conventionally, refresh HANDOFF, and preserve journals/private state. No live TV commands or native app launches during the current restriction.
