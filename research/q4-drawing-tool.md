# Q4 — Shapes for v1

Answered: 2026-10-29 17:35 PDT (Telegram msg 73958). He rejected the premise of a fixed shape set.

His words (condensed; full text in answers-state.md): "Honestly, neither. I want to make a shape multispectral creation tool. Think of it like autocad or 3dprinter, but it never leaves the screen... and you get a detailed summary of every change that's added. Be it a line, stroke, brush, space, layer, shift, angle, a blind drawing tool is really what I'm after that is 1d, 2d, 3d, 4d capable etc... you should as a blind person be able to draw it with measurements through describing or changing sliders until the values that create this come alive... and it could be verified through an ai to scan what's actually there, versus what was described as wanting to be there... blind digital sculptures is the goal here. the shape itself doesn't matter, the way in which shapes layer on top of each other, does."

## Research (2026-10-29)
- GitHub "drawing blind accessible": EchoPaint ★1 (2026-03) — blind users create art through sound/touch; tiny, no layered parametric model.
- Scene-graph/vector editors on GitHub are small and GUI-first; none is accessibility-first or log-driven.
- AI verify-what's-there: VLM description tooling exists (alt-text generators, gowall's OCR-with-VLM ★2,322) — a render -> describe -> compare loop is buildable with current VLMs.
- Gap confirmed: no existing project does parametric, layer-first, fully logged blind drawing/sculpture. This is a green-field design, not an integration.

## What this means for the build
- v1 architecture changes: the core becomes a layered scene model (layers are first-class; shapes are layer content: line, stroke, brush path, space/negative, shift, angle) where EVERY change is an event in an append-only log — the log is what Q3's source-code document prints.
- Parameters + sliders + typed measurements are the input model; rendering is a view of the log.
- Dimension scaling (his "1d, 2d, 3d, 4d") = layers over layers + time/animation later; v1 stays 2D canvas but the model must not hard-code 2D.
- AI verification scan (render -> VLM description -> diff vs intended) is a named later feature, designed-for now.
- This is a scope jump from the shipped v0.1.0 color tool: color engine stays the foundation; the drawing/sculpture core is the new main build.
