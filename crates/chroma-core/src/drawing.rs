//! drawing: a layered design whose every change is logged, step by step.
//!
//! Layers are the organizing unit: shapes live on a named layer, layers
//! paint in order, and a hidden layer paints nothing. Every mutation —
//! a layer added, a shape shifted, a brush changed — appends one event
//! to the log, and each event describes itself in plain words. The log
//! is the design's memory: read top to bottom, it is the recipe that
//! recreates the image exactly, and it is what the summary export writes.

use crate::animation::Frame;
use crate::design::{Design, Shape, ShapeKind};
use crate::sliders::ColorAdjust;
use crate::{describe, Srgb};

#[derive(Debug, Clone)]
pub struct Layer {
	pub name: String,
	pub visible: bool,
	pub shapes: Vec<Shape>,
}

impl Layer {
	pub fn new(name: &str) -> Layer {
		Layer { name: name.to_string(), visible: true, shapes: Vec::new() }
	}
}

/// One recorded change. `describe` is the step line the summary export
/// (and the app's spoken readout) uses.
#[derive(Debug, Clone)]
pub enum Event {
	CanvasSet { width: u32, height: u32 },
	BackgroundSet { color: Srgb },
	LayerAdded { name: String },
	LayerRenamed { from: String, to: String },
	LayerRemoved { name: String },
	LayerVisibility { name: String, visible: bool },
	ShapeAdded { layer: String, shape: Shape },
	ShapeRemoved { layer: String, shape: Shape },
	ShapeShifted { layer: String, index: usize, dx: i32, dy: i32 },
	ShapeResized { layer: String, index: usize, w: u32, h: u32 },
	ShapeRotated { layer: String, index: usize, angle: f64 },
	ShapeRecolored { layer: String, index: usize, color: Srgb },
	ShapeCaptioned { layer: String, index: usize },
	ShapeOpacitySet { layer: String, index: usize, opacity: u8 },
	ShapeBrushSet { layer: String, index: usize, width: u32 },
	ShapeColorAdjusted {
		layer: String,
		index: usize,
		shade: u8,
		tint: u8,
		tone: u8,
		chroma: u8,
		undertone: i32,
		warmth: i32,
		vibrancy: i32,
		brightness: i32,
		clarity: i32,
		fog: i32,
		hardness: i32,
	},
	LayerColorsAdjusted {
		name: String,
		count: usize,
		shade: u8,
		tint: u8,
		tone: u8,
		chroma: u8,
		undertone: i32,
		warmth: i32,
		vibrancy: i32,
		brightness: i32,
		clarity: i32,
		fog: i32,
		hardness: i32,
	},
	FrameAdded { index: usize, duration_ms: u32 },
	FrameDuplicated { index: usize },
	FrameRemoved { index: usize },
	FrameMoved { from: usize, to: usize },
	FrameDurationSet { index: usize, duration_ms: u32 },
	AllFramesDurationSet { count: usize, duration_ms: u32 },
	FrameLoaded { index: usize },
	FrameUpdated { index: usize },
}

