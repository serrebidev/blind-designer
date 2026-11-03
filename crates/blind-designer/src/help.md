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
  start a new design from a template; compare two images, or describe
  an image as a tactile grid; export the design as PNG, as a caption
  sheet, as CSS variables, as a summary (recreation file), or as a
  standalone HTML page whose shapes carry their captions as
  accessible names; export it for touch as a raised-line SVG, a
  tactile sheet, or braille labels; exit.
- Palette: grow the palette from the selected color — complementary,
  split complementary, analogous, triadic, tetradic, 4 shades,
  4 tints, 4 tones — or remove the selected color.
- Design: canvas size; add a rectangle, ellipse, stroke, text,
  image, or symbol; set the background to the selected color; move,
  rotate, resize, recolor, or remove the selected shape; describe
  one area of the canvas; open the Elements dialog, the Layers
  dialog, and the brush library.
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

TEXT AND IMAGE ELEMENTS

Design, Add Text places words in the design: type the phrase, name
a font, pick a point size, then place it by measurement like any
other shape. Its caption is the phrase itself unless you write a
different one. In an exported HTML page the phrase is real text with
real font CSS; in a PNG the app draws it in a built-in block font
(the named font travels with the design and is used by the HTML
export).

Design, Add Image places an image file in the design, scaled into
the box you give it. The design stores the file's path; if the file
cannot be found when the design is drawn, a framed placeholder in
the color you chose shows where it belongs, captioned with the
file's name.

THE ELEMENTS DIALOG

Design, Elements lists everything in the design in plain language:
what it is, where it sits in words (top left, center, bottom
right), how big, and what color — plus its written caption when it
has one. Select an element and Rename Caption, Describe (the full
description, numbers and all), Move, Resize, Recolor or Delete it,
without hunting through the canvas.

PLAIN-LANGUAGE INPUT

The add, move and resize dialogs accept words as well as numbers.
Position can be "top left", "center", "bottom right", or a nudge
from where the shape is now: "a bit right", "a bit down". Size can
be "a bit wider", "a bit taller", "a bit smaller". Color fields
take names: "red", "sky blue", "dark gray" — or a hex code as
always. Words the app does not understand are refused with an
explanation, never guessed at.

Every shape also has an automatic plain-language description —
"long rising stroke from top left to center, dark blue" — used in
the Elements list, and spoken when a shape is added without a
caption of its own.

SAVED BRUSHES

A stroke's brush is its width, its color, and its pattern (solid,
dashed or dotted). When you add a stroke, the brush values prefill
from the current brush, and whatever you use becomes current.
Design, Save Current Brush keeps it in the brush library under a
name; Design, Brush Library lists the saved brushes, and Apply
makes one current for the next strokes. The library is saved in a
file beside the app, so your brushes are there next time.

ANIMATION

Design, Animation turns the design into a short film made of
frames. A frame is a full copy of the canvas — layers, shapes and
background — plus how long it shows, in milliseconds (500 to
start). Build a film like this:

1. Draw the first pose on the canvas.
2. Open Design, Animation and choose Add Frame (Copy of Canvas).
   Frame 1 is captured, and the list reads what it shows.
3. Close the dialog, change the canvas — move a shape, recolor
   it, add the next pose — then capture again.

Each row of the Animation list says the frame's number, its
duration, and in plain words what it shows (frame 1) or what
changed since the frame before it: "the red ellipse moves right
20 pixels", "a new shape appears", "the background changes from
white to black". Duplicate copies a frame, Delete removes one,
Move Up and Move Down reorder the film, and Set Duration changes
how long the selected frame shows (20 to 60000 milliseconds).

To edit a stored frame, select it and choose Load into Canvas:
the canvas becomes that frame. Change it, open the dialog again
and choose Replace with Canvas to store the edit back (the frame
keeps its duration). Until you replace it, canvas edits never
touch a stored frame. Every frame step is written into the
design's log, and frames are saved inside .bdd design files —
designs saved before animation existed open as still designs.

Three exports play or tell the film, all under File:

- Export Animated GIF writes a looping animated GIF, one picture
  per frame, each shown for its own duration.
- Export Animated HTML writes a standalone web page that steps
  through the frames with a CSS animation — and lists the
  frame-by-frame description as readable text under the stage.
- Export Animation Description writes the film as words: every
  frame in order, with its duration and what it shows or what
  changed, ready to read or send.

TACTILE: FEELING A DESIGN

A design can be read by touch as well as by ear. Four tools, all
under File except Describe Area:

- Export Raised-Line SVG writes the design as outlines only —
  every shape a black stroked outline on white, nothing filled.
  Each layer gets its own line style (solid, long dashes, short
  dashes, dots), so layers can be told apart by hand. Print it on
  swell paper or emboss it and the lines rise. A text element is
  drawn as its braille translation inside an outline box: the
  braille is what rises, not the letter shapes.
- Export Tactile Sheet writes the design as a text walk: layer by
  layer, element by element in reading order (top to bottom, left
  to right), each element with its plain-language description, its
  caption, and its caption again as a braille label. Hidden layers
  are left out and noted.
- Export Braille Labels writes just the labels: one numbered line
  per element, its caption in uncontracted (Grade 1) braille,
  ready to emboss or stick beside the raised lines.
- Design, Describe Area names only the elements inside one part
  of the canvas. Type a name — top left, center, bottom right, or
  just top or left for a whole strip — or a box as four numbers
  (across, down, width, height). Elements only partly inside are
  marked "partly in this area".

Worked example: draw the Simple house template (File, New from
Template) and export the tactile sheet. The walk starts with the
sun (top right), then the walls, the roof stroke, the door, the
windows, and the grass along the bottom — the order your hand
would meet them.

SYMBOLS AND TEMPLATES

Design, Add Symbol stamps a ready-made symbol — star, heart,
arrow, check, cross, sun, moon — wherever you place it, at
whatever size. A symbol is ordinary strokes under one caption, so
it moves, resizes, recolors and exports like anything else.

File, New from Template starts a design from a layout: a birthday
card, a simple house, a face, or a chart. Loading a template
replaces the design you have open, and the template is a normal
design from then on — change anything, then Save Design.

PICTURES BY TOUCH AND COMPARISON

File, Describe Image as Tactile Grid turns any picture into a
grid you can feel cell by cell: 24 by 12 cells by default (you
choose the size). Each cell is the picture's average shade there,
shown two ways — as a braille shade pattern (more dots means
darker, the scale is printed at the top) and in words, row by
row: "columns 1 to 6 bright; column 7 dark". The sheet ends with
where the darkest and brightest parts sit. A screenshot becomes
readable this way: dark text blocks and bright backgrounds show
up as shade shapes.

File, Compare Images sets two pictures side by side and reports,
in plain sentences: whether they are the same size; which palette
colors they share and which belong to only one of them; which is
brighter overall; and which is sharper — the blurrier one is
named, so you can tell an original from a soft copy.
