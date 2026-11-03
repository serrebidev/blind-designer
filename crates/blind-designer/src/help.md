Blind Designer — Help and Worked Examples

Blind Designer is a color creator and layered drawing tool built to be
used entirely by ear. Colors are never mixed or blended: every color
is computed from numbers you set (lightness, chroma, hue), so a result
is known before it is made. Every change to a drawing is written into
an event log, and that log is the design's memory — export it and you
have a step-by-step recipe that recreates the image exactly.

THE MAIN WINDOW

- Palette list: your working colors. Moving to a color selects it and
  reads out its full description: name in words, hex code, lightness,
  chroma, hue, its complementary color, and its contrast against the
  design background with WCAG AA and AAA pass or fail.
- Hex value field and Apply Hex: type a hex code (like 3A7CA5) and
  apply it to replace the selected palette color, or add it.
- Lightness, Chroma, and Hue fields and Apply L C H: build a color
  from its OKLCH numbers instead. Lightness 0 to 1, chroma 0 to 0.4,
  hue 0 to 360 degrees.
- Shapes list: every shape in the drawing, layer by layer, each with
  its generated caption (or its own caption, if it has one).

THE MENUS

- File: open an image to extract its palette; import colors from a
  CSS file, or import a simple HTML page (inline styles) as a design;
  export the design as PNG, as a caption sheet, as CSS variables, as
  a summary (recreation file), or as a standalone HTML page whose
  shapes carry their captions as accessible names; exit.
- Palette: grow the palette from the selected color — complementary,
  split complementary, analogous, triadic, tetradic, 4 shades,
  4 tints, 4 tones — or remove the selected color.
- Design: canvas size; add a rectangle, ellipse, or stroke; set the
  background to the selected color; move, rotate, resize, recolor, or
  remove the selected shape; open the Layers dialog.
- Help: the design-terms Dictionary, this help file, the Sound Cues
  toggle, and About.

SOUND CUES

While you draw, each change also plays a short tone, alongside the
spoken readout. The tone tells you where the shape is:

- Left-right position becomes stereo position: a shape on the left
  sounds in the left ear, a shape on the right in the right ear.
- Top-bottom position becomes pitch: the top of the canvas sounds
  highest (880 Hz), the bottom lowest (220 Hz).
- Size becomes length: a bigger shape sounds longer.

Turn cues off or on under Help, Sound Cues. Speech readouts are
unaffected either way.

THE DICTIONARY

Help, Dictionary opens the built-in dictionary of color and design
terms. Type part of a word in the search field and press Search, or
browse the full list. Every term the app itself uses is defined there
in plain language: hue, chroma, lightness, gamut, layer, stroke,
contrast ratio, and many more.

WORKED EXAMPLE 1: SUNSET STRIPES

Goal: three horizontal bands — a deep orange sky band, a red band,
and a dark ground band — on the default 800 by 600 canvas.

1. In the Hex field, type F2994A and choose Apply Hex. The readout
   describes a light vivid orange.
2. Open the Design menu and choose Add Rectangle. Set X 0, Y 0,
   Width 800, Height 300. The top half of the canvas is now orange.
   Listen to the cue: centered, mid pitch, long — a big shape in the
   middle.
3. In the Hex field, type D64541 and Apply Hex.
4. Design, Add Rectangle: X 0, Y 300, Width 800, Height 150. A red
   band sits under the orange. Its cue sounds lower than the first:
   it is further down the canvas.
5. Hex: 402E32, Apply Hex. Design, Add Rectangle: X 0, Y 450,
   Width 800, Height 150. The dark ground band fills the bottom, and
   its cue is the lowest of the three.
6. File, Export Summary (recreation file) and save it. That file now
   holds the numbered recipe for this exact picture. File, Export PNG
   saves the picture itself.

WORKED EXAMPLE 2: A SIMPLE HOUSE WITH A SUN

Goal: a house body, a door, and a round sun, using two layers.

1. Hex: 3A7CA5, Apply Hex (this blue is already in the palette when
   the app starts). Design, Add Rectangle: X 300, Y 320, Width 200,
   Height 160. That is the house body, just below center — its cue
   pans to the middle and sounds below middle pitch.
2. Hex: 8A5A2B, Apply Hex. Design, Add Rectangle: X 375, Y 400,
   Width 50, Height 80. The door stands in the middle of the body.
