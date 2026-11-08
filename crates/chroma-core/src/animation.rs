//! animation: a design as a short film — an ordered list of frames.
//!
//! The model is full-state snapshots, chosen over element-change
//! deltas: a frame is a copy of the drawing's layers and background,
//! exactly what `Drawing::flatten` already renders, so every frame
//! paints through the existing render pipeline with no new renderer
//! and no inverse operations to keep in sync. The workflow matches
//! how the frames are made: draw a pose on the canvas, capture it
//! with `add_frame`, change the canvas, capture again. A stored
//! frame is edited by loading it onto the canvas, changing it, and
//! storing it back with `update_frame`.
//!
//! An empty frame list is a static design (every design made before
//! animation existed). The exports treat that as a one-frame film
//! whose only frame is the canvas itself.
//!
//! Frame-to-frame changes are described in plain words by matching
//! shapes by their position in the layer's shape list — the same
//! order the log and the elements list use.
//! ponytail: index matching by design — upgrade path is identity
//! tracking if shapes ever gain stable ids.

use patois::t;

use crate::design::{Design, Shape, ShapeKind};
use crate::drawing::{Drawing, Event, Layer};
use crate::{color_words, Srgb};

/// How long a freshly captured frame shows, in milliseconds.
pub const DEFAULT_FRAME_MS: u32 = 500;
/// The shortest a frame may show (10 ms is one GIF centisecond step
/// of headroom; encoders quantize to centiseconds).
pub const MIN_FRAME_MS: u32 = 20;
/// The longest a frame may show.
pub const MAX_FRAME_MS: u32 = 60_000;

/// One frame of an animation: a full drawing state — the background
/// and every layer with its shapes — plus how long it shows. The
/// canvas size is the drawing's; all frames share it.
#[derive(Debug, Clone)]
pub struct Frame {
	pub duration_ms: u32,
	pub background: Srgb,
	pub layers: Vec<Layer>,
}

impl Frame {
	/// The frame as it is right now on the drawing's canvas.
	fn snapshot(drawing: &Drawing) -> Frame {
		Frame {
			duration_ms: DEFAULT_FRAME_MS,
			background: drawing.background,
			layers: drawing.layers.clone(),
		}
	}

	/// The flat design this frame renders: visible layers only,
	/// bottom layer first — the same flattening the canvas uses.
	pub fn flatten(&self, width: u32, height: u32) -> Design {
		let mut design = Design::new(width, height, self.background);
		for layer in &self.layers {
			if layer.visible {
				design.shapes.extend(layer.shapes.iter().cloned());
			}
		}
		design
	}
}

impl Drawing {
	/// Capture the canvas as it is now as a new frame at the end of
	/// the animation. Returns the new frame's index.
	pub fn add_frame(&mut self) -> usize {
		let frame = Frame::snapshot(self);
		let duration_ms = frame.duration_ms;
		self.frames.push(frame);
		let index = self.frames.len() - 1;
		self.log.push(Event::FrameAdded { index, duration_ms });
		index
	}

	/// Copy a frame, inserting the copy right after it. Returns the
	/// copy's index, or `None` when there is no such frame.
	pub fn duplicate_frame(&mut self, index: usize) -> Option<usize> {
		let frame = self.frames.get(index)?.clone();
		self.frames.insert(index + 1, frame);
		self.log.push(Event::FrameDuplicated { index });
		Some(index + 1)
	}

	/// Remove a frame. Removing the last one leaves a static design.
	pub fn remove_frame(&mut self, index: usize) -> bool {
		if index >= self.frames.len() {
			return false;
		}
		self.frames.remove(index);
		self.log.push(Event::FrameRemoved { index });
		true
	}

	/// Move a frame one step toward the start (`up`) or the end of
	/// the animation. At either end nothing moves and this is false.
	pub fn move_frame(&mut self, index: usize, up: bool) -> bool {
		if index >= self.frames.len() {
			return false;
		}
		let to = if up {
			if index == 0 {
				return false;
			}
			index - 1
		} else {
			if index + 1 >= self.frames.len() {
				return false;
			}
			index + 1
		};
		self.frames.swap(index, to);
		self.log.push(Event::FrameMoved { from: index, to });
		true
	}

