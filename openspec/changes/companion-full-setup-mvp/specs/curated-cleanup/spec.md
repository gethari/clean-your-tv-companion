# Spec Delta

## Purpose

Offer explicit reversible cleanup for tested TCL firmware while protecting essential TV functions and pre-existing package choices.

## ADDED Requirements

### Requirement: Gate cleanup on validated firmware profiles
Cleanup SHALL use versioned rules with model/firmware constraints and verification evidence. Brand detection or package-name similarity MUST NOT authorize modification. Other TVs SHALL retain supported inspection and setup features while unvalidated cleanup remains unavailable.

#### Scenario: Unvalidated TCL firmware
- **WHEN** the manufacturer is TCL but no tested profile matches
- **THEN** the app shows inspection/comparison without enabling package modification

### Requirement: Require explicit optional cleanup selection
No cleanup item SHALL be selected by default. Each candidate SHALL explain its effect and protected dependencies. Unknown packages and essential HDMI, input, network, remote, casting, accessibility and update functions MUST remain enabled. System packages MUST NOT be uninstalled.

#### Scenario: Bundled essential feature
- **WHEN** a telemetry-labelled package also owns casting or voice functionality
- **THEN** that package is excluded from MVP cleanup rather than silently removing the bundled feature

### Requirement: Verify and precisely restore package changes
Cleanup SHALL execute reviewed operations sequentially with fresh state checks and observed postconditions. Previously disabled packages SHALL produce no disable operation. Restore MUST change only recorded companion-owned package states and detect later drift.

#### Scenario: Package disabled before setup
- **WHEN** a package was already disabled at baseline
- **THEN** neither Apply nor Restore changes its state
