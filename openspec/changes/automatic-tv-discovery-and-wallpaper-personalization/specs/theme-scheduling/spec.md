# Spec Delta

## Purpose

Apply launcher themes at fixed times or solar events on the TV independently of companion availability and wallpaper rotation.

## ADDED Requirements

### Requirement: Configure independent theme schedules
Users SHALL configure daily fixed-time or sunrise/sunset rules selecting theme appearance and wallpaper collection. Wallpaper rotation intervals SHALL remain independent. Review SHALL show the timezone, upcoming effective transitions, and validation errors before Apply.

#### Scenario: Evening theme with rotation
- **WHEN** a user schedules an evening collection at sunset with a 15 minute interval
- **THEN** the TV changes to that collection at sunset and rotates its images every 15 minutes

### Requirement: Calculate solar events from explicit location
Solar schedules SHALL use a user-selected location and timezone without requiring device location permission. If a solar event does not occur on a date, the schedule SHALL use its configured fixed-time fallback and display that behavior in the preview.

#### Scenario: No sunset occurs
- **WHEN** the selected location has no sunset on the scheduled date
- **THEN** the evening rule uses its displayed fixed-time fallback

### Requirement: Resolve schedule conflicts deterministically
The app SHALL reject duplicate fixed-time boundaries and require a visible rule priority for coincident solar and fixed-time boundaries. The schedule SHALL define an initial default theme and a manual override that lasts until the next scheduled transition or user cancellation.

#### Scenario: Rules coincide
- **WHEN** a fixed-time and solar rule resolve to the same instant
- **THEN** the TV applies the rule with the displayed higher priority exactly once

### Requirement: Persist and reconcile schedules on the TV
The TV SHALL persist schedules and apply the currently effective theme after startup, wake, clock changes, or timezone changes without replaying missed transitions. Repeated daylight-saving times SHALL trigger once; nonexistent times SHALL resolve to the first valid instant afterward. Offline operation SHALL use saved rules and available images.

#### Scenario: TV sleeps through sunset
- **WHEN** it wakes after a scheduled evening transition while the laptop is closed
- **THEN** it selects the effective evening theme without replaying intermediate themes

#### Scenario: Network is unavailable
- **WHEN** a scheduled collection cannot fetch new images
- **THEN** the TV uses its available collection images or retains the last usable background
