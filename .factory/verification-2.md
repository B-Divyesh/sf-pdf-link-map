# Independent verification report — FAIL

Work order: `pdf-link-map-verify-2`
Verified: 2026-08-28
Candidate commit: `b322620915b16c78f99cf84f221bca2776b8d8d4`
Live URL: `https://pdf-link-map.sociobot.in/`

## Decision

**FAIL.** Fresh evidence confirms that the repaired candidate is deployed and that the core local CLI works. Two release defects remain: the advertised paid checkout is unavailable, and the live site has severe horizontal overflow at the required 390px viewport. No product source code was changed during this verification.

## Defects

### High — production Team checkout is unavailable

The deployed purchase link correctly targets the required production URL, `https://api.sociobot.in/api/v1/products/pdf-link-map/checkout`, but a fresh `HEAD` request returned **HTTP 404** (2026-08-28). It therefore cannot redirect a customer to checkout. The published site advertises a $29 one-time Team unlock, so this fails the paid-unlock contract. This is a factory billing-registration/deployment dependency, but it is still a release blocker.

### Medium — 390px mobile layout horizontally overflows by 209px

Fresh Chromium at a 390 × 844 viewport measured `documentElement.clientWidth = 390` and `scrollWidth = 599`; `document.documentElement.scrollWidth <= innerWidth` was false. The install section's computed grid track is 366px, but its code-note child expands to 587.4px because the long command's min-content width escapes the single-column mobile grid. The specimen table also has a deliberate 600px width, but its own scroll region is separate; the page-level overflow comes from the install code area. This violates the required mobile fit and makes the primary install guidance require sideways page scrolling.

### Low / environment-qualified — Lighthouse result was not stable

Two fresh mobile Lighthouse 13.4.1 runs wrote reports but each ended non-zero after Chromium's final full-page-screenshot target crash. Reported scores were 86 and 90 Performance; Accessibility, Best Practices, and SEO were 100 in both. The reports measured FCP 1.0–1.2s, LCP 1.3s, CLS 0, and TBT 400–560ms. Treat the TBT/first-run performance budget miss as needing a repeat in the deployment runner, rather than as a standalone product defect, because the browser crashed after artifact collection. Direct Playwright desktop/mobile checks had no page or console errors.

## Passed verification

### Clean checkout and quality gates

- The checkout started clean at exactly the candidate SHA.
- `npm ci` passed with 0 vulnerabilities.
- `npm test` passed: 2 Rust unit tests, 5 Rust integration tests, 1 doctest, 5 static-site tests, and the Playwright site/PWA browser test. A direct repeat of `node site/tests/browser.mjs` also passed.
- `npm run build` passed and created `target/release/pdf-link-map` plus `dist/site/`.
- `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings` passed.
- `cargo package -p pdf-link-map --allow-dirty` passed and verified `pdf-link-map-0.1.0` (8 files, 69.4 KiB source package).

### Clean-consumer CLI checks

- Installed the packaged crate into an empty prefix with `cargo install --path target/package/pdf-link-map-0.1.0 --root <empty-prefix>`; it installed the documented single `pdf-link-map` binary.
- Its `--help` documents the local-only guarantee, report path, manifest, `--json`, `--fail-on`, and stable options.
- A fresh valid one-page PDF made by Chromium exercised the normal consumer path: exit 0, JSON report, standalone HTML map, and the actionable `no_links` warning.
- An invalid empty manifest returned the documented parse error. The repository integration suite independently exercised valid/broken named destinations, an external URI, duplicate names, `--fail-on broken` exit 1, malformed PDF exit 2, manifest page/anchor findings, and refusal to overwrite the source PDF.
- The source and report path are local; normal auditing never requests external URIs. The test fixture verifies external targets are recorded rather than dereferenced.

### Live deployment, privacy, PWA, and response policy

- Fresh SHA-256 comparisons exactly matched local candidate build output for `index.html`, `main-CdEh2TOR.js`, `style-CT0uKtjr.css`, `sw.js`, and `link-map-notebook-640.webp`.
- Live `/`, `/privacy/`, `/terms/`, and `/sw.js` returned 200. The production bundle has no pilot billing host.
- Headers include an enforcing self-only CSP with `frame-ancestors 'none'`, `X-Frame-Options: DENY`, HSTS `max-age=63072000; includeSubDomains; preload`, `nosniff`, strict-origin referrer policy, and restrictive camera/microphone/geolocation permissions. Main hashed JS uses `public, max-age=31536000, immutable`; `sw.js` uses `no-cache`; WebP uses one-day caching.
- Built payloads are within static budgets: main JS 5,597 bytes, CSS 11,708 bytes, mobile WebP 29,348 bytes, and no shipped web fonts.
- A normal fresh visit made no cross-origin requests and produced no page or console errors. A deliberately supplied license token was stored, stripped from the URL, sent only to the specified Sociobot verification endpoint, then recovered to a locked quiet status after the invalid response; the free experience remained available.
- Fresh mobile and desktop Chromium checks found one `h1`, `main`, `lang=en`, a 3px visible focus outline, keyboard Space operation of the specimen control, zero serious/critical axe findings, and reduced-motion transition/animation duration of `0.01ms`.
- Service worker registration was automatic and active at the live root. `navigator.serviceWorker.ready`, `registration.update()`, and controller checks passed. After first visit, an offline reload rendered the `h1`, retained a controller, and stated the offline status.

## Required next steps

1. Register/enable the `pdf-link-map` product in the production Sociobot billing engine, then verify a real checkout redirect and return-token verification/restore flow on the public site.
2. Correct the mobile grid/min-width behavior so the document does not exceed 390 CSS px; rerun desktop and 390px visual/keyboard/a11y checks.
3. Repeat Lighthouse in the deployment runner after the mobile repair and investigate the 400–560ms TBT and runner target crash if they persist.
