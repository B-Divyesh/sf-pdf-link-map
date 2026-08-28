# Verification report — FAIL

Work order: `pdf-link-map-verify-1`
Verified: 2026-08-28
Candidate: `a20cc8ad8bc18067cd68237f25bf399553e8fc9d`
Live URL: `https://pdf-link-map.sociobot.in/`

## Decision

**FAIL.** The core local CLI is release-ready, but the public deployment fails two required live behaviours: the advertised Team checkout is not available, and the PWA does not automatically register a service worker in a fresh browser. These are deployment/release blockers. No product source was modified during verification.

## Blocking defects

### High — advertised Team checkout cannot be used

The live candidate's **Buy Team unlock** link is `https://pilot-api.sociobot.in/api/v1/products/pdf-link-map/checkout`, not the required production Sociobot API host. A fresh `HEAD` request on 2026-08-28 returned **404** from that exact pilot endpoint. The production equivalent also returned 404, so no registered checkout is currently available. This prevents the advertised $29 one-time unlock and violates the release billing contract.

### High — live PWA service worker does not auto-register; offline/update requirement fails

In a fresh Chromium context against the live URL, after `load` plus five seconds, `navigator.serviceWorker.getRegistrations()` returned `[]`; no request for `/sw.js` occurred. `navigator.serviceWorker.ready` was still unresolved after 15 seconds. Consequently a normal first visit has no controller and cannot satisfy offline reload or update behavior. Manually calling `navigator.serviceWorker.register('/sw.js')` did create the `pdf-link-map-v1` cache, which isolates the problem to automatic live registration/activation rather than browser support. The local browser test passed because its local static-server scenario does register and cache the worker; this is therefore a live deployment discrepancy.

## Additional finding

### Medium — public response policy lacks CSP and clickjacking protection

The live HTML response has no `Content-Security-Policy`, `X-Frame-Options`, or CSP `frame-ancestors` directive. It also uses HSTS `max-age=10886400` (126 days), below the one-year/preload-strength policy normally expected for a public HTTPS product. The response does correctly set `nosniff`, `strict-origin-when-cross-origin`, a restrictive camera/microphone/geolocation `Permissions-Policy`, and HTTPS/HSTS. Lighthouse's security diagnostics likewise reported no enforcing CSP and low HSTS max-age.

## Passed evidence

### Clean build and package gates

- Started from a clean checkout at exactly `a20cc8ad8bc18067cd68237f25bf399553e8fc9d`.
- `npm ci`: passed; 0 audit vulnerabilities.
- `npm test`: passed: 2 Rust unit tests, 5 Rust integration tests, 1 compiling doctest, 3 site/build tests, and the bundled Playwright browser/PWA test.
- `npm run build`: passed. It produced `target/release/pdf-link-map` and `dist/site/`.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo package -p pdf-link-map --allow-dirty`: passed and verified `pdf-link-map-0.1.0` (8 files; 69.4 KiB source package).

### Clean-consumer CLI exercise

Installed the packaged crate into an empty temporary consumer prefix with:

```sh
CARGO_TARGET_DIR=/work/repo/target cargo install --path target/package/pdf-link-map-0.1.0 --root <clean-prefix>
```

The installed `pdf-link-map --help` documents the single binary, JSON output, fail thresholds, stable exit behavior, and non-network/non-modification guarantees. A separately generated two-page PDF contained one valid named destination, one broken named destination, one external URI, and a duplicate name. Results:

- Normal audit with a valid manifest: exit `0`; standalone 3,161-byte HTML report and 1,761-byte JSON report created.
- `--fail-on broken` with an expected bad manifest: exit `1`; JSON reported 3 links (1 valid internal, 1 broken internal, 1 external), plus duplicate-destination, broken-link, page-mismatch, and missing-anchor findings.
- Invalid non-array manifest: exit `2`; no report written.
- Report path equal to the input PDF: exit `2`; source was refused.
- Generated report at 390 px: one `h1`, `lang=en`, `main`, no serious/critical axe findings, no console errors, and no automatic outbound requests. The external URI is recorded as a `rel=noreferrer` link; it is never fetched during auditing or report load.

### Live candidate identity, privacy, UI, accessibility, and performance

- Live `/`, main JS, CSS, and 640 px hero asset SHA-256 hashes exactly equal the fresh candidate build. The live index is 9,987 bytes, `main-DJV3LUkb.js` 5,537 bytes, `style-CT0uKtjr.css` 11,708 bytes, and mobile hero 29,348 bytes. These are below the 200 KB JS, 50 KB CSS, and 300 KB mobile-image budgets; there are no shipped font files.
- Live response and legal routes `/privacy/` and `/terms/` returned HTTP 200. Hashed assets return `cache-control: public, max-age=31536000, immutable`; the service worker returns `no-cache`; WebP uses one-day caching.
- Fresh live Playwright checks at 1440 px and 390 × 844: one `h1`, `main`, `lang=en`, keyboard operation of the specimen's empty state, correct `aria-pressed` state, no page/console/request failures, and no cross-origin request on a normal free visit. The skip link and keyboard-selected specimen control each had a visible 3 px blue focus outline. Reduced motion resolved animation and transition duration to `0.01ms`.
- Axe found zero serious or critical findings at both sizes. Visual inspection found the notebook layout legible and intentionally stacked at 390 px, with no overlap or clipped controls.
- Lighthouse 12.8.2 mobile: Performance 96, Accessibility 100, Best Practices 100, SEO 100; FCP 2.2 s, LCP 2.2 s, TBT 40 ms, CLS 0. Lighthouse emitted scores despite a final headless full-page-screenshot target crash; the direct Playwright console/error checks above were clean.

## Required next steps

1. Register the production product and deploy with `VITE_BILLING_API_BASE=https://api.sociobot.in`; verify that the public checkout returns a redirect rather than 404, and that a return token verifies/restores correctly.
2. Fix and re-test live automatic service-worker registration, activation, update, and first-visit offline reload. Test against the public deployment, not only the local static server.
3. Add an enforcing Content Security Policy with an appropriate `frame-ancestors` policy, frame protection, and a stronger HSTS max-age; recheck headers on the public URL.
4. Re-run this verification after deployment and retain the current CLI/package evidence as the non-regressed baseline.
