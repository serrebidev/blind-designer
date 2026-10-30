# Blind Designer

An accessible multispectral color creator for web design, built for screen
reader users first. Free: palette extraction costs nothing, and no color is
ever guessed.

Colors live in OKLCH, a perceptual color space. Nothing is mixed or blended:
complementary colors are a hue rotation of exactly 180 degrees, shades are
lightness steps, tints are lightness steps up, tones are chroma steps down.
Change one parameter and you know what every other value becomes, so schemes
and complementary shades are predictable instead of eyeballed.

## Features

- **Palette extraction**: feed it a background, header or cover image and it
  reads the palette natively from that image — the extracted colors, ranked
  by coverage, each one spoken as a caption ("dark moderate teal (#0F5E68),
  lightness 44 percent, chroma 0.07, hue 208 degrees").
- **Pure parameter color design**: set lightness, chroma and hue directly, or
  type a hex value. Out-of-gamut results are gamut-mapped by reducing chroma
  only, holding lightness and hue exact — never channel-clamped, so a
  complement stays a complement.
- **Schemes and ladders**: complementary, split complementary, analogous,
  triadic and tetradic schemes, plus shade, tint and tone ladders, all as
  exact parameter transforms of the selected color.
- **WCAG contrast readouts**: every selected color reports its contrast
  ratio against the design background with AA/AAA pass or fail for normal
  and large text.
- **Designs**: build a design from rectangles and ellipses filled with
  palette colors, at set pixel dimensions on a set canvas. Export the
  rendered PNG, a caption sheet (every shape described in words, including
  hand-written captions where you gave them and generated fill-ins only
  where you didn't), and the palette as CSS custom properties with the
  spoken caption of each color as its comment.

## Accessibility

Built with wxDragon (wxWidgets) and trypsynth's quiet-list, so the palette
and shape lists behave properly with NVDA: rows are announced by their
captions, and list updates never re-read the list. Every control is labeled,
every action is in a menu, and the status bar speaks the result of every
action.

## Status

0.1.0, in development. Ideas and the original request came from Elshara
Silverheart.

## License

MIT.
