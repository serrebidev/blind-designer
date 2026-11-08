//! design: a 2D canvas of palette-colored shapes, exportable as an image.
//!
//! A design is plain data: a size, a background color, and a list of
//! shapes in paint order. Captions are micro generative fill-ins — a shape
//! whose caption is empty gets one generated from a template (what it is,
//! what color it is in words, how it relates to the background); a shape
//! with a hand-written caption keeps it untouched. Generation only ever
//! fills gaps, never overwrites.

use patois::t;

use crate::{color_words, describe, font, Srgb};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
	Rectangle,
	Ellipse,
	Stroke,
	Text,
	Image,
}

impl ShapeKind {
	pub fn name(self) -> String {
		match self {
			ShapeKind::Rectangle => t("rectangle"),
			ShapeKind::Ellipse => t("ellipse"),
			ShapeKind::Stroke => t("stroke"),
			ShapeKind::Text => t("text"),
			ShapeKind::Image => t("image"),
		}
	}
}

/// How much a text element speaks like a heading: plain body
/// text, or one of three heading levels. The level matters to
/// the document-shaped exports (HTML headings, markdown
/// headings) and to how the text describes itself; it never
/// changes the glyph size — that stays the font size's business.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Heading {
	#[default]
	Body,
	H1,
	H2,
	H3,
}

impl Heading {
	/// Every level, body first: the order choosers list them.
	pub const ALL: [Heading; 4] = [Heading::Body, Heading::H1, Heading::H2, Heading::H3];

	/// The spoken name: "body text", "heading 1", and so on.
	pub fn name(self) -> String {
		match self {
			Heading::Body => t("body text"),
			Heading::H1 => t("heading 1"),
			Heading::H2 => t("heading 2"),
			Heading::H3 => t("heading 3"),
		}
	}

	/// The stable token project files store.
	pub fn token(self) -> &'static str {
		match self {
			Heading::Body => "body",
			Heading::H1 => "h1",
			Heading::H2 => "h2",
			Heading::H3 => "h3",
		}
	}

	pub fn from_token(token: &str) -> Option<Heading> {
		Heading::ALL.iter().find(|h| h.token() == token).copied()
	}
}

/// Where a text element's lines sit inside its box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
	#[default]
	Left,
	Center,
	Right,
}

impl TextAlign {
	/// Every alignment, left first: the order choosers list them.
	pub const ALL: [TextAlign; 3] = [TextAlign::Left, TextAlign::Center, TextAlign::Right];

	/// The spoken name: "left", "centered", "right-aligned".
	pub fn name(self) -> String {
		match self {
			TextAlign::Left => t("left"),
			TextAlign::Center => t("centered"),
			TextAlign::Right => t("right-aligned"),
		}
	}

	/// The plain chooser label: "Left", "Center", "Right".
	pub fn label(self) -> String {
		match self {
			TextAlign::Left => t("Left"),
			TextAlign::Center => t("Center"),
			TextAlign::Right => t("Right"),
		}
	}

	/// The stable token project files store.
	pub fn token(self) -> &'static str {
		match self {
			TextAlign::Left => "left",
			TextAlign::Center => "center",
			TextAlign::Right => "right",
		}
	}

	pub fn from_token(token: &str) -> Option<TextAlign> {
		TextAlign::ALL.iter().find(|a| a.token() == token).copied()
	}
}

/// The style a text element carries: emphasis flags, a heading
/// level, and an alignment. The default — no emphasis, body
/// text, left-aligned — is the state every text element made
/// before styles existed is in, and it writes nothing extra to
/// the project file.
///
/// Rendering honesty: the HTML export turns every part of the
/// style into real markup. The PNG renderer paints bold (drawn
/// twice, a pixel over), underline (a ruled line under the
/// words) and alignment (where each line starts in the box),
/// but italic is spoken, not slanted — the built-in bitmap font
/// has no italic letterforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextStyle {
	pub bold: bool,
	pub italic: bool,
	pub underline: bool,
	pub heading: Heading,
	pub align: TextAlign,
}

impl TextStyle {
	/// True when the style is the plain default: no emphasis,
	/// body text, left-aligned.
	pub fn is_default(&self) -> bool {
		*self == TextStyle::default()
	}

	/// The style in words, as descriptions speak it: "bold",
	/// "italic", "underlined", "heading 1", "centered" — only
	/// the parts that differ from plain body text, in a stable
	/// order. Empty for the default style.
	pub fn words(&self) -> Vec<String> {
		let mut out = Vec::new();
		if self.bold {
			out.push(t("bold"));
		}
		if self.italic {
			out.push(t("italic"));
		}
		if self.underline {
			out.push(t("underlined"));
		}
		if self.heading != Heading::Body {
			out.push(self.heading.name());
		}
		if self.align != TextAlign::Left {
			out.push(self.align.name());
		}
		out
	}

	/// The project-file token: `style:` plus the emphasis flags
	/// (b, i, u, in that order, any subset, possibly none), the
	/// heading token and the alignment token, dash-joined —
	/// "style:bi-h2-center". Written only for styled text, so
	/// plain text files keep their old shape byte for byte.
	pub fn file_token(&self) -> String {
		let mut flags = String::new();
		if self.bold {
			flags.push('b');
		}
		if self.italic {
			flags.push('i');
		}
		if self.underline {
			flags.push('u');
		}
		format!(
			"style:{flags}-{}-{}",
			self.heading.token(),
			self.align.token()
		)
	}

	/// Parse a file token written by `file_token`.
	pub fn from_file_token(token: &str) -> Option<TextStyle> {
		let spec = token.strip_prefix("style:")?;
		let mut parts = spec.split('-');
		let flags = parts.next()?;
		let heading = Heading::from_token(parts.next()?)?;
		let align = TextAlign::from_token(parts.next()?)?;
		if parts.next().is_some() {
			return None;
		}
		let mut style = TextStyle {
			heading,
			align,
			..Default::default()
		};
		for c in flags.chars() {
			match c {
				'b' => style.bold = true,
				'i' => style.italic = true,
				'u' => style.underline = true,
				_ => return None,
			}
		}
		Some(style)
	}
}

