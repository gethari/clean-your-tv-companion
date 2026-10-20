# Spec Delta

## Purpose

Find Android TVs on the current local network and reconnect returning users without confusing discovery with verified device identity.

## ADDED Requirements

### Requirement: Discover automatically on native startup
The companion SHALL start bounded discovery of connected devices and advertised debugging services on native launch, expose search status and retry, and retain manual address entry. Browser demo mode MUST NOT perform network discovery.

#### Scenario: Startup search finds candidates
- **WHEN** the desktop app opens on a reachable local network
- **THEN** it starts searching without a button click and presents discovered candidates as unverified until inspected

#### Scenario: Search cannot find a TV
- **WHEN** discovery times out, is denied, or returns no candidates
- **THEN** the app ends its pending state and offers retry, manual entry, and relevant network/debugging guidance

### Requirement: Reconnect only a verified remembered TV
The companion SHALL automatically reconnect only when exactly one candidate matches the previously selected TV identity and existing authorization. IP address, model name, or advertisement alone MUST NOT establish identity. New or ambiguous candidates SHALL require selection and authorization as needed.

#### Scenario: Returning TV changes connection port
- **WHEN** one authorized candidate is verified as the remembered TV at a new endpoint
- **THEN** the app refreshes its inspection and opens that TV dashboard without repeating pairing

#### Scenario: Identity is ambiguous
- **WHEN** multiple candidates match or identity cannot be verified
- **THEN** the app requests selection and performs no configuration writes

### Requirement: Recover discovery without changing TV configuration
Discovery and reconnection SHALL remain read-only with respect to TV configuration, deduplicate candidate endpoints, and prevent overlapping searches or stale results from replacing a newer session.

#### Scenario: Network changes during a search
- **WHEN** a search is superseded by retry or a network change
- **THEN** results from the earlier search cannot select a TV or overwrite the current search state
