# Changelog

Readable release history for Blind Designer (renamed from Transversion at 0.2.0), newest first. Each release's
section is its GitHub release note, taken from this file by the release
workflow.

## 0.16.0

- Feedback fixes, from hands-on testing of 0.15.0. The
  Sliders tab is never empty again: a project with no sliders
  pinned now shows every slider (pinning in Project Options
  trims the tab to a chosen set, as before), under a note
  saying so. The setup wizard assigns the sliders each
  project kind works with — finishing it pins the kind's
  default set (restating a goal tops the pins up), and its
  status line now names what it did: the goal set, how many
  sliders were pinned, and the seeded canvas. The wizard
  also opens by itself at first launch, on the blank
  project; cancelling leaves the blank project untouched.
  The element wizard's Finish now says on the status line
  what it set. Menus regrouped: Embed Feed Snapshot and
  Embed Pasted Post moved from Design into File, Import,
  where they belong; Design gained Text (Text Studio,
  Replacement Variables) and Review (Inspect Selected,
  Viewability, Recognizability, Project Statistics)
  submenus; View's fourteen color formats now sit in one
  Color Format submenu. Focus Mode brings its window to
  the front and puts the keyboard in its text. Lists Side
  by Side lands the keyboard on the Palette list after the
  switch. Unsaved work can no longer be lost silently:
  opening a design, starting from a template, a pattern or
  the wizard, and exiting all ask first — Save, Don't Save,
  or Cancel — whenever the project has changed since it was
  last saved or opened. New: File, New Window opens a
  second Blind Designer window (a separate copy of the
  app) so two projects can be open at once.

## 0.15.0

- Text Studio wave — styled text, text transforms, embeds,
  practice sheets, PDF and document/presentation exports.
  Text elements gain a style: bold, italic, underline, a
  heading level (body, heading 1–3) and alignment (left,
  centered, right), persisted as one trailing note in the
  design file (unstyled files are byte-identical) and spoken
  in every description. Design, Text Studio: an editor over
  the project's text elements — rewrite words, set the
  style, sort an element's lines A–Z or Z–A, spread all text
  elements into N columns in reading order, and build a
  grid of text elements from a pasted tab/newline table,
  each change one logged step. HTML export honors styles as
  real markup (strong/em/u, h1–h3, text-align); the PNG
  renderer draws bold heavier, underlines, and aligns the
  block — italic is spoken and real in markup but drawn
  upright, because the built-in block font has no slanted
  letters. Design, Embed Feed Snapshot: fetch an RSS/Atom
  feed and place its latest item titles as one captioned
  text element — a snapshot that refreshes only when you
  re-embed; fetch failures are reported in plain words.
  Design, Embed Pasted Post: paste any copied text (a
  Facebook post, another Serrebi tool's words) as a text
  element captioned as pasted. File, Export, Print-Writing
  Practice Sheet: every text element printed large as block
  letters with a legibility line (size in words, contrast
  verdict labeled a rough guide). File, Export, PDF: a
  hand-built two-page PDF — the design raster as a
  FlateDecode image, then the words as real selectable text
  (Helvetica). File, Export, Presentation HTML: one slide
  per animation frame (or one for a still design), picture
  and words side by side. File, Export, Document
  (Markdown): the styled words in reading order plus every
  element's caption, with the design's PNG written beside
  it and referenced. File, Print via PDF: writes the PDF
  to a temp file and opens it in the default viewer —
  print from the viewer; the app does not drive the
  printer itself.

## 0.14.0

- Workflow wave B — replacement variables, focus mode,
  pointer drawing. Design, Replacement Variables: text
  elements and captions may carry {name} tokens (lowercase
  letters, digits, hyphens); the dialog offers one value
  field per token in the design and substitutes either in
  this project (one logged step; animation frames included)
  or in a copy saved as a new design file, the template
  staying untouched — tokens given no value stay as written
  and are named in the report either way. View, Focus Mode:
  a small non-modal window showing only the current focus,
  the statistics, and the goal report's built / not-yet
  lines, kept current as you work. Design, Draw with
  Pointer: press and drag (mouse, or touch delivered as
  mouse events) on a canvas-sized surface and the thinned
  path becomes one stroke in the current brush; keys 1 to 9
  set the brush width live, C cycles the brush colour
  through the palette, Escape cancels.

## 0.13.0

- Workflow wave A — the wizard frame and project goals
  (Elshara's design). One reusable step dialog (header always
  announcing "Step N of M", Back/Next/Finish/Cancel, Finish
  offered on every step) carries three wizards. The setup
  wizard (File, New with Wizard; Design, Setup Wizard) opens
  with "What do you want to create or build?" — the kind
  chosen (writing, image creation, cosmetic work, mixed,
  accessibility review, conversion) decides the focus
  questions, and any answer may be "no fixed answer", leaving
  that part of the goal deliberately open. The answers are
  the goal: the main window's statistics line and Project
  Statistics now carry goal match (settled criteria met) and
  creation (all criteria, open answers counted as not yet)
  percentages with per-criterion lines and suggested
  improvements. New wizard projects are seeded per kind
  (writing starts with a text element, image creation with a
  background-role layer and a grey ground, cosmetic work with
  the gray ramp). The element wizard (Design, Element Wizard;
  Elements context menu) walks one element's stack (what sits
  on it, what it sits on), slider edits (opacity,
  roundedness), and the inspector's review. The export wizard
  (File, Export, Export Wizard) steps through format,
  embedding (separate file, inline in an HTML page with a
  companion page for pictures, caption text alongside) and a
  spoken confirmation of what will be written. Goals and
  answers save in the design file.