#[derive(Debug, Clone)]
pub struct Shape {
	pub kind: ShapeKind,
	pub x: i32,
	pub y: i32,
	pub w: u32,
	pub h: u32,
	pub fill: Srgb,
	/// Stroke path points (Stroke kind only; empty otherwise).
	pub points: Vec<(i32, i32)>,
	/// Brush width in pixels (Stroke kind only).
	pub brush: u32,
	/// Stamp pattern name (Stroke kind only): "solid", "dashed" or
	/// "dotted" — see `crate::brushes::PATTERNS`. Empty means solid.
	pub pattern: String,
	/// The phrase a Text shape shows (Text kind only).
	pub text: String,
	/// Font name a Text shape is set in (Text kind only). The raster
	/// renderer draws every font with the built-in bitmap font; the
	/// name is carried for the HTML export and the spoken description.
	pub font: String,
	/// Font size in points (Text kind only).
	pub font_size: u32,
	/// The text's style: emphasis, heading level, alignment
	/// (Text kind only). See `TextStyle`.
	pub style: TextStyle,
	/// Image file path an Image shape draws (Image kind only).
	pub image_path: String,
	/// Rotation in degrees, clockwise, around the shape's centre.
	pub angle: f64,
	/// Corner roundedness of a rectangle, as a percent: 0 paints
	/// sharp corners, 50 is fully rounded (each corner's arc has
	/// radius half the shorter side, measured as that percent of
	/// it). Only rectangles read it; ellipses are already round
	/// and strokes have no corners.
	pub corner: u8,
	/// How opaque the shape paints, as a percent: 100 paints the
	/// fill color exactly, lower values blend with whatever is
	/// already painted beneath, in paint order.
	pub opacity: u8,
	/// Locked in place against the bulk changes: layer-wide and
	/// depth-wide color adjustments skip a locked shape and say
	/// so in their counts. Direct edits to the shape itself —
	/// move, resize, recolor it one by one — still work; the
	/// lock guards against sweeping changes, not the owner's
	/// hand. Dynamic (unlocked) is the default and the state
	/// every shape made before locks existed is in.
	pub locked: bool,
	/// Hand-written caption, if any. `None` means "generate one for me".
	pub caption: Option<String>,
}

impl Shape {
	pub fn new(kind: ShapeKind, x: i32, y: i32, w: u32, h: u32, fill: Srgb) -> Shape {
		Shape {
			kind,
			x,
			y,
			w,
			h,
			fill,
			points: Vec::new(),
			brush: 3,
			pattern: String::new(),
			text: String::new(),
			font: String::new(),
			font_size: 0,
			style: TextStyle::default(),
			image_path: String::new(),
			angle: 0.0,
			corner: 0,
			opacity: 100,
			locked: false,
			caption: None,
		}
	}

	/// A text element: a phrase set in a named font at a point size,
	/// placed by its box like any other shape. Its caption defaults
	/// to the phrase itself.
	pub fn text(
		phrase: &str,
		font: &str,
		font_size: u32,
		x: i32,
		y: i32,
		w: u32,
		h: u32,
		fill: Srgb,
	) -> Shape {
		let mut s = Shape::new(ShapeKind::Text, x, y, w, h, fill);
		s.text = phrase.to_string();
		s.font = font.to_string();
		s.font_size = font_size.max(1);
		s
	}

	/// An image element: an image file drawn scaled into its box.
	/// `fill` is the placeholder color, painted as a framed outline
	/// when the file cannot be read.
	pub fn image(path: &str, x: i32, y: i32, w: u32, h: u32, fill: Srgb) -> Shape {
		let mut s = Shape::new(ShapeKind::Image, x, y, w, h, fill);
		s.image_path = path.to_string();
		s
	}

	/// A freehand-style stroke through `points` with a brush of `brush`
	/// pixels. The bounding box fields follow the points.
	pub fn stroke(points: Vec<(i32, i32)>, brush: u32, fill: Srgb) -> Shape {
		let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
		for &(x, y) in &points {
			x0 = x0.min(x);
			y0 = y0.min(y);
			x1 = x1.max(x);
			y1 = y1.max(y);
		}
		let (x, y, w, h) = if points.is_empty() {
			(0, 0, 0, 0)
		} else {
			(x0, y0, (x1 - x0) as u32, (y1 - y0) as u32)
		};
		let mut s = Shape::new(ShapeKind::Stroke, x, y, w, h, fill);
		s.points = points;
		s.brush = brush;
		s
	}

	/// A stroke drawn with a pointer: the raw path the hand
	/// travelled, thinned by `sample_points` at `min_dist`
	/// pixels, painted with the given brush width and color.
	pub fn stroke_from_points(raw: &[(i32, i32)], min_dist: i32, brush: u32, fill: Srgb) -> Shape {
		Shape::stroke(sample_points(raw, min_dist), brush, fill)
	}
}

/// Thin a raw pointer path to the points a stroke keeps: the
/// first point, every later point at least `min_dist` pixels
/// (straight-line distance) from the last kept one, and the
/// final point where the pointer lifted — so a fast scribble
/// becomes a smooth stroke instead of a thousand-point one,
/// and the stroke still ends where the hand stopped. A path of
/// zero or one points passes through unchanged. Distances are
/// compared squared, so no square roots are taken.
pub fn sample_points(raw: &[(i32, i32)], min_dist: i32) -> Vec<(i32, i32)> {
	let min = min_dist.max(0) as i64;
	let min2 = min * min;
	let mut kept: Vec<(i32, i32)> = Vec::new();
	for &p in raw {
		match kept.last() {
			None => kept.push(p),
			Some(&last) => {
				let dx = p.0 as i64 - last.0 as i64;
				let dy = p.1 as i64 - last.1 as i64;
				if dx * dx + dy * dy >= min2 {
					kept.push(p);
				}
			}
		}
	}
	if let Some(&last) = raw.last() {
		if kept.last() != Some(&last) {
			kept.push(last);
		}
	}
	kept
}

impl Shape {
	/// Centre the rotation and measurements turn around.
	pub fn center(&self) -> (f64, f64) {
		(
			self.x as f64 + self.w as f64 / 2.0,
			self.y as f64 + self.h as f64 / 2.0,
		)
	}
}

/// Paint one pixel of a shape. At full opacity the color lands
/// exactly — the design's promise that output colors are palette
/// colors. Below full opacity the color blends with the pixel
/// already painted, in paint order.
fn put_shape_pixel(img: &mut image::RgbImage, x: u32, y: u32, px: image::Rgb<u8>, opacity: u8) {
	if opacity >= 100 {
		img.put_pixel(x, y, px);
		return;
	}
	let old = img.get_pixel(x, y);
	let a = opacity as u32;
	let mix = |new: u8, old: u8| ((new as u32 * a + old as u32 * (100 - a)) / 100) as u8;
	img.put_pixel(
		x,
		y,
		image::Rgb([mix(px[0], old[0]), mix(px[1], old[1]), mix(px[2], old[2])]),
	);
}

