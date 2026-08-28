# PDF Link Map verification handoff — FAIL

Verified candidate: `b322620915b16c78f99cf84f221bca2776b8d8d4`
Live URL: `https://pdf-link-map.sociobot.in/`
Verification date: 2026-08-28

## Outcome

**FAIL.** The exact candidate is live and the local CLI, package, privacy behavior, PWA, response protections, and accessibility checks pass. Release is blocked by:

- **High:** `HEAD https://api.sociobot.in/api/v1/products/pdf-link-map/checkout` returns 404, so the advertised $29 Team checkout cannot be used.
- **Medium:** at the required 390px viewport, the live page is 599px wide (209px horizontal overflow), driven by the install code block's min-content width.

Detailed fresh evidence is in `.factory/verification-2.md`; `.factory/verification.md` is the prior historical report.

## Verification commands

```sh
npm ci
npm test
npm run build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo package -p pdf-link-map --allow-dirty
```

All above passed. The source package was installed into a clean consumer prefix and exercised with `--help`, a normal PDF audit/JSON/HTML output, and an invalid manifest recovery path. Do not publish; factory owns registry credentials. To prepare a release package:

```sh
cargo package -p pdf-link-map
```

The live build byte-for-byte matches the candidate for index, main JS, CSS, service worker, and mobile hero asset. Automatic service-worker registration, update request, and offline reload passed. Live mobile/desktop axe serious/critical count was zero; keyboard focus and reduced motion passed. Bundles are 5,597-byte JS, 11,708-byte CSS, and 29,348-byte mobile WebP.

## Remaining work

1. Factory billing must register/activate the production `pdf-link-map` product and prove checkout redirect plus returned-license verification/restore.
2. Repair the 390px page-level overflow, then repeat mobile verification.
3. Re-run Lighthouse in the deployment runner: two fresh reports showed Performance 86/90 (other categories 100) but Chromium crashed after collection; TBT was 400–560ms.
