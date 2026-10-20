# Caveat register

Sources: upstream README, PROMPT.md, tools/{disable,restore,theme}.{sh,ps1}, launcher/build-and-install.sh,
docs/TROUBLESHOOTING.md at local review on 2026-10-20; plus our TCL Q6K session. This is an audit of
observed failure modes, not a claim that upstream lacks all safeguards. Several already have guards in
the scripts; the companion must retain and explain them. The list grows with device evidence.

| Trigger / evidence | Beginner-facing response | Required gate |
|---|---|---|
| Android 11+ TLS debugging: pairing port differs from connection port | Separate labeled steps; TV prompt illustration | Never treat pair success as connected |
| TCL port changed repeatedly during this session | Re-discover and reconnect same identity; manual new port | Never apply stale plan to new device at old IP |
| mDNS unavailable, guest Wi-Fi, VPN, VLAN, host firewall | Explain same LAN and local network access; manual address | Bounded discovery, explicit interface selection |
| macOS privacy blocks local access; Windows firewall | Show OS-specific guidance | Don't disable firewall or security automatically |
| Legacy 5555 may be plaintext/unsecured | Opt-in legacy path, close-debugging checklist | No silent tcpip 5555 downgrade |
| Unsupported Tizen/webOS or restricted Fire OS | Clear unsupported outcome | No generic brand package list |
| Generic reported model (our TCL says Smart TV Pro) | Show reported identity and allow user nickname | Never choose destructive profile from model string alone |
| Google setupwraith owns saved Wi-Fi | Explain forgetting/rejoining network using remote | Block its disable if ownership present OR unreadable |
| Critical input, tuner, Bluetooth, remote, keyboard, Play services | Protected reasons and kept defaults | Protected-package dependency rules, unknown = kept |
| Casting, voice, updates and streaming apps are user choices | Plain-language feature checklist | No default disable for any of these |
| Unknown package functionality | Show observed metadata, keep it | No heuristic package-name debloating |
| Chained pm calls stop silently on some firmware | Honest operation progress | One command per operation; parse output and re-query |
| Existing disabled apps predate companion | Distinguish observed state from our history | Restore only journal-owned deltas |
| HOME chooser / multiple candidates / another third-party launcher | Show candidates and current choice | Start new HOME, verify before handover, retain fallback |
| APK install blocked / HOME signature requirement / Android 14 confirmation | Prompt user, recognize known failure, stop retries | Never disable working HOME after failed install |
| Signature mismatch / downgrade / min SDK / ABI mismatch | Explain incompatibility without data deletion | Inspect artifact and installed signing identity; no uninstall workaround |
| Changing HOME candidates clears preference | Re-query HOME after each change | Rollback on unexpected resolver |
| Force-stop loses accessibility helper bindings | Explain physical Input/Home button test | Save/restore exact service list, recheck enabled AND bound |
| Injected key works but physical remote doesn't | Ask user to press actual buttons | Distinct user-confirmed checkpoint |
| Live TV / HDMI screenshot protected or empty | Check activity + user observation | Never infer broken input from black capture |
| USB debugging cuts USB power on some sets | Ask about attached USB devices | User confirms before enabling/debugging changes |
| ADB file push hangs / install waits for confirmation | Bounded timeout; resumable transfer option | Hash verified temp staging then atomic replace |
| Existing theme/settings overrides, external credentials | Preserve unknown keys, explain effective settings | Merge patch, baseline and revision handling |
| Wallpaper interval only while Home visible; overdue on return | Say when it changes | No promise of changes during playback |
| Unsplash key shared/distributed or private state unreadable | Bring own key, optional provider; Unknown status | No bundled shared key; official API hotlink/credit/tracking requirements |
| Unsplash restricts wallpaper-centric applications | Provider integration conditional, offline folders always usable | Review terms/obtain clarification before public provider release |
| Low-RAM TV + effects + full-res photos | Calm default, preview dimming/effects | Bound decode size and downloads; no video wallpapers |
| Private DNS can break access and doesn't block YouTube ads | Optional toggle with playback check | Restore exact prior mode/specifier, not a guessed default |
| Multi-user/device-owner/security policy blocks an action | Explain limitation, skip action | Identify user; don't assume user 0 across firmware |
| Power loss or Wi-Fi drop mid-change | Reconnect + reconcile, no blind retry | Durable intent/result journal, precondition recheck |
| Factory reset or firmware update changes identity/capabilities | Fresh audit, preserve historical backup separately | Invalidate stale plan, no replay across reset |
| Debugging closed at finish | Explain future maintenance requires reconnect | Closing connection is expected, not failed setup |
| Unsigned Windows/macOS preview installers | Honest testing instructions | Signing/notarization gate for polished public distribution |
| Platform-tools redistribution / licenses | Bundle official OS/arch tools and notices | Pin versions/hash manifest; no random mirror |
| Running headless on Linux | Build/test with virtual display; later laptop smoke test | Real native launch before claiming desktop verified |

## Coverage policy

A profile records exact firmware evidence, tested operations, protected packages, source provenance and
last verification date. Community examples inform review; they are not executable trust. An exact model
match alone does not unlock disabling. Generic audit and existing-state inspection remain useful when
no approved profile matches. Never label all 66 upstream examples as companion-certified.
