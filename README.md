# PDF Link Map

PDF Link Map is a local-first command-line checker for operations and technical-document teams. It inventories internal and external PDF link annotations, resolves named and explicit destinations, spots broken links and duplicate destination names, optionally compares destinations with a heading manifest, and writes a self-contained clickable HTML report.

It never opens external URLs and never modifies the input PDF—including signed files.

## Install

Download a release binary, or build it with a current Rust toolchain:

```sh
cargo install --path crates/pdf-link-map
pdf-link-map --help
```

## Usage

Try the full audit with the bundled sample PDF. It creates a temporary input,
heading manifest, and standalone HTML report, then prints their paths:

```sh
pdf-link-map --demo
```

Audit a PDF and create a report next to it:

```sh
pdf-link-map handbook.pdf
```

Choose the report path and emit machine-readable findings to stdout:

```sh
pdf-link-map handbook.pdf --output audit/link-map.html --json
```

Compare named destinations with the headings you expected the converter to preserve:

```sh
pdf-link-map handbook.pdf --manifest headings.json --fail-on broken
```

The manifest is a JSON array. `anchor` is optional; when present it must match a named PDF destination.

```json
[
  { "title": "Installation", "anchor": "install", "page": 3 },
  { "title": "Troubleshooting", "anchor": "troubleshooting" }
]
```

Exit codes are stable: `0` completed (even with findings unless a threshold is requested), `1` the chosen `--fail-on` policy failed, and `2` input/configuration or PDF parsing failed. `--json` writes only JSON to stdout; human progress goes to stderr.

## What it validates

- URI annotations (recorded but never dereferenced)
- explicit page destinations and named destinations
- missing pages, missing named destinations, malformed actions, and duplicate destination names
- optional manifest anchors and expected pages
- empty PDFs and PDFs with no links, with actionable output rather than a crash

Encrypted PDFs are reported as unsupported in v1. The parser is defensive but intentionally does not repair malformed files.

## Develop and verify

Requirements: Rust 1.85+, Node.js 20+, npm 10+.

```sh
npm ci
npm test
npm run build
```

`npm test` runs the Rust unit/integration suite and site tests. `npm run build` creates the static deployment in `dist/site/` and a release CLI binary under `target/release/`. To preview the site: `npm run dev`.

Create a publishable source package without publishing it:

```sh
cargo package -p pdf-link-map
```

## Privacy

PDF analysis is entirely local and there is no telemetry. The website has no accounts, checkout, or license storage while the optional Team rollout kit is unavailable. See the site's privacy and terms pages.

## License

MIT. See [LICENSE](LICENSE).
