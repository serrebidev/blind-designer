# Q3 — CSS variables / palette file export

Answered: 2026-10-29 17:35 PDT (Telegram msg 73958).

His words: "Yes, but more importantly, just documenting this in a summary file would be enough. think of it like an entire image source code, if you were to describe it to someone looking to make that exact image, with steps."

## Research (2026-10-29)
- Scene/vector sources: SVG is the open, text-readable "source code" of an image; JSON scene graphs are the editable equivalent (editor projects found are small — volkovsh/Graph-Editor ★0, Luseefor/vinculum ★0 — no dominant open format; design our own and document it).
- Design-token tooling (recent: mrodgersjs-web/design-studio, 2026-10-30) validates/exports tokens + WCAG — confirms CSS-variable export is a standard expectation.

## What this means for the build
- Primary export artifact = a step-by-step recreation document ("image source code"): ordered steps, every shape/layer with exact measurements and colors (named + OKLCH + hex), so a person or the app itself can rebuild the image exactly.
- CSS variables + palette file ship alongside, as he approved, but they are the secondary export.
- The design file format is the source of truth; PNG is a render of it.
