# Spec Delta

## Purpose

Manage personal and optional online photo collections as launcher backgrounds with independent rotation and recoverable delivery to the TV.

## ADDED Requirements

### Requirement: Import and preview local collections
Users SHALL be able to select or drop local image files and folders, review accepted images, remove items, and preview crop and dimming before applying. Invalid files SHALL receive individual reasons; valid files MUST remain selectable. Collections SHALL affect launcher backgrounds only.

#### Scenario: Mixed file drop
- **WHEN** a drop includes valid images and unsupported or corrupt files
- **THEN** accepted thumbnails and individual rejection reasons appear without changing the TV

### Requirement: Configure wallpaper rotation
Users SHALL choose a collection, sequential or shuffled order, and a positive custom interval in minutes, with 5, 15, and 30 minute presets. The TV SHALL persist rotation and continue using available images without the companion. An invalid interval MUST NOT be applied.

#### Scenario: Laptop closes
- **WHEN** a verified collection is active and the companion closes
- **THEN** the TV continues rotation using its saved interval and available images

#### Scenario: Collection contains one image
- **WHEN** rotation has only one usable image
- **THEN** that image remains displayed without repeated empty transitions

### Requirement: Transfer collections recoverably
Applying local media SHALL report actual progress and verify transferred content before activating the new collection. Interrupted or insufficient-storage transfers SHALL preserve the previously active collection and expose retry or cancellation.

#### Scenario: Transfer is interrupted
- **WHEN** connectivity is lost before collection activation
- **THEN** the TV retains its previous collection and reconnect reconciles staged content before resuming

### Requirement: Offer optional provider sources honestly
The app SHALL offer Unsplash and Reddit collection configuration only when the adapter has required access and satisfies provider requirements. Missing access, authentication failure, rate limits, and offline status SHALL be distinct from empty collections. Existing usable backgrounds MUST survive provider failures.

#### Scenario: Provider access is unavailable
- **WHEN** a user selects a provider without approved access or required credentials
- **THEN** the app explains its unavailable state and offers local import without claiming a connected collection

#### Scenario: Online retrieval fails
- **WHEN** an enabled provider is unreachable or rate limited
- **THEN** the TV keeps available backgrounds and exposes retry status without clearing the collection

### Requirement: Do not imply unsupported Google Photos synchronization
The app MUST NOT advertise Google Photos live album sync or anonymous private album access without a validated supported integration. Local imports of exported photos SHALL work without a Google login in the companion.

#### Scenario: User imports exported Google Photos files
- **WHEN** a user selects supported exported image files
- **THEN** they are treated as local media without requesting Google authorization