	/// Set how long a frame shows, clamped to the range the
	/// frame-duration slider definition carries (the single source
	/// of that range). Consecutive sets on the same frame — a
	/// slider dragging through values — coalesce into one log
	/// event with the final duration.
	pub fn set_frame_duration(&mut self, index: usize, duration_ms: u32) -> bool {
		let Some(frame) = self.frames.get_mut(index) else {
			return false;
		};
		let duration_ms = duration_ms.clamp(MIN_FRAME_MS, MAX_FRAME_MS);
		frame.duration_ms = duration_ms;
		self.log_slider_event(
			Event::FrameDurationSet { index, duration_ms },
			|e| matches!(e, Event::FrameDurationSet { index: i, .. } if *i == index),
		);
		true
	}

	/// Set EVERY frame's duration at once — the whole film at one
	/// pace; this is what the frame-duration slider drives.
	/// Clamped like a single frame's duration, and logged as ONE
	/// event for the whole change, coalesced while a slider drags.
	/// False when the design has no frames yet.
	pub fn set_all_frame_durations(&mut self, duration_ms: u32) -> bool {
		if self.frames.is_empty() {
			return false;
		}
		let duration_ms = duration_ms.clamp(MIN_FRAME_MS, MAX_FRAME_MS);
		for frame in &mut self.frames {
			frame.duration_ms = duration_ms;
		}
		let count = self.frames.len();
		self.log_slider_event(Event::AllFramesDurationSet { count, duration_ms }, |e| {
			matches!(e, Event::AllFramesDurationSet { .. })
		});
		true
	}

	/// Load a frame onto the canvas for editing: the canvas layers
	/// and background become the frame's. The stored frame is not
	/// changed until `update_frame` stores the canvas back.
	pub fn load_frame(&mut self, index: usize) -> bool {
		let Some(frame) = self.frames.get(index).cloned() else {
			return false;
		};
		self.layers = frame.layers;
		self.background = frame.background;
		self.log.push(Event::FrameLoaded { index });
		true
	}

	/// Replace a stored frame with the canvas as it is now (the
	/// frame keeps its duration).
	pub fn update_frame(&mut self, index: usize) -> bool {
		if index >= self.frames.len() {
			return false;
		}
		self.frames[index].background = self.background;
		self.frames[index].layers = self.layers.clone();
		self.log.push(Event::FrameUpdated { index });
		true
	}

	/// The frames as flat designs, in play order. A static design
	/// (no stored frames) yields its canvas as the only frame.
	pub fn frame_designs(&self) -> Vec<Design> {
		if self.frames.is_empty() {
			return vec![self.flatten()];
		}
		self.frames
			.iter()
			.map(|f| f.flatten(self.width, self.height))
			.collect()
	}

	/// Each frame's duration in play order (a static design reports
	/// one default-length frame, matching `frame_designs`).
	pub fn frame_durations(&self) -> Vec<u32> {
		if self.frames.is_empty() {
			return vec![DEFAULT_FRAME_MS];
		}
		self.frames.iter().map(|f| f.duration_ms).collect()
	}

	/// The whole film's running time in milliseconds.
	pub fn total_duration_ms(&self) -> u32 {
		self.frame_durations().iter().sum()
	}

	/// One line about a frame: frame 1 gets a description of what it
	/// shows; every later frame gets what changed since the frame
	/// before it ("the red ellipse moves right 20 pixels"). This is
	/// the line the Animation dialog lists and the spoken export
	/// speaks. `None` when there is no such frame — frame 1 here is
	/// index 0 of the stored frames, or the canvas itself when the
	/// design is static.
	pub fn frame_change_description(&self, index: usize) -> Option<String> {
		let snapshot;
		let frames: Vec<&Frame> = if self.frames.is_empty() {
			snapshot = Frame::snapshot(self);
			vec![&snapshot]
		} else {
			self.frames.iter().collect()
		};
		let frame = frames.get(index)?;
		if index == 0 {
			return Some(describe_contents(self.width, self.height, frame));
		}
		Some(describe_diff(
			self.width,
			self.height,
			frames[index - 1],
			frame,
		))
	}

	/// The spoken/text export: every frame in order, with its
	/// duration and its description, one frame per line.
	pub fn animation_description(&self) -> String {
		let count = if self.frames.is_empty() {
			1
		} else {
			self.frames.len()
		};
		let total = self.total_duration_ms();
		let mut out = t(
			"Animation description: %s frame%s, total running time %s milliseconds (%s seconds).\n",
		)
		.replacen("%s", &(count).to_string(), 1)
		.replacen("%s", if count == 1 { "" } else { "s" }, 1)
		.replacen("%s", &(total).to_string(), 1)
		.replacen("%s", &format!("{:.1}", total as f64 / 1000.0), 1);
		for (i, ms) in self.frame_durations().iter().enumerate() {
			let what = self
				.frame_change_description(i)
				.unwrap_or_else(|| "nothing".to_string());
			out.push_str(
				&t("Frame %s (%s milliseconds): %s\n")
					.replacen("%s", &(i + 1).to_string(), 1)
					.replacen("%s", &(ms).to_string(), 1)
					.replacen("%s", &(what).to_string(), 1),
			);
		}
		out
	}

