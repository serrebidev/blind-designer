Blind Designer — Help and Worked Examples

Blind Designer is a color creator and layered drawing tool built to be
used entirely by ear. Colors are never mixed or blended: every color
is computed from numbers you set (lightness, chroma, hue), so a result
is known before it is made. Every change to a drawing is written into
an event log, and that log is the design's memory — export it and you
have a step-by-step recipe that recreates the image exactly.

THE MAIN WINDOW

The window's lists live in a tabbed notebook: four tabs — Palette,
Elements, Layers, Sliders — that you reach with Tab and move
between with Ctrl+Tab or the arrow keys. Every list shows only
what the project actually contains: a new project starts with an
empty palette, no elements, and its one starting layer. Nothing
is listed before it exists.

- Palette tab: your working colors. A new palette is empty — add
  colors with the hex field, by extracting them from an image, by
  importing them, or with Color, Load Starter Colors. Moving to a
  color selects it and reads out its full description: name in
  words, hex code, lightness, chroma, hue, its complementary color,
  and its contrast against the design background with WCAG AA and
  AAA pass or fail.
- Elements tab: every element in the drawing, in plain language —
  what it is, where it sits in words, how big, what color — layer
  by layer. Selecting an element here is what the Design menu's
  Arrange commands act on.
- Layers tab: the drawing's layers, with shape counts; the current
  layer (where new shapes land) is marked. Selecting a layer here
  makes it current. Renaming, removing and hiding layers live in
  Design, Layers.
- Selected color readout: the full description of the palette color
  you last selected.
- Hex value field and Apply Hex: type a hex code (like 3A7CA5) and
  apply it to replace the selected palette color, or add it.
- Lightness, Chroma, and Hue fields and Apply L C H: build a color
  from its OKLCH numbers instead. Lightness 0 to 1, chroma 0 to 0.4,
  hue 0 to 360 degrees.
- Project statistics line: kept current under the color fields —
  how many layers, elements, palette colors and animation frames
  the project has, and the canvas size. Design, Project Statistics
  gives the full count, element kind by element kind.
- Sliders tab: the sliders this project has pinned — see THE
  SLIDERS TAB below. A new project pins none.
- Lists side by side: View, Lists Side by Side replaces the
  Palette and Elements tabs with one Lists tab holding the two
  lists side by side, so both are on screen (and in tab order)
  at once. Turn it off and the tabs come back. It lasts for
  this session, like the other View choices.

THE INSPECTOR

Press Enter on any list — Palette, Elements or Layers — and the
inspector answers three questions about the thing under the
cursor, in a report you can read and review:

- What it is. An element: its kind, caption and plain-language
  description, its size and place, its opacity, roundedness and
  angle. A layer: its contents by kind, where it paints. A
  color: its description and where it came from.
- Where it is used. An element: its layer and that layer's
  depth role, whether the layer is shown (hidden layers stay
  out of exports), and in how many animation frames it appears.
  A color: every element that uses it, and whether it is the
  background.
- What can change it — and what happens if nothing does. An
  unlocked element is reached by the color adjustments aimed at
  its layer or its depth; a locked one is skipped (see LOCKING
  ELEMENTS). And left alone, everything stays exactly as it is:
  nothing in Blind Designer changes a design by itself.

Design, Inspect Selected does the same for the selected element
without leaving the keyboard.

LOCKING ELEMENTS

An element can be locked in place: Design, Lock/Unlock Selected
toggles it, and the Elements list says "Locked" on its row.
A locked element is skipped by the bulk changes — the layer-wide
and depth-wide color adjustments — and those announcements count
it honestly: "3 elements changed, 2 locked stayed the same."
Direct edits to the element alone — move, resize, recolor,
rename — still work; locking is protection from group changes,
not from you. Locking is saved in the design file.

CONTEXT MENUS

Every list has a context menu: right-click it, or press the
Applications key or Shift+F10 while it has focus. The menu is
the list's own actions — the same commands as the menu bar, not
copies.

- Palette: with a color selected — inspect it, set it as the
  background, remove it. With no colors at all, the menu still
  opens and offers the three ways colors arrive: add one by
  hex, load the starter colors, extract from an image.
- Elements: with an element selected — inspect, lock or unlock,
  move, rotate, resize, recolor, remove. Empty or not, the Add
  items are always there, so an empty list can start the design
  from its own menu.
- Layers: inspect the selected layer, or open the Layers dialog
  where layers are added. (A design always has at least one
  layer, so this list is never empty.)
