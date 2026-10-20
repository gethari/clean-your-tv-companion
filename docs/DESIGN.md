# Design direction

Subject: a laptop helping a household TV feel like their own again. Single job: explain the TV's current
state and the next useful action. Quiet, reassuring, never a cleaning utility that invents problems.

Palette: porcelain #F6F8FC, ink #192C48, ocean #315FDB, pale blue #EAF0FF, mint #CDECDD,
amber #9B5C12. Status always has text/icon, never color alone. Body uses the platform UI font for
native familiarity; restrained rounded display uses system rounded fallback; utility data uses monospace.
No remote fonts, images or CDN. Spacious two-column desktop layout; narrow widths stack without loss.

Signature: a little illustrated TV with expressive eyes. Its screen becomes a landscape on completion,
a gentle antenna scan during discovery, and a paused face on disconnect. Drawn in local SVG/CSS so it is
crisp, small, original and usable offline. No continuous flashing, celebratory noise or forced confetti.

Flow: left rail for Connect → Check TV → Make it yours → Review → Finish; overview content with a TV
identity panel, source freshness and actual results. Available options live below completed features,
not as alarming uncompleted tasks. Current preview labels unavailable writing flows as planned text;
there are no pretend Apply/Restore buttons.

Motion: 160ms focus/hover, 220ms section reveal, brief scan only while waiting. Respect
prefers-reduced-motion: disable transforms/loops and keep status text. No artificial delay to show art.
Buttons describe action, pending actions disable duplicate submits, errors have a direct recovery step.
Keyboard focus is visible; form inputs have labels; progress/status uses polite live announcements;
buttons ≥40px; screen-reader labels describe the illustration only when it carries new information.

Copy examples:
- Existing setup: "Your TV already feels like home." / "Here’s what’s already in place."
- No pending work: "Nothing needs changing." / "You can close the app and enjoy your TV."
- Unknown: "Couldn't check wallpaper connection." / "Private app settings aren't visible over ADB."
- Lost connection: "Your TV went offline." / "Keep it awake and check the connection port on the TV."
- Unsupported: "This TV uses a different system." / "This version supports Android TV and Google TV."
- Partial setup: "A few things are ready. Let's check the rest." Never "Your TV is dirty" or a fake score.
