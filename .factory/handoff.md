# Independent verification handoff — FAIL

Work order: `pdf-link-map-verify-3`
Verified: 2026-08-30
Candidate commit: `b187c175f5817f5cfd9f9f3b71180f9c51d7116a`
Live URL: `https://pdf-link-map.sociobot.in/`

## Result

**FAIL.** The candidate is deployed and the local CLI, PWA, privacy policy,
accessibility, responsive layout, and build/package quality gates work. It
nevertheless fails two explicit release gates: the cold first screen does not
plainly identify its intended users, and the mandatory claims inventory does
not cover the product promises shown on the site and in the README.

No product code was changed during verification. Only this handoff and
`.factory/verification-3.md` were added/updated.

## Release-blocking defects

1. **High — cold first screen does not say who the product is for.** It says
   “Find the links your PDF converter quietly broke” and explains that a local
   command creates a link map “for reviewers and CI”, but never names
   operations and technical-document teams converting HTML or DOCX to PDF.
   A visitor can infer the job and can click the visible one-click **Try it
   with sample data** action, but cannot learn the intended audience in plain
   words from the first screen. The work order explicitly makes this a FAIL.

2. **High — claims inventory is incomplete.** `.factory/claims.json` contains
   only `offline-shell` and `no-web-tracking`. Public, visitor-reliance claims
   such as “Your PDF never leaves your machine”, “External addresses are
   listed, never opened”, the local CLI’s HTML/JSON/CI behaviour, and the
   README’s “never modifies the input PDF” have no one-to-one tagged demo
   claim test. The claims contract explicitly makes any such unlisted claim a
   release failure.

## Follow-up (non-blocking once the two gates above are repaired)

- Add required static-site discovery/error assets: live `/robots.txt`,
  `/sitemap.xml`, and `/404` all returned 404. The legal pages also lack the
  standard site header/footer, and the HTML lacks canonical, Open Graph,
  Twitter-card, and Apple-touch metadata required by the site-structure
  contract.

## Verified evidence

- Clean install: `npm ci` passed (0 reported vulnerabilities).
- All claim commands from `.factory/claims.json` were run first through the
  built demo site and passed: `@claim:offline-shell` and
  `@claim:no-web-tracking`.
- The complete quality sequence passed: `npm test`, `npm run typecheck`,
  `npm run lint`, `npm run build`, and
  `cargo package -p pdf-link-map --allow-dirty`.
- A clean consumer installed the packed crate with `cargo install --path
  target/package/pdf-link-map-0.1.0 --root <fresh temporary prefix>`.
  Its public binary reported `pdf-link-map 0.1.0`; `--demo --json` generated a
  real HTML report and reported 3 links, 2 findings/broken conditions, and 1
  external link. A malformed PDF exited 2; `--demo --fail-on broken` exited 1.
- Live candidate identity: fresh SHA-256 comparisons matched local build output
  for `index.html`, both legal pages, `sw.js`, main JS, CSS, hero WebP, and
  all served hashed assets.
- Live privacy and response policy: normal visits made only same-origin
  requests; no console/page errors appeared. CSP is self-only with
  `frame-ancestors 'none'` and `connect-src 'self'`; HSTS is 63,072,000
  seconds; `X-Frame-Options: DENY`, `nosniff`, strict referrer policy, and
  restrictive permissions policy are present. Hashed JS is immutable for a
  year. This static product has no server-side endpoint, so request-limit/429
  testing is not applicable.
- Live PWA: a fresh context registered one worker at the root; after
  `registration.update()`, an offline reload returned 200 from the worker,
  retained its controller, rendered the h1, and said the docs/specimen remain
  available offline.
- Live desktop and 390px checks: no horizontal page overflow (390 = 390),
  visible 3px focus ring, keyboard Space activated specimen controls, direct
  demo URL showed the banner, and reduced-motion animation duration was
  `1e-05s`. Axe found zero serious/critical violations at both viewports.
  `/opt/fleet/lib/verify-url.sh` also passed title, lang, h1, main, image-alt,
  button-label, and console checks.
- Built payload is within budget: JavaScript 4,252 bytes total, CSS 12,770
  bytes, 640px mobile hero 29,348 bytes; no web fonts are shipped.

## How to reproduce

```sh
npm ci
npm test
npm run typecheck
npm run lint
npm run build
cargo package -p pdf-link-map --allow-dirty
target/release/pdf-link-map --demo --json
```

See `.factory/verification-3.md` for commands, results, and exact live
evidence.
