//! drawing: a layered design whose every change is logged, step by step.
//!
//! Layers are the organizing unit: shapes live on a named layer, layers
//! paint in order, and a hidden layer paints nothing. Every mutation —
//! a layer added, a shape shifted, a brush changed — appends one event
//! to the log, and each event describes itself in plain words. The log
//! is the design's memory: read top to bottom, it is the recipe that
//! recreates the image exactly, and it is what the summary export writes.

use crate::design::{Design, Shape, ShapeKind};
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
		self.log.push(Event::ShapeResized { layer: name, index, w, h });
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