- Sliders page: Project Options — where the pinned sliders are
  chosen — one step from the page they appear on.

SEARCH AND INSERT

Design, Search and Insert (Ctrl+F) searches everything the
design can gain, in one field: palette colors (by their words
or any part of their hex code), the starter colors, the design's
own elements (by caption, kind or layer), the symbols, the
templates and the patterns. Type a word, press Search, and the
results list everything that matches; an empty search lists the
whole catalog. Press Enter on a result — or choose it and press
Insert — and it goes where the cursor is:

- A color becomes the brush color, the color the next thing
  drawn will use. A starter color joins the palette first if
  it is not there already.
- An element is copied: right after the element you have
  selected, stepped a little so the copy is findable, or at the
  centre of the canvas when nothing is selected.
- A symbol is stamped at the centre of the canvas in the brush
  color.
- A template starts a new design, as File, New from Template
  does.
- A pattern opens the Pattern dialog, because it needs its two
  colors first.

A report then confirms what was placed, and where.

THE SLIDERS TAB

Instead of a dialog for every setting, the settings you keep
coming back to can live as sliders on their own tab, and they
apply the moment you move them. Each slider spans the smallest to
the largest value it can take, with the finest step there is:

- Arrow keys move a slider by its smallest step (one unit: one
  pixel, one level of red, one percent). Type-exact values still
  exist everywhere they did before — the dialogs and fields all
  still work, and they read their ranges from the same
  definitions, so the numbers always agree.
- Page Up and Page Down jump by a percent of the slider's range —
  10 percent to start; change it in Project Options.
- Home and End jump to the smallest and largest values.
- Ctrl plus an arrow key hops from slider to slider without
  changing any value: once your hand knows the tab, you can move
  through your kept settings by feel.
- Every slider carries its name and a spoken readout of its value
  in words beside it: "Brush width, 12 pixels".

Moving a slider says what changed and that everything else stayed
the same — "Brush width 12 pixels. Everything else unchanged." —
on the status line. View, Announce Slider Changes turns the
"everything else" wording off, leaving just the name and value.

Which sliders are pinned is the project's own choice, made in
Design, Project Options (below): check the ones you want to keep,
and they are saved in the design file with the project. A new
project starts from scratch — no sliders pinned at all. Project
Options is also where adding and removing lives now: the starter
colors load there too, alongside Color, Load Starter Colors.

The sliders there are: brush width (the current brush); the
current color's red, green and blue; transparency (how see-through
NEW shapes are painted — 0 is fully solid); element transparency
and element scale (the Elements-tab selection: scale grows or
shrinks it from the size it had when you started); stroke width
(a selected stroke); frame duration (the whole film at one pace —
every frame set at once); the extract-colors count; the tactile
grid's columns and rows; the sound cue volume; the color
channels (next section); and the color adjustments.

The color adjustments — shade, tint, tone, chroma, undertone, and
the descriptive ones: temperature (warm or cool), vibrancy
(vibrant or dull), brightness, clarity, fogginess, hardness (hard
or soft) — act on a target you choose: the current color (the
brush color new strokes paint with), the selected element, or the
whole current layer at once. Picking the target is the picker at
the top of the Sliders tab when any adjustment is pinned, and
also lives in Project Options. An adjustment is always worked out
from the color the target had when you started moving its
sliders, never piled on top of the last move, so sliding back to
zero restores the original color exactly. A layer adjustment is
one step in the log for the whole layer, spoken in words.

COLOR CHANNEL SLIDERS

Pin "Color channels (in current format)" in Project Options and
the Sliders tab grows a group under a "Color channels" heading:
one slider for every channel of the color format you picked in
the View menu — hex's red, green and blue; LCH's lightness,
chroma and hue; CMYK's four inks; and so on for every system.
Each slider carries its spoken name ("LCH hue") and a readout of
its value, and moving one says what changed and that everything
else stayed the same: "LCH lightness 62.0. Everything else
unchanged." You hear what you are changing while you change it,
instead of typing a number and finding out after.

The channels act on the same target as the color adjustments —
the picker at the top of the tab: the current color, or the
selected element. (A layer or a depth is many colors at once,
and channels edit one color at a time, so for those targets the
channel sliders grey out and a note under the group says so.)
Changing the View menu's format rebuilds the group as the new
format's channels. The keys are the tab's own: arrows move by
the channel's smallest step, Page Up and Page Down jump by your
page-jump percent, Home and End go to the ends, Ctrl plus an
arrow hops from channel to channel.

