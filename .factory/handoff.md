# Repair handoff — PDF Link Map

Work order: `pdf-link-map-repair-4`
Repaired/deployed commit: `dbee73bc6940c600e4d9e11d6e0029fc3bef2e7b`
Live URL: https://pdf-link-map.sociobot.in/
Verified: 2026-08-30

## Result

The verifier’s two release blockers are repaired and deployed.

- The first screen now names the intended users: operations and
  technical-document teams converting HTML or DOCX to PDF. It keeps the
  visible sample action and its adjacent result description.
- Every public reliance claim is registered in `.factory/claims.json` and has
  an exact tagged regression test. New coverage proves the CLI records an
  external URI without making a request, leaves the input hash unchanged,
  writes its HTML/JSON output, and uses the documented exit behavior.
- Added the missed site-structure requirements: `robots.txt`, `sitemap.xml`,
  designed not-found page, canonical/Open Graph/Twitter/Apple-touch metadata,
  and the common header/footer on legal pages. The social image is a local
  crop of the existing original hero; provenance is recorded in
  `.factory/design.md`.
- The direct demo URL is now precached, so the advertised offline behavior
  works inside the required sample-data sandbox as well as at the landing URL.

The paid Team rollout remains honestly unavailable: there is no checkout,
license storage, billing request, or gated free CLI behavior.

## Local verification

Clean install:

```sh
npm ci
```

Passed with 0 reported npm vulnerabilities.

Ran each registered claim command first, from the built demo entry point:

```sh
npm run build:cli && npm run build:site && node --test --test-name-pattern='@claim:offline-shell' site/tests/claims.test.mjs
npm run build:cli && npm run build:site && node --test --test-name-pattern='@claim:no-web-tracking' site/tests/claims.test.mjs
npm run build:cli && npm run build:site && node --test --test-name-pattern='@claim:local-only-cli' site/tests/claims.test.mjs
npm run build:cli && npm run build:site && node --test --test-name-pattern='@claim:cli-audit-and-ci' site/tests/claims.test.mjs
```

All four passed. The local-only test rewrites the bundled demo URI to a local
HTTP probe, observes zero requests, confirms `Recorded only; never requested`,
and compares the PDF’s SHA-256 before and after audit.

Complete checks passed:

```sh
npm test
npm run lint
npm run build
cargo package -p pdf-link-map --allow-dirty
```

`npm test` passed 2 Rust unit tests, 7 Rust integration tests, 1 doctest, 7
site/build tests, the desktop and 390px Playwright/axe browser suite, and 4
claim tests. The added Rust regression test proves explicit destinations
resolve to page 2. `npm run lint` passed strict TypeScript, rustfmt, and
clippy with `-D warnings`. The production build produced `dist/site/` and
`target/release/pdf-link-map`; the package verifier produced an 8-file,
75.4 KiB source package.

Clean consumer check passed:

```sh
cargo install --path target/package/pdf-link-map-0.1.0 --root <fresh-prefix>
<fresh-prefix>/bin/pdf-link-map --demo --json
```

The installed binary reported `pdf-link-map 0.1.0`; its sample emitted 3
links, 1 external link, and 2 broken conditions. `--demo --fail-on broken`
exited 1 as documented.

`/opt/fleet/lib/verify-url.sh http://127.0.0.1:4178/` passed with one h1,
main, `lang=en`, title, image alt text, labelled buttons, and no console
errors. Browser checks covered desktop and 390px mobile, keyboard Space on
the specimen control, focus visibility, no mobile overflow, serious/critical
axe findings, and reduced motion.

## Deployment and live verification

Deployed the built static artifact with:

```sh
/opt/fleet/lib/deploy-static.sh pdf-link-map dist/site
```

The deploy completed as Static Web App `sf-pdf-link-map` in `eastus2` and the
custom HTTPS URL returned 200. SHA-256 checks matched every tested deployed
file to `dist/site/`: landing, legal pages, 404 page, discovery files, service
worker, favicon/touch/social images, notebook images, CSS, and all JavaScript
chunks.

Live checks passed:

- `/`, `/privacy/`, `/terms/`, `/robots.txt`, and `/sitemap.xml` returned
  200. An unknown path returned the designed Page not found document with a
  404 response; direct `/404` opens that designed document.
- The live CSP is self-only with `frame-ancestors 'none'` and
  `connect-src 'self'`; HSTS is 63,072,000 seconds with preload; X-Frame-
  Options is DENY; nosniff, strict-origin referrer policy, and restrictive
  camera/microphone/geolocation permissions are present.
- `/opt/fleet/lib/verify-url.sh https://pdf-link-map.sociobot.in/` passed:
  200, title, lang, one h1, main, all image alts, labelled buttons, and zero
  console errors.
- Fresh Playwright desktop (1440px) and mobile (390×844) contexts had no page
  overflow, no console errors, no non-origin requests, and zero serious or
  critical axe findings. Keyboard Space rendered the specimen empty state and
  the sample-action focus ring was visible.
- A fresh `?demo=1` context registered and updated its worker; after going
  offline, it reloaded with its demo banner, h1, offline notice, and active
  controller intact. Reduced-motion hero animation measured `1e-05s`.

Lighthouse 13.4.0 mobile emitted a valid JSON report: Performance 100,
Accessibility 100, Best Practices 100, SEO 100; FCP 1.0 s, LCP 1.2 s, TBT
0 ms, CLS 0. The launcher exited 1 after report generation because Chromium
reported `TARGET_CRASHED` during its final browser cleanup; the valid report
and independent Playwright checks are retained as the evidence.

## Known gaps and next steps

No known release-blocking product gaps remain. The optional Team rollout kit
should stay unavailable until the factory registers a real production billing
product; this repair deliberately does not advertise or call an unavailable
checkout. Future paid work must implement the Sociobot billing flow only after
that registration exists.

## How to run

```sh
npm ci
npm test
npm run lint
npm run build
target/release/pdf-link-map --demo --json
```
