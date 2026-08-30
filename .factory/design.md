# PDF Link Map visual thesis

## Direction: handwritten lab notebook

PDF navigation failures are invisible until somebody tries a link. The site makes that forensic work tangible: an off-white engineer's notebook, ruled in muted blue, annotated with ink, stamps, arrows, and page tabs. The illustration shows a document's link path being traced from a contents entry to a destination—explanation, not decoration. It deliberately avoids glossy SaaS gradients and generic card grids.

The site is explicitly single-mode. A warm paper canvas and dense ink are essential to the notebook metaphor; forcing a dark theme would turn paper into a screen and weaken the identity. Contrast is maintained in the chosen light treatment.

## Tokens

- `paper #F4EEDB`: warm stock background.
- `paper-raised #FFFDF5`: pasted note and code surfaces.
- `ink #19232B`: graphite-blue primary text (contrast 13.5:1 on paper).
- `ink-muted #51616B`: secondary pencil note (6.2:1 on paper).
- `rule #A8C1C5`: notebook rules and structure, never body text.
- `red-ink #A33A2B`: primary action and failed-link marks; white text is 6.3:1.
- `blue-ink #1F5872`: links and verified marks (6.4:1 on paper).
- `green-ink #2D684E`: valid status.
- `amber-ink #8A570C`: unresolved/warning status.
- `danger #8E2D25`: failure state.

## Type and spacing

- Interface and long text: native humanist system stack (`Avenir Next`, `Segoe UI`, sans-serif), avoiding a font download entirely.
- Notebook annotations and display moments: `Segoe Print`, `Bradley Hand`, `Comic Sans MS`, cursive. Used sparingly; body copy stays readable.
- Scale: 16px body, 18px lead, 22px h3, 29px h2, clamp(40–64px) h1. Line height 1.55; reading measure 68 characters.
- An 8px base rhythm with 4px half-steps. Major sections use 72–112px separation. Controls are at least 44px high.

## Interaction grammar

Primary buttons resemble a red proofing stamp and depress by 2px. Links receive a hand-drawn underline. Tabs and disclosure notes look like attached paper scraps. Focus uses a 3px blue-ink ring with a paper offset. Status is always expressed by icon/word as well as color.

The browser demo processes only an included specimen report; actual PDF inspection belongs to the downloadable local CLI, making the privacy boundary obvious. License restore uses an inline notebook strip, not a modal.

## Motion

On entry, the traced link route and notes resolve once over 240–500ms using opacity and transforms. Hover and press feedback is 160ms. Nothing loops. Under `prefers-reduced-motion: reduce`, movement and smooth scrolling are removed and state changes are immediate.

## Asset plan and provenance

- Hero: original raster illustration generated for this product, depicting a top-down lab notebook with a printed PDF contents sheet, a red pencil tracing a link path, verification marks, and no legible generated text. Generated with `/opt/fleet/lib/gen-image.sh` using the factory image deployment on 2026-08-28; source PNG and prompt metadata are retained in `site/src/assets/provenance/`, optimized WebP is shipped. License: created for this repository under MIT.
- Sharing image: `site/public/link-map-social-card.jpg` is a 1200×630 center crop derived locally from that same original hero WebP on 2026-08-30; it carries no new generated text or third-party asset. The 180×180 Apple touch icon is a locally drawn raster version of the repository’s original SVG route mark.
- Icons and link-route marks: original CSS shapes or inline, accessible characters; no external icon pack.
- Paper rules and grain: CSS gradients rendered locally, no network asset.

The final generation prompt is recorded alongside the source asset and repeated in the handoff.
