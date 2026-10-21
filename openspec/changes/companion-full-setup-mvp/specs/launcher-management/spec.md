# Spec Delta

## Purpose

Install and select a verified compatible upstream launcher while preserving a working fallback and existing application data.

## ADDED Requirements

### Requirement: Install trusted compatible launcher artifacts
The companion SHALL use a pinned artifact with source, version, SHA-256, package, signing identity and compatibility metadata. It MUST verify bytes and compatibility before installation and verify the installed package afterward. A missing trusted artifact SHALL block installation with a clear reason.

#### Scenario: Signature mismatch
- **WHEN** a candidate APK is signed incompatibly with the installed launcher
- **THEN** installation is blocked without uninstalling or clearing application data

#### Scenario: Desired launcher already installed
- **WHEN** inspection verifies the required compatible launcher version
- **THEN** installation is a no-op

### Requirement: Verify launcher handover and retain fallback
The companion SHALL guide HOME selection and verify the active HOME before dependent operations. The original launcher MUST remain enabled as a fallback. Restoring HOME SHALL use its recorded prior value when no later conflicting change exists.

#### Scenario: User declines HOME chooser
- **WHEN** installation succeeds but the intended launcher does not become HOME
- **THEN** the app reports installation separately from unverified handover and pauses dependent operations

### Requirement: Treat upstream runtime as an external dependency
The companion SHALL configure interfaces verified in the pinned upstream launcher. It MUST NOT infer provider or companion IPC support from a version name, and MUST expose unsupported operations as unavailable. Runtime changes require a separately scoped launcher deliverable.

#### Scenario: Provider exists without a supported configuration bridge
- **WHEN** the launcher can fetch Unsplash photos but lacks the required safe companion interface
- **THEN** the companion explains the missing integration without exposing an Apply action that cannot work
