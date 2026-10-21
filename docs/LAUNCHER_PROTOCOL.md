# Companion launcher protocol · version 1

Status: implementation contract. Existing launcher 2.3/2.4 builds do not implement this protocol.
See ARCHITECTURE.md for source ownership and pinned legacy release evidence. The companion must
require capabilities rather than inferring support from versionName. No wallpaper writes are enabled
by this document alone.

## Transport and caller boundary

Authority: `com.gathin.tvhome.companion`; base URI: `content://com.gathin.tvhome.companion/v1`.
Rust uses fixed `adb -s <validated-device> shell content call` operations. The Android provider accepts
only the authorized Android shell UID (2000) or its own process UID, checked with Binder.getCallingUid
at every entry point before accessing files or clearing caller identity. Other apps cannot call it.
ADB authorization is the trust boundary; there is no network listener or cloud service.

`call(method, arg, extras)` accepts a base64url UTF-8 JSON request in `arg` and returns a Bundle with
one base64url JSON `reply` string. Payloads have no padding and contain only ASCII letters, digits,
`-` and `_`; no command interpolation of JSON, filenames, credentials or user text. Envelope size is
limited to 256 KiB decoded; media transfers use the separate bounded stream below. Unknown methods,
protocol versions, unsafe identifiers and malformed envelopes fail before mutation. Every reply has
`protocol_version: 1`, `request_id` (32 lowercase hex characters), `ok`, and either `data` or
`error: {code, message}`. Errors never echo secrets or source paths.

ContentProvider and Binder references:
https://developer.android.com/reference/android/content/ContentProvider
https://developer.android.com/reference/android/os/Binder#getCallingUid()

## Capabilities and safe state

`capabilities` takes `{protocol_version:1, request_id}` and returns:

```json
{"protocol_version":1,"request_id":"0123456789abcdef0123456789abcdef","ok":true,"data":{"launcher_version":"2.5-companion","features":["collections_v1","atomic_activation_v1","revision_checks_v1","readback_v1"],"limits":{"asset_bytes":20971520,"collection_bytes":268435456,"asset_count":200}}}
```

These four features are mandatory for any personalization. `theme_schedules_v1` and
`private_credentials_v1` are separate optional features; absent features disable their controls.
An absent provider, version other than 1, malformed limits or missing mandatory feature is read-only.
Do not fall back to writing external config.json.

`state` returns `{revision, settings, overrides, collections, active_collection, operation}`.
`overrides` contains companion-owned values; `settings` overlays them on the existing theme. Removing
a key removes only its companion override, revealing the existing theme value. The companion journals
override presence/value separately from effective settings so restore can recover exact prior absence.
The revision also covers legacy theme contents and TV-side preference values, so later manual changes
produce a conflict even when an override currently masks them. `revision` is a
64-character lowercase SHA-256 digest covering all persisted personalization state, including opaque
unknown settings. `settings` contains only safe owned settings; credentials and unknown settings are
excluded from readback. Unknown settings stay on the TV and contribute to revision checks. `operation`
contains last activated request_id and result revision so a disconnected client can reconcile.
`collections` exposes opaque collection IDs, hashes, byte sizes and playback preferences, never
absolute paths. No secrets appear in capabilities, state, errors, logs or exports.

## Typed patches

A settings patch is `{set:{...},remove:[...]}`. Allowed keys are `wallpaper`,
`wallpaper_rotation_minutes`, `wallpaper_dim`, `wallpaper_order`, `active_collection`, `theme_schedule`,
`background_color`, `accent_color`, `reduced_motion`, and `theme_schedule_enabled`. Set and remove must not overlap. Changes
merge with persisted configuration, preserving every unrelated/unknown key and its original value.
Removal restores absence rather than a guessed default. Rotation is an integer from 1 through 1440
minutes; dimming is an integer from 0 through 100; order is `sequential` or `shuffle`. Colors use
`#RRGGBB`. IDs contain 32 lowercase hex characters. Theme schedules use a separately validated schema
and are accepted only when `theme_schedules_v1` is advertised. Credentials never enter this patch.

## Staging and streaming

`stage` takes `{protocol_version,request_id,expected_revision,patch,collections:[{id,assets:[{sha256,bytes,mime}],order,interval_minutes}]}`.
It checks revision, patch shape, feature support, unique IDs, safe hashes, MIME types (JPEG/PNG/WebP),
individual/aggregate limits, available storage and rollback retention before making a durable session.
The session ID is the request_id. Exact retries return the same session; reusing an ID with a different
payload returns `request_conflict`. Staging does not change active config or the displayed collection.

Assets are written using fixed `content write --uri` paths:
`content://com.gathin.tvhome.companion/v1/sessions/<session-id>/assets/<sha256>`.
Rust streams bytes on stdin. The provider returns a pipe, never an unrestricted private-file descriptor,
and validates bytes against the session manifest, declared length, magic/decoded format, dimensions
and SHA-256. Reject traversal, symlinks, duplicate concurrent writers and unlisted hashes. An incomplete
or oversized stream is discarded; EOF alone does not acknowledge validity. Only verified complete
assets become staged objects. Already verified hashes can be reused without retransmission.