The ranges are each system's full range, and the fine-grained
systems move finer than their readouts print: XYZ channels run
0 to 110 in hundredths (the scale is relative to the D65 white
point, not a percent — white's own Z is 108.88, so the range
passes 100), xyY chromaticity moves in ten-thousandths, and the
Lab family's values move in tenths. Three honest exceptions:

- Decimal is one packed number, 0 to 16777215. A single slider
  over sixteen million steps cannot be stepped through, so its
  channels are its three bytes — the red, green and blue bytes
  the number is made of. Slide a byte and you are editing the
  decimal number, one base-256 digit at a time.
- Web safe sliders step by 51: the web-safe grid has only six
  stops per channel (0, 51, 102, 153, 204, 255), and the color
  snaps to the grid — the sliders show the color that landed.
- Binary has no channel sliders at all. Binary is a readout,
  not an editing system: it writes the color out as bits, and
  bits are not values you can slide. The group shows that
  sentence instead; pick any other format to edit channels.

THE MENUS

Menus are grouped by the job at hand, and menu items that cannot
do anything right now are greyed out: the color schemes need a
palette color selected, the Arrange commands need an element
selected, and resting on any menu item shows a line in the status
bar saying what it does.

- File: open or save a design; start one from a template; Import —
  bring in an image's colors, a CSS file's colors, or an HTML page
  as a design, compare two images, or describe an image as a
  tactile grid; Export — write the design out as a PNG, a caption
  sheet, CSS variables, a summary (recreation file), an HTML page,
  an animated GIF or HTML page, an animation description, or for
  touch as a raised-line SVG, a tactile sheet, or braille labels;
  exit.
- Color: grow the palette from the selected color — Schemes
  (complementary, split complementary, analogous, triadic,
  tetradic) and Shades, Tints and Tones; load the starter colors;
  remove the selected color; set the design background to the
  selected color.
- Design: canvas size; Add — a rectangle, ellipse, stroke, text,
  image, or symbol; Arrange Selected Shape — move, rotate, resize,
  recolor, or remove the element selected in the Elements list or
  tab; the Elements and Layers dialogs; Animation; Describe Area;
  Project Statistics; Project Options; save the current brush and
  open the brush library.
- View: how color values are shown and spoken — hex, RGB, HSL or
  HSV — and whether slider changes announce that everything else
  stayed the same.
- Help: the design-terms Dictionary, this help file, the Sound Cues
  toggle, and About.

SETTINGS: WHAT EACH ONE CHANGES

- Sound Cues (Help menu, starts on): whether drawing changes also
  play a tone. The tone's stereo position says left or right, its
  pitch says height, its length says size. Turning it off changes
  nothing about the spoken readouts.
- Color Values (View menu, starts as Hex): the format every color
  value is shown and spoken in — palette list, readout, status
  line. Hex, RGB, HSL or HSV. Saved files (captions, summaries)
  stay in hex so they never change under you. The choice lasts for
  the session.
- Canvas Size (Design menu): the drawing's width and height in
  pixels. Everything is placed inside it; the statistics line and
  the tactile grid describe positions against it.
- The current brush (set in Design, Add, Stroke, and by applying a
  brush from the Brush Library): the width in pixels, the color,
  and the pattern (solid, dashed or dotted) that new strokes are
  drawn with. Changing it never alters strokes already drawn.
- Extract count (asked by File, Import, Open Image and Extract
  Colors from Image): how many colors to take from the image,
  1 to 64, 8 by default.
- Frame duration (Animation dialog, Set Duration): how long one
  animation frame shows, 20 to 60000 milliseconds. Each frame has
  its own.
- Tactile grid size (File, Import, Describe Image as Tactile
  Grid): the grid's columns and rows, 24 by 12 by default. More
  cells mean finer detail and a longer sheet.
- Text size (asked by Design, Add, Add Text): the placed text's
  point size, 6 to 200. It sets the text element's box and, in the
  HTML export, its real font size.
- Pinned sliders (Design, Project Options): which sliders this
  project keeps on its Sliders tab. Starts empty — pin only what
  you use. Saved in the design file.
- Color adjustments act on (Project Options, and the picker at
  the top of the Sliders tab): the current color, the selected
  element, or the whole current layer. Saved in the design file.
- Page jump (Project Options): how far Page Up and Page Down move
  a slider, as a percent of its range — 10 percent to start.
  Saved in the design file.
