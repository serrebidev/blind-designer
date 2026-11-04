//! project: save and load a whole design as one text file.
//!
//! The file holds everything the app state holds: the canvas, the
//! background, every layer with its shapes (captions, angles, brush
//! widths, stroke points), the palette with each color's provenance,
//! the full event log, and — when the design is animated — every
//! frame (duration, background, layers) in an optional section after
//! the log. Files written before animation existed have no such
//! section and still load, with zero frames — so an opened design
//! is the same design, history included, not just the same picture.
//! A second optional section carries the project's slider settings
//! (pinned sliders, adjustment target, page jump) when they differ
//! from the defaults; a shape below full opacity stores its percent
//! as one trailing field on its shape line, so fully opaque designs
//! keep the exact file shape they had before opacity existed.
//!
//! The format is hand-rolled line-oriented text in the recreation
//! file's spirit (serde is not a workspace dependency, and one design
//! per file needs no more than this): one record per line, space-
//! separated fields, strings double-quoted with backslash escapes.
//! Numbers that must round-trip exactly (angles, coverage) are written
//! with `{:?}`, Rust's shortest round-trip float form.
//! ponytail: hand-rolled format by design — upgrade path is serde_json
//! if designs ever need to be read by other tools.

use crate::animation::Frame;
use crate::design::{Shape, ShapeKind};
use crate::drawing::{Drawing, Event, Layer};
use crate::palette::{PaletteColor, Provenance};
use crate::sliders::AdjustTarget;
use crate::Srgb;

/// A whole saved design: the drawing plus the palette.
#[derive(Debug, Clone)]
pub struct Project {
	pub drawing: Drawing,
	pub palette: Vec<PaletteColor>,
	/// The sliders this project pins to its Sliders tab, by
	/// registry id (see `crate::sliders`). Empty — a project that
	/// has not chosen any, which includes every file written
	/// before sliders existed.
	pub pinned_sliders: Vec<String>,
	/// What the color-adjustment sliders act on.
	pub slider_target: AdjustTarget,
	/// The Page Up/Down jump on the Sliders tab, as a percent of
	/// each slider's range. Ten when the file does not say.
	pub page_jump_percent: u8,
}

/// The page-jump percent a project starts with, and the value a
/// file means when it does not carry one.
pub const DEFAULT_PAGE_JUMP_PERCENT: u8 = 10;

/// What a project actually contains, counted: canvas size, layers,
/// elements by kind, palette colors, animation frames. The main
/// window keeps the one-line form current; the Project Statistics
/// readout speaks the full form. Counts are only ever what is really
/// in the drawing — an empty project reports zeros, never defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProjectStats {
	pub canvas_width: u32,
	pub canvas_height: u32,
	pub layers: usize,
	pub elements: usize,
	pub rectangles: usize,
	pub ellipses: usize,
	pub strokes: usize,
	pub texts: usize,
	pub images: usize,
	pub colors: usize,
	pub frames: usize,
}

fn counted(n: usize, one: &str, many: &str) -> String {
	format!("{n} {}", if n == 1 { one } else { many })
}

impl ProjectStats {
	/// Count a drawing and its palette.
	pub fn gather(drawing: &Drawing, colors: usize) -> ProjectStats {
		let mut stats = ProjectStats {
			canvas_width: drawing.width,
			canvas_height: drawing.height,
			layers: drawing.layers.len(),
			colors,
			frames: drawing.frames.len(),
			..ProjectStats::default()
		};
		for layer in &drawing.layers {
			for shape in &layer.shapes {
				stats.elements += 1;
				match shape.kind {
					ShapeKind::Rectangle => stats.rectangles += 1,
					ShapeKind::Ellipse => stats.ellipses += 1,
					ShapeKind::Stroke => stats.strokes += 1,
					ShapeKind::Text => stats.texts += 1,
					ShapeKind::Image => stats.images += 1,
				}
			}
		}
		stats
	}

	/// The element breakdown in words, kinds with none left out:
	/// "3 rectangles, 1 ellipse, 2 strokes, 1 text element,
	/// 0 image elements" — or "no elements" when the canvas is bare.
	fn kinds_text(&self) -> String {
		if self.elements == 0 {
			return "no elements".to_string();
		}
		[
			counted(self.rectangles, "rectangle", "rectangles"),
			counted(self.ellipses, "ellipse", "ellipses"),
			counted(self.strokes, "stroke", "strokes"),
			counted(self.texts, "text element", "text elements"),
			counted(self.images, "image element", "image elements"),
		]
		.join(", ")
	}

	/// One line for the main window's statistics readout.
	pub fn one_line(&self) -> String {
		let frames = if self.frames == 0 {
			"no animation frames".to_string()
		} else {
			counted(self.frames, "animation frame", "animation frames")
		};
		format!(
			"Project: {}, {}, {}, {}, canvas {} by {}",
			counted(self.layers, "layer", "layers"),
			counted(self.elements, "element", "elements"),
			counted(self.colors, "palette color", "palette colors"),
			frames,
			self.canvas_width,
			self.canvas_height,
		)
	}

	/// The full readout: every count on its own line, in plain words.
	pub fn report(&self) -> String {
		format!(
			"Project statistics\n\
			 Canvas: {} by {} pixels\n\
			 Layers: {}\n\
			 Elements: {} ({})\n\
			 Palette colors: {}\n\
			 Animation frames: {}",
			self.canvas_width,
			self.canvas_height,
			self.layers,
			self.elements,
			self.kinds_text(),
			self.colors,
			self.frames,
		)
	}
}

/// A string as a quoted, escaped token.
pub(crate) fn quote(s: &str) -> String {
	let mut out = String::from("\"");
	for c in s.chars() {
		match c {
			'"' => out.push_str("\\\""),
			'\\' => out.push_str("\\\\"),
			'\n' => out.push_str("\\n"),
			'\t' => out.push_str("\\t"),
			_ => out.push(c),
		}
	}
	out.push('"');
	out
}