/// Is a point at shape-local coordinates (lx, ly) — offsets from
/// the rectangle's centre — inside the rectangle with its corner
/// roundedness applied? The corner arcs have radius
/// corner-percent of half the shorter side, so 50 percent rounds
/// fully over. Sharp corners (0) are the plain box test.
fn rect_local_inside(shape: &Shape, lx: f64, ly: f64) -> bool {
	let (hw, hh) = (shape.w as f64 / 2.0, shape.h as f64 / 2.0);
	if lx.abs() > hw || ly.abs() > hh {
		return false;
	}
	let r = shape.corner.min(50) as f64 / 100.0 * hw.min(hh);
	if r <= 0.0 {
		return true;
	}
	// In a corner region, the point must fall within the arc
	// around that corner's centre; everywhere else it's inside.
	let (ax, ay) = (lx.abs() - (hw - r), ly.abs() - (hh - r));
	if ax <= 0.0 || ay <= 0.0 {
		return true;
	}
	ax * ax + ay * ay <= r * r
}

#[derive(Debug, Clone)]
pub struct Design {
	pub width: u32,
	pub height: u32,
	pub background: Srgb,
	pub shapes: Vec<Shape>,
}

impl Design {
	pub fn new(width: u32, height: u32, background: Srgb) -> Design {
		Design {
			width,
			height,
			background,
			shapes: Vec::new(),
		}
	}

	/// The hue relationship between two colors, in words, when it is one of
	/// the classic predictable ones. `None` when there is no clean relation
	/// (or either color is achromatic and hue means nothing).
	pub fn relation(a: Srgb, b: Srgb) -> Option<String> {
		let (oa, ob) = (a.to_oklch(), b.to_oklch());
		if oa.c < 0.02 || ob.c < 0.02 {
			return None;
		}
		let d = (oa.h - ob.h).abs();
		let d = if d > 180.0 { 360.0 - d } else { d };
		match d {
			x if x <= 10.0 => Some(t("same hue as")),
			x if (x - 30.0).abs() <= 10.0 => Some(t("analogous to")),
			x if (x - 120.0).abs() <= 10.0 => Some(t("triadic with")),
			x if (x - 180.0).abs() <= 10.0 => Some(t("complementary to")),
			_ => None,
		}
	}

	/// The caption for one shape: the hand-written one if present, else a
	/// generated fill-in describing the shape against the background.
	/// A text element's caption defaults to its phrase; an image
	/// element's names its file.
	pub fn caption_for(&self, shape: &Shape) -> String {
		if let Some(c) = &shape.caption {
			return c.clone();
		}
		if shape.kind == ShapeKind::Text {
			return shape.text.clone();
		}
		if shape.kind == ShapeKind::Image {
			return t("image from file %s, %s by %s pixels at %s, %s")
				.replacen("%s", &(image_file_name(&shape.image_path)).to_string(), 1)
				.replacen("%s", &(shape.w).to_string(), 1)
				.replacen("%s", &(shape.h).to_string(), 1)
				.replacen("%s", &(shape.x).to_string(), 1)
				.replacen("%s", &(shape.y).to_string(), 1);
		}
		let mut s = if shape.kind == ShapeKind::Stroke {
			t("stroke through %s points with a %s pixel brush, in %s")
				.replacen("%s", &(shape.points.len()).to_string(), 1)
				.replacen("%s", &(shape.brush).to_string(), 1)
				.replacen("%s", &(describe(shape.fill)).to_string(), 1)
		} else {
			t("%s filled with %s, %s by %s pixels at %s, %s")
				.replacen("%s", &(shape.kind.name()).to_string(), 1)
				.replacen("%s", &(describe(shape.fill)).to_string(), 1)
				.replacen("%s", &(shape.w).to_string(), 1)
				.replacen("%s", &(shape.h).to_string(), 1)
				.replacen("%s", &(shape.x).to_string(), 1)
				.replacen("%s", &(shape.y).to_string(), 1)
		};
		if shape.angle != 0.0 {
			s.push_str(&t("; rotated %s degrees").replacen("%s", &(shape.angle).to_string(), 1));
		}
		if let Some(rel) = Self::relation(shape.fill, self.background) {
			s.push_str(
				&t("; %s the background %s")
					.replacen("%s", &(rel).to_string(), 1)
					.replacen("%s", &(describe(self.background)).to_string(), 1),
			);
		}
		let ratio = shape.fill.contrast(self.background);
		s.push_str(&t("; contrast against background %s to 1").replacen(
			"%s",
			&format!("{:.1}", ratio),
			1,
		));
		s
	}

	/// Captions for the whole design: a one-line header plus one line per
	/// shape, in paint order.
	pub fn caption_sheet(&self) -> String {
		let mut out = t("Design %s by %s pixels, background %s.\n")
			.replacen("%s", &(self.width).to_string(), 1)
			.replacen("%s", &(self.height).to_string(), 1)
			.replacen("%s", &(describe(self.background)).to_string(), 1);
		for (i, shape) in self.shapes.iter().enumerate() {
			out.push_str(&format!("{}. {}\n", i + 1, self.caption_for(shape)));
		}
		out
	}

	/// Where a point sits on the canvas, in plain words: a 3 by 3 grid
	/// over the canvas — "top left", "center", "bottom right",
	/// "top center", "middle left" and so on. Fractions are 0.0 at the
	/// left/top edge to 1.0 at the right/bottom.
	pub fn position_word(nx: f32, ny: f32) -> String {
		let col = if nx < 1.0 / 3.0 {
			0
		} else if nx < 2.0 / 3.0 {
			1
		} else {
			2
		};
		let row = if ny < 1.0 / 3.0 {
			0
		} else if ny < 2.0 / 3.0 {
			1
		} else {
			2
		};
		match (row, col) {
			(0, 0) => t("top left"),
			(0, 1) => t("top center"),
			(0, 2) => t("top right"),
			(1, 0) => t("middle left"),
			(1, 1) => t("center"),
			(1, 2) => t("middle right"),
			(2, 0) => t("bottom left"),
			(2, 1) => t("bottom center"),
			_ => t("bottom right"),
		}
	}

	fn spot(&self, x: f64, y: f64) -> String {
		Self::position_word(
			(x / self.width.max(1) as f64) as f32,
			(y / self.height.max(1) as f64) as f32,
		)
	}