3. Design, Layers. Choose Add Layer, name it Sky, then Set Current,
   then Close. New shapes now land on the Sky layer.
4. Hex: F2C230, Apply Hex. Design, Add Ellipse: X 620, Y 60,
   Width 110, Height 110. Equal width and height make the ellipse a
   circle: the sun. Its cue pans right and sounds high — top right,
   exactly where a sun belongs.
5. Shapes list check: three shapes, the sun listed under layer Sky.
6. File, Export Captions saves a sheet describing every shape in
   words, including how each fill contrasts with the background.

WORKED EXAMPLE 3: A FACE

Goal: a round head, two eyes, and a smile drawn as a stroke.

1. Hex: F2C09A, Apply Hex. Design, Add Ellipse: X 250, Y 100,
   Width 300, Height 300. The head, centered, near the top.
2. Hex: 26221F, Apply Hex. Design, Add Ellipse: X 330, Y 210,
   Width 30, Height 40 for the left eye, then again with X 440 for
   the right eye. The right eye's cue pans a little further right.
3. Design, Add Stroke. Enter the points: 340,320 400,350 460,320 —
   three points dipping in the middle, a smile. Set the brush width
   to 6. The shapes list describes it as a stroke through 3 points.
4. To nudge the smile down: select it in the Shapes list, then
   Design, Move Selected Shape, and shift down by 10. The readout
   announces the shift, and the cue sounds a touch lower.

WORKED EXAMPLE 4: A PALETTE FROM A PHOTO, THEN SHADES

Goal: take the colors of a photo you already like, then build a
matching set of darker shades for text or borders.

1. File, Open Image, and pick the photo. The palette is replaced by
   the image's dominant colors, largest first, each with the percent
   of the image it covers. The largest becomes the design background.
2. Move through the palette list and listen to the readouts. Note the
   contrast line for any color you might put text on: AA normal text
   needs 4.5 to 1 or more.
3. Select a color, then Palette, Add 4 Shades. Four darker versions
   join the palette, same hue and chroma, lightness stepped down.
4. Select one of the shades, then Design, Set Background to Selected
   Color, and hear how the readouts of the other colors change:
   contrast is always judged against the current background.
5. File, Export CSS saves the whole palette as CSS variables, each
   with its spoken caption as a comment, ready for a web page.

TIPS

- Predict before you act: a complementary is always hue plus 180,
  shades only step lightness down, tones only scale chroma down.
  Nothing in this app blends, so the numbers never surprise you.
- The event log is complete. If a drawing goes wrong, export the
  summary and read the steps: the step where it went wrong is the
  step to redo.
- A stroke cannot be resized by width and height — its size is its
  points. Move it, or add a new stroke with the points you want.
- A drawing always keeps at least one layer. Hide a layer to compare
  the design with and without its shapes; hidden layers do not paint
  and do not export to PNG.

SAVING AND OPENING DESIGNS

- File, Save Design writes the whole design to a .bdd file: the
  canvas, the background, every layer and shape, the palette with
  each color's provenance, and the full step-by-step log.
- File, Open Design reads one back. The opened design replaces the
  current one, history included — export a summary afterwards and
  the steps are the same steps you saved.

WHERE EACH COLOR CAME FROM

Every palette entry says how it got there, right in the palette
list: "typed in by hand", "extracted from image photo.jpg" (with
the percent of the image it covers), "generated as complementary
of" a base color, or "imported from" the CSS or HTML source it was
found in. When a design is saved, the provenance is saved with it.

Worked example: press Apply Hex with a color selected and the entry
is restamped "typed in by hand" — the palette always tells the
truth about the latest way a color arrived.

COLOR VALUES YOUR WAY

View menu: Color Values as Hex, RGB, HSL or HSV. Every spoken and
displayed color value follows the choice — the palette list, the
selected-color readout, the status line. Captions and the summary
(recreation) file keep hex, because those are saved file formats
and stay stable. The choice is yours per session; the app starts
in hex.

EXTRACTING MORE COLORS

File, Open Image and File, Extract Colors from Image both ask how
many colors to extract: 8 by default, up to 64. Open Image replaces
the palette and sets the background to the largest color, as
before. Extract Colors from Image only adds to the palette — use it
to mine any image for colors without touching your design.
