# Changelog

Readable release history for Blind Designer (renamed from Transversion at 0.2.0), newest first. Each release's
section is its GitHub release note, taken from this file by the release
workflow.

## 0.1.0

- First release. An accessible multispectral color creator: colors live in
  OKLCH and are never mixed or blended — complementary, split
  complementary, analogous, triadic and tetradic schemes, and shade, tint
  and tone ladders are pure parameter changes (hue angle, lightness step,
  chroma step), so every result is predictable before it is seen.
- Palette extraction from any image you feed it (background, header,
  cover): up to 8 colors ranked by coverage, each spoken as a caption.
  Free — no paid palette service.
- Every color is described in words: family, lightness and chroma
  percentages, hue angle, hex. WCAG contrast ratio against the design
  background with AA/AAA pass or fail, spoken.
- Designs: rectangles and ellipses filled with palette colors at set
  pixel positions and sizes on a set canvas. Export the rendered PNG, a
  caption sheet (hand-written captions win; generated fill-ins only where
  a shape has none), and the palette as CSS custom properties with each
  color's caption as its comment.
- Built with wxDragon and quiet-list for NVDA-first list behavior.
