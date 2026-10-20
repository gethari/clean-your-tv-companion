# Spec Delta

## Purpose

Guide desktop users from discovery to reviewed personalization with clear feedback while preserving existing TV configuration.

## ADDED Requirements

### Requirement: Adapt setup to observed state
The companion SHALL guide new users through selection, pairing when needed, inspection, personalization, review, and completion. Returning verified users SHALL open their dashboard. Existing configuration SHALL populate editing controls; unknown values MUST remain visibly unknown.

#### Scenario: Existing customization is inspected
- **WHEN** a TV already has a compatible configured launcher
- **THEN** setup preserves its values and does not require reinstalling or resetting it

#### Scenario: Pairing succeeds but connection discovery fails
- **WHEN** authorization succeeds without one verified connection candidate
- **THEN** the app retains pairing success and focuses guidance for obtaining the current connection address

### Requirement: Provide accessible feedback tied to real state
The app SHALL provide inline validation, pending feedback, duplicate-submit prevention, visible keyboard focus, and announced status. Discovery, transfer, and completion motion SHALL reflect real work, introduce no artificial delay, and respect reduced-motion settings.

#### Scenario: Reduced motion is enabled
- **WHEN** the user discovers or configures a TV with reduced motion enabled
- **THEN** status remains understandable without animated transforms or looping effects

### Requirement: Review and verify personalization changes
The companion SHALL preview intended changes and require explicit Apply before configuration writes. It SHALL record prior state, verify outcomes, expose partial or unknown results, and support conflict-aware restoration of its own changes. Unsupported launcher versions SHALL show an actionable read-only state.

#### Scenario: Connection drops during apply
- **WHEN** a transfer or configuration operation loses its connection
- **THEN** completion is not claimed and reconnect reconciles observed state before retrying

#### Scenario: Nothing changed
- **WHEN** reviewed preferences already match the verified TV state
- **THEN** the app reports no changes needed and performs no configuration writes

### Requirement: Support desktop input and narrow layouts
Personalization SHALL support keyboard controls and file selection as alternatives to drag-and-drop. Narrow windows SHALL preserve all actions without horizontal overflow. Provider network use and account requirements SHALL be disclosed in context.

#### Scenario: Narrow desktop window
- **WHEN** the preview and controls cannot fit side by side
- **THEN** they stack while preserving labels, status, and access to review and apply
