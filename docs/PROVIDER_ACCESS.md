# Online wallpaper provider access decision

**Reviewed:** 2026-10-20
**Scope:** Unsplash API and Reddit Data API for optional wallpaper collections in Clean Your TV Companion.

This is a provider gate assessment, not evidence of API access. No approved Unsplash or Reddit credentials, provider agreement, or app approval are established in this repository. Do not enable either adapter until the applicable approval and product-use questions below are resolved. The planned local import path is independent of these provider gates.

## Decision

| Provider | Current decision | Gate before enabling |
| --- | --- | --- |
| Unsplash | Unavailable for TV-copied/offline wallpaper collections pending written clarification/approval. | Establish approved API access for this app and confirm in writing that API images may be copied to the TV, retained for offline rotation, and served from TV storage while meeting Unsplash hotlinking and interaction tracking rules. |
| Reddit | Unavailable. No scraping, unauthenticated JSON, or fallback image extraction. | Obtain explicit API access approval for this exact use case, plus written approval for commercial use if applicable. Obtain a permitted-use/retention decision for copying user-submitted images to the TV and retaining them offline. |

## Unsplash

Unsplash API Guidelines require apps to use the API-returned `photo.urls` image URLs directly (hotlinking), including when displaying images. An image selection or a similar event such as setting an image as a background is a “download” event and requires a call to the API-provided `photo.links.download_location`. The endpoint is event tracking; it is not the image URL. Attribution must identify Unsplash and the photographer and link to the photographer’s Unsplash profile. See [Unsplash API Guidelines](https://help.unsplash.com/en/articles/2511245-unsplash-api-guidelines), [API documentation](https://unsplash.com/documentation), and [API Terms](https://unsplash.com/api-terms).

This product’s TV-owned rotation needs verified image bytes on TV storage and must continue while offline. That does not fit the direct hotlinking requirement on its face. Triggering the download event would not itself grant permission to copy, retain, or redistribute the photo outside the API-provided URL. Unsplash’s API Terms also say that each display must carry the required attribution and that each comparable download event, including setting wallpaper, must be reported. Do not interpret the general Unsplash License as overriding these API-specific requirements.

**Recommendation:** keep the Unsplash adapter unavailable for TV transfer/offline rotation until Unsplash confirms the proposed flow in writing. If approval is given, preserve returned attribution metadata and profile URL with each image, hotlink previews through returned URLs, report each qualifying wallpaper selection event, and document any permitted caching and retention period. Do not put an API key in source, URLs exposed in UI, logs, snapshots, or exported reports; use an OS credential store for a locally supplied key. Public distribution and production API access/rate limits must be confirmed with Unsplash before release.

## Reddit

Reddit’s current [Responsible Builder Policy](https://support.reddithelp.com/hc/en-us/articles/42728983564564-Responsible-Builder-Policy) says API users must request access and receive explicit approval before accessing Reddit data. It also requires explicit written approval for commercial use. The [Data API Terms](https://redditinc.com/policies/data-api-terms) license user content only for the approved app use case, require honoring rights and restrictions imposed by the content’s author, prohibit unapproved commercialization, and require immediately deleting data not required for the approved use case. If access ends, cached or stored API content and materials must be deleted. Reddit may revoke access or change the APIs.

The terms do not establish a general right to download and retain Redditors’ images as an offline wallpaper library. Image availability in a post is not proof that the poster owns or licenses the image for redistribution. Attribution requirements also apply as specified by Reddit’s developer documentation, and provider branding must not suggest Reddit endorsement. No approved credentials or access approval are established here.

**Recommendation:** leave Reddit unavailable unless Reddit approves this app and this exact image-display/offline-TV use in writing, including retention, display/attribution, removal, and commercial status. Until then, do not scrape Reddit pages or public JSON endpoints. If approved later, constrain access to user-selected communities and permitted API endpoints, use authenticated API access with truthful app identity, retain only data needed for the approved wallpaper feature, promptly process removals/revocation, and preserve attribution and source links. Store credentials in the OS credential store and keep them out of logs, snapshots, UI URLs, and exports.

## Shared credential and release handling

- No provider credential or approval is checked into this report; no secret values are recorded.
- Keep online providers optional. Provider access, authentication errors, rate limits, or outages must not block local imports or erase an already usable local collection.
- Keep desktop credentials in the OS credential store. Any credential needed by the TV may cross IPC only through an approved, narrowly scoped launcher protocol; never include credentials in operation snapshots, logs, URLs shown to users, or exported reports.
- Recheck provider terms and approval conditions before enabling an adapter and after material terms/API changes. Removal or revocation must follow the provider’s applicable deletion/retention requirements.

## Official sources

- [Unsplash API Guidelines](https://help.unsplash.com/en/articles/2511245-unsplash-api-guidelines)
- [Unsplash API documentation](https://unsplash.com/documentation)
- [Unsplash API Terms](https://unsplash.com/api-terms)
- [Reddit Responsible Builder Policy](https://support.reddithelp.com/hc/en-us/articles/42728983564564-Responsible-Builder-Policy)
- [Reddit Data API Terms](https://redditinc.com/policies/data-api-terms)
