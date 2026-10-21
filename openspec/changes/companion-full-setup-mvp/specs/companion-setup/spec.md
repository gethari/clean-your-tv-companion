# Spec Delta

## Purpose

Guide desktop users from device discovery through verified setup and later maintenance without terminal instructions.

## ADDED Requirements

### Requirement: Discover and authorize the intended TV
The app SHALL start bounded native discovery, retain manual connection, and guide pairing. It MUST reconnect automatically only to one authorized candidate matching the remembered identity. Demo mode MUST NOT invoke device operations.

#### Scenario: Returning TV changes port
- **WHEN** exactly one authorized candidate matches the remembered TV at a new endpoint
- **THEN** the app refreshes inspection and opens its dashboard

#### Scenario: Discovery cannot verify a candidate
- **WHEN** discovery is empty, ambiguous, unauthorized or interrupted
- **THEN** the app ends its pending state and offers selection, retry or manual connection

### Requirement: Adapt setup to current observations
The app SHALL distinguish installed, active, compatible, unknown and failed states. It SHALL offer only relevant setup steps and preserve existing customization. Configuration writes MUST require a reviewed explicit Apply.

#### Scenario: Already customized TV
- **WHEN** a compatible launcher is installed and active
- **THEN** the app offers maintenance without requiring reinstall or resetting preferences

### Requirement: Provide accessible desktop feedback
The app SHALL support keyboard controls, narrow layouts, reduced motion, inline validation and announced operation status. Progress MUST reflect observed work. Failures SHALL provide a useful next action.

#### Scenario: Novice desktop setup
- **WHEN** a user follows setup on a supported packaged Windows or Linux app
- **THEN** all steps are available without terminal commands or developer tooling
