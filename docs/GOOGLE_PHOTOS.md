# Google Photos feasibility for wallpaper collections

**Reviewed:** 2026-10-20
**Status:** Investigation only. This report does not establish Google API credentials, partner acceptance, OAuth verification, or approval for the app’s proposed transfer/display behavior.

## Recommendation

Do not add a Google Photos connect control in this change. Use local file/folder import as the intended route once that implementation is complete. A future integration should first seek Google’s review of the intended product flow and then choose one of two distinct routes:

1. **Picker API for a deliberate one-time import:** The user selects specific items in Google Photos, then the companion imports the selected bytes into a local collection and, after an explicit transfer action, copies them to the compatible TV. Treat this as a candidate user-initiated export flow, subject to policy and OAuth review. It is not an album subscription or automatic sync.
2. **Ambient API for a Google-managed ambient slideshow:** This is the API expressly designed to show a user’s selected Google Photos on ambient devices such as smart TVs and photo frames. It requires partner-program acceptance and Google reviews. It may be a better technical fit for a continuing Google Photos slideshow, but it does not establish eligibility for this app’s launcher-background feature or its private launcher protocol. Confirm that scope with Google before designing UI or claiming support.

Neither route is an approved integration today. The repository’s current scope displays wallpapers through a compatible launcher background. Google documentation supports user-selected sharing to an Ambient device, but does not expressly say that any third-party launcher wallpaper implementation qualifies. Ambient use should therefore remain a follow-up feasibility question.

## Picker API: bounded user selection

The Picker API is the supported API for letting a user select items from their library. The API provides a picker URI/session, waits for user selection, then lists and retrieves only items selected in that session. It uses the `photospicker.mediaitems.readonly` OAuth scope; the user must authenticate and consent. Google’s former broad Library API read scopes were removed after March 31, 2025, and the Library API now focuses on app-created content. Do not build on the old `photoslibrary.readonly` model or assume broad album access. See [Google’s Photos API changes](https://developers.google.com/photos/support/updates), [Picker sessions](https://developers.google.com/photos/picker/guides/sessions), and [Picker media retrieval](https://developers.google.com/photos/picker/guides/media-items).

Picker media `baseUrl` values require an OAuth bearer token and expire after 60 minutes (or earlier after permission revocation). Therefore the app would have to fetch selected bytes promptly and treat the URL as temporary. This is technically compatible with a one-time import pipeline, but the API’s short-lived URL does not itself grant indefinite retention. Google’s policy requires a visible user-benefiting feature, clear disclosure immediately before consent, affirmative consent, secure handling, data deletion controls, and respect for deletion requests. It permits data transfer only for the user-facing feature with consent, among limited exceptions. Before implementing persistent TV copies, confirm that this explicit user-directed wallpaper export meets Google’s policy and review requirements. See [Photos API User Data and Developer Policy](https://developers.google.com/photos/support/api-policy).

For a future implementation, request authorization only when the user starts import; fetch only the selected items; remove Picker sessions after retrieval; do not retain the expiring URL as a durable media reference; explain which copies are stored on the desktop and TV; and provide a way to delete those copies and revoke the integration. Protect OAuth tokens at rest and in transit. The user can also use Google Photos’ existing export/download facility and import files through the app’s local picker without granting the companion Google account access.

## Ambient API: partner-gated slideshow

Ambient is designed for apps that connect an ambient device to a user’s Google Photos account and display selected photos/videos. It uses OAuth 2.0 for TVs and limited-input devices, requires the user to sign in with a valid Google Account, and does not support service accounts. A user authorizes in a richer-input browser/device and selects media sources in the Google Photos app. A public app must pass OAuth verification. See [Ambient app configuration](https://developers.google.com/photos/ambient/guides/configure-your-app) and [Ambient flow](https://developers.google.com/photos/ambient/guides/get-started).

Access is gated: Google says an app must be accepted into the Google Photos Partner Program to use Ambient. After acceptance, launch still requires OAuth verification and a Google Photos API integration review. Picker and Library APIs do not require partner membership, though quota increases may. See the [Google Photos Partner Program](https://developers.google.com/photos/partner-program/overview).

The API requires the user to choose media sources for the device, then the app polls and retrieves the curated items for display. Google provides ambient-specific UX guidance, including an accessible way to identify the connected account, an accessible Google Photos attribution path, refreshing the selected sources, and respecting automated content filtering. See [Ambient media retrieval](https://developers.google.com/photos/ambient/guides/media-items) and [Ambient UX guidelines](https://developers.google.com/photos/support/ambient-ux-guidelines).

This looks more suitable than Picker for a maintained Google Photos slideshow, but would require a Google-authenticated TV-side device integration and network-dependent media retrieval. It has not been shown to work through this project’s launcher contract. The Ambient API’s existence is not evidence that a third-party launcher’s background setter is an eligible Ambient device or that this app can substitute its own wallpaper rotation/storage flow. Seek partner feedback on the exact launcher use case before investing in an adapter.

## Retention and eligibility summary

| Question | Finding |
| --- | --- |
| Does the user need to sign in? | Yes for both Picker and Ambient API access. Picker requires Google OAuth consent for the selected-items scope; Ambient uses OAuth for TV/limited-input devices. |
| Can Picker provide ongoing album sync? | No. It gives access to user-picked items for a session; it is a bounded import route, not a general album subscription. |
| Can the app keep a local copy? | Picker supplies short-lived fetch URLs. Google policy permits appropriate, user-facing use and limited transfers with consent, but it does not give a blanket perpetual-retention grant. Confirm the explicit desktop-to-TV export and retention/deletion design with Google before implementation. |
| Does Ambient provide persistent selected-source display? | It is explicitly designed for ambient displays, with user-selected media sources and refresh behavior. Partner acceptance and launch reviews are required. |
| Is this launcher background feature eligible for Ambient? | Unknown from public documentation. A compatible TV launcher wallpaper is not automatically equivalent to an Ambient API device/slideshow. Obtain Google’s written guidance and partner acceptance before representing it as supported. |
| Should a connect button ship now? | No. Keep this investigation separate from shipped capability and keep local import fully functional without Google login. |

## Official sources

- [Google Photos API updates](https://developers.google.com/photos/support/updates)
- [Picker API sessions](https://developers.google.com/photos/picker/guides/sessions)
- [Picker API media retrieval and URL lifetime](https://developers.google.com/photos/picker/guides/media-items)
- [Photos API User Data and Developer Policy](https://developers.google.com/photos/support/api-policy)
- [Ambient configuration and OAuth](https://developers.google.com/photos/ambient/guides/configure-your-app)
- [Ambient API getting started](https://developers.google.com/photos/ambient/guides/get-started)
- [Google Photos Partner Program](https://developers.google.com/photos/partner-program/overview)
- [Ambient media retrieval](https://developers.google.com/photos/ambient/guides/media-items)
- [Ambient UX guidelines](https://developers.google.com/photos/support/ambient-ux-guidelines)