impl Event {
	pub fn describe(&self, drawing: &Drawing) -> String {
		match self {
			Event::CanvasSet { width, height } => {
				format!("Set the canvas to {width} by {height} pixels.")
			}
			Event::BackgroundSet { color } => {
				format!("Set the background to {} ({}).", describe(*color), color.to_hex())
			}
			Event::LayerAdded { name } => format!("Add layer \"{name}\"."),
			Event::LayerRenamed { from, to } => format!("Rename layer \"{from}\" to \"{to}\"."),
			Event::LayerRemoved { name } => format!("Remove layer \"{name}\" and its shapes."),
			Event::LayerVisibility { name, visible } => {
				format!("{} layer \"{name}\".", if *visible { "Show" } else { "Hide" })
			}
			Event::ShapeAdded { layer, shape } => {
				format!("On layer \"{layer}\", add {}.", drawing.caption_of(shape))
			}
			Event::ShapeRemoved { layer, shape } => {
				format!("On layer \"{layer}\", remove {}.", drawing.caption_of(shape))
			}
			Event::ShapeShifted { layer, index, dx, dy } => format!(
				"On layer \"{layer}\", shift shape {} by {dx} across and {dy} down.",
				index + 1
			),
			Event::ShapeResized { layer, index, w, h } => format!(
				"On layer \"{layer}\", resize shape {} to {w} by {h} pixels.",
				index + 1
			),
			Event::ShapeRotated { layer, index, angle } => format!(
				"On layer \"{layer}\", rotate shape {} to {angle} degrees.",
				index + 1
			),
			Event::ShapeRecolored { layer, index, color } => format!(
				"On layer \"{layer}\", recolor shape {} to {} ({}).",
				index + 1,
				describe(*color),
				color.to_hex()
			),
			Event::ShapeCaptioned { layer, index } => format!(
				"On layer \"{layer}\", set the caption of shape {}.",
				index + 1
			),
			Event::ShapeOpacitySet { layer, index, opacity } => format!(
				"On layer \"{layer}\", set shape {} to {opacity} percent opaque.",
				index + 1
			),
			Event::ShapeBrushSet { layer, index, width } => format!(
				"On layer \"{layer}\", set the brush width of shape {} to {width} pixels.",
				index + 1
			),
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
			} => {
				format!(
					"On layer \"{layer}\", adjust the color of shape {}: {}.",
					index + 1,
					crate::sliders::ColorAdjust {
						shade: *shade,
						tint: *tint,
						tone: *tone,
						chroma: *chroma,
						undertone: *undertone,
						warmth: *warmth,
						vibrancy: *vibrancy,
						brightness: *brightness,
						clarity: *clarity,
						fog: *fog,
						hardness: *hardness,
					}
					.describe()
				)
			}
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
			} => {
				format!(
					"Adjust the colors of all {count} elements on layer \"{name}\": {}.",
					crate::sliders::ColorAdjust {
						shade: *shade,
						tint: *tint,
						tone: *tone,
						chroma: *chroma,
						undertone: *undertone,
						warmth: *warmth,
						vibrancy: *vibrancy,
						brightness: *brightness,
						clarity: *clarity,
						fog: *fog,
						hardness: *hardness,
					}
					.describe()
				)
			}
			Event::FrameAdded { index, duration_ms } => format!(
				"Add animation frame {}: a copy of the canvas as it is now, shown for {duration_ms} milliseconds.",
				index + 1
			),
			Event::FrameDuplicated { index } => {
				format!("Duplicate animation frame {}.", index + 1)
			}
			Event::FrameRemoved { index } => {
				format!("Remove animation frame {}.", index + 1)
			}
			Event::FrameMoved { from, to } => format!(
				"Move animation frame {} to position {}.",
				from + 1,
				to + 1
			),
			Event::FrameDurationSet { index, duration_ms } => format!(
				"Set animation frame {} to show for {duration_ms} milliseconds.",
				index + 1
			),
			Event::AllFramesDurationSet { count, duration_ms } => format!(
				"Set all {count} animation frames to show for {duration_ms} milliseconds each."
			),
			Event::FrameLoaded { index } => format!(
				"Load animation frame {} onto the canvas for editing.",
				index + 1
			),
			Event::FrameUpdated { index } => format!(
				"Replace animation frame {} with the canvas as it is now.",
				index + 1
			),
		}
	}
}

#[derive(Debug, Clone)]
pub struct Drawing {
	pub width: u32,
	pub height: u32,
	pub background: Srgb,
	pub layers: Vec<Layer>,
	pub log: Vec<Event>,
	/// The animation frames, in play order: full snapshots of the
	/// layers and background, captured by `add_frame` and friends
	/// (see `crate::animation`). Empty means a static design — the
	/// canvas itself is the only picture.
	pub frames: Vec<Frame>,
}

impl Drawing {
	/// A fresh drawing: one visible layer, and the log already holding
	/// the canvas, background and layer steps that recreate it.
	pub fn new(width: u32, height: u32, background: Srgb) -> Drawing {
		let mut d = Drawing {
			width,
			height,
			background,
			layers: vec![Layer::new("Layer 1")],
			log: Vec::new(),
			frames: Vec::new(),
		};
		d.log.push(Event::CanvasSet { width, height });
		d.log.push(Event::BackgroundSet { color: background });
		d.log.push(Event::LayerAdded { name: "Layer 1".to_string() });
		d
	}

	fn caption_of(&self, shape: &Shape) -> String {
		Design { width: self.width, height: self.height, background: self.background, shapes: Vec::new() }
			.caption_for(shape)
	}

