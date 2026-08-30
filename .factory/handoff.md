# PDF Link Map repair handoff

Work order: `pdf-link-map-repair-3`
Verifier report: `6b782ae3a7943abcfbcc8110778590efb5980693`
Verified candidate reproduced: `b322620915b16c78f99cf84f221bca2776b8d8d4`
Repair branch base: `1a45068dd6424b12b44e33d34fec716314471701`

## Outcome

Both release blockers from the independent report are repaired without changing
the core local audit workflow.

- The exact candidate was rebuilt in a detached worktree and inspected at
  390 × 844. It reproduced the verifier's measurement exactly:
  `clientWidth = 390`, `scrollWidth = 599`, and the install grid grew to
  `587.438px`. The long `cargo install` command was imposing its min-content
  width on the single-column mobile track.
- The released layout explicitly releases the grid items' automatic minimum
  width. The command now scrolls only inside its labelled, keyboard-focusable
  `pre` element. The repaired 390px build measures `clientWidth = 390` and
  `scrollWidth = 390`.
- The unavailable $29 Team checkout is not advertised or called. The site has
  no checkout link, billing API URL, license storage, `fetch`, or external
  runtime connection; it plainly says the Team rollout kit is not for sale.
  The complete CLI audit, HTML/JSON export, manifests, thresholds, and safety
  features remain free and available.
- Added a real CLI demo: `pdf-link-map --demo` creates a temporary two-page
  specimen PDF and heading manifest, audits them using the production parser,
  writes a standalone HTML report, and prints its path. The browser now has a
  direct `/?demo=1#demo` sample mode with the required demo banner.

## Regression coverage

- `site/tests/browser.mjs` checks the exact mobile root cause: the document
  width equals the 390px viewport, the install grid is constrained, and the
  long command's own horizontal scroller has `overflow-x: auto`. It also
  verifies the code scroller is keyboard focusable, the demo URL/banner,
  specimen keyboard operation, desktop/mobile axe serious/critical count,
  zero console errors, PWA registration, and offline reload.
- `crates/pdf-link-map/tests/fixture_audit.rs` runs `--demo --json`, asserts
  its valid/broken/external sample report, and checks that the emitted HTML
  report exists.
- `.factory/claims.json` registers one isolated test per public web claim:
  the offline shell and no-web-tracking request behavior. Both use fresh
  browser contexts; the offline test closes only its own context.
- `site/tests/site.test.mjs` continues to reject unavailable checkout/API/
  local-storage/fetch code in the production bundle and asserts the response
  policy and size budgets.

## Verification evidence

The current worker image completed these commands successfully:

```sh
npm ci
npm test
npm run lint
npm run build
cargo package -p pdf-link-map --allow-dirty
```

Results: 2 Rust unit tests, 6 Rust integration tests, 1 compiling doctest,
5 built-site tests, browser desktop/mobile/PWA checks, and 2 claim tests all
passed. Strict TypeScript, rustfmt, and clippy passed. The production build
created `target/release/pdf-link-map` and `dist/site/`; the shipped JS is
4.25 KB total, CSS is 12.77 KB, and the mobile WebP remains 29.35 KB.

The packed `pdf-link-map-0.1.0` crate installed into a new empty consumer
prefix with `cargo install --path target/package/pdf-link-map-0.1.0 --root
<prefix>`. Its `--help` worked, and `--demo --json` produced a 3-link audit
with one external URI and a real standalone HTML report.

`/opt/fleet/lib/verify-url.sh http://127.0.0.1:4178/ <temp-evidence>` passed:
HTTP 200, title, `lang=en`, one `h1`, `main`, no missing image alt text, no
unlabelled buttons, and no browser console errors. Fresh desktop and mobile
screenshots were visually inspected; the 390px page has no horizontal page
scrolling. Lighthouse mobile reported Performance 100, Accessibility 100,
Best Practices 100, and SEO 100 (FCP 1.0s, LCP 1.4s, TBT 0ms, CLS 0). Like the
previous runner, Lighthouse printed a final browser-tab crash after emitting a
complete report; the command returned zero and independent Playwright checks
completed without errors.

The deployed-site verification and deployment identifier will be appended
after the static deployment completes.

## How to run and release

```sh
npm ci
npm test
npm run lint
npm run build
/opt/fleet/lib/deploy-static.sh pdf-link-map dist/site
```

Run the CLI sample with `target/release/pdf-link-map --demo`, or see
`.factory/demo.md`. To prepare a registry package without publishing it:

```sh
cargo package -p pdf-link-map
```

## Known boundary / next step

No billing, DNS, payment, or unrelated service resource was read or changed.
The optional Team rollout kit remains intentionally unavailable until the
factory registers and activates its production billing product. A future,
separately authorized paid release must restore checkout and license features
only after a real checkout redirect and return-token smoke test pass.