/// Split one line into tokens; a double-quoted token may contain
/// spaces and backslash escapes.
pub(crate) fn tokens(line: &str) -> Result<Vec<String>, String> {
	let mut out = Vec::new();
	let mut chars = line.chars().peekable();
	loop {
		while chars.peek() == Some(&' ') {
			chars.next();
		}
		let Some(&c) = chars.peek() else { break };
		if c == '"' {
			chars.next();
			let mut tok = String::new();
			loop {
				match chars.next() {
					None => return Err(format!("unterminated string in line: {line}")),
					Some('"') => break,
					Some('\\') => match chars.next() {
						Some('n') => tok.push('\n'),
						Some('t') => tok.push('\t'),
						Some(other) => tok.push(other),
						None => return Err(format!("dangling escape in line: {line}")),
					},
					Some(other) => tok.push(other),
				}
			}
			out.push(tok);
		} else {
			let mut tok = String::new();
			while let Some(&c) = chars.peek() {
				if c == ' ' {
					break;
				}
				tok.push(c);
				chars.next();
			}
			out.push(tok);
		}
	}
	Ok(out)
}

fn num<T: std::str::FromStr>(toks: &[String], i: usize, what: &str) -> Result<T, String> {
	toks.get(i)
		.ok_or_else(|| format!("missing {what}"))
		.and_then(|t| t.parse::<T>().map_err(|_| format!("bad {what}: {t}")))
}

fn hex(toks: &[String], i: usize) -> Result<Srgb, String> {
	let t = toks.get(i).ok_or("missing color")?;
	Srgb::from_hex(t).ok_or_else(|| format!("bad color: {t}"))
}

/// The layer lines of the file format: a "layer" header plus one
/// "shape" line per shape. Shared by the drawing's own layers and by
/// each animation frame's layers.
fn layer_lines(layer: &Layer) -> String {
	let mut out = format!(
		"layer {} {} {}\n",
		quote(&layer.name),
		if layer.visible { "visible" } else { "hidden" },
		layer.shapes.len()
	);
	for shape in &layer.shapes {
		out.push_str(&format!("shape {}\n", shape_fields(shape)));
	}
	out
}

/// The fields of one shape, in file order. Shared by the layer shape
/// lines and the shape-added / shape-removed log events.
///
/// The first ten fields are the original format — every file written
/// before text, image and patterned-brush shapes existed has exactly
/// those ten, and still loads. The new kinds append their extra
/// fields after them, only when they carry any: text appends the
/// phrase, the font name and the point size; image appends the file
/// path; a stroke appends its pattern name unless it is solid.
fn shape_fields(s: &Shape) -> String {
	let kind = match s.kind {
		ShapeKind::Rectangle => "rectangle",
		ShapeKind::Ellipse => "ellipse",
		ShapeKind::Stroke => "stroke",
		ShapeKind::Text => "text",
		ShapeKind::Image => "image",
	};
	let points = if s.points.is_empty() {
		"-".to_string()
	} else {
		s.points.iter().map(|(x, y)| format!("{x},{y}")).collect::<Vec<_>>().join(";")
	};
	let caption = match &s.caption {
		None => "-".to_string(),
		Some(c) => quote(c),
	};
	let mut out = format!(
		"{kind} {} {} {} {} {} {:?} {} {points} {caption}",
		s.x,
		s.y,
		s.w,
		s.h,
		s.fill.to_hex(),
		s.angle,
		s.brush
	);
	match s.kind {
		ShapeKind::Text => {
			out.push_str(&format!(
				" {} {} {}",
				quote(&s.text),
				quote(&s.font),
				s.font_size
			));
		}
		ShapeKind::Image => {
			out.push_str(&format!(" {}", quote(&s.image_path)));
		}
		ShapeKind::Stroke if !s.pattern.is_empty() && s.pattern != "solid" => {
			out.push_str(&format!(" {}", quote(&s.pattern)));
		}
		_ => {}
	}
	// A shape below full opacity carries its percent as one last
	// field; a fully opaque shape writes nothing extra, so older
	// files and fully opaque designs stay byte-identical.
	if s.opacity < 100 {
		out.push_str(&format!(" {}", s.opacity));
	}
	out
}

fn parse_shape(toks: &[String]) -> Result<Shape, String> {
	if toks.len() < 10 {
		return Err(format!("a shape needs at least 10 fields, got {}", toks.len()));
	}
	let kind = match toks[0].as_str() {
		"rectangle" => ShapeKind::Rectangle,
		"ellipse" => ShapeKind::Ellipse,
		"stroke" => ShapeKind::Stroke,
		"text" => ShapeKind::Text,
		"image" => ShapeKind::Image,
		other => return Err(format!("unknown shape kind: {other}")),
	};
	let points = if toks[8] == "-" {
		Vec::new()
	} else {
		toks[8]
			.split(';')
			.map(|p| {
				let (x, y) = p.split_once(',').ok_or_else(|| format!("bad point: {p}"))?;
				Ok((
					x.parse::<i32>().map_err(|_| format!("bad point: {p}"))?,
					y.parse::<i32>().map_err(|_| format!("bad point: {p}"))?,
				))
			})
			.collect::<Result<Vec<(i32, i32)>, String>>()?
	};
	let mut shape = Shape {
		kind,
		x: num(&toks, 1, "shape x")?,
		y: num(&toks, 2, "shape y")?,
		w: num(&toks, 3, "shape width")?,
		h: num(&toks, 4, "shape height")?,
		fill: hex(&toks, 5)?,
		points,
		brush: num(&toks, 7, "brush width")?,
		angle: num(&toks, 6, "angle")?,
		caption: if toks[9] == "-" { None } else { Some(toks[9].clone()) },
		pattern: String::new(),
		text: String::new(),
		font: String::new(),
		font_size: 0,
		image_path: String::new(),
		opacity: 100,
	};
	// The appended fields of the newer kinds; absent in old files.
	// Each kind has a known count of them; one field beyond that
	// count can only be the opacity percent (written last, and
	// only when the shape is not fully opaque).
	let mut extra: &[String] = &toks[10..];
	let expected = match kind {
		ShapeKind::Text => 3,
		ShapeKind::Image => 1,
		_ => 0,
	};
	if kind != ShapeKind::Stroke && extra.len() == expected + 1 {
		shape.opacity = num(extra, expected, "shape opacity")?;
		extra = &extra[..expected];
	}
	match kind {
		ShapeKind::Text => {
			if extra.len() != 3 {
				return Err(format!("a text shape needs 13 fields, got {}", toks.len()));
			}
			shape.text = extra[0].clone();
			shape.font = extra[1].clone();
			shape.font_size = num(extra, 2, "font size")?;
		}
		ShapeKind::Image => {
			if extra.len() != 1 {
				return Err(format!("an image shape needs 11 fields, got {}", toks.len()));
			}
			shape.image_path = extra[0].clone();
		}
		ShapeKind::Stroke => {
			// A stroke's extras are its pattern (a word) and then
			// the opacity (a number); either may appear alone.
			if extra.len() > 2 {
				return Err(format!("a stroke needs at most 12 fields, got {}", toks.len()));
			}
			if extra.len() == 2 {
				shape.pattern = extra[0].clone();
				shape.opacity = num(extra, 1, "shape opacity")?;
			} else if extra.len() == 1 {
				match extra[0].parse::<u8>() {
					Ok(opacity) => shape.opacity = opacity,
					Err(_) => shape.pattern = extra[0].clone(),
				}
			}
		}
		_ => {
			if !extra.is_empty() {
				return Err(format!("a shape needs 10 fields, got {}", toks.len()));
			}
		}
	}
	shape.opacity = shape.opacity.min(100);
	Ok(shape)
}

