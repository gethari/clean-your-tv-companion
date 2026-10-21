# Spec Delta

## Purpose

Keep reviewed companion changes recoverable across interruptions and desktop restarts without overwriting later user changes.

## ADDED Requirements

### Requirement: Review identity-bound changes before writing
Plans SHALL contain exact prior state, intended changes, identity, preconditions and expiry. The app MUST persist a baseline and intent before mutation, hold a per-device lock and verify postconditions. No-op plans SHALL issue no configuration writes. Editing a draft MUST invalidate its review.

#### Scenario: Different TV or stale review
- **WHEN** identity, revision or expiry no longer matches the plan
- **THEN** the app refuses writes and requests refreshed review

### Requirement: Recover persisted operation history
The companion SHALL list safe operation summaries for the verified TV across restarts and expose resumable, uncertain, verified, conflicting and restored states. Invalid records SHALL be reported without hiding valid history. History MUST NOT expose another TV's plans or secret fields.

#### Scenario: Desktop reopens after activation disconnect
- **WHEN** a saved activation has an uncertain outcome
- **THEN** history offers reconciliation and queries actual state before any retry or success claim

### Requirement: Restore only unchanged companion-owned state
Restore SHALL create a separate reviewed inverse plan, preserving exact prior values and absence. Current owned values MUST match the recorded applied state before restoration. Existing pre-disabled packages and unrelated settings MUST be preserved. APK/data downgrade SHALL NOT be promised.

#### Scenario: Manual change after apply
- **WHEN** the current value differs from the last verified companion-applied value
- **THEN** restore reports a conflict without overwriting that value

### Requirement: Preserve active state during failed transfers
Media SHALL be hash-verified and storage-checked before atomic activation. Interrupted or cancelled staging MUST preserve active and rollback media. Unknown activation outcomes MUST remain visibly uncertain until verified.

#### Scenario: Insufficient TV storage
- **WHEN** staging cannot retain both active/rollback media and the new assets
- **THEN** the current collection remains active and the app offers a smaller collection or safe retry