	/// The animation as an animated GIF: one rendered image per
	/// frame, each shown for its duration, looping forever. A static
	/// design exports as a one-frame GIF.
	pub fn to_gif(&self) -> Result<Vec<u8>, String> {
		use image::codecs::gif::{GifEncoder, Repeat};
		let mut buf: Vec<u8> = Vec::new();
		{
			let mut encoder = GifEncoder::new(&mut buf);
			encoder.set_repeat(Repeat::Infinite).map_err(|e| {
				t("the GIF would not start looping: %s").replacen("%s", &(e).to_string(), 1)
			})?;
			for (design, ms) in self.frame_designs().iter().zip(self.frame_durations()) {
				let rgba = image::DynamicImage::ImageRgb8(design.render()).to_rgba8();
				let frame =
					image::Frame::from_parts(rgba, 0, 0, image::Delay::from_numer_denom_ms(ms, 1));
				encoder.encode_frame(frame).map_err(|e| {
					t("a frame would not encode as GIF: %s").replacen("%s", &(e).to_string(), 1)
				})?;
			}
		}
		Ok(buf)
	}
}

/// What a frame shows, in one line: its shapes in plain words over
/// its background — "medium dark red ellipse at center (…); … on a
/// white background", or "a blank white canvas".
fn describe_contents(width: u32, height: u32, frame: &Frame) -> String {
	let design = frame.flatten(width, height);
	if design.shapes.is_empty() {
		return t("a blank %s canvas").replacen(
			"%s",
			&(color_words(frame.background)).to_string(),
			1,
		);
	}
	let parts: Vec<String> = design
		.shapes
		.iter()
		.map(|s| design.describe_plain(s))
		.collect();
	t("%s on a %s background")
		.replacen("%s", &(parts.join("; ")).to_string(), 1)
		.replacen("%s", &(color_words(frame.background)).to_string(), 1)
}

/// A short name for a shape in change sentences: "red ellipse",
/// "blue stroke", "text \"Hi\"", "image photo.png".
fn shape_label(shape: &Shape) -> String {
	match shape.kind {
		ShapeKind::Text => t("text \"%s\"").replacen("%s", &(shape.text).to_string(), 1),
		ShapeKind::Image => t("image %s").replacen(
			"%s",
			&(crate::design::image_file_name(&shape.image_path)).to_string(),
			1,
		),
		kind => format!("{} {}", color_words(shape.fill), kind.name()),
	}
}

fn shapes_equal(a: &Shape, b: &Shape) -> bool {
	a.kind == b.kind
		&& a.x == b.x
		&& a.y == b.y
		&& a.w == b.w
		&& a.h == b.h
		&& a.fill == b.fill
		&& a.points == b.points
		&& a.brush == b.brush
		&& a.pattern == b.pattern
		&& a.text == b.text
		&& a.font == b.font
		&& a.font_size == b.font_size
		&& a.image_path == b.image_path
		&& a.angle == b.angle
		&& a.caption == b.caption
}