- Announce Slider Changes (View menu, starts on): moving a slider
  also says that everything else stayed the same. Off: only the
  slider's name and new value are spoken.
- Sound cue volume (a slider, when pinned): how loud the sound
  cues play, 0 to 100 percent. 100 is how they have always
  sounded. Lasts for the session.

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
4. Design, Add, Rectangle: X 0, Y 300, Width 800, Height 150. A red
   band sits under the orange. Its cue sounds lower than the first:
   it is further down the canvas.
5. Hex: 402E32, Apply Hex. Design, Add, Rectangle: X 0, Y 450,
   Width 800, Height 150. The dark ground band fills the bottom, and
   its cue is the lowest of the three.
6. File, Export, Summary (recreation file) and save it. That file now
   holds the numbered recipe for this exact picture. File, Export, PNG
   saves the picture itself.

WORKED EXAMPLE 2: A SIMPLE HOUSE WITH A SUN

Goal: a house body, a door, and a round sun, using two layers.

1. Hex: 3A7CA5, Apply Hex (Color, Load Starter Colors adds this
   same blue). Design, Add, Rectangle: X 300, Y 320, Width 200,
   Height 160. That is the house body, just below center — its cue
   pans to the middle and sounds below middle pitch.
2. Hex: 8A5A2B, Apply Hex. Design, Add, Rectangle: X 375, Y 400,
   Width 50, Height 80. The door stands in the middle of the body.
3. Design, Layers. Choose Add Layer, name it Sky, then Set Current,
   then Close. New shapes now land on the Sky layer.
4. Hex: F2C230, Apply Hex. Design, Add, Ellipse: X 620, Y 60,
   Width 110, Height 110. Equal width and height make the ellipse a
   circle: the sun. Its cue pans right and sounds high — top right,
   exactly where a sun belongs.
5. Elements tab check: three elements, the sun listed under layer Sky.
6. File, Export, Captions saves a sheet describing every shape in
   words, including how each fill contrasts with the background.

WORKED EXAMPLE 3: A FACE

Goal: a round head, two eyes, and a smile drawn as a stroke.

1. Hex: F2C09A, Apply Hex. Design, Add, Ellipse: X 250, Y 100,
   Width 300, Height 300. The head, centered, near the top.
2. Hex: 26221F, Apply Hex. Design, Add, Ellipse: X 330, Y 210,
   Width 30, Height 40 for the left eye, then again with X 440 for
   the right eye. The right eye's cue pans a little further right.
3. Design, Add, Stroke. Enter the points: 340,320 400,350 460,320 —
   three points dipping in the middle, a smile. Set the brush width
   to 6. The shapes list describes it as a stroke through 3 points.
4. To nudge the smile down: select it in the Elements list, then
   Design, Arrange Selected Shape, Move, and shift down by 10. The
   readout announces the shift, and the cue sounds a touch lower.

WORKED EXAMPLE 4: A PALETTE FROM A PHOTO, THEN SHADES

Goal: take the colors of a photo you already like, then build a
matching set of darker shades for text or borders.

1. File, Import, Open Image, and pick the photo. The palette is replaced by
   the image's dominant colors, largest first, each with the percent
   of the image it covers. The largest becomes the design background.
2. Move through the palette list and listen to the readouts. Note the
   contrast line for any color you might put text on: AA normal text
   needs 4.5 to 1 or more.
3. Select a color, then Color, Shades Tints and Tones, Add 4 Shades.
   Four darker versions join the palette, same hue and chroma,
   lightness stepped down.
4. Select one of the shades, then Color, Set Background to Selected
   Color, and hear how the readouts of the other colors change:
   contrast is always judged against the current background.
5. File, Export, CSS saves the whole palette as CSS variables, each
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

View menu: Color Values as Hex, RGB, HSL or HSV — and now also
CMYK, CIE XYZ, xyY, CIE Lab, CIE LCH, CIE LUV, Hunter Lab, decimal,
web safe and binary. Every spoken and displayed color value follows
the choice — the palette list, the selected-color readout, the
status line. Captions and the summary (recreation) file keep hex,
because those are saved file formats and stay stable. The choice is
yours per session; the app starts in hex.

What each system is:

- Hex, RGB, HSL, HSV: the screen systems, as before.
- CMYK: print inks — cyan, magenta, yellow and black percents.
- CIE XYZ: the master numbers every CIE system is computed from
  (D65 white). xyY is the same color as chromaticity x and y plus
  luminance — the form the wavelength math uses.