fn provenance_fields(p: &Provenance) -> String {
	match p {
		Provenance::Starter => "starter".to_string(),
		Provenance::Typed => "typed".to_string(),
		Provenance::Extracted { image } => format!("extracted {}", quote(image)),
		Provenance::Generated { scheme } => format!("generated {}", quote(scheme)),
		Provenance::Imported { source } => format!("imported {}", quote(source)),
	}
}

fn parse_provenance(toks: &[String]) -> Result<Provenance, String> {
	match toks.first().map(String::as_str) {
		Some("starter") => Ok(Provenance::Starter),
		Some("typed") => Ok(Provenance::Typed),
		Some("extracted") => Ok(Provenance::Extracted {
			image: toks.get(1).ok_or("missing image name")?.clone(),
		}),
		Some("generated") => Ok(Provenance::Generated {
			scheme: toks.get(1).ok_or("missing scheme")?.clone(),
		}),
		Some("imported") => Ok(Provenance::Imported {
			source: toks.get(1).ok_or("missing import source")?.clone(),
		}),
		_ => Err("unknown provenance".to_string()),
	}
}

fn event_line(e: &Event) -> String {
	match e {
		Event::CanvasSet { width, height } => format!("event canvas {width} {height}"),
		Event::BackgroundSet { color } => format!("event background {}", color.to_hex()),
		Event::LayerAdded { name } => format!("event layer-added {}", quote(name)),
		Event::LayerRenamed { from, to } => {
			format!("event layer-renamed {} {}", quote(from), quote(to))
		}
		Event::LayerRemoved { name } => format!("event layer-removed {}", quote(name)),
		Event::LayerVisibility { name, visible } => format!(
			"event layer-visibility {} {}",
			quote(name),
			if *visible { "visible" } else { "hidden" }
		),
		Event::ShapeAdded { layer, shape } => {
			format!("event shape-added {} {}", quote(layer), shape_fields(shape))
		}
		Event::ShapeRemoved { layer, shape } => {
			format!("event shape-removed {} {}", quote(layer), shape_fields(shape))
		}
		Event::ShapeShifted { layer, index, dx, dy } => {
			format!("event shape-shifted {} {index} {dx} {dy}", quote(layer))
		}
		Event::ShapeResized { layer, index, w, h } => {
			format!("event shape-resized {} {index} {w} {h}", quote(layer))
		}
		Event::ShapeRotated { layer, index, angle } => {
			format!("event shape-rotated {} {index} {angle:?}", quote(layer))
		}
		Event::ShapeRecolored { layer, index, color } => {
			format!("event shape-recolored {} {index} {}", quote(layer), color.to_hex())
		}
		Event::ShapeCaptioned { layer, index } => {
			format!("event shape-captioned {} {index}", quote(layer))
		}
		Event::ShapeOpacitySet { layer, index, opacity } => {
			format!("event shape-opacity {} {index} {opacity}", quote(layer))
		}
		Event::ShapeBrushSet { layer, index, width } => {
			format!("event shape-brush {} {index} {width}", quote(layer))
		}
		Event::ShapeColorAdjusted {
			layer,
			index,
			shade,
			tint,
			tone,
			chroma,
			undertone,
			warmth,
			vibrancy,
			brightness,
			clarity,
			fog,
			hardness,
		} => format!(
			"event shape-adjusted {} {index} {shade} {tint} {tone} {chroma} {undertone} {warmth} {vibrancy} {brightness} {clarity} {fog} {hardness}",
			quote(layer)
		),
		Event::LayerColorsAdjusted {
			name,
			count,
			shade,
			tint,
			tone,
			chroma,
			undertone,
			warmth,
			vibrancy,
			brightness,
			clarity,
			fog,
			hardness,
		} => format!(
			"event layer-adjusted {} {count} {shade} {tint} {tone} {chroma} {undertone} {warmth} {vibrancy} {brightness} {clarity} {fog} {hardness}",
			quote(name)
		),
		Event::FrameAdded { index, duration_ms } => {
			format!("event frame-added {index} {duration_ms}")
		}
		Event::FrameDuplicated { index } => format!("event frame-duplicated {index}"),
		Event::FrameRemoved { index } => format!("event frame-removed {index}"),
		Event::FrameMoved { from, to } => format!("event frame-moved {from} {to}"),
		Event::FrameDurationSet { index, duration_ms } => {
			format!("event frame-duration {index} {duration_ms}")
		}
		Event::AllFramesDurationSet { count, duration_ms } => {
			format!("event frames-duration-all {count} {duration_ms}")
		}
		Event::FrameLoaded { index } => format!("event frame-loaded {index}"),
		Event::FrameUpdated { index } => format!("event frame-updated {index}"),
	}
}

