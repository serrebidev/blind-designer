# Q0 — Elshara's own suggestion: a color-design terminology dictionary

Answered: 2026-10-29 17:35 PDT (Telegram msg 73958), before Q1.

His words: "Get a dictionary of all the values of terms necessary for color design, and support it."

## Research (2026-10-29)
- GitHub, most stars: meodai/color-names ★3,090 — large handpicked color-name list (30k+ names), actively updated (2026-10-29). The standard open color-name dictionary; nearest-name lookup APIs and datasets derive from it.
- Also relevant: meodai's color-names API + the XKCD color survey names are the usual second source.
- Web: W3C/CSS named colors (148) are the floor, not a design dictionary; design terms needed go beyond names: hue/chroma/lightness, harmony terms (complementary, triadic, analogous), WCAG terms, gamut, etc.

## What this means for the build
- Ship a built-in dictionary: color names (meodai/color-names data, license-checked at integration) for naming any picked color by nearest name, plus a glossary of design terms the app itself uses, searchable and spoken.
- Every value the app reports should resolve through the dictionary: a color gets a name, a term gets a definition.