`validate` takes `{protocol_version,request_id,session_id}` and returns `{ready,missing_assets,revision}`.
All assets must be verified, limits/storage rechecked and the expected revision still current. A stale
revision returns `revision_conflict`; the companion must refresh/review rather than overwrite.

## Activation, readback and recovery

`activate` takes `{protocol_version,request_id,session_id,expected_revision}`. Under one launcher-side
mutation lock, recheck revision, fsync durable session/asset metadata, and atomically replace a single
active-state pointer/config revision. The persisted state includes operation identity so activation
and its acknowledgment cannot disagree after restart. Only then refresh Home and return
`{revision,operation_id}`. Zero exit status is not proof: the companion queries `state` and verifies
operation ID, settings and referenced hashes. Repeating activation of the same session is idempotent.

`result` takes `{protocol_version,request_id,operation_id}` and returns `activated` with session/revision,
or `unknown` when no committed operation is recorded. Stage and validation status use their own methods. Query after disconnect; never blindly repeat an operation
whose outcome is unknown. A crash before atomic replacement keeps the previous active state; after
replacement it recovers the new state and operation ID. No partially activated collection is displayed.

`discard` takes `{protocol_version,request_id,session_id}` and idempotently deletes only unactivated
staging. It must not remove active or rollback-referenced media. Explicit safe-boundary cancellation
uses discard after reconciliation. The current implementation retains staged records and immutable assets; automatic expiry and garbage
collection are not implemented. Storage preflight may therefore require a smaller collection. It never
deletes active or rollback media to make room.

Restoration is another reviewed stage/activate transaction, using the companion journal's exact prior
values and a fresh expected_revision. It never resets all settings or overwrites later manual changes.
The TV retains previous referenced assets until the companion explicitly releases its rollback manifest;
insufficient space to retain them blocks activation. APK/data downgrades are outside the protocol.

## Capability fallback verification

`tests/fixtures/launcher-protocol/capabilities.json` covers a supported launcher, the legacy 2.4 shape,
a newer protocol, a missing mandatory feature and invalid storage limits. The Rust fixture test
`launcher_protocol::tests::fixture_capabilities_fail_closed_for_legacy_newer_and_incomplete_versions`
confirms that every unsupported shape is read-only. The browser test
`tests/ui/personalization.spec.ts` (“unsupported launcher remains read-only…”) confirms the UI shows the
inspection reason and exposes no Apply or Restore action. A compatible launcher must advertise protocol
version 1 with all four mandatory personalization features; version names, provider presence or partial
features never enable editing. The exact baseline path is shown in each review, and restore limits are
listed below.

## Error codes and verification

Stable error codes: `unsupported_protocol`, `unsupported_feature`, `invalid_request`, `access_denied`,
`revision_conflict`, `request_conflict`, `missing_assets`, `invalid_asset`, `storage_full`, `busy`,
`unknown_session`, and `internal_error`. Failed calls are distinguishable from empty collections.

Contract fixtures under tests/fixtures/launcher-protocol cover supported, legacy, newer and incomplete
capabilities and merge behavior. The launcher must additionally verify unauthorized Binder callers,
traversal, truncated streams, incorrect hashes, crash boundaries, stale revisions, idempotent activation,
discard safety and private-field exclusion before any physical TV configuration test.

## Desktop journal and restore limits

Plans and exact prior overrides are stored below the operating system app-data directory at
`io.gethari.cleanyourtv.companion/operations/plans/<operation-id>.json` (Linux normally
`~/.local/share/io.gethari.cleanyourtv.companion/operations`). The review displays the actual baseline
path. Unix directories are private (0700) and records are 0600. Do not delete these records if you
need restoration. The cache is under app-data `media`; no original desktop folder is synchronized.

Reviews expire after 15 minutes. A device/firmware/user or revision mismatch requires a new review.
Disconnects during transfer retain the previous active collection; retry sends only missing verified
hashes. Activation intent is saved durably before the call. A missing response is an uncertain outcome;
retry queries committed operation history and safe state before using the same idempotent operation ID.
Success requires matching operation/session/revision, owned overrides and manifests. Manual changes
that prevent verification move the operation to `needs_review` instead of claiming success.

Restore requires a separately reviewed inverse plan. It restores only owned overrides whose current
values still match the verified apply, including exact prior absence, and replaced manifests that have
not subsequently changed. Added immutable assets/manifests can remain on the TV; restore is not a media
purge or an APK/data downgrade. Unknown and unrelated settings are preserved. Native Windows/macOS
crash durability and physical apply/restore acceptance remain unverified; offline unit tests are not a
substitute for those checks.

## MVP transfer regression checks

Offline fake-device tests now inject a disconnect after one verified asset, then confirm that resume
sends only the remaining hash before activation. Storage rejection before staging, cancellation after
partial transfer, an unsolicited requested hash, and a rejected/unverified asset all leave the active
state unchanged. Cache tests independently reject altered bytes, wrong lengths and unsafe identifiers.
These checks validate the companion coordinator; they do not establish support in an upstream launcher
or replace a physical transfer/restore check. See UPSTREAM_SUPPORT.md for that external dependency.