	/// A plain-language description of one shape — the automatic
	/// description the elements list uses and the app speaks when a
	/// shape is created without a caption: "long rising stroke from
	/// top left to center, dark blue". No hex codes, no coordinates:
	/// position and size in words, exact pixel numbers in parentheses
	/// where they pin the shape down.
	pub fn describe_plain(&self, shape: &Shape) -> String {
		let (cx, cy) = shape.center();
		let at = self.spot(cx, cy);
		let words = color_words(shape.fill);
		let mut out = match shape.kind {
			ShapeKind::Stroke => {
				if shape.points.len() < 2 {
					t("single brush dot at %s, %s")
						.replacen("%s", &(at).to_string(), 1)
						.replacen("%s", &(words).to_string(), 1)
				} else {
					let (fx, fy) = shape.points[0];
					let (lx, ly) = shape.points[shape.points.len() - 1];
					let from = self.spot(fx as f64, fy as f64);
					let to = self.spot(lx as f64, ly as f64);
					let (dx, dy) = (lx - fx, ly - fy);
					let dir = if dx.abs() <= 2 && dy.abs() > 2 {
						"vertical"
					} else if dy < -2 {
						"rising"
					} else if dy > 2 {
						"falling"
					} else {
						"level"
					};
					let len = ((dx as f64).powi(2) + (dy as f64).powi(2)).sqrt();
					let diag = ((self.width as f64).powi(2) + (self.height as f64).powi(2)).sqrt();
					let size = if len < diag * 0.2 {
						"short"
					} else if len < diag * 0.55 {
						"medium-length"
					} else {
						"long"
					};
					let mut s = t("%s %s stroke from %s to %s, %s")
						.replacen("%s", size, 1)
						.replacen("%s", dir, 1)
						.replacen("%s", &(from).to_string(), 1)
						.replacen("%s", &(to).to_string(), 1)
						.replacen("%s", &(words).to_string(), 1);
					if !shape.pattern.is_empty() && shape.pattern != "solid" {
						s.push_str(&t(", %s pattern").replacen(
							"%s",
							&(shape.pattern).to_string(),
							1,
						));
					}
					s
				}
			}
			ShapeKind::Text => {
				let font = if shape.font.is_empty() {
					t("the default font")
				} else {
					t("%s font").replacen("%s", &(shape.font).to_string(), 1)
				};
				let style_words = shape.style.words();
				let style = if style_words.is_empty() {
					String::new()
				} else {
					format!(", {}", style_words.join(", "))
				};
				t("text \"%s\" in %s at %s point%s, at %s, %s")
					.replacen("%s", &(shape.text).to_string(), 1)
					.replacen("%s", &(font).to_string(), 1)
					.replacen("%s", &(shape.font_size).to_string(), 1)
					.replacen("%s", &(style).to_string(), 1)
					.replacen("%s", &(at).to_string(), 1)
					.replacen("%s", &(words).to_string(), 1)
			}
			ShapeKind::Image => t("image %s at %s (%s by %s pixels)")
				.replacen("%s", &(image_file_name(&shape.image_path)).to_string(), 1)
				.replacen("%s", &(at).to_string(), 1)
				.replacen("%s", &(shape.w).to_string(), 1)
				.replacen("%s", &(shape.h).to_string(), 1),
			kind => {
				let area = (shape.w as f64 * shape.h as f64)
					/ (self.width.max(1) as f64 * self.height.max(1) as f64);
				let size = if area < 0.02 {
					"small"
				} else if area < 0.12 {
					"medium"
				} else if area < 0.35 {
					"large"
				} else {
					"huge"
				};
				let (kind_word, corners) = if kind == ShapeKind::Rectangle && shape.corner > 0 {
					(
						t("rounded rectangle"),
						t(", corners %s percent").replacen("%s", &(shape.corner).to_string(), 1),
					)
				} else {
					(kind.name(), String::new())
				};
				t("%s %s %s at %s (%s by %s pixels)%s")
					.replacen("%s", size, 1)
					.replacen("%s", &(words).to_string(), 1)
					.replacen("%s", &(kind_word).to_string(), 1)
					.replacen("%s", &(at).to_string(), 1)
					.replacen("%s", &(shape.w).to_string(), 1)
					.replacen("%s", &(shape.h).to_string(), 1)
					.replacen("%s", &(corners).to_string(), 1)
			}
		};
		if shape.opacity < 100 {
			out.push_str(&t(", %s percent opaque").replacen("%s", &(shape.opacity).to_string(), 1));
		}
		out
	}

	/// Render the design to an image: background fill, then each shape in
	/// order. Rectangles and ellipses with hard edges. A fully opaque
	/// shape's colors land exactly — the output colors are the palette
	/// colors, pixel for pixel; a shape below full opacity blends with
	/// what is already painted beneath it, in paint order.
	/// Strokes are stamped along their points with a square brush; a shape
	/// with a rotation angle turns around its centre.
	pub fn render(&self) -> image::RgbImage {
		let mut img = image::RgbImage::from_pixel(
			self.width,
			self.height,
			image::Rgb([self.background.r, self.background.g, self.background.b]),
		);
		for shape in &self.shapes {
			let px = image::Rgb([shape.fill.r, shape.fill.g, shape.fill.b]);
			if shape.kind == ShapeKind::Stroke {
				Self::paint_stroke(&mut img, shape, px);
				continue;
			}
			if shape.kind == ShapeKind::Text {
				Self::paint_text(&mut img, shape, px);
				continue;
			}
			if shape.kind == ShapeKind::Image {
				Self::paint_image(&mut img, shape, px);
				continue;
			}
			let (x0, y0) = (shape.x, shape.y);
			let (x1, y1) = (shape.x + shape.w as i32, shape.y + shape.h as i32);
			if shape.angle == 0.0 {
				for y in y0.max(0)..y1.min(self.height as i32) {
					for x in x0.max(0)..x1.min(self.width as i32) {
						let inside = match shape.kind {
							ShapeKind::Rectangle => rect_local_inside(
								shape,
								x as f64 + 0.5 - (x0 as f64 + shape.w as f64 / 2.0),
								y as f64 + 0.5 - (y0 as f64 + shape.h as f64 / 2.0),
							),
							_ => {
								let (cx, cy) = (
									x0 as f32 + shape.w as f32 / 2.0,
									y0 as f32 + shape.h as f32 / 2.0,
								);
								let (rx, ry) = (shape.w as f32 / 2.0, shape.h as f32 / 2.0);
								if rx <= 0.0 || ry <= 0.0 {
									false
								} else {
									let dx = (x as f32 + 0.5 - cx) / rx;
									let dy = (y as f32 + 0.5 - cy) / ry;
									dx * dx + dy * dy <= 1.0
								}
							}
						};
						if inside {
							put_shape_pixel(&mut img, x as u32, y as u32, px, shape.opacity);
						}
					}
				}
			} else {
				// Rotated: test each pixel near the shape by un-rotating its
				// centre around the shape centre, then the plain tests.
				let (cx, cy) = shape.center();
				let rad = ((shape.w as f64 / 2.0).powi(2) + (shape.h as f64 / 2.0).powi(2))
					.sqrt()
					.ceil() as i32 + 1;
				let (sin, cos) = (-shape.angle).to_radians().sin_cos();
				for y in (cy as i32 - rad).max(0)..(cy as i32 + rad).min(self.height as i32) {
					for x in (cx as i32 - rad).max(0)..(cx as i32 + rad).min(self.width as i32) {
						let (dx, dy) = (x as f64 + 0.5 - cx, y as f64 + 0.5 - cy);
						let (lx, ly) = (dx * cos - dy * sin, dx * sin + dy * cos);
						let inside = match shape.kind {
							ShapeKind::Rectangle => rect_local_inside(shape, lx, ly),
							_ => {
								let (rx, ry) = (shape.w as f64 / 2.0, shape.h as f64 / 2.0);
								rx > 0.0
									&& ry > 0.0 && (lx / rx) * (lx / rx) + (ly / ry) * (ly / ry) <= 1.0
							}
						};
						if inside {
							put_shape_pixel(&mut img, x as u32, y as u32, px, shape.opacity);
						}
					}
				}
			}
		}
		img
	}

