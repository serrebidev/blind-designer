//! design: a 2D canvas of palette-colored shapes, exportable as an image.
//!
//! A design is plain data: a size, a background color, and a list of
//! shapes in paint order. Captions are micro generative fill-ins — a shape
//! whose caption is empty gets one generated from a template (what it is,
//! what color it is in words, how it relates to the background); a shape
//! with a hand-written caption keeps it untouched. Generation only ever
//! fills gaps, never overwrites.

use crate::{describe, Srgb};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
	Rectangle,
	Ellipse,
}

impl ShapeKind {
	pub fn name(self) -> &'static str {
		match self {
			ShapeKind::Rectangle => "rectangle",
			ShapeKind::Ellipse => "ellipse",
		}
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
	/// Hand-written caption, if any. `None` means "generate one for me".
	pub caption: Option<String>,
}

impl Shape {
	pub fn new(kind: ShapeKind, x: i32, y: i32, w: u32, h: u32, fill: Srgb) -> Shape {
		Shape { kind, x, y, w, h, fill, caption: None }
	}
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
		Design { width, height, background, shapes: Vec::new() }
	}

	/// The hue relationship between two colors, in words, when it is one of
	/// the classic predictable ones. `None` when there is no clean relation
	/// (or either color is achromatic and hue means nothing).
	pub fn relation(a: Srgb, b: Srgb) -> Option<&'static str> {
		let (oa, ob) = (a.to_oklch(), b.to_oklch());
		if oa.c < 0.02 || ob.c < 0.02 {
			return None;
		}
		let d = (oa.h - ob.h).abs();
		let d = if d > 180.0 { 360.0 - d } else { d };
		match d {
			x if x <= 10.0 => Some("same hue as"),
			x if (x - 30.0).abs() <= 10.0 => Some("analogous to"),
			x if (x - 120.0).abs() <= 10.0 => Some("triadic with"),
			x if (x - 180.0).abs() <= 10.0 => Some("complementary to"),
			_ => None,
		}
	}

	/// The caption for one shape: the hand-written one if present, else a
	/// generated fill-in describing the shape against the background.
	pub fn caption_for(&self, shape: &Shape) -> String {
		if let Some(c) = &shape.caption {
			return c.clone();
		}
		let mut s = format!(
			"{} filled with {}, {} by {} pixels at {}, {}",
			shape.kind.name(),
			describe(shape.fill),
			shape.w,
			shape.h,
			shape.x,
			shape.y
		);
		if let Some(rel) = Self::relation(shape.fill, self.background) {
			s.push_str(&format!("; {} the background {}", rel, describe(self.background)));
		}
		let ratio = shape.fill.contrast(self.background);
		s.push_str(&format!("; contrast against background {:.1} to 1", ratio));
		s
	}

	/// Captions for the whole design: a one-line header plus one line per
	/// shape, in paint order.
	pub fn caption_sheet(&self) -> String {
		let mut out = format!(
			"Design {} by {} pixels, background {}.\n",
			self.width,
			self.height,
			describe(self.background)
		);
		for (i, shape) in self.shapes.iter().enumerate() {
			out.push_str(&format!("{}. {}\n", i + 1, self.caption_for(shape)));
		}
		out
	}

	/// Render the design to an image: background fill, then each shape in
	/// order. Rectangles and ellipses with hard edges (no blending — colors
	/// in the output are exactly the palette colors, pixel for pixel).
	pub fn render(&self) -> image::RgbImage {
		let mut img = image::RgbImage::from_pixel(
			self.width,
			self.height,
			image::Rgb([self.background.r, self.background.g, self.background.b]),
		);
		for shape in &self.shapes {
			let px = image::Rgb([shape.fill.r, shape.fill.g, shape.fill.b]);
			let (x0, y0) = (shape.x, shape.y);
			let (x1, y1) = (shape.x + shape.w as i32, shape.y + shape.h as i32);
			for y in y0.max(0)..y1.min(self.height as i32) {
				for x in x0.max(0)..x1.min(self.width as i32) {
					let inside = match shape.kind {
						ShapeKind::Rectangle => true,
						ShapeKind::Ellipse => {
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
						img.put_pixel(x as u32, y as u32, px);
					}
				}
			}
		}
		img
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
	fn render_paints_shapes_in_order() {
		let mut d = Design::new(10, 10, Srgb::WHITE);
		d.shapes
			.push(Shape::new(ShapeKind::Rectangle, 0, 0, 10, 10, Srgb::new(255, 0, 0)));
		d.shapes
			.push(Shape::new(ShapeKind::Ellipse, 2, 2, 6, 6, Srgb::new(0, 0, 255)));
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
		assert_eq!(Design::relation(base, comp), Some("complementary to"));
		assert_eq!(Design::relation(base, base), Some("same hue as"));
		assert_eq!(Design::relation(base, Srgb::WHITE), None);
	}

	#[test]
	fn css_export_lists_each_color_once() {
		let mut d = Design::new(10, 10, Srgb::new(15, 94, 104));
		d.shapes
			.push(Shape::new(ShapeKind::Rectangle, 0, 0, 5, 5, Srgb::new(15, 94, 104)));
		d.shapes
			.push(Shape::new(ShapeKind::Rectangle, 5, 5, 5, 5, Srgb::new(255, 128, 0)));
		let css = d.to_css("brand");
		assert_eq!(css.matches("--brand-").count(), 2);
		assert!(css.contains("#0F5E68"));
		assert!(css.contains("#FF8000"));
	}
}
