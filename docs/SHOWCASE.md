# Future GitHub Pages showcase

The repository is public so hosted Actions runners can build the desktop previews. Pages is reserved
for a later showcase; it is not a dependency of the installed app. A future Pages deployment can show
screenshots, honest compatibility, download links, novice instructions and a clearly labeled demo.

Keep the showcase statically hosted, with no account, analytics requirement or AI service. Browser demo
has no ADB/localhost bridge and never implies it is inspecting the visitor's TV. Downloads point to
versioned, signed desktop releases when available; unsigned preview builds must retain their labels.
Set Vite's base to /clean-your-tv-companion/ if the current demo is later deployed to project Pages,
and test direct navigation/assets under that prefix before enabling the deployment workflow.

Do not publish support exports, TV screenshots with identifiers, local addresses, journal files,
credential examples containing real values, API keys or ADB keys. Current screenshots use synthetic
explicitly labeled demo data. Configure Pages through GitHub Actions only when the showcase is ready.