fn parse_event(toks: &[String]) -> Result<Event, String> {
	// toks[0] is "event"; toks[1] is the kind.
	let name = |i: usize| toks.get(i).cloned().ok_or("missing layer name");
	match toks.get(1).map(String::as_str) {
		Some("canvas") => Ok(Event::CanvasSet {
			width: num(toks, 2, "canvas width")?,
			height: num(toks, 3, "canvas height")?,
		}),
		Some("background") => Ok(Event::BackgroundSet { color: hex(toks, 2)? }),
		Some("layer-added") => Ok(Event::LayerAdded { name: name(2)? }),
		Some("layer-renamed") => Ok(Event::LayerRenamed { from: name(2)?, to: name(3)? }),
		Some("layer-removed") => Ok(Event::LayerRemoved { name: name(2)? }),
		Some("layer-visibility") => Ok(Event::LayerVisibility {
			name: name(2)?,
			visible: match toks.get(3).map(String::as_str) {
				Some("visible") => true,
				Some("hidden") => false,
				_ => return Err("bad layer visibility".to_string()),
			},
		}),
		Some("shape-added") => Ok(Event::ShapeAdded {
			layer: name(2)?,
			shape: parse_shape(&toks[3..])?,
		}),
		Some("shape-removed") => Ok(Event::ShapeRemoved {
			layer: name(2)?,
			shape: parse_shape(&toks[3..])?,
		}),
		Some("shape-shifted") => Ok(Event::ShapeShifted {
			layer: name(2)?,
			index: num(toks, 3, "shape index")?,
			dx: num(toks, 4, "shift x")?,
			dy: num(toks, 5, "shift y")?,
		}),
		Some("shape-resized") => Ok(Event::ShapeResized {
			layer: name(2)?,
			index: num(toks, 3, "shape index")?,
			w: num(toks, 4, "shape width")?,
			h: num(toks, 5, "shape height")?,
		}),
		Some("shape-rotated") => Ok(Event::ShapeRotated {
			layer: name(2)?,
			index: num(toks, 3, "shape index")?,
			angle: num(toks, 4, "angle")?,
		}),
		Some("shape-recolored") => Ok(Event::ShapeRecolored {
			layer: name(2)?,
			index: num(toks, 3, "shape index")?,
			color: hex(toks, 4)?,
		}),
		Some("shape-captioned") => Ok(Event::ShapeCaptioned {
			layer: name(2)?,
			index: num(toks, 3, "shape index")?,
		}),
		Some("shape-opacity") => Ok(Event::ShapeOpacitySet {
			layer: name(2)?,
			index: num(toks, 3, "shape index")?,
			opacity: num(toks, 4, "shape opacity")?,
		}),
		Some("shape-brush") => Ok(Event::ShapeBrushSet {
			layer: name(2)?,
			index: num(toks, 3, "shape index")?,
			width: num(toks, 4, "brush width")?,
		}),
		Some("shape-adjusted") => Ok(Event::ShapeColorAdjusted {
			layer: name(2)?,
			index: num(toks, 3, "shape index")?,
			shade: num(toks, 4, "shade")?,
			tint: num(toks, 5, "tint")?,
			tone: num(toks, 6, "tone")?,
			chroma: num(toks, 7, "chroma")?,
			undertone: num(toks, 8, "undertone")?,
			warmth: num(toks, 9, "warmth")?,
			vibrancy: num(toks, 10, "vibrancy")?,
			brightness: num(toks, 11, "brightness")?,
			clarity: num(toks, 12, "clarity")?,
			fog: num(toks, 13, "fog")?,
			hardness: num(toks, 14, "hardness")?,
		}),
		Some("layer-adjusted") => Ok(Event::LayerColorsAdjusted {
			name: name(2)?,
			count: num(toks, 3, "element count")?,
			shade: num(toks, 4, "shade")?,
			tint: num(toks, 5, "tint")?,
			tone: num(toks, 6, "tone")?,
			chroma: num(toks, 7, "chroma")?,
			undertone: num(toks, 8, "undertone")?,
			warmth: num(toks, 9, "warmth")?,
			vibrancy: num(toks, 10, "vibrancy")?,
			brightness: num(toks, 11, "brightness")?,
			clarity: num(toks, 12, "clarity")?,
			fog: num(toks, 13, "fog")?,
			hardness: num(toks, 14, "hardness")?,
		}),
		Some("frame-added") => Ok(Event::FrameAdded {
			index: num(toks, 2, "frame index")?,
			duration_ms: num(toks, 3, "frame duration")?,
		}),
		Some("frame-duplicated") => Ok(Event::FrameDuplicated {
			index: num(toks, 2, "frame index")?,
		}),
		Some("frame-removed") => Ok(Event::FrameRemoved {
			index: num(toks, 2, "frame index")?,
		}),
		Some("frame-moved") => Ok(Event::FrameMoved {
			from: num(toks, 2, "frame index")?,
			to: num(toks, 3, "frame index")?,
		}),
		Some("frame-duration") => Ok(Event::FrameDurationSet {
			index: num(toks, 2, "frame index")?,
			duration_ms: num(toks, 3, "frame duration")?,
		}),
		Some("frames-duration-all") => Ok(Event::AllFramesDurationSet {
			count: num(toks, 2, "frame count")?,
			duration_ms: num(toks, 3, "frame duration")?,
		}),
		Some("frame-loaded") => Ok(Event::FrameLoaded {
			index: num(toks, 2, "frame index")?,
		}),
		Some("frame-updated") => Ok(Event::FrameUpdated {
			index: num(toks, 2, "frame index")?,
		}),
		_ => Err("unknown event kind".to_string()),
	}
}

/// One layer block (header + shape lines), read through the same
/// line source as the rest of the file. Used for the drawing's own
/// layers and for each animation frame's layers.
fn parse_layer(
	next: &mut impl FnMut(&str) -> Result<Vec<String>, String>,
) -> Result<Layer, String> {
	let toks = next("a layer")?;
	if toks.first().map(String::as_str) != Some("layer") {
		return Err("missing layer line".to_string());
	}
	let mut layer = Layer {
		name: toks.get(1).ok_or("missing layer name")?.clone(),
		visible: match toks.get(2).map(String::as_str) {
			Some("visible") => true,
			Some("hidden") => false,
			_ => return Err("bad layer visibility".to_string()),
		},
		shapes: Vec::new(),
	};
	let shapes: usize = num(&toks, 3, "shape count")?;
	for _ in 0..shapes {
		let toks = next("a shape")?;
		if toks.first().map(String::as_str) != Some("shape") {
			return Err("missing shape line".to_string());
		}
		layer.shapes.push(parse_shape(&toks[1..])?);
	}
	Ok(layer)
}