	/// The shape's caption as the flat design computes it (its own
	/// caption when it has one, else the generated one).
	pub fn description_of(&self, shape: &Shape) -> String {
		self.caption_of(shape)
	}

	/// The shape's automatic plain-language description (see
	/// `Design::describe_plain`): what the elements list shows and the
	/// app speaks when a shape is created without a caption.
	pub fn plain_of(&self, shape: &Shape) -> String {
		Design { width: self.width, height: self.height, background: self.background, shapes: Vec::new() }
			.describe_plain(shape)
	}

	pub fn set_canvas(&mut self, width: u32, height: u32) {
		self.width = width;
		self.height = height;
		self.log.push(Event::CanvasSet { width, height });
	}

	pub fn set_background(&mut self, color: Srgb) {
		self.background = color;
		self.log.push(Event::BackgroundSet { color });
	}

	/// Adds a layer on top and returns its index.
	pub fn add_layer(&mut self, name: &str) -> usize {
		self.layers.push(Layer::new(name));
		self.log.push(Event::LayerAdded { name: name.to_string() });
		self.layers.len() - 1
	}

	pub fn rename_layer(&mut self, index: usize, name: &str) -> bool {
		let Some(layer) = self.layers.get_mut(index) else { return false };
		let from = std::mem::replace(&mut layer.name, name.to_string());
		self.log.push(Event::LayerRenamed { from, to: name.to_string() });
		true
	}

	/// Removes a layer and its shapes. The drawing always keeps at
	/// least one layer; removing the last one fails.
	pub fn remove_layer(&mut self, index: usize) -> bool {
		if self.layers.len() <= 1 || index >= self.layers.len() {
			return false;
		}
		let gone = self.layers.remove(index);
		self.log.push(Event::LayerRemoved { name: gone.name });
		true
	}

	pub fn set_layer_visible(&mut self, index: usize, visible: bool) -> bool {
		let Some(layer) = self.layers.get_mut(index) else { return false };
		layer.visible = visible;
		let name = layer.name.clone();
		self.log.push(Event::LayerVisibility { name, visible });
		true
	}

	pub fn add_shape(&mut self, layer: usize, shape: Shape) -> Option<usize> {
		let l = self.layers.get_mut(layer)?;
		l.shapes.push(shape.clone());
		let index = l.shapes.len() - 1;
		let name = l.name.clone();
		self.log.push(Event::ShapeAdded { layer: name, shape });
		Some(index)
	}

	pub fn remove_shape(&mut self, layer: usize, index: usize) -> Option<Shape> {
		let l = self.layers.get_mut(layer)?;
		if index >= l.shapes.len() {
			return None;
		}
		let gone = l.shapes.remove(index);
		let name = l.name.clone();
		self.log.push(Event::ShapeRemoved { layer: name, shape: gone.clone() });
		Some(gone)
	}

	pub fn shift_shape(&mut self, layer: usize, index: usize, dx: i32, dy: i32) -> bool {
		let Some(l) = self.layers.get_mut(layer) else { return false };
		let Some(s) = l.shapes.get_mut(index) else { return false };
		s.x += dx;
		s.y += dy;
		for p in &mut s.points {
			p.0 += dx;
			p.1 += dy;
		}
		let name = l.name.clone();
		self.log.push(Event::ShapeShifted { layer: name, index, dx, dy });
		true
	}

	pub fn resize_shape(&mut self, layer: usize, index: usize, w: u32, h: u32) -> bool {
		let Some(l) = self.layers.get_mut(layer) else { return false };
		let Some(s) = l.shapes.get_mut(index) else { return false };
		if s.kind == ShapeKind::Stroke {
			return false; // a stroke's size is its points; shift or redraw it
		}
		s.w = w;
		s.h = h;
		let name = l.name.clone();
		// A resize slider drags through many sizes in a row; the
		// log keeps the final one (see log_slider_event). A resize
		// after any other change logs fresh, as before.
		self.log_slider_event(
			Event::ShapeResized { layer: name.clone(), index, w, h },
			|e| matches!(e, Event::ShapeResized { layer: l, index: i, .. } if *l == name && *i == index),
		);
		true
	}

	pub fn rotate_shape(&mut self, layer: usize, index: usize, angle: f64) -> bool {
		let Some(l) = self.layers.get_mut(layer) else { return false };
		let Some(s) = l.shapes.get_mut(index) else { return false };
		s.angle = angle;
		let name = l.name.clone();
		self.log.push(Event::ShapeRotated { layer: name, index, angle });
		true
	}