- Recognizability (Design, Recognizability): per project
  type, how clearly someone else would recognize what is
  being created — word contrast for writing, element counts,
  standout contrast and depth layering for pictures — plainly
  labelled a rough guide, not a measurement.
- Grayscale start: new projects begin in grayscale (Project
  Options toggles it per project; saved in the file), the
  gray ramp joins the palette from Project Options, and
  Design, Apply Colour Treatment gives a layer's or a whole
  depth's grayscale shapes the palette colours in paint
  order (locked shapes stay grayscale and are counted;
  coloured shapes are never touched).

## 0.12.0

- Interface wave 2 (Elshara's hands-on requests): elements can
  be locked (Design, Lock/Unlock Selected) — bulk layer and
  depth color adjustments skip locked elements and announce
  "N changed, M locked stayed the same"; direct edits still
  work, and the lock is saved in the design file. Enter on any
  main-window list opens the inspector: what the element, layer
  or color is, where it is used, what can change it, and what
  happens if it is left alone (nothing — nothing changes a
  design by itself). Every list has a context menu, empty lists
  included: an empty palette offers the three ways colors
  arrive, an empty elements list offers the Add items, the
  sliders page offers Project Options where the pins live.
  Design, Search and Insert (Ctrl+F) searches palette colors,
  starter colors, elements, symbols, templates and patterns by
  name or hex fragment, and inserts the chosen thing at the
  cursor — colors to the brush, elements and symbols onto the
  canvas — with a report of what landed where. File, New from
  Pattern weaves a checkerboard, stripes or dots design from
  two palette colors. View, Lists Side by Side puts the Palette
  and Elements lists on one tab in a splitter (this session
  only, like the other View choices).

## 0.11.0

- Color channels as sliders (Elshara: "the hex/lch values
  should be just a selectable slider rather than just editable"):
  pin "Color channels (in current format)" in Project Options
  and the Sliders tab gains one slider per channel of the
  current View color format — hex bytes spoken as hex, LCH
  lightness/chroma/hue, CMYK inks, XYZ in hundredths, xyY
  chromaticity in ten-thousandths, the Lab family in tenths —
  each with a spoken name and readout, announcing what changed
  as it changes. The channels edit the adjustments' target
  color (current color or selected element) with direct-set
  semantics, and changing format rebuilds the group. Decimal
  edits as its three bytes (one packed number cannot be
  stepped through); binary stays a readout, and the group says
  why instead of offering sliders.

## 0.10.0

- Depth (Elshara's vision: shape, movement, depth of
  visibility): layers gain a depth role — background, midground,
  foreground, sideground, or no role — set in Design, Layers,
  spoken in the layer lists, saved in the design file, and the
  tactile sheet now describes the background first, as the base.
  The color adjustments can act on a whole depth at once:
  Background layers, Midground layers and Foreground layers join
  the adjustment target picker, one logged step per depth.
  Rectangles gain roundedness: a new Roundedness slider (0 sharp
  to 50 fully rounded) and a Corners field in the Elements
  dialog's Resize — rounded rectangles paint round in the PNG,
  carry a border-radius in HTML, and describe themselves as
  "rounded rectangle, corners 30 percent". New Design,
  Viewability reports how the design reads from a viewing angle
  and depth: each element viewable, partly hidden or hidden
  behind what paints over it, turned away at so many degrees, or
  outside the chosen depth (box maths, stated in the report).
  Scenes within scenes: File, Save Scene and Insert Scene store
  a design as a reusable group, its elements landing on the
  current layer with the scene's name prefixed to their
  captions. Two new templates, Portrait and Landscape, are built
  in depth style with their layers already tagged by role.
- Every color system (Elshara's colorhexa ask): the View menu's
  color format choice grows from 4 to 14 — CMYK, CIE XYZ, xyY,
  CIE Lab, CIE LCH, CIE LUV, Hunter Lab, decimal, web safe and
  binary join hex, RGB, HSL and HSV, and every spoken color value
  follows the choice. New Color, Color Tools dialog: blend two
  palette colors in OKLab at a slider percent, subtract, build a
  gradient ramp into the palette, random colors, a WCAG contrast
  checker with verdicts, and a dominant-wavelength readout —
  purples honestly report a complementary wavelength, grays none.
  Color, Text Readability reports every text element's contrast
  against the background as easy to read, hard to read, or
  unreadable. Scheme Settings (Color, Schemes) exposes the
  analogous spread in degrees and a second harmony wheel: RYB
  paint, where red's opposite is green. Experimental frequency
  to color folds a sound frequency up by octaves into the visible
  band, labelled experimental in the dialog and the help.
- Sliders (Elshara's request): a fourth notebook tab, Sliders,
  holds the settings a project wants to keep as native sliders —
  brush width, the current color's red/green/blue, transparency,
  element transparency and scale, stroke width, frame duration
  (the whole film at one pace), extract count, tactile grid size,
  sound cue volume, and eleven color adjustments (shade, tint,
  tone, chroma, undertone, temperature, vibrancy, brightness,
  clarity, fogginess, hardness). A project starts from scratch:
  NO sliders are pinned until Design, Project Options pins them;
  the pins, the adjustment target and the Page Up/Down jump
  percent are saved in the design file. Adjustments act on the
  current color, the selected element, or a whole layer at once,
  always computed from the color the target started with, so
  sliding back to zero restores it exactly; a layer adjustment is
  one logged step. Moving a slider announces what changed and
  that everything else stayed the same (View, Announce Slider
  Changes turns that wording off). Keys: arrows take the smallest
  step, Page Up/Down jump by the project's percent, Home/End go
  to the ends, Ctrl+arrows hop focus between sliders. The brush,
  animation-duration, extract-count and tactile dialogs now read
  their ranges from the same slider definitions, and new shapes
  are painted at the transparency slider's opacity (shape opacity
  is stored per element and renders blended).
- Elshara's feedback wave: lists now show only what the project
  actually contains — a new project starts with an empty palette
  (the starter colors arrive via Color, Load Starter Colors), and
  the Palette, Elements and Layers lists live in a tabbed notebook
  in the main window. Menus regrouped by task: File gains Import
  and Export submenus, Palette becomes Color (Schemes and Shades,
  Tints and Tones submenus), Design gains Add and Arrange Selected
  Shape submenus, and menu items and dialog buttons grey out when
  they need a selection that does not exist yet. A project
  statistics line stays current in the main window, with the full
  count under Design, Project Statistics. Every menu item now
  carries a plain-language help line, and the help file documents
  what every setting changes.
- Tactile output (issue #2): File, Export Raised-Line SVG writes the
  design as stroked outlines with a different line style per layer
  and text elements as braille labels; File, Export Tactile Sheet
  walks the design element by element in reading order with each
  element's plain-language description, caption and braille label;
  File, Export Braille Labels writes the elements' captions in
  Grade 1 (uncontracted) braille, one numbered label per element.
- Describe Area: Design, Describe Area names only the elements in
  one part of the canvas — a named region (top left, center, …) or
  a typed box — marking elements that are only partly inside.
- Symbols: Design, Add Symbol stamps star, heart, arrow, check,
  cross, sun and moon as ordinary strokes under one caption, placed
  in numbers or plain words like any other element.
- Templates: File, New from Template starts a design from a
  birthday card, a simple house, a face, or a chart; a template is
  an ordinary design that edits, saves and exports as usual.
- Image comparison: File, Compare Images reports two pictures'
  sizes, shared and unique palette colors, which is brighter, and
  which is sharper (the blurrier one is named).
- Tactile grid: File, Describe Image as Tactile Grid renders any
  picture (a screenshot, say) as a coarse grid of shades — braille
  shade patterns plus the same grid in words, row by row — with the
  darkest and brightest parts named.
- Animation: a design can be a short film of frames, each a full
  copy of the canvas with its own duration in milliseconds.
  Design, Animation lists the frames in plain language — what
  frame 1 shows, and what changed in each later frame ("the red
  ellipse moves right 20 pixels") — and adds, duplicates, deletes,
  reorders and re-times them, with Load into Canvas / Replace with
  Canvas as the edit loop. Frames save inside .bdd files (older
  files still open, as still designs). File gains three exports:
  a looping animated GIF, an animated HTML page (CSS animation
  plus the frame-by-frame description as readable text), and a
  spoken-style animation description file.
- Save and open designs: File, Save Design writes the whole design —
  canvas, background, layers, shapes, palette and the full step log —
  to a .bdd project file; File, Open Design reads it back.
- Palette provenance (spec q5): every palette color now says how it
  was added — typed in, extracted from a named image, generated by a
  scheme, or imported from a CSS/HTML source — in the palette list
  and in the saved project file.
- Color value preference: the View menu shows every spoken and
  displayed color value as hex, RGB, HSL or HSV.
- Extraction count: Open Image and the new File, Extract Colors from
  Image ask how many colors to extract (default 8, up to 64) instead
  of a fixed cap; Extract adds to the palette from any image without
  touching the design.

## 0.9.0

- Every color format, spoken your way: the View menu's color format
  now covers hex, rgb, rgba, hsl, hsv, CMYK, XYZ, xyY, CIE-LAB,
  CIE-LCH, CIE-LUV, Hunter-Lab, decimal, web safe, and binary — the
  whole app reads colors in the system you pick.
- Wavelength: colors report their dominant wavelength where one
  exists (purples honestly report a complementary wavelength, grays
  report none).
- Color, Color Tools: blend two colors by slider, subtract one from
  another, build a gradient into your palette, roll a random color,
  and check contrast between any two colors with a plain verdict.
- Color, Text Readability: every text element judged against the
  background — easy to read, hard to read, or unreadable.
- Schemes gain settings: analogous spread in degrees, and the RYB
  paint wheel as an alternate way to find harmonies.
- Frequency to color (experimental): type a sound frequency in Hz
  and hear the color it maps to by octaves into visible light.

## 0.8.0

- Sliders, Elshara's way: a Sliders tab in the main window with 24
  descriptive sliders — brush width, transparency, color channels,
  element scale, frame duration, sound volume, and the color
  adjustments: shade, tint, tone, chroma, undertone, temperature,
  vibrancy, brightness, clarity, fogginess, hardness.
- A new project pins NONE of them: Design, Project Options is where
  you choose your sliders, set what they aim at (the brush color, the
  selected piece, or the whole layer), and set your page-jump size.
- Every change announces itself: "Brush width 12 pixels. Everything
  else unchanged." (View, Announce Slider Changes turns it off.)
- Keys: arrows fine-step, Page up/down jump by your percentage, Home
  and End go to the ends, Control+arrows hop between sliders.
- Elements have real transparency now, set by the Transparency
  slider and spoken in their descriptions.

## 0.7.0

- Your lists tell the truth now: a new project starts empty — no
  colors, no shapes, nothing pretending to be there. Load Starter
  Colors (Color menu) adds the starter palette only when you ask.
- The lists live in the main window as tabs: Palette, Elements,
  Layers, right where you work, with the same actions as the menus.
- Menus rebuilt: Import and Export are submenus under File, Palette is
  now the Color menu with its groups tucked into submenus, and Design
  groups adding and arranging the same way. Shorter paths, spoken
  cleanly.
- Project statistics: a one-line summary stays current in the main
  window, and Design, Project Statistics reads the full counts.
- Every setting now says what it changes (Help, and the status line as
  you rest on each menu item); actions that need a selection grey out
  when there is nothing to act on.

## 0.6.0

- Feel your designs: File, Export Raised-Line SVG writes the design as
  outlines for swell-paper and embosser workflows — one line style per
  layer, text as braille. File, Export Tactile Sheet walks the design
  piece by piece in reading order, with braille labels. File, Export
  Braille Labels writes the labels on their own.
- Describe any area: Design, Describe Area hears only what is in one
  part of the canvas — "top left", "center", or your own box.
- Symbols: Design, Add Symbol stamps a star, heart, arrow, check,
  cross, sun or moon at any size, in any color.
- Templates: File, New from Template starts you from a birthday card,
  a simple house, a face, or a chart — then it is yours to change.
- Compare two images: File, Compare Images reports their sizes,
  shared and unshared colors, which is brighter, and which is blurrier.
- Any picture as touch: File, Describe Image as Tactile Grid turns a
  photo or screenshot into a grid of braille shades with the same grid
  in words, so you can feel where things are, cell by cell.

## 0.5.0

- Animation: Design, Animation turns your design into a film, pose by
  pose. Draw a pose, capture it as a frame, change the canvas, capture
  again. Frames can be duplicated, reordered, deleted, and given their
  own durations, and each frame is listed with what changed from the
  one before it, in plain words ("the red ellipse moves right 20
  pixels").
- Export the film three ways: an animated GIF, an animated HTML page,
  and a spoken frame-by-frame description you can read or share.
- Animations save inside your design files; older design files still
  open exactly as before.

## 0.4.0

- Save and open your designs: File, Save Design and Open Design keep the
  whole design — canvas, layers, shapes, palette, and the step-by-step
  log — in one file you can reopen and keep working on.
- Every palette color remembers where it came from: typed in, extracted
  from an image (with the image's name), generated from another color,
  or imported — and the palette reads that memory out.
- Your choice of color format: View menu picks hex, rgb, hsl or hsv,
  and everything the app speaks follows it.
- Extract colors from any image, as many as you ask for (up to 64),
  not just eight from a background.
- Text elements: add a phrase in a chosen font and size, placed by
  measurement like any other shape, captioned with the phrase itself.
- Image elements: place an image file into the design with a caption;
  it draws scaled into its box, and a missing file shows a captioned
  placeholder instead of failing.
- Saved brushes: keep named brushes (width, color, solid/dashed/dotted)
  and reuse them for strokes instead of rebuilding them each time.
- Elements dialog: Design, Elements lists every piece of the design in
  plain language — rename its caption, hear a full description, move,
  resize, recolor or delete it from one place.
- Plain-language input: position and size fields take words as well as
  numbers ("top left", "center", "a bit wider"), and color fields take
  names ("sky blue", "forest green").
- Everything you add is described as it lands: "long rising stroke from
  top left to center, dark blue."

## 0.3.0

- Sound cues while drawing: adding, moving, rotating or resizing a shape
  now plays a gentle tone alongside the spoken values — pitch follows
  height on the canvas, stereo position follows left-right, and bigger
  shapes sound longer. The decision Elshara left to our intuition,
  delivered: cues and speech together.
- Design dictionary: Help, Design Dictionary opens a searchable list of
  color and design terms, each with a plain-language definition.
- Help file: Help, Help and Worked Examples opens a guide with
  step-by-step examples — make a rectangle, a layered scene, a stroke —
  written to be followed by ear.
- Startup warnings fixed (thanks to Elshara's report): the app now
  carries a proper Windows manifest, and the status bar no longer trips
  a wxWidgets invalid-ID alert at launch.
- Web export: File, Export HTML Page writes the design as a standalone
  web page — palette as CSS variables, every shape positioned and
  captioned for screen readers.
- Web import: File, Import CSS Colors brings a stylesheet's colors into
  the palette, each with a note of where it came from. File, Import
  HTML Design turns a simple HTML page's styled elements into shapes
  with captions, so a site design can be explored and reworked by ear.

## 0.2.0

- Renamed: Transversion is now Blind Designer, the name Elshara chose.
  Same color engine, same designs — a new name and a bigger job.
- Drawing core: designs are now layered. Design, Layers adds, renames,
  removes, shows or hides named layers, and picks the layer new shapes
  land on. Hidden layers paint nothing.
- Every change is logged, step by step: canvas, background, layer,
  shape, shift, angle, brush. Each change is spoken with its exact
  values as it happens, and File, Export Summary writes the whole log
  as a recreation file — numbered instructions that rebuild the exact
  image from nothing.
- New shape: strokes. Design, Add Stroke takes typed x,y points and a
  brush width; the stroke paints through its points in the selected
  palette color.
- Shapes change by measurement: Design, Move Selected Shape (shift
  across and down), Rotate Selected Shape (angle in degrees), Resize
  Selected Shape, Set Selected Shape to Palette Color — all logged.
  Rectangles, ellipses and strokes all render rotated.

## 0.1.0

- First release. An accessible multispectral color creator: colors live in
  OKLCH and are never mixed or blended — complementary, split
  complementary, analogous, triadic and tetradic schemes, and shade, tint
  and tone ladders are pure parameter changes (hue angle, lightness step,
  chroma step), so every result is predictable before it is seen.
- Palette extraction from any image you feed it (background, header,
  cover): up to 8 colors ranked by coverage, each spoken as a caption.
  Free — no paid palette service.
- Every color is described in words: family, lightness and chroma
  percentages, hue angle, hex. WCAG contrast ratio against the design
  background with AA/AAA pass or fail, spoken.
- Designs: rectangles and ellipses filled with palette colors at set
  pixel positions and sizes on a set canvas. Export the rendered PNG, a
  caption sheet (hand-written captions win; generated fill-ins only where
  a shape has none), and the palette as CSS custom properties with each
  color's caption as its comment.
- Built with wxDragon and quiet-list for NVDA-first list behavior.