impl Project {
	/// The whole project as project-file text.
	pub fn to_text(&self) -> String {
		let d = &self.drawing;
		let mut out = String::from("blind-designer-project 1\n");
		out.push_str(&format!("canvas {} {}\n", d.width, d.height));
		out.push_str(&format!("background {}\n", d.background.to_hex()));
		out.push_str(&format!("palette {}\n", self.palette.len()));
		for pc in &self.palette {
			let cov = match pc.coverage {
				Some(c) => format!("{c:?}"),
				None => "-".to_string(),
			};
			out.push_str(&format!(
				"color {} {} {}\n",
				pc.color.to_hex(),
				cov,
				provenance_fields(&pc.provenance)
			));
		}
		out.push_str(&format!("layers {}\n", d.layers.len()));
		for layer in &d.layers {
			out.push_str(&layer_lines(layer));
		}
		out.push_str(&format!("log {}\n", d.log.len()));
		for event in &d.log {
			out.push_str(&event_line(event));
			out.push('\n');
		}
		// The animation section is written only when the design has
		// frames, so files for static designs keep the exact shape
		// they had before animation existed (and old readers are not
		// the concern — old FILES are: see from_text).
		if !d.frames.is_empty() {
			out.push_str(&format!("frames {}\n", d.frames.len()));
			for frame in &d.frames {
				out.push_str(&format!(
					"frame {} {} {}\n",
					frame.duration_ms,
					frame.background.to_hex(),
					frame.layers.len()
				));
				for layer in &frame.layers {
					out.push_str(&layer_lines(layer));
				}
			}
		}
		// The sliders section likewise appears only when the
		// project has slider settings of its own (pinned sliders,
		// a non-default adjustment target, a non-default page
		// jump), so a project that never chose any keeps the
		// exact file shape it had before.
		if !self.pinned_sliders.is_empty()
			|| self.slider_target != AdjustTarget::BaseColor
			|| self.page_jump_percent != DEFAULT_PAGE_JUMP_PERCENT
		{
			out.push_str(&format!("sliders {}\n", self.pinned_sliders.len()));
			for id in &self.pinned_sliders {
				out.push_str(&format!("slider {id}\n"));
			}
			if self.slider_target != AdjustTarget::BaseColor {
				out.push_str(&format!("slider-target {}\n", self.slider_target.id()));
			}
			if self.page_jump_percent != DEFAULT_PAGE_JUMP_PERCENT {
				out.push_str(&format!("slider-page-jump {}\n", self.page_jump_percent));
			}
		}
		out.push_str("end\n");
		out
	}