	pub fn recolor_shape(&mut self, layer: usize, index: usize, color: Srgb) -> bool {
		let Some(l) = self.layers.get_mut(layer) else { return false };
		let Some(s) = l.shapes.get_mut(index) else { return false };
		s.fill = color;
		let name = l.name.clone();
		self.log.push(Event::ShapeRecolored { layer: name, index, color });
		true
	}

	pub fn caption_shape(&mut self, layer: usize, index: usize, caption: String) -> bool {
		let Some(l) = self.layers.get_mut(layer) else { return false };
		let Some(s) = l.shapes.get_mut(index) else { return false };
		s.caption = Some(caption);
		let name = l.name.clone();
		self.log.push(Event::ShapeCaptioned { layer: name, index });
		true
	}

	/// Log an event for a slider-driven change. When the newest
	/// log entry is the same kind of change to the same target, it
	/// is replaced instead of appended: a slider dragged through
	/// fifty values is one step in the design's story, told with
	/// its final value, not fifty steps.
	pub(crate) fn log_slider_event(&mut self, event: Event, replaces: impl Fn(&Event) -> bool) {
		match self.log.last_mut() {
			Some(last) if replaces(last) => *last = event,
			_ => self.log.push(event),
		}
	}

	/// Set a shape's opacity (percent, clamped to 100).
	pub fn set_shape_opacity(&mut self, layer: usize, index: usize, opacity: u8) -> bool {
		let opacity = opacity.min(100);
		let name = {
			let Some(l) = self.layers.get_mut(layer) else { return false };
			let Some(s) = l.shapes.get_mut(index) else { return false };
			s.opacity = opacity;
			l.name.clone()
		};
		self.log_slider_event(
			Event::ShapeOpacitySet { layer: name.clone(), index, opacity },
			|e| matches!(e, Event::ShapeOpacitySet { layer: l, index: i, .. } if *l == name && *i == index),
		);
		true
	}

	/// Set a stroke's brush width in pixels. Only strokes have a
	/// brush width; anything else fails.
	pub fn set_shape_brush_width(&mut self, layer: usize, index: usize, width: u32) -> bool {
		let width = width.max(1);
		let name = {
			let Some(l) = self.layers.get_mut(layer) else { return false };
			let Some(s) = l.shapes.get_mut(index) else { return false };
			if s.kind != ShapeKind::Stroke {
				return false;
			}
			s.brush = width;
			l.name.clone()
		};
		self.log_slider_event(
			Event::ShapeBrushSet { layer: name.clone(), index, width },
			|e| matches!(e, Event::ShapeBrushSet { layer: l, index: i, .. } if *l == name && *i == index),
		);
		true
	}

	/// Adjust one shape's color: the amounts apply to `base`
	/// — the color the shape had when the adjustment began — so
	/// the result never compounds as a slider moves, and zeroing
	/// the sliders restores the original color exactly. Logged as
	/// one event per adjustment (see `log_slider_event`).
	pub fn adjust_shape_color(
		&mut self,
		layer: usize,
		index: usize,
		base: Srgb,
		adjust: ColorAdjust,
	) -> bool {
		let name = {
			let Some(l) = self.layers.get_mut(layer) else { return false };
			let Some(s) = l.shapes.get_mut(index) else { return false };
			s.fill = adjust.apply(base);
			l.name.clone()
		};
		self.log_slider_event(
			Event::ShapeColorAdjusted {
				layer: name.clone(),
				index,
				shade: adjust.shade,
				tint: adjust.tint,
				tone: adjust.tone,
				chroma: adjust.chroma,
				undertone: adjust.undertone,
				warmth: adjust.warmth,
				vibrancy: adjust.vibrancy,
				brightness: adjust.brightness,
				clarity: adjust.clarity,
				fog: adjust.fog,
				hardness: adjust.hardness,
			},
			|e| matches!(e, Event::ShapeColorAdjusted { layer: l, index: i, .. } if *l == name && *i == index),
		);
		true
	}

