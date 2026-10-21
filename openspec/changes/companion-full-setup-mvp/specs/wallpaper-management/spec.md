# Spec Delta

## Purpose

Manage launcher backgrounds from local photos and supported upstream providers through reviewed companion configuration.

## ADDED Requirements

### Requirement: Import and preview local collections
The companion SHALL accept native file/folder selection and drop, preserve accepted images from mixed imports, deduplicate, enforce aggregate limits and show per-file rejection reasons. Users SHALL edit crop, dimming and collection membership before review. Photos MUST appear as launcher backgrounds only.

#### Scenario: Crop fails
- **WHEN** a requested crop cannot be prepared
- **THEN** the previous verified crop and its controls remain consistent, with a useful retry message

#### Scenario: Mixed imports exceed collection quota
- **WHEN** multiple imports would exceed the draft collection limits
- **THEN** accepted photos remain available and excess items are rejected with their reasons

### Requirement: Configure independent playback
Users SHALL select supported sequential/shuffle order and 5/15/30/custom positive whole-minute intervals. The TV SHALL persist playback and continue without the desktop. Unsupported source controls SHALL explain their limitation.

#### Scenario: One image and laptop closed
- **WHEN** a verified single-image collection is active and the companion closes
- **THEN** the TV retains that image without empty transitions

### Requirement: Configure existing upstream wallpaper providers
The app SHALL show the current source and supported choices, including local, Unsplash and automatic selection when supported upstream. It SHALL support an Unsplash Access Key and multiple collection IDs, keeping an existing key, replacement and explicit removal. It MUST NOT require the Secret Key or expose stored keys in readback.

#### Scenario: Existing Unsplash key
- **WHEN** the TV reports a saved credential and the user edits collection IDs only
- **THEN** the app preserves that credential without asking for it again

### Requirement: Distinguish provider validation and runtime results
Provider setup SHALL distinguish invalid credentials, inaccessible or empty collections, rate limits, offline state, persisted configuration and verified image retrieval. Failure MUST preserve the usable background. Switching to local photos MUST preserve provider configuration unless removal is explicitly selected.

#### Scenario: Key saved while offline
- **WHEN** persisted configuration is verified but image retrieval cannot be verified
- **THEN** the app reports the saved configuration and pending provider check separately without claiming a working connection

### Requirement: Keep credentials within scoped trusted channels
New keys SHALL remain in desktop memory only until completion or dismissal and SHALL enter TV-private storage through a verified narrow interface. Logs, process arguments, ordinary journals and support exports MUST exclude credentials. Credential restoration SHALL use TV-private rollback material and opaque references only.

#### Scenario: Interrupted credential update
- **WHEN** connection ends during a reviewed credential update
- **THEN** recovery reconciles private transaction status without exporting or guessing credential contents