	/// Read a project file back. Anything malformed is an `Err` with
	/// a plain-words reason; a partial project is never returned.
	pub fn from_text(text: &str) -> Result<Project, String> {
		// Lines are indexed through a Cell so the parser can also
		// peek at the upcoming line (the optional animation section
		// is detected that way) without fighting the borrow checker.
		let all: Vec<&str> = text.lines().collect();
		let pos = std::cell::Cell::new(0usize);
		let mut next = |what: &str| -> Result<Vec<String>, String> {
			let line =
				all.get(pos.get()).ok_or_else(|| format!("file ends before {what}"))?;
			pos.set(pos.get() + 1);
			tokens(line)
		};
		let head = next("the header")?;
		if head.len() != 2 || head[0] != "blind-designer-project" || head[1] != "1" {
			return Err("not a Blind Designer project file".to_string());
		}
		let canvas = next("the canvas")?;
		if canvas.first().map(String::as_str) != Some("canvas") {
			return Err("missing canvas line".to_string());
		}
		let (width, height) = (num(&canvas, 1, "canvas width")?, num(&canvas, 2, "canvas height")?);
		let bg = next("the background")?;
		if bg.first().map(String::as_str) != Some("background") {
			return Err("missing background line".to_string());
		}
		let background = hex(&bg, 1)?;

		let head = next("the palette")?;
		if head.first().map(String::as_str) != Some("palette") {
			return Err("missing palette line".to_string());
		}
		let count: usize = num(&head, 1, "palette size")?;
		let mut palette = Vec::with_capacity(count);
		for _ in 0..count {
			let toks = next("a palette color")?;
			if toks.first().map(String::as_str) != Some("color") {
				return Err("missing palette color line".to_string());
			}
			let coverage = if toks.get(2).map(String::as_str) == Some("-") {
				None
			} else {
				Some(num(&toks, 2, "coverage")?)
			};
			palette.push(PaletteColor {
				color: hex(&toks, 1)?,
				coverage,
				provenance: parse_provenance(&toks[3..])?,
			});
		}

		let head = next("the layers")?;
		if head.first().map(String::as_str) != Some("layers") {
			return Err("missing layers line".to_string());
		}
		let count: usize = num(&head, 1, "layer count")?;
		let mut layers = Vec::with_capacity(count);
		for _ in 0..count {
			layers.push(parse_layer(&mut next)?);
		}

		let head = next("the event log")?;
		if head.first().map(String::as_str) != Some("log") {
			return Err("missing log line".to_string());
		}
		let count: usize = num(&head, 1, "log size")?;
		let mut log = Vec::with_capacity(count);
		for _ in 0..count {
			let toks = next("a log event")?;
			if toks.first().map(String::as_str) != Some("event") {
				return Err("missing event line".to_string());
			}
			log.push(parse_event(&toks)?);
		}
		// The animation and sliders sections are both optional
		// and come after the log, in the order to_text writes
		// them: every project file written before either existed
		// goes straight from the log to the end marker, and loads
		// with zero frames and no pinned sliders.
		let mut frames: Vec<Frame> = Vec::new();
		let mut pinned_sliders: Vec<String> = Vec::new();
		let mut slider_target = AdjustTarget::BaseColor;
		let mut page_jump_percent = DEFAULT_PAGE_JUMP_PERCENT;
		loop {
			let section = all
				.get(pos.get())
				.and_then(|line| tokens(line).ok())
				.and_then(|toks| toks.first().cloned());
			match section.as_deref() {
				Some("frames") => {
					let head = next("the animation frames")?;
					let count: usize = num(&head, 1, "frame count")?;
					for _ in 0..count {
						let toks = next("an animation frame")?;
						if toks.first().map(String::as_str) != Some("frame") {
							return Err("missing frame line".to_string());
						}
						let layer_count: usize = num(&toks, 3, "frame layer count")?;
						let mut frame_layers = Vec::with_capacity(layer_count);
						for _ in 0..layer_count {
							frame_layers.push(parse_layer(&mut next)?);
						}
						frames.push(Frame {
							duration_ms: num(&toks, 1, "frame duration")?,
							background: hex(&toks, 2)?,
							layers: frame_layers,
						});
					}
				}
				Some("sliders") => {
					let head = next("the pinned sliders")?;
					let count: usize = num(&head, 1, "pinned slider count")?;
					for _ in 0..count {
						let toks = next("a pinned slider")?;
						if toks.first().map(String::as_str) != Some("slider") {
							return Err("missing pinned slider line".to_string());
						}
						pinned_sliders
							.push(toks.get(1).ok_or("missing slider id")?.clone());
					}
					// The adjustment target and the page jump may
					// follow the pins, in the order to_text writes.
					let peek_is = |word: &str| {
						all.get(pos.get())
							.and_then(|line| tokens(line).ok())
							.and_then(|toks| toks.first().map(|t| t == word))
							.unwrap_or(false)
					};
					if peek_is("slider-target") {
						let toks = next("the slider target")?;
						let id = toks.get(1).ok_or("missing slider target")?;
						slider_target = AdjustTarget::from_id(id)
							.ok_or_else(|| format!("unknown slider target: {id}"))?;
					}
					if peek_is("slider-page-jump") {
						let toks = next("the slider page jump")?;
						page_jump_percent = num(&toks, 1, "page jump percent")?;
					}
				}
				_ => break,
			}
		}
		let tail = next("the end marker")?;
		if tail.first().map(String::as_str) != Some("end") {
			return Err("missing end marker".to_string());
		}
		Ok(Project {
			drawing: Drawing { width, height, background, layers, log, frames },
			palette,
			pinned_sliders,
			slider_target,
			page_jump_percent,
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::design::ShapeKind;
	use crate::ColorFormat;

	#[test]
	fn stats_of_a_fresh_project_are_all_real_zeros() {
		let d = Drawing::new(800, 600, Srgb::WHITE);
		let stats = ProjectStats::gather(&d, 0);
		assert_eq!(stats.layers, 1);
		assert_eq!(stats.elements, 0);
		assert_eq!(stats.colors, 0);
		assert_eq!(stats.frames, 0);
		assert_eq!(
			stats.one_line(),
			"Project: 1 layer, 0 elements, 0 palette colors, no animation frames, canvas 800 by 600"
		);
		assert!(stats.report().contains("Elements: 0 (no elements)"), "{}", stats.report());
	}

	#[test]
	fn stats_count_every_element_kind_and_frames() {
		let mut d = Drawing::new(640, 480, Srgb::WHITE);
		d.add_shape(0, Shape::new(ShapeKind::Rectangle, 0, 0, 10, 10, Srgb::BLACK));
		d.add_shape(0, Shape::new(ShapeKind::Rectangle, 20, 0, 10, 10, Srgb::BLACK));
		d.add_shape(0, Shape::new(ShapeKind::Ellipse, 40, 0, 10, 10, Srgb::BLACK));
		d.add_shape(0, Shape::stroke(vec![(0, 0), (5, 5)], 3, Srgb::BLACK));
		d.add_shape(0, Shape::text("hi", "Arial", 24, 0, 20, 30, 30, Srgb::BLACK));
		d.add_shape(0, Shape::image("a.png", 0, 60, 30, 30, Srgb::BLACK));
		d.add_frame();
		let stats = ProjectStats::gather(&d, 4);
		assert_eq!(stats.elements, 6);
		assert_eq!(stats.rectangles, 2);
		assert_eq!(stats.ellipses, 1);
		assert_eq!(stats.strokes, 1);
		assert_eq!(stats.texts, 1);
		assert_eq!(stats.images, 1);
		assert_eq!(stats.frames, 1);
		assert_eq!(stats.colors, 4);
		let line = stats.one_line();
		assert!(line.contains("6 elements"), "{line}");
		assert!(line.contains("1 animation frame"), "{line}");
		assert!(line.contains("canvas 640 by 480"), "{line}");
		let report = stats.report();
		assert!(report.contains("2 rectangles, 1 ellipse, 1 stroke, 1 text element, 1 image element"), "{report}");
	}

	/// A drawing that exercises every moving part: two layers (one
	/// hidden, one renamed), a captioned rotated rectangle, a stroke
	/// with points and a brush, and edits of every kind in the log.
	fn rich_project() -> Project {
		let mut d = Drawing::new(800, 600, Srgb::WHITE);
		d.set_canvas(1024, 768);
		d.set_background(Srgb::new(238, 238, 238));
		let sky = d.add_layer("Sky \"top\" \\ layer");
		let mut rect = Shape::new(ShapeKind::Rectangle, 10, 20, 100, 50, Srgb::new(15, 94, 104));
		rect.caption = Some("a \"quoted\" caption\nwith a newline".to_string());
		d.add_shape(sky, rect);
		d.add_shape(sky, Shape::stroke(vec![(1, 2), (30, 40), (50, 8)], 7, Srgb::new(255, 128, 0)));
		d.shift_shape(sky, 0, 5, -3);
		d.resize_shape(sky, 0, 120, 60);
		d.rotate_shape(sky, 0, 33.25);
		d.recolor_shape(sky, 1, Srgb::new(200, 40, 30));
		d.caption_shape(sky, 1, "the sun path".to_string());
		d.rename_layer(0, "Ground");
		d.set_layer_visible(0, false);
		let gone = d.add_layer("Scratch");
		d.add_shape(gone, Shape::new(ShapeKind::Ellipse, 0, 0, 9, 9, Srgb::BLACK));
		d.remove_shape(gone, 0);
		d.remove_layer(gone);
		Project {
			drawing: d,
			palette: vec![
				PaletteColor::starter(Srgb::new(58, 124, 165)),
				PaletteColor::typed(Srgb::new(15, 94, 104)),
				PaletteColor::extracted(Srgb::new(255, 128, 0), 0.42, "photo.jpg"),
				PaletteColor::generated(Srgb::new(200, 40, 30), "complementary of #3A7CA5"),
				PaletteColor::imported(Srgb::new(0, 0, 255), "custom property --brand-1"),
			],
			pinned_sliders: Vec::new(),
			slider_target: AdjustTarget::BaseColor,
			page_jump_percent: DEFAULT_PAGE_JUMP_PERCENT,
		}
	}

	#[test]
	fn project_round_trips_everything() {
		let project = rich_project();
		let text = project.to_text();
		let back = Project::from_text(&text).expect("a saved project reads back");
		// Serializing the loaded project reproduces the file byte for
		// byte: nothing was lost or altered anywhere.
		assert_eq!(back.to_text(), text);
		assert_eq!(back.drawing.summary(), project.drawing.summary());
		assert_eq!(back.drawing.log.len(), project.drawing.log.len());
		assert_eq!(back.palette, project.palette);
		// Spot-check the fields a byte comparison could theoretically
		// hide behind a shared bug in both directions.
		let sky = &back.drawing.layers[1];
		assert_eq!(sky.name, "Sky \"top\" \\ layer");
		assert!(!back.drawing.layers[0].visible);
		let rect = &sky.shapes[0];
		assert_eq!(rect.caption.as_deref(), Some("a \"quoted\" caption\nwith a newline"));
		assert_eq!(rect.angle, 33.25);
		assert_eq!((rect.x, rect.y, rect.w, rect.h), (15, 17, 120, 60));
		let stroke = &sky.shapes[1];
		assert_eq!(stroke.kind, ShapeKind::Stroke);
		assert_eq!(stroke.points, vec![(1, 2), (30, 40), (50, 8)]);
		assert_eq!(stroke.brush, 7);
		assert_eq!(stroke.caption.as_deref(), Some("the sun path"));
		let extracted = &back.palette[2];
		assert_eq!(extracted.coverage, Some(0.42));
		assert!(extracted.spoken(ColorFormat::Hex).contains("extracted from image photo.jpg"));
	}

	#[test]
	fn a_fresh_drawing_round_trips() {
		let project = Project {
			drawing: Drawing::new(100, 80, Srgb::new(15, 94, 104)),
			palette: Vec::new(),
			pinned_sliders: Vec::new(),
			slider_target: AdjustTarget::BaseColor,
			page_jump_percent: DEFAULT_PAGE_JUMP_PERCENT,
		};
		let back = Project::from_text(&project.to_text()).unwrap();
		assert_eq!(back.to_text(), project.to_text());
		assert_eq!(back.drawing.layers.len(), 1);
		assert_eq!(back.drawing.log.len(), 3);
	}

	#[test]
	fn text_image_and_patterned_shapes_round_trip() {
		let mut d = Drawing::new(800, 600, Srgb::WHITE);
		d.add_shape(0, Shape::text("Hello world", "Comic Sans MS", 28, 10, 20, 300, 40, Srgb::BLACK));
		d.add_shape(0, Shape::image("/pics/my photo.png", 50, 60, 200, 100, Srgb::new(255, 0, 0)));
		let mut dashed = Shape::stroke(vec![(1, 1), (90, 90)], 5, Srgb::new(0, 0, 255));
		dashed.pattern = "dashed".to_string();
		d.add_shape(0, dashed);
		let project = Project { drawing: d, palette: Vec::new(), pinned_sliders: Vec::new(), slider_target: AdjustTarget::BaseColor, page_jump_percent: DEFAULT_PAGE_JUMP_PERCENT };
		let text = project.to_text();
		let back = Project::from_text(&text).expect("the new kinds read back");
		assert_eq!(back.to_text(), text);
		let shapes = &back.drawing.layers[0].shapes;
		assert_eq!(shapes[0].kind, ShapeKind::Text);
		assert_eq!(shapes[0].text, "Hello world");
		assert_eq!(shapes[0].font, "Comic Sans MS");
		assert_eq!(shapes[0].font_size, 28);
		assert_eq!(shapes[1].kind, ShapeKind::Image);
		assert_eq!(shapes[1].image_path, "/pics/my photo.png");
		assert_eq!(shapes[2].pattern, "dashed");
	}

	#[test]
	fn files_written_before_the_new_kinds_still_load() {
		// A hand-written project in the original ten-field shape
		// format: one rectangle, one plain stroke, one log event.
		let legacy = "\
blind-designer-project 1
canvas 100 80
background #FFFFFF
palette 0
layers 1
layer \"Layer 1\" visible 2
shape rectangle 1 2 30 40 #FF0000 0 3 - -
shape stroke 5 5 10 10 #0000FF 0 2 5,5;15,15 \"a stroke\"
log 1
event canvas 100 80
end
";
		let project = Project::from_text(legacy).expect("a legacy file loads");
		let shapes = &project.drawing.layers[0].shapes;
		assert_eq!(shapes.len(), 2);
		assert_eq!(shapes[0].kind, ShapeKind::Rectangle);
		assert_eq!(shapes[0].pattern, "");
		assert_eq!(shapes[1].kind, ShapeKind::Stroke);
		assert_eq!(shapes[1].points, vec![(5, 5), (15, 15)]);
		assert_eq!(shapes[1].caption.as_deref(), Some("a stroke"));
	}

	#[test]
	fn animation_frames_round_trip() {
		let mut d = Drawing::new(100, 100, Srgb::WHITE);
		d.add_shape(0, Shape::new(ShapeKind::Ellipse, 40, 40, 20, 20, Srgb::new(200, 40, 30)));
		d.add_frame();
		d.set_frame_duration(0, 250);
		d.shift_shape(0, 0, 20, 0);
		d.set_background(Srgb::BLACK);
		d.add_layer("Back");
		d.add_frame();
		let project = Project { drawing: d, palette: Vec::new(), pinned_sliders: Vec::new(), slider_target: AdjustTarget::BaseColor, page_jump_percent: DEFAULT_PAGE_JUMP_PERCENT };
		let text = project.to_text();
		assert!(text.contains("frames 2\n"), "{text}");
		let back = Project::from_text(&text).expect("a project with frames reads back");
		assert_eq!(back.to_text(), text);
		assert_eq!(back.drawing.frames.len(), 2);
		assert_eq!(back.drawing.frames[0].duration_ms, 250);
		assert_eq!(back.drawing.frames[0].background, Srgb::WHITE);
		assert_eq!(back.drawing.frames[0].layers.len(), 1);
		assert_eq!(back.drawing.frames[0].layers[0].shapes[0].x, 40);
		assert_eq!(back.drawing.frames[1].background, Srgb::BLACK);
		assert_eq!(back.drawing.frames[1].layers.len(), 2);
		assert_eq!(back.drawing.frames[1].layers[0].shapes[0].x, 60);
		// The frame operations are log events and round-trip too
		// (byte-for-byte equality above covers it; say it out loud).
		let steps: Vec<String> =
			back.drawing.log.iter().map(|e| e.describe(&back.drawing)).collect();
		assert!(steps.iter().any(|s| s.contains("Add animation frame 1")), "{steps:?}");
		assert!(steps.iter().any(|s| s.contains("Set animation frame 1 to show for 250")), "{steps:?}");
	}

	#[test]
	fn files_written_before_animation_still_load() {
		// The pre-animation file shape ends at the end marker right
		// after the log: no frames section at all.
		let legacy = "\
blind-designer-project 1
canvas 100 80
background #FFFFFF
palette 0
layers 1
layer \"Layer 1\" visible 1
shape ellipse 40 40 20 20 #C8281E 0 3 - -
log 3
event canvas 100 80
event background #FFFFFF
event layer-added \"Layer 1\"
end
";
		let project = Project::from_text(legacy).expect("a pre-animation file loads");
		assert!(project.drawing.frames.is_empty());
		// And re-saving it does not invent an animation section.
		assert!(!project.to_text().contains("frames"));
		// The existing legacy test's file is the same story.
		let back = Project::from_text(&rich_project().to_text()).unwrap();
		assert!(back.drawing.frames.is_empty());
	}

	#[test]
	fn pinned_sliders_target_and_page_jump_round_trip() {
		let mut project = rich_project();
		project.pinned_sliders =
			vec!["brush-width".to_string(), "adjust-fog".to_string(), "frame-duration".to_string()];
		project.slider_target = AdjustTarget::Layer;
		project.page_jump_percent = 25;
		let text = project.to_text();
		assert!(text.contains("sliders 3\nslider brush-width\nslider adjust-fog\nslider frame-duration\nslider-target layer\nslider-page-jump 25\n"), "{text}");
		let back = Project::from_text(&text).expect("a project with slider settings reads back");
		assert_eq!(back.to_text(), text);
		assert_eq!(back.pinned_sliders, project.pinned_sliders);
		assert_eq!(back.slider_target, AdjustTarget::Layer);
		assert_eq!(back.page_jump_percent, 25);
	}

	#[test]
	fn projects_without_slider_settings_write_no_sliders_section() {
		// Defaults all the way down: no section, like every file
		// written before sliders existed.
		let project = rich_project();
		assert!(!project.to_text().contains("slider"), "{}", project.to_text());
		let back = Project::from_text(&project.to_text()).unwrap();
		assert!(back.pinned_sliders.is_empty());
		assert_eq!(back.slider_target, AdjustTarget::BaseColor);
		assert_eq!(back.page_jump_percent, DEFAULT_PAGE_JUMP_PERCENT);
	}

	#[test]
	fn files_written_before_sliders_load_with_no_pins() {
		// Elshara: a project starts from scratch — a file with no
		// sliders section loads with an EMPTY pin set.
		let legacy = "\
blind-designer-project 1
canvas 100 80
background #FFFFFF
palette 0
layers 1
layer \"Layer 1\" visible 0
log 3
event canvas 100 80
event background #FFFFFF
event layer-added \"Layer 1\"
end
";
		let project = Project::from_text(legacy).expect("a pre-sliders file loads");
		assert!(project.pinned_sliders.is_empty());
		assert_eq!(project.slider_target, AdjustTarget::BaseColor);
		assert_eq!(project.page_jump_percent, DEFAULT_PAGE_JUMP_PERCENT);
		assert!(!project.to_text().contains("slider"));
	}

	#[test]
	fn shape_opacity_round_trips_as_a_trailing_field() {
		let mut d = Drawing::new(800, 600, Srgb::WHITE);
		let mut ghost = Shape::new(ShapeKind::Rectangle, 10, 10, 50, 50, Srgb::BLACK);
		ghost.opacity = 40;
		d.add_shape(0, ghost);
		let mut veil = Shape::stroke(vec![(1, 1), (90, 90)], 5, Srgb::new(0, 0, 255));
		veil.pattern = "dashed".to_string();
		veil.opacity = 25;
		d.add_shape(0, veil);
		d.add_shape(0, Shape::new(ShapeKind::Ellipse, 5, 5, 20, 20, Srgb::WHITE));
		let project = Project { drawing: d, palette: Vec::new(), pinned_sliders: Vec::new(), slider_target: AdjustTarget::BaseColor, page_jump_percent: DEFAULT_PAGE_JUMP_PERCENT };
		let text = project.to_text();
		// The translucent shapes carry their percent last; the
		// fully opaque ellipse's line is exactly the old shape.
		assert!(text.contains("shape rectangle 10 10 50 50 #000000 0.0 3 - - 40\n"), "{text}");
		assert!(text.contains("shape stroke 1 1 89 89 #0000FF 0.0 5 1,1;90,90 - \"dashed\" 25\n"), "{text}");
		assert!(text.contains("shape ellipse 5 5 20 20 #FFFFFF 0.0 3 - -\n"), "{text}");
		let back = Project::from_text(&text).expect("translucent shapes read back");
		assert_eq!(back.to_text(), text);
		let shapes = &back.drawing.layers[0].shapes;
		assert_eq!(shapes[0].opacity, 40);
		assert_eq!(shapes[1].opacity, 25);
		assert_eq!(shapes[1].pattern, "dashed");
		assert_eq!(shapes[2].opacity, 100);
	}

	#[test]
	fn slider_events_round_trip_in_the_log() {
		let mut d = Drawing::new(100, 100, Srgb::WHITE);
		d.add_shape(0, Shape::new(ShapeKind::Rectangle, 5, 5, 10, 10, Srgb::new(200, 40, 30)));
		d.set_shape_opacity(0, 0, 45);
		d.adjust_shape_color(
			0,
			0,
			Srgb::new(200, 40, 30),
			crate::sliders::ColorAdjust { shade: 10, fog: -20, ..Default::default() },
		);
		let bases: Vec<Srgb> = d.layers[0].shapes.iter().map(|s| s.fill).collect();
		d.adjust_layer_colors(0, &bases, crate::sliders::ColorAdjust { warmth: 15, ..Default::default() });
		let project = Project { drawing: d, palette: Vec::new(), pinned_sliders: Vec::new(), slider_target: AdjustTarget::BaseColor, page_jump_percent: DEFAULT_PAGE_JUMP_PERCENT };
		let text = project.to_text();
		assert!(text.contains("event shape-opacity"), "{text}");
		assert!(text.contains("event shape-adjusted"), "{text}");
		assert!(text.contains("event layer-adjusted"), "{text}");
		let back = Project::from_text(&text).expect("slider events read back");
		assert_eq!(back.to_text(), text);
		assert_eq!(back.drawing.log.len(), project.drawing.log.len());
	}

	#[test]
	fn bad_files_are_rejected_not_half_read() {
		assert!(Project::from_text("hello world").is_err());
		assert!(Project::from_text("").is_err());
		let text = rich_project().to_text();
		// Truncated at any point: an error, never a partial project.
		for cut in [10, text.len() / 3, text.len() / 2, text.len() - 5] {
			assert!(Project::from_text(&text[..cut]).is_err(), "cut at {cut}");
		}
		// A corrupted number is an error too.
		let broken = text.replacen("canvas 1024", "canvas wide", 1);
		assert!(Project::from_text(&broken).is_err());
	}
}
