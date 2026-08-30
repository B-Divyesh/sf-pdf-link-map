# Independent verification report — FAIL

Work order: `pdf-link-map-verify-3`
Verified: 2026-08-30
Candidate commit: `b187c175f5817f5cfd9f9f3b71180f9c51d7116a`
Live URL: `https://pdf-link-map.sociobot.in/`

## Decision

**FAIL.** Fresh evidence shows that the live deployment is exactly the
candidate build and that the CLI fulfils the researched link-audit job. The
candidate still fails the work order’s mandatory cold first-read gate and the
claims contract. These are release blockers even though the automated tests
pass.

## Required first-read result

Opened the live URL in a fresh Chromium context at desktop size before using
the site. The first screen says:

> Find the links your PDF converter quietly broke.
>
> One local command inventories every link annotation, checks where it lands,
> and leaves a clickable map for reviewers and CI.

It clearly says **what** the tool does and provides a visible one-click **Try
it with sample data** action with the adjacent explanation “Opens a bundled
link-map sample in this page.” Clicking it opened `/?demo=1#demo`, changed the
title to `Demo — PDF Link Map`, and showed the persistent “Demo — sample data,
nothing is saved” banner, Reset demo, and Start for real controls.

It does **not** say, in plain words, that it is for operations and
technical-document teams converting HTML or DOCX to PDF. “Reviewers and CI”
describes an output/use context, not the intended person/team. The work order
states that a first screen missing any of what/for whom/what to click first
fails the candidate. **Finding: High, release-blocking.**

## Claims audit

Ran each listed claim command from the clean checkout before the broader test
suite:

| Claim | Exact test | Result |
| --- | --- | --- |
| Works offline after the first visit | `npm run build:site && node --test --test-name-pattern='@claim:offline-shell' site/tests/claims.test.mjs` | PASS — fresh 390px context visited the built site, worker became ready, offline reload retained controller/h1 and showed Offline. |
| The website has no telemetry | `npm run build:site && node --test --test-name-pattern='@claim:no-web-tracking' site/tests/claims.test.mjs` | PASS — fresh context recorded no non-origin requests. |

The tests are valid, but `.factory/claims.json` has only those two entries.
Live and README copy make additional reliance claims without a corresponding
`@claim:<id>` demo test, including:

- “Your PDF never leaves your machine. External addresses are listed, never
  opened.”
- “Internal named + explicit destinations”; “Output HTML + JSON + exit codes”.
- README: “It never opens external URLs and never modifies the input PDF”.
- README: the stated stable exit-code contract and validation behaviour.

The claims rules require every visitor-reliance claim to be listed and tested,
and specifically direct verifiers to fail unlisted claims found on the landing
page or README. **Finding: High, release-blocking.**

## Quality gates from clean checkout

All completed successfully, in this order after the mandatory claim tests:

```sh
npm ci
npm test
npm run typecheck
npm run lint
npm run build
cargo package -p pdf-link-map --allow-dirty
```

`npm test` ran the Rust workspace suite, static-site tests, browser/PWA test,
and both claims. Strict TypeScript, rustfmt, and clippy passed. The build
produced `target/release/pdf-link-map` and `dist/site/`; package output was
present at `target/package/pdf-link-map-0.1.0`.

### Independent packed-consumer CLI exercise

Installed with:

```sh
cargo install --path target/package/pdf-link-map-0.1.0 --root <fresh-temp-prefix>
```

The installed public binary reported `pdf-link-map 0.1.0`. Its `--help`
describes the PDF input, report path, manifest, JSON, failure policy, and
local/no-external-request policy. `--demo --json` created a temporary input,
manifest, and actual standalone HTML report; JSON reported 3 links, 1
external link, and broken/missing-manifest findings. `--demo --fail-on broken`
returned 1. A malformed non-PDF returned 2 with a parse error. Repository
integration coverage also passed for normal/broken destination handling,
duplicate destinations, manifests, no annotations, overwrite refusal, and
the documented JSON CI command.

## Live deployment identity and behaviour

Fresh SHA-256 comparisons of local `dist/site/` to the live response matched
for `index.html`, `privacy/index.html`, `terms/index.html`, `sw.js`, main JS,
both other JS chunks, CSS, and `link-map-notebook.webp`. The live deployment
therefore matches the candidate.

Fresh live Playwright evidence:

- Desktop 1440px and mobile 390 × 844px: one h1; mobile document
  `clientWidth = scrollWidth = 390`; no page/console errors.
- Keyboard: the sample link takes focus with a designed `rgb(31, 88, 114)
  solid 3px` outline; Space selected “No annotations” and rendered its empty
  result. The skip link navigated to `#main` and the next Tab landed on the
  first main action.
- Demo: clicking the first-screen sample action set `?demo=1#demo`, displayed
  the required banner, and retained the specimen state controls.
- `prefers-reduced-motion: reduce`: hero animation duration was `1e-05s`.
- `@axe-core/playwright` had zero serious or critical issues on both viewports.
- Normal visits sent only same-origin requests (document, JS/CSS, self-hosted
  WebP); no analytics or third-party requests were observed.
- `/opt/fleet/lib/verify-url.sh https://pdf-link-map.sociobot.in <existing-temp-dir>`
  passed: HTTP 200; title; `lang=en`; one h1; main; all image alts; labelled
  buttons; no console errors.

### PWA and HTTP policy

A new live context observed one root service-worker registration and an active
controller. `registration.update()` succeeded. After going offline, reload
returned 200 from the worker, retained the controller and h1, and showed
“Offline — docs and specimen remain available.”

The landing and legal responses returned 200 with a self-only enforcing CSP
including `frame-ancestors 'none'` and `connect-src 'self'`, `X-Frame-Options:
DENY`, HSTS `max-age=63072000; includeSubDomains; preload`, nosniff, strict
referrer policy, and camera/microphone/geolocation permissions disabled.
Hashed JS used `public, max-age=31536000, immutable`; landing HTML uses
30-second revalidation. This static product exposes no server-side product or
billing endpoints, so a 429/Retry-After allowance test does not apply.

Total built JS is 4,252 bytes; CSS is 12,770 bytes; the 640px hero is 29,348
bytes. All are within the stated static budgets.

## Additional contract findings

**Medium — required discovery/error assets are absent.** `/robots.txt`,
`/sitemap.xml`, and `/404` each returned 404 on the live deployment. The
site-structure contract requires robots and sitemap plus a designed 404 route.
The legal pages also do not carry the required consistent header/footer, and
the site lacks canonical, Open Graph/Twitter-card, and Apple-touch metadata.

## Remediation before rerun

1. Rewrite the first-screen supporting sentence to explicitly name operations
   and technical-document teams converting HTML/DOCX to PDF, while preserving
   the visible sample action and adjacent outcome.
2. Either add one observable, independently runnable demo test for every
   public product/privacy/CLI claim and register it in `.factory/claims.json`,
   or remove the unsupported claim from all landing and README copy.
3. Add the missing discovery/404 assets and route/metadata skeleton items.
4. Re-run the claim commands first, then the full local and live verification.