	/// Stamp a stroke: walk each segment in half-pixel steps and paint a
	/// square of the brush width at every step. Points rotate around the
	/// stroke's centre when the shape carries an angle. The brush
	/// pattern gates the stamps: "dashed" paints 16 steps on, 16 off;
	/// "dotted" paints every tenth step; anything else is solid.
	fn paint_stroke(img: &mut image::RgbImage, shape: &Shape, px: image::Rgb<u8>) {
		if shape.points.is_empty() {
			return;
		}
		let (cx, cy) = shape.center();
		let (sin, cos) = shape.angle.to_radians().sin_cos();
		let pts: Vec<(f64, f64)> = shape
			.points
			.iter()
			.map(|&(x, y)| {
				if shape.angle == 0.0 {
					(x as f64, y as f64)
				} else {
					let (dx, dy) = (x as f64 - cx, y as f64 - cy);
					(cx + dx * cos - dy * sin, cy + dx * sin + dy * cos)
				}
			})
			.collect();
		let half = shape.brush as f64 / 2.0;
		let mut step: usize = 0;
		let mut stamp = |img: &mut image::RgbImage, sx: f64, sy: f64| {
			let on = match shape.pattern.as_str() {
				"dashed" => (step / 16) % 2 == 0,
				"dotted" => step % 10 == 0,
				_ => true,
			};
			step += 1;
			if !on {
				return;
			}
			let (w, h) = (img.width() as i32, img.height() as i32);
			for y in (sy - half).floor() as i32..(sy + half).ceil() as i32 {
				for x in (sx - half).floor() as i32..(sx + half).ceil() as i32 {
					if x >= 0 && y >= 0 && x < w && y < h {
						put_shape_pixel(img, x as u32, y as u32, px, shape.opacity);
					}
				}
			}
		};
		if pts.len() == 1 {
			stamp(img, pts[0].0, pts[0].1);
		}
		for seg in pts.windows(2) {
			let (ax, ay) = seg[0];
			let (bx, by) = seg[1];
			let len = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt();
			let steps = (len / 0.5).ceil().max(1.0) as u32;
			for i in 0..=steps {
				let t = i as f64 / steps as f64;
				stamp(img, ax + (bx - ax) * t, ay + (by - ay) * t);
			}
		}
	}

	/// Paint a text element with the built-in bitmap font: each glyph
	/// cell scales to the font size (7 rows to `font_size` pixels),
	/// characters advance 6 cells, and the text clips at the canvas
	/// edge. The phrase may run over several lines (feed snapshots
	/// and pasted posts do): each line starts lower by a glyph
	/// height plus a gap row, and the style positions every line
	/// inside the shape's box — left at the box's edge, centered,
	/// or right. Bold paints every glyph twice, the second pass a
	/// glyph pixel over; underline rules a line under each text
	/// line. Italic is spoken in the description but NOT slanted
	/// here: the bitmap font has no italic letterforms. The named
	/// font is not rasterized — see the field docs.
	fn paint_text(img: &mut image::RgbImage, shape: &Shape, px: image::Rgb<u8>) {
		let scale = ((shape.font_size.max(1) as f64 / font::GLYPH_H as f64).round() as i32).max(1);
		let (iw, ih) = (img.width() as i32, img.height() as i32);
		let fallback = font::glyph('?').unwrap();
		let line_step = (font::GLYPH_H as i32 + 2) * scale;
		for (li, line) in shape.text.split('\n').enumerate() {
			let ly = shape.y + li as i32 * line_step;
			if ly >= ih {
				break;
			}
			let chars = line.chars().count() as i32;
			let line_w = if chars == 0 {
				0
			} else {
				chars * font::ADVANCE as i32 * scale - scale
			};
			let start_x = match shape.style.align {
				TextAlign::Left => shape.x,
				TextAlign::Center => shape.x + (shape.w as i32 - line_w) / 2,
				TextAlign::Right => shape.x + shape.w as i32 - line_w,
			};
			let mut cx = start_x;
			for ch in line.chars() {
				let g = font::glyph(ch).unwrap_or(fallback);
				for (row, &bits) in g.iter().enumerate() {
					for col in 0..font::GLYPH_W {
						if (bits >> (font::GLYPH_W - 1 - col)) & 1 == 0 {
							continue;
						}
						// Bold's second pass sits one glyph pixel
						// to the right of the first.
						for pass in 0..if shape.style.bold { 2 } else { 1 } {
							for dy in 0..scale {
								for dx in 0..scale {
									let (x, y) = (
										cx + col as i32 * scale + dx + pass * scale,
										ly + row as i32 * scale + dy,
									);
									if x >= 0 && y >= 0 && x < iw && y < ih {
										put_shape_pixel(img, x as u32, y as u32, px, shape.opacity);
									}
								}
							}
						}
					}
				}
				cx += font::ADVANCE as i32 * scale;
			}
			if shape.style.underline && chars > 0 {
				let uy = ly + font::GLYPH_H as i32 * scale + scale / 2;
				for x in start_x..start_x + line_w {
					for dy in 0..scale {
						let y = uy + dy;
						if x >= 0 && y >= 0 && x < iw && y < ih {
							put_shape_pixel(img, x as u32, y as u32, px, shape.opacity);
						}
					}
				}
			}
		}
	}