/// The changes between two shapes known to differ, as sentence
/// fragments with the shape named once at the front: "the red
/// ellipse moves right 20 pixels and changes color to blue".
fn describe_shape_change(width: u32, height: u32, bg: Srgb, a: &Shape, b: &Shape) -> String {
	let mut parts: Vec<String> = Vec::new();
	if a.kind != b.kind {
		return t("the %s is replaced by a %s")
			.replacen("%s", &(shape_label(a)).to_string(), 1)
			.replacen("%s", &(shape_label(b)).to_string(), 1);
	}
	let (dx, dy) = (b.x - a.x, b.y - a.y);
	if dx != 0 || dy != 0 {
		let mut moves: Vec<String> = Vec::new();
		if dx > 0 {
			moves.push(t("right %s pixels").replacen("%s", &(dx).to_string(), 1));
		} else if dx < 0 {
			moves.push(t("left %s pixels").replacen("%s", &(-dx).to_string(), 1));
		}
		if dy > 0 {
			moves.push(t("down %s pixels").replacen("%s", &(dy).to_string(), 1));
		} else if dy < 0 {
			moves.push(t("up %s pixels").replacen("%s", &(-dy).to_string(), 1));
		}
		parts.push(t("moves %s").replacen("%s", &(moves.join(" and ")).to_string(), 1));
	} else if a.points != b.points && a.kind == ShapeKind::Stroke {
		parts.push(t("is redrawn along a new path"));
	}
	if (a.w, a.h) != (b.w, b.h) {
		parts.push(
			t("resizes to %s by %s pixels")
				.replacen("%s", &(b.w).to_string(), 1)
				.replacen("%s", &(b.h).to_string(), 1),
		);
	}
	if a.angle != b.angle {
		parts.push(t("rotates to %s degrees").replacen("%s", &(b.angle).to_string(), 1));
	}
	if a.fill != b.fill {
		parts.push(t("changes color to %s").replacen("%s", &(color_words(b.fill)).to_string(), 1));
	}
	if a.kind == ShapeKind::Text && a.text != b.text {
		parts.push(t("now reads \"%s\"").replacen("%s", &(b.text).to_string(), 1));
	}
	if a.caption != b.caption {
		parts.push(t("gets a new caption"));
	}
	if parts.is_empty() {
		// Some field the sentences above do not single out changed
		// (brush, pattern, font, image file): name it plainly rather
		// than pretend nothing happened.
		let design = Design {
			width,
			height,
			background: bg,
			shapes: Vec::new(),
		};
		return t("the %s changes: now %s")
			.replacen("%s", &(shape_label(a)).to_string(), 1)
			.replacen("%s", &(design.describe_plain(b)).to_string(), 1);
	}
	t("the %s %s")
		.replacen("%s", &(shape_label(a)).to_string(), 1)
		.replacen("%s", &(parts.join(" and ")).to_string(), 1)
}

