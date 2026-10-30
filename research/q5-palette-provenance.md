# Q5 — Palette size from an image

Answered: 2026-10-29 17:35 PDT (Telegram msg 73958).

His words: "See above. I want as many as is introduced to the project, each contained in its own recorded memory of how and where it is added when and only if it is there."

## Research (2026-10-29)
- Extraction defaults in the field: Color Thief default 10 (cap ~20), Android Palette default 16, median-cut/k-means extractors (zygisS22 ★121, FabriceCastel/gvcci 16 colors ★114).
- His answer rejects the fixed-size model entirely: no cap, no default count — provenance instead.

## What this means for the build
- No palette-size setting. Every color that enters a project (extracted, picked, typed) is recorded with provenance: source, where it is used, when added — the same event log as Q4.
- Extraction from an image records ALL distinct colors it introduces (clustered, but the count is the image's, not a preset).
