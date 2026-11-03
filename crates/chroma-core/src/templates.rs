//! templates: starter designs, built through the normal Drawing
//! API so a loaded template is an ordinary design — its event log
//! describes how it was made, and it saves, exports and edits like
//! anything drawn by hand.

use crate::design::{Shape, ShapeKind};
use crate::drawing::Drawing;
use crate::Srgb;

/// The templates on offer, in menu order.
pub const TEMPLATE_NAMES: &[&str] = &["Birthday card", "Simple house", "Face", "Chart"];

fn shape(kind: ShapeKind, x: i32, y: i32, w: u32, h: u32, color: (u8, u8, u8)) -> Shape {
	Shape::new(kind, x, y, w, h, Srgb::new(color.0, color.1, color.2))
}

/// A built-in starting design by name, or `None`. Canvas is the
/// standard 800 by 600 on white.
pub fn template(name: &str) -> Option<Drawing> {
	let mut d = Drawing::new(800, 600, Srgb::WHITE);
	match name {
		"Birthday card" => {
			d.add_shape(0, shape(ShapeKind::Rectangle, 40, 40, 720, 520, (253, 243, 216)));
			// Three balloons with strings.
			for (x, color) in [(100, (220, 50, 50)), (350, (60, 160, 75)), (600, (50, 90, 220))] {
				d.add_shape(0, shape(ShapeKind::Ellipse, x, 70, 90, 120, color));
				d.add_shape(
					0,
					Shape::stroke(vec![(x + 45, 190), (x + 30, 240), (x + 50, 290)], 2, Srgb::BLACK),
				);
			}
			let mut wish = Shape::text("Happy Birthday", "Arial", 48, 190, 330, 420, 70, Srgb::BLACK);
			wish.caption = Some("the words Happy Birthday".to_string());
			d.add_shape(0, wish);
			d.add_shape(0, shape(ShapeKind::Rectangle, 330, 460, 140, 12, (250, 180, 40)));
		}
		"Simple house" => {
			d.add_shape(0, shape(ShapeKind::Ellipse, 620, 50, 100, 100, (255, 205, 0))); // sun
			d.add_shape(0, shape(ShapeKind::Rectangle, 250, 300, 300, 220, (245, 222, 179)));
			d.add_shape(
				0,
				Shape::stroke(vec![(225, 305), (400, 175), (575, 305)], 10, Srgb::new(150, 75, 40)),
			);
			d.add_shape(0, shape(ShapeKind::Rectangle, 372, 415, 56, 105, (120, 70, 30))); // door
			d.add_shape(0, shape(ShapeKind::Rectangle, 282, 340, 52, 52, (170, 215, 240)));
			d.add_shape(0, shape(ShapeKind::Rectangle, 466, 340, 52, 52, (170, 215, 240)));
			d.add_shape(0, shape(ShapeKind::Rectangle, 0, 520, 800, 80, (110, 180, 90))); // grass
		}
		"Face" => {
			d.add_shape(0, shape(ShapeKind::Ellipse, 250, 60, 300, 420, (255, 220, 185)));
			d.add_shape(0, shape(ShapeKind::Ellipse, 325, 190, 42, 54, (30, 30, 30)));
			d.add_shape(0, shape(ShapeKind::Ellipse, 433, 190, 42, 54, (30, 30, 30)));
			d.add_shape(0, Shape::stroke(vec![(400, 265), (388, 325), (404, 330)], 4, Srgb::BLACK));
			// A smile: an arc dipping below its center line.
			let smile: Vec<(i32, i32)> = (0..=12)
				.map(|i| {
					let a = (20.0 + i as f64 * 140.0 / 12.0).to_radians();
					(400 + (105.0 * a.cos()) as i32, 315 + (105.0 * a.sin()) as i32)
				})
				.collect();
			d.add_shape(0, Shape::stroke(smile, 5, Srgb::new(180, 60, 60)));
		}
		"Chart" => {
			d.add_shape(0, Shape::stroke(vec![(100, 70), (100, 520)], 4, Srgb::BLACK));
			d.add_shape(0, Shape::stroke(vec![(100, 520), (730, 520)], 4, Srgb::BLACK));
			d.add_shape(0, shape(ShapeKind::Rectangle, 170, 390, 90, 130, (220, 50, 50)));
			d.add_shape(0, shape(ShapeKind::Rectangle, 330, 300, 90, 220, (60, 160, 75)));
			d.add_shape(0, shape(ShapeKind::Rectangle, 490, 210, 90, 310, (50, 90, 220)));
			let mut title = Shape::text("Chart", "Arial", 36, 350, 15, 120, 45, Srgb::BLACK);
			title.caption = Some("the word Chart as a title".to_string());
			d.add_shape(0, title);
		}
		_ => return None,
	}
	Some(d)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::project::Project;

	fn shape_count(d: &Drawing) -> usize {
		d.layers.iter().map(|l| l.shapes.len()).sum()
	}

	#[test]
	fn every_template_builds_and_round_trips() {
		for name in TEMPLATE_NAMES {
			let d = template(name).unwrap();
			assert!(shape_count(&d) >= 4, "{name} is too empty: {}", shape_count(&d));
			// A template is an ordinary design: it loads through the
			// normal project path and comes back unchanged.
			let text = Project { drawing: d.clone(), palette: Vec::new() }.to_text();
			let back = Project::from_text(&text).unwrap().drawing;
			assert_eq!(back.summary(), d.summary(), "{name} changed in the project round-trip");
			assert_eq!(back.flatten().caption_sheet(), d.flatten().caption_sheet());
		}
		assert!(template("Castle").is_none());
	}

	#[test]
	fn templates_stay_on_the_canvas() {
		for name in TEMPLATE_NAMES {
			let d = template(name).unwrap();
			for layer in &d.layers {
				for s in &layer.shapes {
					assert!(
						s.x >= 0 && s.y >= 0 && s.x + s.w as i32 <= 800 && s.y + s.h as i32 <= 600,
						"{name}: {:?} escapes the canvas",
						s.kind
					);
				}
			}
		}
	}
}
