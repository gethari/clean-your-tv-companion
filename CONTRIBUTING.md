# Contributing

Use conventional commits and focused PRs. Read docs/PRODUCT.md, docs/CAVEATS.md, docs/ARCHITECTURE.md,
docs/DESIGN.md and plans/companion.md. Never commit personal TV dumps, credentials, APK signing keys,
ADB keys or hardcoded home-network addresses. Demo data must be synthetic and explicitly labeled.

Run pnpm check, pnpm test, pnpm build and cargo test before proposing native changes. Backend
commands accept typed inputs, never arbitrary shell snippets. Unknown observations stay unknown.
A feature that changes a TV needs durable baseline/journal, diff review, postcondition and tested
compensation. Do not expose a UI control until its implementation and recovery are real.