/// What changed between two frames, in one line: changes joined by
/// "; ", or the no-change sentence when the frames are identical.
fn describe_diff(width: u32, height: u32, prev: &Frame, curr: &Frame) -> String {
	let mut changes: Vec<String> = Vec::new();
	if prev.background != curr.background {
		changes.push(
			t("the background changes from %s to %s")
				.replacen("%s", &(color_words(prev.background)).to_string(), 1)
				.replacen("%s", &(color_words(curr.background)).to_string(), 1),
		);
	}
	let design = curr.flatten(width, height);
	for (li, layer) in curr.layers.iter().enumerate() {
		let Some(before) = prev.layers.get(li) else {
			changes.push(
				t("layer \"%s\" is added with %s shapes")
					.replacen("%s", &(layer.name).to_string(), 1)
					.replacen("%s", &(layer.shapes.len()).to_string(), 1),
			);
			continue;
		};
		if before.name != layer.name {
			changes.push(
				t("layer \"%s\" is renamed to \"%s\"")
					.replacen("%s", &(before.name).to_string(), 1)
					.replacen("%s", &(layer.name).to_string(), 1),
			);
		}
		if before.visible != layer.visible {
			changes.push(
				t("layer \"%s\" is %s")
					.replacen("%s", &(layer.name).to_string(), 1)
					.replacen(
						"%s",
						if layer.visible { "shown" } else { "hidden" },
						1,
					),
			);
		}
		for (si, shape) in layer.shapes.iter().enumerate() {
			match before.shapes.get(si) {
				None => changes.push(
					t("a new shape appears on layer \"%s\": %s")
						.replacen("%s", &(layer.name).to_string(), 1)
						.replacen("%s", &(design.describe_plain(shape)).to_string(), 1),
				),
				Some(old) if !shapes_equal(old, shape) => {
					changes.push(describe_shape_change(
						width,
						height,
						curr.background,
						old,
						shape,
					));
				}
				_ => {}
			}
		}
		for gone in before.shapes.iter().skip(layer.shapes.len()) {
			changes.push(
				t("the %s on layer \"%s\" disappears")
					.replacen("%s", &(shape_label(gone)).to_string(), 1)
					.replacen("%s", &(layer.name).to_string(), 1),
			);
		}
	}
	for gone in prev.layers.iter().skip(curr.layers.len()) {
		changes.push(t("layer \"%s\" is removed").replacen("%s", &(gone.name).to_string(), 1));
	}
	if changes.is_empty() {
		t("no change from the previous frame")
	} else {
		changes.join("; ")
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::drawing::Drawing;

	fn red() -> Srgb {
		Srgb::new(200, 40, 30)
	}

	fn drawing_with_circle() -> Drawing {
		let mut d = Drawing::new(100, 100, Srgb::WHITE);
		d.add_shape(0, Shape::new(ShapeKind::Ellipse, 40, 40, 20, 20, red()));
		d
	}

	#[test]
	fn add_frame_captures_the_canvas_as_it_is_now() {
		let mut d = drawing_with_circle();
		let index = d.add_frame();
		assert_eq!(index, 0);
		assert_eq!(d.frames.len(), 1);
		assert_eq!(d.frames[0].duration_ms, DEFAULT_FRAME_MS);
		assert_eq!(d.frames[0].flatten(100, 100).shapes.len(), 1);
		// Later canvas edits do not leak into the stored frame.
		d.shift_shape(0, 0, 30, 0);
		d.add_shape(0, Shape::new(ShapeKind::Rectangle, 0, 0, 5, 5, Srgb::BLACK));
		assert_eq!(d.frames[0].flatten(100, 100).shapes.len(), 1);
		assert_eq!(d.frames[0].layers[0].shapes[0].x, 40);
		// The capture is in the log, in plain words.
		let steps: Vec<String> = d.log.iter().map(|e| e.describe(&d)).collect();
		assert!(
			steps.iter().any(|s| s.contains("Add animation frame 1")),
			"{steps:?}"
		);
	}

	#[test]
	fn duplicate_delete_reorder_and_duration() {
		let mut d = drawing_with_circle();
		d.add_frame();
		d.add_frame();
		assert_eq!(d.duplicate_frame(0), Some(1));
		assert_eq!(d.frames.len(), 3);
		assert_eq!(d.duplicate_frame(9), None);
		// Durations clamp to the supported range.
		assert!(d.set_frame_duration(0, 250));
		assert_eq!(d.frames[0].duration_ms, 250);
		assert!(d.set_frame_duration(0, 5));
		assert_eq!(d.frames[0].duration_ms, MIN_FRAME_MS);
		assert!(d.set_frame_duration(0, 999_999));
		assert_eq!(d.frames[0].duration_ms, MAX_FRAME_MS);
		assert!(!d.set_frame_duration(9, 250));
		assert_eq!(d.total_duration_ms(), MAX_FRAME_MS + 2 * DEFAULT_FRAME_MS);
		// Reorder swaps neighbours and stops at the ends.
		d.frames[0].duration_ms = 111;
		d.frames[1].duration_ms = 222;
		assert!(d.move_frame(0, false));
		assert_eq!(d.frames[0].duration_ms, 222);
		assert_eq!(d.frames[1].duration_ms, 111);
		assert!(!d.move_frame(0, true));
		assert!(!d.move_frame(2, false));
		assert!(d.move_frame(2, true));
		// Delete removes exactly one and refuses bad indexes.
		assert!(!d.remove_frame(9));
		assert!(d.remove_frame(0));
		assert_eq!(d.frames.len(), 2);
	}

	#[test]
	fn set_all_frame_durations_is_one_coalesced_event() {
		let mut d = drawing_with_circle();
		// No frames yet: nothing to pace.
		assert!(!d.set_all_frame_durations(500));
		d.add_frame();
		d.add_frame();
		d.add_frame();
		let logged = d.log.len();
		assert!(d.set_all_frame_durations(500));
		assert!(d.frames.iter().all(|f| f.duration_ms == 500));
		assert_eq!(d.log.len(), logged + 1, "one event for the whole film");
		// A slider dragging through values stays one event.
		assert!(d.set_all_frame_durations(640));
		assert!(d.set_all_frame_durations(700));
		assert_eq!(d.log.len(), logged + 1);
		assert!(d.frames.iter().all(|f| f.duration_ms == 700));
		let text = d.log.last().unwrap().describe(&d);
		assert!(text.contains("all 3 animation frames"), "{text}");
		assert!(text.contains("700 milliseconds"), "{text}");
		// Clamped like a single frame.
		assert!(d.set_all_frame_durations(1));
		assert!(d.frames.iter().all(|f| f.duration_ms == MIN_FRAME_MS));
	}

	#[test]
	fn load_edit_and_update_frame_is_the_editing_loop() {
		let mut d = drawing_with_circle();
		d.add_frame();
		// Wander off and draw something else.
		d.remove_shape(0, 0);
		assert_eq!(d.flatten().shapes.len(), 0);
		// Loading the frame brings its state back to the canvas.
		assert!(d.load_frame(0));
		assert_eq!(d.flatten().shapes.len(), 1);
		assert_eq!(d.flatten().shapes[0].x, 40);
		// Editing the canvas does not touch the stored frame...
		d.shift_shape(0, 0, 10, 0);
		assert_eq!(d.frames[0].layers[0].shapes[0].x, 40);
		// ...until it is stored back, keeping the frame's duration.
		d.set_frame_duration(0, 750);
		assert!(d.update_frame(0));
		assert_eq!(d.frames[0].layers[0].shapes[0].x, 50);
		assert_eq!(d.frames[0].duration_ms, 750);
		assert!(!d.load_frame(4));
		assert!(!d.update_frame(4));
	}

	#[test]
	fn frame_one_is_described_by_what_it_shows() {
		let mut d = drawing_with_circle();
		// Static design: the canvas itself is frame 1.
		let line = d.frame_change_description(0).unwrap();
		assert!(line.contains("ellipse"), "{line}");
		assert!(line.contains("center"), "{line}");
		assert!(line.contains("white background"), "{line}");
		d.add_frame();
		assert_eq!(d.frame_change_description(0).unwrap(), line);
		assert_eq!(d.frame_change_description(1), None);
	}

	#[test]
	fn diff_names_moves_recolors_adds_and_removals() {
		let mut d = drawing_with_circle();
		d.add_frame(); // frame 1: circle at 40,40
		d.shift_shape(0, 0, 20, 0);
		d.add_frame(); // frame 2: circle moved right
		let line = d.frame_change_description(1).unwrap();
		assert!(line.contains("moves right 20 pixels"), "{line}");
		assert!(line.contains("ellipse"), "{line}");
		// Recolor + resize + background in one step.
		d.recolor_shape(0, 0, Srgb::new(0, 0, 255));
		d.resize_shape(0, 0, 30, 10);
		d.set_background(Srgb::BLACK);
		d.add_frame();
		let line = d.frame_change_description(2).unwrap();
		assert!(line.contains("changes color to"), "{line}");
		assert!(line.contains("blue"), "{line}");
		assert!(line.contains("resizes to 30 by 10 pixels"), "{line}");
		assert!(
			line.contains("background changes from white to black"),
			"{line}"
		);
		// An added shape appears, then disappears when removed.
		d.add_shape(
			0,
			Shape::new(ShapeKind::Rectangle, 0, 0, 10, 10, Srgb::BLACK),
		);
		d.add_frame();
		let line = d.frame_change_description(3).unwrap();
		assert!(line.contains("a new shape appears"), "{line}");
		assert!(line.contains("rectangle"), "{line}");
		d.remove_shape(0, 1);
		d.add_frame();
		let line = d.frame_change_description(4).unwrap();
		assert!(line.contains("disappears"), "{line}");
		// Capturing twice without changes says so.
		d.add_frame();
		assert_eq!(
			d.frame_change_description(5).unwrap(),
			"no change from the previous frame"
		);
	}

	#[test]
	fn spoken_export_lists_every_frame_with_its_duration() {
		let mut d = drawing_with_circle();
		d.add_frame();
		d.set_frame_duration(0, 400);
		d.shift_shape(0, 0, 0, 15);
		d.add_frame();
		d.set_frame_duration(1, 600);
		let text = d.animation_description();
		assert!(text.contains("2 frames"), "{text}");
		assert!(
			text.contains("total running time 1000 milliseconds"),
			"{text}"
		);
		assert!(text.contains("Frame 1 (400 milliseconds):"), "{text}");
		assert!(text.contains("Frame 2 (600 milliseconds):"), "{text}");
		assert!(text.contains("moves down 15 pixels"), "{text}");
	}

	#[test]
	fn gif_export_has_a_frame_per_frame_and_loops() {
		use image::AnimationDecoder;
		let mut d = drawing_with_circle();
		d.add_frame();
		d.shift_shape(0, 0, 20, 0);
		d.add_frame();
		let bytes = d.to_gif().unwrap();
		assert_eq!(&bytes[0..6], b"GIF89a");
		let decoder = image::codecs::gif::GifDecoder::new(std::io::Cursor::new(bytes)).unwrap();
		let frames: Vec<_> = decoder.into_frames().collect::<Result<_, _>>().unwrap();
		assert_eq!(frames.len(), 2);
		// A static design still exports: one frame, the canvas itself.
		let still = drawing_with_circle().to_gif().unwrap();
		assert_eq!(&still[0..6], b"GIF89a");
	}
}