	/// Adjust every shape's color on a layer at once: each shape's
	/// new color is the adjustment applied to its entry in `bases`
	/// (the layer's colors when the adjustment began, in shape
	/// order). Returns how many shapes were adjusted — zero when
	/// the layer is missing or `bases` no longer matches it, so a
	/// stale snapshot can never repaint the wrong shapes. Logged
	/// as ONE event for the whole layer.
	pub fn adjust_layer_colors(
		&mut self,
		layer: usize,
		bases: &[Srgb],
		adjust: ColorAdjust,
	) -> usize {
		let (name, count) = {
			let Some(l) = self.layers.get_mut(layer) else { return 0 };
			if bases.len() != l.shapes.len() || bases.is_empty() {
				return 0;
			}
			for (s, base) in l.shapes.iter_mut().zip(bases.iter()) {
				s.fill = adjust.apply(*base);
			}
			(l.name.clone(), l.shapes.len())
		};
		self.log_slider_event(
			Event::LayerColorsAdjusted {
				name: name.clone(),
				count,
				shade: adjust.shade,
				tint: adjust.tint,
				tone: adjust.tone,
				chroma: adjust.chroma,
				undertone: adjust.undertone,
				warmth: adjust.warmth,
				vibrancy: adjust.vibrancy,
				brightness: adjust.brightness,
				clarity: adjust.clarity,
				fog: adjust.fog,
				hardness: adjust.hardness,
			},
			|e| matches!(e, Event::LayerColorsAdjusted { name: n, .. } if *n == name),
		);
		count
	}

	/// The flat design the renderers and caption sheets consume: visible
	/// layers only, bottom layer first, shapes in paint order.
	pub fn flatten(&self) -> Design {
		let mut design = Design::new(self.width, self.height, self.background);
		for layer in &self.layers {
			if layer.visible {
				design.shapes.extend(layer.shapes.iter().cloned());
			}
		}
		design
	}