	/// Paint an image element: the file scaled into the shape's box.
	/// When the file cannot be read, a placeholder is painted instead —
	/// the box outlined and crossed in the shape's fill color — so a
	/// missing image is visible as missing, never silently absent.
	fn paint_image(img: &mut image::RgbImage, shape: &Shape, px: image::Rgb<u8>) {
		let (iw, ih) = (img.width() as i32, img.height() as i32);
		if shape.w >= 1 && shape.h >= 1 {
			if let Ok(src) = image::open(&shape.image_path) {
				let small = src
					.resize_exact(shape.w, shape.h, image::imageops::FilterType::Triangle)
					.to_rgb8();
				for y in 0..shape.h as i32 {
					for x in 0..shape.w as i32 {
						let (tx, ty) = (shape.x + x, shape.y + y);
						if tx >= 0 && ty >= 0 && tx < iw && ty < ih {
							put_shape_pixel(
								img,
								tx as u32,
								ty as u32,
								*small.get_pixel(x as u32, y as u32),
								shape.opacity,
							);
						}
					}
				}
				return;
			}
		}
		// Placeholder: outline the box and cross it corner to corner.
		let (x0, y0) = (shape.x, shape.y);
		let (x1, y1) = (shape.x + shape.w as i32 - 1, shape.y + shape.h as i32 - 1);
		let plot = |img: &mut image::RgbImage, x: i32, y: i32| {
			if x >= 0 && y >= 0 && x < iw && y < ih {
				put_shape_pixel(img, x as u32, y as u32, px, shape.opacity);
			}
		};
		for t in 0..2 {
			for x in x0..=x1 {
				plot(img, x, y0 + t);
				plot(img, x, y1 - t);
			}
			for y in y0..=y1 {
				plot(img, x0 + t, y);
				plot(img, x1 - t, y);
			}
		}
		let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
		for i in 0..=steps {
			let x = x0 + (x1 - x0) * i / steps;
			let y = y0 + (y1 - y0) * i / steps;
			plot(img, x, y);
			plot(img, x, y0 + (y1 - y));
		}
	}

	/// Export the design's colors (background first, then shape fills,
	/// de-duplicated) as CSS custom properties with caption comments —
	/// the web-design hand-off.
	pub fn to_css(&self, prefix: &str) -> String {
		let mut colors: Vec<Srgb> = vec![self.background];
		for s in &self.shapes {
			if !colors.contains(&s.fill) {
				colors.push(s.fill);
			}
		}
		css_for_colors(&colors, prefix)
	}
}

/// The file name at the end of an image path (the whole string when
/// it has no file-name part) — descriptions name the file, not the
/// folder it lives in.
pub fn image_file_name(path: &str) -> String {
	std::path::Path::new(path)
		.file_name()
		.map(|n| n.to_string_lossy().into_owned())
		.unwrap_or_else(|| path.to_string())
}

