# Q6 — WCAG contrast readouts

Answered: 2026-10-29 17:35 PDT (Telegram msg 73958).

His words (condensed): "It's a useful footnote for the documentation to compare parts of an image, so you have interactive logs with other parts of it or a different image or a variable between colored captions/text and the image itself as well as to see how it would look with different dimensions, what would get cropped, what wouldn't, how scalable it gets etc."

## Research (2026-10-29)
- GitHub, most stars: Myndex/SAPC-APCA ★592 (APCA, the WCAG 3 contrast candidate), Tanaguru/Contrast-Finder ★251 (finds valid fg/bg pairs).
- Recent: finalize/contrast-kit (2026-10-30), kinti/a11y-toolkit ★2 (WCAG 2.2 checks for AI agents).

## What this means for the build
- Contrast is a documentation/log feature, not just a text-pair checker: the log can compare any two recorded colors (regions, layers, captions vs image, image vs image) with WCAG ratio + AA/AAA, APCA as a secondary readout.
- Add dimension analysis to the same footnote system: at other sizes/aspect ratios, what crops, what scales — computed from the scene model, reported in the summary document.
