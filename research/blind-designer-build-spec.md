# Blind Designer — consolidated build spec

Consolidated 2026-10-30 from Elshara Silverheart's answers (Telegram, 2026-10-29 17:35 PDT for Q1-Q6 and his dictionary addition; 2026-10-30 06:37 PDT for Q7 deferral and audience). Source detail: q0-dictionary.md through q6-contrast-docs.md and answers-state.md in this folder. Nothing here re-asks an answered question.

## Identity and platform
- Name: Blind Designer (keeps the serrebi project naming line). No collision found (q1).
- Target audience: the blind community (Elshara, 2026-10-30).
- Elshara's OS: Windows 11. Stack: Rust + wxDragon GUI, NVDA accessible throughout; trypsynth's projects (wxDragon, quiet-list, wx-utils, shipfitter) as references, same family as BlindRadio and BlindFB.
- Foundation already built in Transversion v0.1.0 (delivered to Elshara 2026-10-29): OKLCH-native color engine, pure parameter transforms (scheme/shade/hue results predictable by the numbers, gamut mapping holds hue exact), palette extraction from a fed background/header image, spoken color captions, design rendering, CSS export. 11/11 core tests green at v0.1.0.

## The build: rename + layered drawing core
The color engine stays. The next build renames Transversion to Blind Designer and adds the drawing core Elshara specified (q4, the scope change):
- A blind drawing/sculpture tool, not a fixed shape set. Layers are the organizing unit.
- Every change is logged in detail: line, stroke, brush, space, layer, shift, angle — the event log is the design's memory.
- Drawing is by measurements and sliders, not by sighted dragging. Ambition is 1D/2D/3D/4D capable; v1 is the layered 2D core.
- An AI verification scan (does what is actually rendered match what the log describes) is part of the design, scheduled later, not v1.

## Supporting systems
- Dictionary (q0, Elshara's own addition): a supported dictionary of color-design terms and values throughout the tool. Research headline: meodai/color-names (about 3,090 stars) is the standard open color-name dataset.
- Palette (q5): no fixed palette size. Every color introduced gets its own recorded memory of how and where it was added (provenance per color).
- Captions (q2): both outputs — one carries a summary of the creation, the other the exact file.
- Exports (q3): CSS variables yes, but primary is the summary file: step-by-step instructions to recreate the exact image ("an entire image source code").
- Contrast (q6): documentation footnote, not a gate — comparisons between parts of an image, between images, and captions vs image, plus dimension analysis (what crops, how it scales).

## Sound feedback while drawing (Q7)
Elshara deferred the choice to Muse's intuition (2026-10-30, "Sounds good. Use intuition for the question."). Decision, already sent to him 2026-10-30: both — gentle pitch and position cues while drawing (continuous feedback speech cannot give) plus speech/text readouts for exact values. Do not present this as Elshara's pick; it is Muse's call under his deferral.

## Status
- Spec complete as of 2026-10-30. Elshara has been thanked and told the next update will be a first build he can try.
- Next step (Muse's work, no Brandon or Elshara input needed): rename the project to Blind Designer and build the layered, change-logged drawing core on the existing color engine, gated on the full test suite before any release, per Brandon's standing all-tests-passing rule.