/// CSS custom properties for an arbitrary color list, de-duplicated,
/// each with its spoken caption as a comment.
pub fn css_for_colors(colors: &[Srgb], prefix: &str) -> String {
	let mut seen: Vec<Srgb> = Vec::new();
	let mut out = String::from(":root {\n");
	for c in colors {
		if seen.contains(c) {
			continue;
		}
		seen.push(*c);
		out.push_str(&format!(
			"\t--{}-{}: {}; /* {} */\n",
			prefix,
			seen.len(),
			c.to_hex(),
			describe(*c)
		));
	}
	out.push_str("}\n");
	out
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn render_blends_a_translucent_shape_over_what_is_beneath() {
		let mut d = Design::new(4, 4, Srgb::WHITE);
		let mut s = Shape::new(ShapeKind::Rectangle, 0, 0, 4, 4, Srgb::BLACK);
		s.opacity = 50;
		d.shapes.push(s);
		let img = d.render();
		// Half black over white lands exactly halfway.
		assert_eq!(*img.get_pixel(2, 2), image::Rgb([127, 127, 127]));
	}

	#[test]
	fn render_at_full_opacity_is_exact() {
		let mut d = Design::new(4, 4, Srgb::WHITE);
		d.shapes
			.push(Shape::new(ShapeKind::Rectangle, 0, 0, 4, 4, Srgb::BLACK));
		assert_eq!(d.shapes[0].opacity, 100, "shapes start fully opaque");
		assert_eq!(*d.render().get_pixel(2, 2), image::Rgb([0, 0, 0]));
	}

	#[test]
	fn rounded_rectangles_cut_their_corners_and_say_so() {
		let mut d = Design::new(40, 40, Srgb::WHITE);
		let mut s = Shape::new(ShapeKind::Rectangle, 10, 10, 20, 20, Srgb::BLACK);
		d.shapes.push(s.clone());
		// Sharp: the very corner pixel paints.
		assert_eq!(*d.render().get_pixel(10, 10), image::Rgb([0, 0, 0]));
		s.corner = 50;
		d.shapes[0] = s;
		let img = d.render();
		// Fully rounded: the corner pixel is background again,
		// while the middle of each edge still paints.
		assert_eq!(*img.get_pixel(10, 10), image::Rgb([255, 255, 255]));
		assert_eq!(*img.get_pixel(20, 10), image::Rgb([0, 0, 0]));
		assert_eq!(*img.get_pixel(10, 20), image::Rgb([0, 0, 0]));
		assert_eq!(*img.get_pixel(20, 20), image::Rgb([0, 0, 0]));
		// The description names the rounding and its percent;
		// a sharp rectangle says neither.
		let plain = d.describe_plain(&d.shapes[0]);
		assert!(plain.contains("rounded rectangle"), "{plain}");
		assert!(plain.contains("corners 50 percent"), "{plain}");
		let sharp = Design::new(40, 40, Srgb::WHITE);
		assert!(!sharp
			.describe_plain(&Shape::new(ShapeKind::Rectangle, 0, 0, 5, 5, Srgb::BLACK))
			.contains("rounded"));
		// Ellipses ignore the corner field entirely.
		let mut e = Design::new(40, 40, Srgb::WHITE);
		let mut ball = Shape::new(ShapeKind::Ellipse, 10, 10, 20, 20, Srgb::BLACK);
		ball.corner = 50;
		e.shapes.push(ball);
		assert!(!e.describe_plain(&e.shapes[0]).contains("rounded"));
	}

	#[test]
	fn plain_description_mentions_opacity_only_below_full() {
		let d = Design::new(100, 100, Srgb::WHITE);
		let mut s = Shape::new(ShapeKind::Rectangle, 10, 10, 20, 20, Srgb::BLACK);
		assert!(
			!d.describe_plain(&s).contains("opaque"),
			"{}",
			d.describe_plain(&s)
		);
		s.opacity = 40;
		assert!(
			d.describe_plain(&s).ends_with(", 40 percent opaque"),
			"{}",
			d.describe_plain(&s)
		);
	}

	#[test]
	fn render_paints_shapes_in_order() {
		let mut d = Design::new(10, 10, Srgb::WHITE);
		d.shapes.push(Shape::new(
			ShapeKind::Rectangle,
			0,
			0,
			10,
			10,
			Srgb::new(255, 0, 0),
		));
		d.shapes.push(Shape::new(
			ShapeKind::Ellipse,
			2,
			2,
			6,
			6,
			Srgb::new(0, 0, 255),
		));
		let img = d.render();
		assert_eq!(img.get_pixel(0, 0), &image::Rgb([255, 0, 0]));
		assert_eq!(img.get_pixel(5, 5), &image::Rgb([0, 0, 255])); // ellipse center
		assert_eq!(img.get_pixel(2, 2), &image::Rgb([255, 0, 0])); // ellipse corner is outside
	}

	#[test]
	fn captions_fill_only_gaps() {
		let mut d = Design::new(100, 50, Srgb::WHITE);
		let mut s = Shape::new(ShapeKind::Rectangle, 0, 0, 10, 10, Srgb::new(15, 94, 104));
		let auto = d.caption_for(&s);
		assert!(auto.contains("rectangle"), "{auto}");
		assert!(auto.contains("teal"), "{auto}");
		s.caption = Some("my own words".to_string());
		assert_eq!(d.caption_for(&s), "my own words");
	}

	#[test]
	fn relation_detects_complementary() {
		let base = Srgb::new(200, 60, 40);
		let comp = base.to_oklch().complementary().to_srgb();
		assert_eq!(Design::relation(base, comp), Some("complementary to".to_string()));
		assert_eq!(Design::relation(base, base), Some("same hue as".to_string()));
		assert_eq!(Design::relation(base, Srgb::WHITE), None);
	}

	#[test]
	fn text_elements_render_their_phrase_and_caption_with_it() {
		let mut d = Design::new(120, 40, Srgb::WHITE);
		let t = Shape::text("Hi", "Arial", 14, 2, 2, 60, 20, Srgb::BLACK);
		// The caption defaults to the phrase itself.
		assert_eq!(d.caption_for(&t), "Hi");
		d.shapes.push(t);
		let img = d.render();
		// 'H' paints its left stem in the first glyph cell (scale 2).
		assert_eq!(img.get_pixel(2, 2), &image::Rgb([0, 0, 0]));
		// A hand-written caption still wins over the phrase.
		let mut t2 = Shape::text("Hi", "Arial", 14, 2, 2, 60, 20, Srgb::BLACK);
		t2.caption = Some("greeting".to_string());
		assert_eq!(d.caption_for(&t2), "greeting");
		// Plain description names the phrase, the font, the size, the spot.
		let plain = d.describe_plain(&d.shapes[0]);
		assert!(plain.contains("text \"Hi\""), "{plain}");
		assert!(plain.contains("Arial font"), "{plain}");
		assert!(plain.contains("14 point"), "{plain}");
		assert!(plain.contains("top left"), "{plain}");
	}

	#[test]
	fn image_elements_render_the_file_or_a_placeholder() {
		// A real file on disk: its pixels land in the shape's box.
		let dir = std::env::temp_dir();
		let path = dir.join(format!("blind-designer-test-{}.png", std::process::id()));
		image::RgbImage::from_pixel(4, 4, image::Rgb([10, 200, 30]))
			.save(&path)
			.unwrap();
		let path_str = path.to_string_lossy().into_owned();
		let mut d = Design::new(40, 40, Srgb::WHITE);
		d.shapes
			.push(Shape::image(&path_str, 5, 5, 8, 8, Srgb::new(255, 0, 0)));
		let img = d.render();
		assert_eq!(img.get_pixel(6, 6), &image::Rgb([10, 200, 30]));
		assert_eq!(img.get_pixel(0, 0), &image::Rgb([255, 255, 255]));
		let caption = d.caption_for(&d.shapes[0]);
		assert!(caption.contains("image from file"), "{caption}");
		assert!(caption.contains(".png"), "{caption}");
		std::fs::remove_file(&path).ok();
		// A missing file: the placeholder outline shows, in fill color.
		let mut d2 = Design::new(40, 40, Srgb::WHITE);
		d2.shapes.push(Shape::image(
			"/no/such/file.png",
			5,
			5,
			10,
			10,
			Srgb::new(255, 0, 0),
		));
		let img2 = d2.render();
		assert_eq!(img2.get_pixel(5, 5), &image::Rgb([255, 0, 0])); // corner of the frame
		assert_eq!(img2.get_pixel(20, 20), &image::Rgb([255, 255, 255])); // outside the box
		let plain = d2.describe_plain(&d2.shapes[0]);
		assert!(plain.contains("image file.png"), "{plain}");
	}

	#[test]
	fn plain_descriptions_speak_position_size_and_color() {
		let mut d = Design::new(800, 600, Srgb::WHITE);
		// A stroke from the top-left corner rising to the centre.
		let s = Shape::stroke(vec![(20, 500), (400, 300)], 3, Srgb::new(0, 0, 139));
		d.shapes.push(s);
		let plain = d.describe_plain(&d.shapes[0]);
		assert!(plain.contains("rising stroke"), "{plain}");
		assert!(plain.contains("from bottom left"), "{plain}");
		assert!(plain.contains("to center"), "{plain}");
		assert!(plain.contains("blue"), "{plain}");
		// A small rectangle in the top-left corner.
		let r = Shape::new(ShapeKind::Rectangle, 0, 0, 40, 30, Srgb::new(255, 0, 0));
		let plain = d.describe_plain(&r);
		assert!(plain.contains("small"), "{plain}");
		assert!(plain.contains("rectangle at top left"), "{plain}");
		assert!(plain.contains("(40 by 30 pixels)"), "{plain}");
		// The 3 by 3 position grid.
		assert_eq!(Design::position_word(0.0, 0.0), "top left");
		assert_eq!(Design::position_word(0.5, 0.5), "center");
		assert_eq!(Design::position_word(1.0, 1.0), "bottom right");
		assert_eq!(Design::position_word(0.5, 0.0), "top center");
		assert_eq!(Design::position_word(0.0, 0.5), "middle left");
	}

	#[test]
	fn brush_patterns_paint_less_than_solid() {
		let count = |pattern: &str| {
			let mut d = Design::new(120, 20, Srgb::WHITE);
			let mut s = Shape::stroke(vec![(2, 10), (117, 10)], 1, Srgb::BLACK);
			s.pattern = pattern.to_string();
			d.shapes.push(s);
			d.render()
				.pixels()
				.filter(|p| **p == image::Rgb([0, 0, 0]))
				.count()
		};
		let solid = count("solid");
		let dashed = count("dashed");
		let dotted = count("dotted");
		assert!(solid > 100, "solid stroke painted only {solid} pixels");
		assert!(
			dashed < solid && dashed > solid / 3,
			"dashed {dashed} vs solid {solid}"
		);
		assert!(dotted < dashed, "dotted {dotted} vs dashed {dashed}");
	}

	#[test]
	fn sampling_keeps_first_far_and_final_points() {
		// One pixel steps along a line, sampled at 4: every
		// fourth point survives, plus the exact lift-off point.
		let raw: Vec<(i32, i32)> = (0..=10).map(|x| (x, 0)).collect();
		assert_eq!(
			sample_points(&raw, 4),
			vec![(0, 0), (4, 0), (8, 0), (10, 0)]
		);
		// Diagonal distance counts, not just axis steps:
		// (3,3) is ~4.24 away, (2,2) only ~2.83.
		assert_eq!(
			sample_points(&[(0, 0), (2, 2), (3, 3)], 4),
			vec![(0, 0), (3, 3)]
		);
		// A press and release on one spot stays one point;
		// an empty path stays empty.
		assert_eq!(sample_points(&[(5, 5)], 4), vec![(5, 5)]);
		assert!(sample_points(&[], 4).is_empty());
	}

	#[test]
	fn stroke_from_points_samples_and_bounds() {
		let raw: Vec<(i32, i32)> = (0..=10).map(|x| (x * 2, x)).collect();
		let s = Shape::stroke_from_points(&raw, 4, 3, Srgb::BLACK);
		assert_eq!(s.kind, ShapeKind::Stroke);
		assert_eq!(s.points, sample_points(&raw, 4));
		assert!(s.points.len() < raw.len());
		assert_eq!((s.x, s.y, s.w, s.h), (0, 0, 20, 10));
		assert_eq!(s.brush, 3);
	}

	#[test]
	fn text_styles_speak_their_words_and_default_speaks_none() {
		let mut d = Design::new(300, 100, Srgb::WHITE);
		let plain = Shape::text("Notes", "Arial", 14, 5, 5, 100, 20, Srgb::BLACK);
		assert!(plain.style.is_default());
		let words = d.describe_plain(&plain);
		assert!(!words.contains("bold"), "{words}");
		assert!(!words.contains("heading"), "{words}");
		assert!(!words.contains("centered"), "{words}");
		let mut styled = plain.clone();
		styled.style = TextStyle {
			bold: true,
			underline: true,
			heading: Heading::H1,
			align: TextAlign::Center,
			..Default::default()
		};
		let words = d.describe_plain(&styled);
		assert!(
			words.contains("bold, underlined, heading 1, centered"),
			"{words}"
		);
		// The file token round-trips through its parser.
		assert_eq!(
			TextStyle::from_file_token(&styled.style.file_token()),
			Some(styled.style)
		);
		assert_eq!(styled.style.file_token(), "style:bu-h1-center");
		assert_eq!(
			TextStyle::from_file_token("style:-body-left"),
			Some(TextStyle::default())
		);
		assert_eq!(TextStyle::from_file_token("style:x-body-left"), None);
		assert_eq!(TextStyle::from_file_token("style:b-h9-left"), None);
	}

	#[test]
	fn styled_text_renders_bold_wider_underlined_and_centered() {
		let render = |style: TextStyle| {
			let mut d = Design::new(200, 60, Srgb::WHITE);
			let mut t = Shape::text("I", "Arial", 14, 10, 4, 100, 40, Srgb::BLACK);
			t.style = style;
			d.shapes.push(t);
			d.render()
		};
		let count = |img: &image::RgbImage| {
			img.pixels()
				.filter(|p| **p == image::Rgb([0, 0, 0]))
				.count()
		};
		let plain = render(TextStyle::default());
		// Bold paints the glyph twice over: strictly more ink.
		let bold = render(TextStyle {
			bold: true,
			..Default::default()
		});
		assert!(
			count(&bold) > count(&plain),
			"bold {} vs plain {}",
			count(&bold),
			count(&plain)
		);
		// Underline adds a rule BELOW the glyph rows (glyph rows
		// end at y = 4 + 7*2 = 18 at this scale; the rule sits
		// at 4 + 14 + 1 = 19).
		let under = render(TextStyle {
			underline: true,
			..Default::default()
		});
		assert_eq!(*under.get_pixel(10, 19), image::Rgb([0, 0, 0]));
		assert_eq!(*plain.get_pixel(10, 19), image::Rgb([255, 255, 255]));
		// Centered: the narrow glyph moves to the middle of its
		// box. 'I' row 1 sets only its middle column: at scale 2
		// that is x = 10 + 2*2 = 14 from the left edge, and
		// x = 55 + 4 = 59 when the 10-pixel line is centered in
		// the 100-pixel box starting at 10.
		assert_eq!(*plain.get_pixel(14, 6), image::Rgb([0, 0, 0]));
		let centered = render(TextStyle {
			align: TextAlign::Center,
			..Default::default()
		});
		assert_eq!(*centered.get_pixel(14, 6), image::Rgb([255, 255, 255]));
		assert_eq!(*centered.get_pixel(59, 6), image::Rgb([0, 0, 0]));
		assert!(count(&centered) == count(&plain));
	}

	#[test]
	fn multi_line_text_paints_every_line() {
		let mut d = Design::new(200, 80, Srgb::WHITE);
		d.shapes
			.push(Shape::text("A\nB", "Arial", 14, 2, 2, 100, 60, Srgb::BLACK));
		let img = d.render();
		// Line one at the top; line two a glyph height plus gap
		// lower (step 18 at scale 2), its own pixels present.
		assert_eq!(*img.get_pixel(2, 2), image::Rgb([255, 255, 255])); // 'A' top row is 01110: first cell clear
		assert!(
			img.pixels()
				.filter(|p| **p == image::Rgb([0, 0, 0]))
				.count() > 20
		);
		let second_line_ink = (0..40).any(|x| *img.get_pixel(x, 2 + 18) == image::Rgb([0, 0, 0]));
		assert!(second_line_ink, "second line painted nothing");
	}

	#[test]
	fn css_export_lists_each_color_once() {
		let mut d = Design::new(10, 10, Srgb::new(15, 94, 104));
		d.shapes.push(Shape::new(
			ShapeKind::Rectangle,
			0,
			0,
			5,
			5,
			Srgb::new(15, 94, 104),
		));
		d.shapes.push(Shape::new(
			ShapeKind::Rectangle,
			5,
			5,
			5,
			5,
			Srgb::new(255, 128, 0),
		));
		let css = d.to_css("brand");
		assert_eq!(css.matches("--brand-").count(), 2);
		assert!(css.contains("#0F5E68"));
		assert!(css.contains("#FF8000"));
	}
}
