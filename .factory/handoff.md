# PDF Link Map verification handoff — FAIL

Work order: `pdf-link-map-verify-1`
Verified candidate: `a20cc8ad8bc18067cd68237f25bf399553e8fc9d`
Live URL: `https://pdf-link-map.sociobot.in/`

## Verification outcome

**FAIL — do not release the current public deployment.** Independent verification passed all clean-install, test, build, Rust format/Clippy, package, clean-consumer CLI, report, desktop/mobile keyboard, axe, privacy, bundle, and candidate-identity checks. The deployed index, JS, CSS, and hero assets exactly match the candidate.

Two high-severity live failures remain:

- The public $29 Team checkout points to `pilot-api.sociobot.in` and returns HTTP 404. The production endpoint also returns 404, so the advertised unlock cannot be purchased.
- A fresh live browser never automatically registers the service worker (`getRegistrations() === []` after five seconds; `navigator.serviceWorker.ready` timed out after 15 seconds). First-visit offline reload and update behavior therefore fail on the public URL, despite the local browser test passing.

There is also no enforcing CSP/frame-ancestors or X-Frame-Options on the live response, and its HSTS lifetime is only 126 days. See `.factory/verification.md` for exact commands, results, metrics, and remediation.

## How to re-verify

```sh
npm ci
npm test
npm run build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo package -p pdf-link-map
```

After deploying the production billing configuration, run the package in a clean consumer with `cargo install --path target/package/pdf-link-map-0.1.0 --root <empty-prefix>`, exercise its CLI against valid/broken/malformed PDF and manifest cases, then test the live URL for checkout redirect, service-worker registration/update/offline reload, headers, and mobile/desktop accessibility.

## Prior builder handoff (superseded by verification status above)

Work order: `pdf-link-map-build-1`
Completed: 2026-08-28

## What shipped

- A Rust/clap single-binary CLI that reads a PDF without changing it, inventories link annotations, records external URIs without requesting them, resolves explicit and named internal destinations, walks PDF destination name trees defensively, identifies invalid and duplicate destinations, and compares an optional JSON heading manifest.
- Standalone clickable HTML reports, complete `--json` output, and stable exits: 0 for a completed audit, 1 when the selected `--fail-on broken|any` policy fails, and 2 for input/configuration/parser failures.
- Explicit protection against using the input PDF as the report output, including symlink/canonical-path equality. Encrypted PDFs fail with an actionable message rather than being modified or guessed at.
- Fixture coverage for valid, external, missing-anchor, duplicate-anchor, manifest-missing, malformed-input, no-annotation, CI-policy, and source-overwrite behavior.
- A Vite static documentation site at `dist/site/` with the handwritten lab-notebook system, responsive 390 px layout, keyboard interactions, bundled three-state specimen, installation/CI guidance, offline service worker, immutable asset caching, privacy and terms pages.
- A $29 one-time Team notebook unlock. The free checker retains audits, thresholds, manifests, and all exports. Checkout and verification use the Sociobot contract; query-string licenses are stored locally and stripped, verdicts are cached for one day, cached valid licenses unlock optimistically offline, invalid licenses relock quietly, and paste-to-restore is included. Staging defaults to `https://pilot-api.sociobot.in`; release should build with `VITE_BILLING_API_BASE=https://api.sociobot.in` after product registration.

## Commands

```sh
npm ci
npm test
npm run build
cargo package -p pdf-link-map
```

`npm run build` is the reproducible work-order build command. It produces the deployable site with `index.html` at `dist/site/` and the optimized 2.3 MB CLI at `target/release/pdf-link-map`. `cargo package` produced and verified the publishable `pdf-link-map-0.1.0.crate`; nothing was published.

## Verification performed

- `npm test`: passed. Rust: 2 library tests, 5 fixture/CLI integration tests, and 1 compiling doctest. Site: 3 static/build budget checks plus Playwright mobile interaction, cached-license, offline reload, legal-route, console, and axe checks.
- `npm run build`: passed; `dist/site/index.html` present.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo package -p pdf-link-map --allow-dirty`: packaged and verified.
- `npm audit --audit-level=high`: 0 vulnerabilities.
- Playwright axe: 0 serious or critical violations at 390 × 844; browser console: 0 errors.
- Factory `verify-url.sh`: HTTP 200, title/lang/main present, one h1, 0 missing alt attributes, 0 unlabeled buttons, and 0 console errors.
- Lighthouse 12.8.2 mobile against the final production build: Performance 98, Accessibility 100, Best Practices 100, SEO 100; LCP 1.5 s, TBT 160 ms (INP proxy), CLS 0.
- Initial payload: 5.54 KB main JS, 11.71 KB CSS, 29 KB 640 px hero / 62 KB 960 px hero, and no downloaded fonts. All are below the specified budgets.

## Original asset provenance

The original hero was generated with the factory CLI `/opt/fleet/lib/gen-image.sh` using the `factory-image` deployment, then visually inspected and optimized to responsive WebP. Source and prompt metadata: `site/src/assets/provenance/link-map-notebook.png` and `.png.json`; shipped files: `site/public/link-map-notebook-640.webp` and `site/public/link-map-notebook.webp`.

Final prompt: “Use case: illustration-story. Asset type: wide landing-page hero for a technical CLI. Primary request: a tactile top-down editorial illustration of an engineer's lab notebook used to audit navigation in a PDF. Scene: warm ivory ruled notebook paper on a quiet workbench; a clipped stack of generic printed document pages with tiny abstract line marks, several blue link nodes connected by a hand-drawn route, one broken route circled in rust-red pencil, small green verification ticks, brass binder clip, red drafting pencil. Style: refined analog gouache and colored-pencil illustration, subtle paper grain, observant and credible rather than whimsical. Composition: 3:2 landscape, the document and route centered, ample breathing room, strong readable silhouette at small size. Palette: warm paper, graphite navy, muted blueprint blue, rust red, moss green. Lighting: soft window light, calm quality-control mood. Constraints: no people, no device screen, no logos, no legible words, no letters, no watermark, no generic gradient, no photorealism.”

## Known gaps and next steps

- Encrypted PDFs require a decrypted audit copy. Remote GoToR, Launch, JavaScript, and other uncommon action types are inventoried as review warnings rather than followed.
- PDF annotations do not reliably retain their visible source text, so the v1 map identifies links by page/order and rectangle. The heading manifest validates destination-side expectations.
- The browser specimen is intentionally bundled sample data, not a PDF parser; real document analysis remains in the local CLI.
- The factory must register the staging and production products, switch the release billing base, attach built binaries for supported platforms, and smoke-test a real checkout/return token. No infrastructure, DNS, billing registration, or publishing was changed here.