- CIE Lab and CIE LCH: lightness with opponent axes (Lab), or
  lightness, chroma and hue (LCH, the polar form of Lab).
- CIE LUV: Lab's sibling system, built for lights and displays.
- Hunter Lab: the older 1948 system. It shares Lab's name and
  nothing else — its numbers for a color are different numbers.
- Decimal: the color as one number, 0 (black) to 16777215 (white).
- Web safe: the color snapped to the nearest of the 216 web-safe
  colors (channels in steps of 51). This one shows the snapped
  color, not your exact color.
- Binary: the color as 24 ones and zeros, one byte per channel.

COLOR TOOLS

Color, Color Tools is the workshop. Pick a first and a second
palette color, then:

- Blend mixes them in OKLab — perceptual mixing, so 50 percent
  looks halfway, not just averaged. The slider sets how much of
  the second color is in the mix.
- Subtract takes each channel of the second color away from the
  first (floored at zero). Subtracting black changes nothing;
  subtracting white gives black.
- Gradient builds a ramp between the two colors — you say how
  many steps — and adds the whole ramp to the palette.
- Contrast Check reports the WCAG contrast ratio between the two
  colors with a verdict: excellent (7 to 1 and up, any text size),
  good (4.5, normal text), fair (3, large text only), poor.
- Dominant Wavelength reports the first color's dominant
  wavelength in nanometres, with its excitation purity. Honest
  limits: grays have no wavelength at all, and purple-family
  colors have no *dominant* wavelength — theirs is reported as a
  complementary wavelength (the spectrum color that mixes with
  them to make white), never silently swapped.
- Random Color adds a color nobody picked.
- Frequency to Color (experimental) folds a sound frequency up
  by octaves into the visible band and gives that light's color:
  440 hertz lands at about 620 nanometres, orange-red. This is an
  artistic convention — octaves mapped to octaves because both
  are waves — not physics; a note is not really a color.

Color, Text Readability checks every text element in the design
against the design background and reports each one as easy to
read (contrast 4.5 to 1 or better), hard to read (3 to 1), or
unreadable (below 3 to 1). It measures against the background
only; a shape painted directly behind a text element is not
accounted for, and the report says so.

SCHEME SETTINGS: TWO WHEELS AND A SPREAD

