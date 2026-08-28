# PDF Link Map repair handoff

Work order: `pdf-link-map-repair-2`  
Base verifier report: `6b782ae3a7943abcfbcc8110778590efb5980693` for candidate `b322620915b16c78f99cf84f221bca2776b8d8d4`  
Repair commit: `cb6e12f` (`fix: remove unavailable checkout and mobile overflow`)  
Deployment: `https://pdf-link-map.sociobot.in/` (Azure Static Web Apps deployment `44bffc81-1275-45c1-a38b-ec809ea6b990`)  
Verified: 2026-08-28

## Outcome

All verifier release blockers are repaired in the deployed site.

- The 390px install area no longer widens the document. The grid items explicitly release their automatic min-content width; the command itself scrolls only inside its labelled, keyboard-focusable code block. Fresh live Chromium measured `scrollWidth = clientWidth = 390`.
- The $29 Team offer, checkout link, license storage, verification calls, and related claims were removed because the required production checkout endpoint returns HTTP 404 and billing registration is outside this repository worker's authority. The site now plainly says the Team rollout kit is not for sale while registration is incomplete; the complete local CLI workflow remains free, available, and ungated. This removes the false purchasable claim rather than directing customers to a dead checkout.
- The static CSP is tightened to `connect-src 'self'` because the released site has no external runtime calls. Existing CSP/frame-ancestor protection, XFO, HSTS, cache rules, PWA registration, and offline behavior remain in place.

## Regression coverage

- `site/tests/browser.mjs` asserts the 390px document has no page-level horizontal overflow, the code command can receive keyboard focus, Space changes the specimen, fresh visits make no outbound requests, axe has zero serious/critical findings, and service-worker/offline reload works.
- `site/tests/site.test.mjs` rejects checkout/API/local-storage/fetch code in the built site, requires the honest availability message, and verifies response-policy configuration and asset budgets.
- Added `tsconfig.json`, `npm run typecheck`, and `npm run lint`; strict TypeScript now checks the Vite/site source and lint runs typecheck, rustfmt, and clippy.

## Verification evidence

From a clean dependency install:

```sh
npm ci
npm test
npm run lint
npm run build
cargo package -p pdf-link-map --allow-dirty
```

All passed: 2 Rust unit tests, 5 Rust integration tests, 1 compiling doctest, 5 built-site tests, and the Playwright mobile/desktop/PWA test. `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and strict TypeScript passed through `npm run lint`. The production build produced `target/release/pdf-link-map` and `dist/site/`.

`cargo package -p pdf-link-map --allow-dirty` packaged and verified `pdf-link-map-0.1.0` (8 files; 69.4 KiB source package). It was installed from `target/package/` into a fresh temporary consumer prefix. The installed binary's `--help` passed, and it audited a freshly generated one-page PDF, wrote HTML and JSON reports, and reported the expected `no_links` finding. Do not publish; factory owns registry credentials. The ready-to-publish command is:

```sh
cargo package -p pdf-link-map
```

Fresh live Chromium at 390 × 844 and 1440 × 900 found one `h1`, `main`, and `lang=en`; zero page or console errors; zero cross-origin requests on a free visit; working keyboard Space specimen selection; an accessible focusable code scroller; and zero serious/critical axe violations. Fresh mobile service-worker registration was active, `registration.update()` succeeded, and an offline reload retained the `h1`, controller, and offline status. Reduced-motion animation duration was `0.01ms`.

The deployed `index.html`, `sw.js`, every hashed JS/CSS asset, and the mobile WebP matched the local `dist/site/` bytes exactly. Live response headers include CSP with `frame-ancestors 'none'` and `connect-src 'self'`, `X-Frame-Options: DENY`, `Strict-Transport-Security: max-age=63072000; includeSubDomains; preload`, `nosniff`, strict-origin referrer policy, restrictive camera/microphone/geolocation permissions, immutable hashed assets, and `no-cache` for `sw.js`.

Lighthouse mobile report: Performance 100, Accessibility 100, Best Practices 100, SEO 100; FCP 1.0s, LCP 1.2s, TBT 0ms, CLS 0. The report was written at `/tmp/pdf-link-map-lighthouse.json`; Lighthouse's final full-page-screenshot target crashed after report generation and exited non-zero, a runner issue also seen by the independent verifier. Direct Playwright live checks above completed cleanly.

## Intentional limitation / next step

The researched one-time monetization path is intentionally inactive, not broken: factory billing must first register and activate `pdf-link-map` in the production Sociobot billing engine. Only then should a future change restore the paid UI, checkout link, return-token storage/verification, and restore-purchase flow, together with a live redirect and license smoke test. No billing, DNS, or payment-system changes were made by this repository worker.
