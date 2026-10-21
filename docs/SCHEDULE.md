# Theme schedules · preview semantics

The Backgrounds page includes a local schedule editor and transition preview. It validates an explicit
location and timezone, fixed-time and sunrise/sunset rules, priorities and fallback times, then shows the
next changes. **It does not write to a TV.** The launcher capability `theme_schedules_v1` remains absent
from the verified upstream release, so TV-side evaluation, persistence and wake/restart behavior are not
claimed. Physical delivery is a separate launcher deliverable.

## Example fixture

The editor opens with this fixture, which matches the automated tests in `tests/schedule.test.mjs`:

```json
{
  "defaultTheme": "warm",
  "location": { "latitude": 51.5072, "longitude": -0.1276, "timezone": "Europe/London" },
  "rules": [
    { "id": "morning", "name": "Weekday morning", "kind": "fixed", "time": "07:00", "theme": "warm", "priority": 10 },
    { "id": "evening", "name": "Evening", "kind": "solar", "event": "sunset", "offsetMinutes": 0, "fallbackTime": "18:00", "theme": "calm", "priority": 20 }
  ]
}
```

`upcomingTransitions(schedule, now, 4)` returns at most four distinct instants in chronological order,
formatted with the schedule’s timezone. A rule’s `priority` breaks ties at the same instant, and only the
highest-priority rule is applied once. Duplicate fixed-time boundaries and duplicate priorities are
validation errors.

## Solar fallback and polar dates

Solar times use the saved latitude and longitude; the app never requests device location permission. If
the selected date has no sunrise or sunset (for example latitude `89` in late December), the rule uses its
explicit fixed-time fallback and the preview marks “fixed fallback used.” The fixture test asserts that
`Arctic/Longyearbyen` at latitude `89` on 20 December produces a fallback at `18:00`, not an invented
solar time.

## Timezones and daylight saving

Rules are wall-clock times in the selected IANA timezone, not the laptop’s current zone. Changing the
timezone in the editor changes the preview. The scheduler conversion follows the TV contract:

- A repeated time during a daylight-saving fall-back resolves to the earliest occurrence and triggers once.
- A nonexistent time during a spring-forward gap resolves to the first valid instant afterward.
- Missed transitions are not replayed; the preview shows the effective upcoming changes only.

The fixture tests cover `Europe/Berlin` on 2026-03-29 (`02:30` becomes `03:00`) and 2026-10-25 (`02:30`
resolves to the first occurrence, `00:30 UTC`).

## Offline operation

The intended TV behavior is local evaluation from saved rules and available collection images, without a
network or the companion. That behavior requires the launcher-side scheduler and is not enabled in this
build. The desktop editor and preview work offline and make no network or TV calls. Once the capability is
available, delivery must use the versioned contract, and physical wake/reboot/offline checks are required
before enabling the controls.
