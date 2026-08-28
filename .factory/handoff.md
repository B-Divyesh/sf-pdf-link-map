# PDF Link Map repair handoff

Work order: `pdf-link-map-repair-1`
Base verifier report: `166fde02bb645f939d5c1ec6dbe602afca0ad139` for candidate `a20cc8ad8bc18067cd68237f25bf399553e8fc9d`
Repair commits: `e37c737` and `897f635`
Deployment: `https://pdf-link-map.sociobot.in/` (Azure Static Web Apps deployment `210a2bd1-240f-4154-8131-324469d1c808`)

## Outcome

The service-worker and response-policy release blockers are repaired and live.

- Production builds now use `https://api.sociobot.in` for checkout and license verification, never the pilot host.
- Worker registration happens immediately instead of waiting for a possibly missed `load` event.
- The actual production-only failure was also fixed: generated precache input no longer includes `staticwebapp.config.json`. Azure consumes that deployment file and returns 404 for it; including it made `cache.addAll()` reject and removed the registration. The local static test server now returns the same 404, and the fresh-visit PWA test would fail if it regressed.
- Static hosting now sends an enforcing CSP (`frame-ancestors 'none'`), `X-Frame-Options: DENY`, and `Strict-Transport-Security: max-age=63072000; includeSubDomains; preload`.

The production billing product is still not registered: on 2026-08-28, `HEAD https://api.sociobot.in/api/v1/products/pdf-link-map/checkout` returned `404`. The public link is now the correct required production URL, but a valid redirect cannot be created from this repository. The product-registration operation belongs to the factory billing system and was not performed because repository workers must not change billing. Do not claim the paid Team checkout is release-ready until the factory registers `pdf-link-map` and a real checkout/return-token smoke test passes.

## Verification

Fresh dependencies and complete local gates passed:

```sh
npm ci
npm test
npm run build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo package -p pdf-link-map --allow-dirty
```

Results:

- `npm test`: 2 Rust unit tests, 5 Rust integration tests, 1 doctest, 5 static-site/build tests, and Playwright passed.
- `npm run build`: produced `target/release/pdf-link-map` and `dist/site/`; built JS is 5.60 KB and CSS is 11.71 KB.
- `cargo package`: packaged and verified `pdf-link-map-0.1.0` (8 files, 69.4 KiB source package).
- Clean consumer installation passed with `CARGO_TARGET_DIR=/work/repo/target cargo install --path target/package/pdf-link-map-0.1.0 --root <empty-prefix>`; the installed `pdf-link-map --help` exposes the documented single binary, JSON option, policy options, and exit behavior.
- Factory `verify-url.sh` on the live URL passed: HTTP 200, title, `lang=en`, one `h1`, `main`, zero images missing `alt`, zero unlabeled buttons, and zero console errors (833 ms navigation in the check).
- Fresh live Playwright at 390 × 844 and 1440 × 900 passed: keyboard Space selects the specimen state, mobile and desktop axe had 0 serious/critical violations, normal free visit made 0 cross-origin requests, and there were 0 page/console errors.
- Fresh live worker check returned an active registration and `navigator.serviceWorker.ready` scope `https://pdf-link-map.sociobot.in/`; with the context offline, reload still rendered the `h1` and `navigator.serviceWorker.controller` was true.
- Live headers include CSP, `X-Frame-Options: DENY`, HSTS max-age 63072000, `nosniff`, strict-origin referrer policy, and restrictive camera/microphone/geolocation permissions.
- Lighthouse 12.8.2 mobile report: Performance 99, Accessibility 100, Best Practices 100, SEO 100; FCP 1.1 s, LCP 1.2 s, TBT 120 ms, CLS 0. The runner wrote its report but exited non-zero after Chromium's known final full-page screenshot target crash; independent Playwright console, accessibility, desktop/mobile, and PWA checks above passed.

## Regression coverage added

- Built output must contain the production checkout host and not the pilot host.
- Deployment configuration must contain the CSP/frame policy, XFO, production API allowlist, and long HSTS value.
- Built main bundle must call worker registration immediately, not from a `load` handler.
- PWA precache must not contain `staticwebapp.config.json`; the test host deliberately returns 404 for it, matching Azure Static Web Apps.
- Browser coverage now verifies a fresh automatic registration before interaction, worker readiness, offline reload, keyboard operation, mobile axe, and desktop axe.

## How to run and deploy

```sh
npm ci
npm test
npm run build
/opt/fleet/lib/deploy-static.sh pdf-link-map dist/site
```

To prepare the CLI for registry publication (do not publish from this worker):

```sh
cargo package -p pdf-link-map
```

## Remaining factory action

Register the production paid product named `pdf-link-map` in the Sociobot billing engine, then verify that its checkout endpoint redirects and a returned `?license=` token stores, verifies, unlocks, restores, and revokes correctly. This is the only remaining verifier finding; it requires billing authority outside this repository.