	/// The recreation file: step-by-step instructions that rebuild this
	/// exact drawing from nothing, generated from the event log, with
	/// the final state's shape list appended as a check.
	pub fn summary(&self) -> String {
		let mut out = String::from(
			"Blind Designer recreation file.\n\
			 Follow the steps in order to recreate the image exactly.\n\n",
		);
		for (i, event) in self.log.iter().enumerate() {
			out.push_str(&format!("{}. {}\n", i + 1, event.describe(self)));
		}
		let shapes: usize = self.layers.iter().map(|l| l.shapes.len()).sum();
		out.push_str(&format!(
			"\nFinal state: {} layer{}, {} shape{}, canvas {} by {} pixels, background {} ({}).\n",
			self.layers.len(),
			if self.layers.len() == 1 { "" } else { "s" },
			shapes,
			if shapes == 1 { "" } else { "s" },
			self.width,
			self.height,
			describe(self.background),
			self.background.to_hex()
		));
		for layer in &self.layers {
			out.push_str(&format!(
				"Layer \"{}\"{}: {} shape{}.\n",
				layer.name,
				if layer.visible { "" } else { " (hidden)" },
				layer.shapes.len(),
				if layer.shapes.len() == 1 { "" } else { "s" }
			));
			for (i, shape) in layer.shapes.iter().enumerate() {
				out.push_str(&format!("  {}. {}\n", i + 1, self.caption_of(shape)));
			}
		}
		out
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn red() -> Srgb {
		Srgb::new(200, 40, 30)
	}

	#[test]
	fn new_drawing_logs_its_own_setup() {
		let d = Drawing::new(100, 80, Srgb::WHITE);
		assert_eq!(d.log.len(), 3);
		assert_eq!(d.layers.len(), 1);
		let s = d.summary();
		assert!(s.contains("1. Set the canvas to 100 by 80 pixels."), "{s}");
		assert!(s.contains("3. Add layer \"Layer 1\"."), "{s}");
	}

	#[test]
	fn every_change_lands_in_the_log() {
		let mut d = Drawing::new(100, 100, Srgb::WHITE);
		let top = d.add_layer("Sky");
		d.add_shape(top, Shape::new(ShapeKind::Ellipse, 10, 10, 30, 20, red()));
		d.shift_shape(top, 0, 5, -2);
		d.rotate_shape(top, 0, 45.0);
		d.recolor_shape(top, 0, Srgb::new(0, 0, 255));
		d.set_layer_visible(0, false);
		let steps: Vec<String> = d.log.iter().map(|e| e.describe(&d)).collect();
		assert_eq!(steps.len(), 3 + 6);
		assert!(steps[5].contains("shift shape 1 by 5 across and -2 down"), "{}", steps[5]);
		assert!(steps[6].contains("rotate shape 1 to 45 degrees"), "{}", steps[6]);
		assert!(steps[7].contains("recolor shape 1"), "{}", steps[7]);
		assert!(steps[8].contains("Hide layer \"Layer 1\""), "{}", steps[8]);
		let shape = &d.layers[top].shapes[0];
		assert_eq!((shape.x, shape.y), (15, 8));
		assert_eq!(shape.angle, 45.0);
	}

	#[test]
	fn flatten_skips_hidden_layers_and_keeps_order() {
		let mut d = Drawing::new(10, 10, Srgb::WHITE);
		d.add_shape(0, Shape::new(ShapeKind::Rectangle, 0, 0, 10, 10, red()));
		let top = d.add_layer("Top");
		d.add_shape(top, Shape::new(ShapeKind::Rectangle, 0, 0, 5, 5, Srgb::new(0, 0, 255)));
		assert_eq!(d.flatten().shapes.len(), 2);
		d.set_layer_visible(top, false);
		let flat = d.flatten();
		assert_eq!(flat.shapes.len(), 1);
		assert_eq!(flat.shapes[0].fill, red());
		// The hidden layer's pixels never reach the render.
		assert_eq!(flat.render().get_pixel(1, 1), &image::Rgb([200, 40, 30]));
	}

	#[test]
	fn the_last_layer_cannot_be_removed() {
		let mut d = Drawing::new(10, 10, Srgb::WHITE);
		assert!(!d.remove_layer(0));
		let top = d.add_layer("Top");
		assert!(d.remove_layer(top));
		assert_eq!(d.layers.len(), 1);
	}

	#[test]
	fn strokes_render_and_shift_with_their_points() {
		let mut d = Drawing::new(20, 20, Srgb::WHITE);
		let s = Shape::stroke(vec![(2, 10), (17, 10)], 1, red());
		d.add_shape(0, s);
		let img = d.flatten().render();
		assert_eq!(img.get_pixel(10, 10), &image::Rgb([200, 40, 30]));
		assert_eq!(img.get_pixel(10, 4), &image::Rgb([255, 255, 255]));
		d.shift_shape(0, 0, 0, -5);
		let img = d.flatten().render();
		assert_eq!(img.get_pixel(10, 5), &image::Rgb([200, 40, 30]));
		assert_eq!(img.get_pixel(10, 10), &image::Rgb([255, 255, 255]));
	}

	#[test]
	fn text_and_image_shapes_live_in_layers_and_the_log() {
		let mut d = Drawing::new(800, 600, Srgb::WHITE);
		d.add_shape(0, Shape::text("Title", "Arial", 32, 10, 10, 200, 40, Srgb::BLACK));
		d.add_shape(0, Shape::image("photo.png", 0, 100, 100, 80, red()));
		// Both kinds resize by their box and shift like other shapes.
		assert!(d.resize_shape(0, 0, 300, 50));
		assert!(d.shift_shape(0, 1, 5, 5));
		let steps: Vec<String> = d.log.iter().map(|e| e.describe(&d)).collect();
		// The text shape's log line reads its phrase (its caption).
		assert!(steps[3].contains("Title"), "{}", steps[3]);
		assert!(steps[4].contains("image from file photo.png"), "{}", steps[4]);
		let plain = d.plain_of(&d.layers[0].shapes[0]);
		assert!(plain.contains("text \"Title\""), "{plain}");
		assert!(plain.contains("32 point"), "{plain}");
		// The flat design carries both kinds to the renderers.
		let flat = d.flatten();
		assert_eq!(flat.shapes.len(), 2);
		assert_eq!(flat.shapes[1].kind, ShapeKind::Image);
	}

	#[test]
	fn slider_opacity_sets_and_coalesces_in_the_log() {
		let mut d = Drawing::new(40, 40, Srgb::WHITE);
		d.add_shape(0, Shape::new(ShapeKind::Rectangle, 5, 5, 10, 10, red()));
		let logged = d.log.len();
		assert!(d.set_shape_opacity(0, 0, 80));
		assert!(d.set_shape_opacity(0, 0, 55));
		assert!(d.set_shape_opacity(0, 0, 30));
		assert_eq!(d.layers[0].shapes[0].opacity, 30);
		// Three drags through the slider are ONE new log event.
		assert_eq!(d.log.len(), logged + 1, "log: {:?}", d.log.last());
		assert!(!d.set_shape_opacity(9, 0, 30));
		assert!(!d.set_shape_opacity(0, 9, 30));
	}

	#[test]
	fn slider_brush_width_only_touches_strokes() {
		let mut d = Drawing::new(40, 40, Srgb::WHITE);
		d.add_shape(0, Shape::new(ShapeKind::Rectangle, 5, 5, 10, 10, red()));
		assert!(!d.set_shape_brush_width(0, 0, 9));
		d.add_shape(0, Shape::stroke(vec![(1, 1), (20, 20)], 3, red()));
		assert!(d.set_shape_brush_width(0, 1, 9));
		assert_eq!(d.layers[0].shapes[1].brush, 9);
	}

	#[test]
	fn layer_adjustment_recolors_every_shape_as_one_event() {
		let mut d = Drawing::new(100, 100, Srgb::WHITE);
		d.add_shape(0, Shape::new(ShapeKind::Rectangle, 5, 5, 10, 10, red()));
		d.add_shape(0, Shape::new(ShapeKind::Ellipse, 30, 30, 10, 10, Srgb::new(58, 124, 165)));
		d.add_shape(0, Shape::new(ShapeKind::Rectangle, 60, 60, 10, 10, Srgb::BLACK));
		let bases: Vec<Srgb> = d.layers[0].shapes.iter().map(|s| s.fill).collect();
		let logged = d.log.len();
		let adj = ColorAdjust { shade: 50, ..Default::default() };
		// All three shapes adjusted, counted, one log event.
		assert_eq!(d.adjust_layer_colors(0, &bases, adj), 3);
		assert_eq!(d.log.len(), logged + 1);
		for (shape, base) in d.layers[0].shapes.iter().zip(bases.iter()) {
			assert_eq!(shape.fill, adj.apply(*base));
			assert!(shape.fill.to_oklch().l <= base.to_oklch().l);
		}
		// Dragging further replaces the event, never adds another.
		let adj2 = ColorAdjust { shade: 80, ..Default::default() };
		assert_eq!(d.adjust_layer_colors(0, &bases, adj2), 3);
		assert_eq!(d.log.len(), logged + 1);
		// A stale snapshot (wrong shape count) changes nothing.
		assert_eq!(d.adjust_layer_colors(0, &bases[..2], adj), 0);
		assert_eq!(d.adjust_layer_colors(9, &bases, adj), 0);
		// The event describes itself in words.
		let text = d.log.last().unwrap().describe(&d);
		assert!(text.contains("all 3 elements"), "{text}");
		assert!(text.contains("shaded 80 percent"), "{text}");
	}

	#[test]
	fn shape_adjustment_applies_from_the_base_and_restores() {
		let mut d = Drawing::new(40, 40, Srgb::WHITE);
		d.add_shape(0, Shape::new(ShapeKind::Rectangle, 5, 5, 10, 10, red()));
		let base = red();
		let adj = ColorAdjust { tint: 40, warmth: 30, ..Default::default() };
		assert!(d.adjust_shape_color(0, 0, base, adj));
		assert_eq!(d.layers[0].shapes[0].fill, adj.apply(base));
		assert_ne!(d.layers[0].shapes[0].fill, base);
		// Back to zero restores the original color exactly.
		assert!(d.adjust_shape_color(0, 0, base, ColorAdjust::default()));
		assert_eq!(d.layers[0].shapes[0].fill, base);
		assert!(!d.adjust_shape_color(0, 7, base, adj));
	}

	#[test]
	fn rotation_changes_the_render() {
		let mut d = Drawing::new(40, 40, Srgb::WHITE);
		d.add_shape(0, Shape::new(ShapeKind::Rectangle, 15, 5, 10, 30, red()));
		let plain = d.flatten().render();
		assert_eq!(plain.get_pixel(20, 6), &image::Rgb([200, 40, 30])); // top middle
		d.rotate_shape(0, 0, 90.0);
		let turned = d.flatten().render();
		assert_eq!(turned.get_pixel(20, 6), &image::Rgb([255, 255, 255])); // now empty
		assert_eq!(turned.get_pixel(6, 20), &image::Rgb([200, 40, 30])); // arm reaches left
	}
}