Color, Schemes, Scheme Settings holds the scheme parameters.
The harmony wheel chooses how hue schemes rotate: the screen
wheel (RGB hues, what the app has always done) or the paint
wheel (RYB — the artists' wheel, where red's opposite is green
and yellow's opposite is violet, the way paint actually mixes).
The analogous spread sets how far the Analogous scheme steps
either side of the base color, 5 to 90 degrees (30 to start).
Shades, tints and tones are lightness and chroma ladders; the
wheel never affects them.

EXTRACTING MORE COLORS

File, Import, Open Image and File, Import, Extract Colors from Image both ask how
many colors to extract: 8 by default, up to 64. Open Image replaces
the palette and sets the background to the largest color, as
before. Extract Colors from Image only adds to the palette — use it
to mine any image for colors without touching your design.

TEXT AND IMAGE ELEMENTS

Design, Add, Text places words in the design: type the phrase, name
a font, pick a point size, then place it by measurement like any
other shape. Its caption is the phrase itself unless you write a
different one. In an exported HTML page the phrase is real text with
real font CSS; in a PNG the app draws it in a built-in block font
(the named font travels with the design and is used by the HTML
export).

Design, Add, Image places an image file in the design, scaled into
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

Three exports play or tell the film, all under File, Export:

- Animated GIF writes a looping animated GIF, one picture
  per frame, each shown for its own duration.
- Animated HTML writes a standalone web page that steps
  through the frames with a CSS animation — and lists the
  frame-by-frame description as readable text under the stage.
- Animation Description writes the film as words: every
  frame in order, with its duration and what it shows or what
  changed, ready to read or send.

TACTILE: FEELING A DESIGN

A design can be read by touch as well as by ear. Four tools, all
under File, Export, except Describe Area:

- Raised-Line SVG writes the design as outlines only —
  every shape a black stroked outline on white, nothing filled.
  Each layer gets its own line style (solid, long dashes, short
  dashes, dots), so layers can be told apart by hand. Print it on
  swell paper or emboss it and the lines rise. A text element is
  drawn as its braille translation inside an outline box: the
  braille is what rises, not the letter shapes.
- Tactile Sheet writes the design as a text walk: layer by
  layer, element by element in reading order (top to bottom, left
  to right), each element with its plain-language description, its
  caption, and its caption again as a braille label. Hidden layers
  are left out and noted.
- Braille Labels writes just the labels: one numbered line
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

DEPTH: BACK, MIDDLE AND FRONT

A design can say how deep its scene runs. Every layer can carry
a depth role — background, midground, foreground or sideground —
set in Design, Layers: select the layer, then choose its role
from the Depth role choice under the list. A layer with no role
behaves exactly as layers always have; the role never changes
how anything paints. It changes what the depth tools say and do:

- The tactile sheet describes the background first, as the base
  the rest stands on, then the midground, the foreground and
  the sideground. Layers with no role keep their paint order,
  last.
- The color adjustments can act on a whole depth at once. When
  an adjustment slider is pinned, the target picker on the
  Sliders tab (and in Project Options) offers Background
  layers, Midground layers and Foreground layers alongside the
  current color, the selected element and the current layer:
  fog the background, sharpen the foreground, and everything
  else stays put. One change to a whole depth is one step in
  the log.
- Design, Viewability reports how the design reads from where
  a viewer stands. Give a viewing angle in degrees — 0 is
  straight on from the front, and an element rotated more than
  90 degrees away from your angle shows its back — and a depth
  to judge (all depths, or one of background, midground,
  foreground). Each element is reported in plain words:
  viewable; partly hidden or hidden behind the element painted
  over it (an element counts as hidden when one box covers
  nearly all of it, partly hidden from 60 percent covered);
  turned away at so many degrees; or outside the chosen depth.
  The maths uses each element's box and rotation only, so
  curves, holes and see-through parts are not taken into
  account — the report says so at its top.

Sharp and rounded: a rectangle's corners can round over. The
Roundedness slider (pin it in Project Options) runs from 0 —
sharp corners — to 50 — fully rounded, each corner's arc half
the rectangle's shorter side. The Resize dialog in Design,
Elements carries the same Corners value for a selected
rectangle. Rounded rectangles paint round in the PNG, carry a
border-radius in the HTML export, and describe themselves as
"rounded rectangle, corners 30 percent". Ellipses are already
round and strokes have no corners, so roundedness does not
apply to them.

Scenes within scenes: File, Save Scene writes the current
design as a scene file (an ordinary .bdd design file). File,
Insert Scene places a saved scene's elements onto the current
layer of the design you have open, as one group: every inserted
element's caption gains the scene's name in front of it —
"Harbour: ..." — so the group reads as a group in the elements
list, the tactile sheet and every export. Build a harbour once;
drop it into every poster after that.

The Portrait and Landscape templates (File, New from Template)
are built in depth style: their layers arrive already tagged —
a wall as background, a framed picture as midground, the sitter
as foreground; a sky and far hills as background, the near
ground as foreground.

SYMBOLS AND TEMPLATES

Design, Add, Symbol stamps a ready-made symbol — star, heart,
arrow, check, cross, sun, moon — wherever you place it, at
whatever size. A symbol is ordinary strokes under one caption, so
it moves, resizes, recolors and exports like anything else.

File, New from Template starts a design from a layout: a birthday
card, a simple house, a face, a chart, a portrait or a landscape.
Loading a template replaces the design you have open, and the
template is a normal design from then on — change anything, then
Save Design.

File, New from Pattern starts a design as a pattern woven from
two of your palette colors: a checkerboard, stripes, or dots.
You pick which color makes the marks and which makes the ground;
the design's background is the ground color, and the marks sit
on one layer tagged as the background depth. From there it is a
normal design — recolor the marks, add elements over the weave.

PICTURES BY TOUCH AND COMPARISON

File, Import, Describe Image as Tactile Grid turns any picture into a
grid you can feel cell by cell: 24 by 12 cells by default (you
choose the size). Each cell is the picture's average shade there,
shown two ways — as a braille shade pattern (more dots means
darker, the scale is printed at the top) and in words, row by
row: "columns 1 to 6 bright; column 7 dark". The sheet ends with
where the darkest and brightest parts sit. A screenshot becomes
readable this way: dark text blocks and bright backgrounds show
up as shade shapes.

File, Import, Compare Images sets two pictures side by side and reports,
in plain sentences: whether they are the same size; which palette
colors they share and which belong to only one of them; which is
brighter overall; and which is sharper — the blurrier one is
named, so you can tell an original from a soft copy.
