//! templates: starter designs, built through the normal Drawing
//! API so a loaded template is an ordinary design — its event log
//! describes how it was made, and it saves, exports and edits like
//! anything drawn by hand.

use crate::design::{Shape, ShapeKind};
use crate::drawing::{DepthRole, Drawing};
use crate::Srgb;

/// The templates on offer, in menu order.
pub const TEMPLATE_NAMES: &[&str] = &[
	"Birthday card",
	"Simple house",
	"Face",
	"Chart",
	"Portrait",
	"Landscape",
];

/// The pattern templates on offer, in menu order: two-color
/// grids and repeats that need the user's two colors before
/// they can be built (see `pattern_template`).
pub const PATTERN_NAMES: &[&str] = &["Checkerboard", "Stripes", "Dots"];

/// A two-color pattern design by name, or `None`. Like a
/// template, it is an ordinary design built through the Drawing
/// API on the standard 800 by 600 canvas — but where a template
/// brings its own colors, a pattern is woven from the two the
/// user picks: the canvas background is the second color, the
/// marks on it (the light squares, the stripes, the dots) are
/// the first, all on one layer tagged as the background depth,
/// named "Pattern". A checkerboard's cells are 100 pixels,
/// stripes run vertically 100 pixels wide, dots stand on a
/// 100-pixel grid.
pub fn pattern_template(kind: &str, first: Srgb, second: Srgb) -> Option<Drawing> {
	let mut d = Drawing::new(800, 600, second);
	d.rename_layer(0, "Pattern");
	d.set_layer_depth(0, DepthRole::Background);
	match kind {
		"Checkerboard" => {
			for row in 0..6 {
				for col in 0..8 {
					if (row + col) % 2 == 0 {
						d.add_shape(
							0,
							Shape::new(ShapeKind::Rectangle, col * 100, row * 100, 100, 100, first),
						);
					}
				}
			}
		}
		"Stripes" => {
			for col in (0..8).step_by(2) {
				d.add_shape(
					0,
					Shape::new(ShapeKind::Rectangle, col * 100, 0, 100, 600, first),
				);
			}
		}
		"Dots" => {
			for row in 0..6 {
				for col in 0..8 {
					d.add_shape(
						0,
						Shape::new(
							ShapeKind::Ellipse,
							col * 100 + 25,
							row * 100 + 25,
							50,
							50,
							first,
						),
					);
				}
			}
		}
		_ => return None,
	}
	Some(d)
}

fn shape(kind: ShapeKind, x: i32, y: i32, w: u32, h: u32, color: (u8, u8, u8)) -> Shape {
	Shape::new(kind, x, y, w, h, Srgb::new(color.0, color.1, color.2))
}

/// A built-in starting design by name, or `None`. Canvas is the
/// standard 800 by 600 on white.
pub fn template(name: &str) -> Option<Drawing> {
	let mut d = Drawing::new(800, 600, Srgb::WHITE);
	match name {
		"Birthday card" => {
			d.add_shape(
				0,
				shape(ShapeKind::Rectangle, 40, 40, 720, 520, (253, 243, 216)),
			);
			// Three balloons with strings.
			for (x, color) in [
				(100, (220, 50, 50)),
				(350, (60, 160, 75)),
				(600, (50, 90, 220)),
			] {
				d.add_shape(0, shape(ShapeKind::Ellipse, x, 70, 90, 120, color));
				d.add_shape(
					0,
					Shape::stroke(
						vec![(x + 45, 190), (x + 30, 240), (x + 50, 290)],
						2,
						Srgb::BLACK,
					),
				);
			}
			let mut wish = Shape::text(
				"Happy Birthday",
				"Arial",
				48,
				190,
				330,
				420,
				70,
				Srgb::BLACK,
			);
			wish.caption = Some("the words Happy Birthday".to_string());
			d.add_shape(0, wish);
			d.add_shape(
				0,
				shape(ShapeKind::Rectangle, 330, 460, 140, 12, (250, 180, 40)),
			);
		}
		"Simple house" => {
			d.add_shape(
				0,
				shape(ShapeKind::Ellipse, 620, 50, 100, 100, (255, 205, 0)),
			); // sun
			d.add_shape(
				0,
				shape(ShapeKind::Rectangle, 250, 300, 300, 220, (245, 222, 179)),
			);
			d.add_shape(
				0,
				Shape::stroke(
					vec![(225, 305), (400, 175), (575, 305)],
					10,
					Srgb::new(150, 75, 40),
				),
			);
			d.add_shape(
				0,
				shape(ShapeKind::Rectangle, 372, 415, 56, 105, (120, 70, 30)),
			); // door
			d.add_shape(
				0,
				shape(ShapeKind::Rectangle, 282, 340, 52, 52, (170, 215, 240)),
			);
			d.add_shape(
				0,
				shape(ShapeKind::Rectangle, 466, 340, 52, 52, (170, 215, 240)),
			);
			d.add_shape(
				0,
				shape(ShapeKind::Rectangle, 0, 520, 800, 80, (110, 180, 90)),
			); // grass
		}
		"Face" => {
			d.add_shape(
				0,
				shape(ShapeKind::Ellipse, 250, 60, 300, 420, (255, 220, 185)),
			);
			d.add_shape(0, shape(ShapeKind::Ellipse, 325, 190, 42, 54, (30, 30, 30)));
			d.add_shape(0, shape(ShapeKind::Ellipse, 433, 190, 42, 54, (30, 30, 30)));
			d.add_shape(
				0,
				Shape::stroke(vec![(400, 265), (388, 325), (404, 330)], 4, Srgb::BLACK),
			);
			// A smile: an arc dipping below its center line.
			let smile: Vec<(i32, i32)> = (0..=12)
				.map(|i| {
					let a = (20.0 + i as f64 * 140.0 / 12.0).to_radians();
					(
						400 + (105.0 * a.cos()) as i32,
						315 + (105.0 * a.sin()) as i32,
					)
				})
				.collect();
			d.add_shape(0, Shape::stroke(smile, 5, Srgb::new(180, 60, 60)));
		}
		"Chart" => {
			d.add_shape(
				0,
				Shape::stroke(vec![(100, 70), (100, 520)], 4, Srgb::BLACK),
			);
			d.add_shape(
				0,
				Shape::stroke(vec![(100, 520), (730, 520)], 4, Srgb::BLACK),
			);
			d.add_shape(
				0,
				shape(ShapeKind::Rectangle, 170, 390, 90, 130, (220, 50, 50)),
			);
			d.add_shape(
				0,
				shape(ShapeKind::Rectangle, 330, 300, 90, 220, (60, 160, 75)),
			);
			d.add_shape(
				0,
				shape(ShapeKind::Rectangle, 490, 210, 90, 310, (50, 90, 220)),
			);
			let mut title = Shape::text("Chart", "Arial", 36, 350, 15, 120, 45, Srgb::BLACK);
			title.caption = Some("the word Chart as a title".to_string());
			d.add_shape(0, title);
		}
		"Portrait" => {
			// A portrait in depth: a backdrop layer tagged
			// background, a framed picture in the midground, and
			// the sitter in the foreground.
			d.rename_layer(0, "Backdrop");
			d.set_layer_depth(0, DepthRole::Background);
			d.add_shape(
				0,
				shape(ShapeKind::Rectangle, 0, 0, 800, 600, (216, 205, 190)),
			); // wall
			d.add_shape(
				0,
				shape(ShapeKind::Rectangle, 0, 470, 800, 130, (150, 120, 95)),
			); // floor
			let mid = d.add_layer("Picture");
			d.set_layer_depth(mid, DepthRole::Midground);
			d.add_shape(
				mid,
				shape(ShapeKind::Rectangle, 540, 70, 180, 140, (120, 90, 60)),
			); // frame
			d.add_shape(
				mid,
				shape(ShapeKind::Rectangle, 555, 85, 150, 110, (170, 200, 225)),
			); // its sky
			let front = d.add_layer("Sitter");
			d.set_layer_depth(front, DepthRole::Foreground);
			d.add_shape(
				front,
				shape(ShapeKind::Ellipse, 290, 90, 220, 280, (255, 220, 185)),
			); // head
			d.add_shape(
				front,
				Shape::stroke(
					vec![(305, 150), (400, 95), (495, 150)],
					12,
					Srgb::new(90, 60, 35),
				),
			); // hair
			d.add_shape(
				front,
				shape(ShapeKind::Ellipse, 345, 205, 30, 40, (30, 30, 30)),
			); // eye
			d.add_shape(
				front,
				shape(ShapeKind::Ellipse, 425, 205, 30, 40, (30, 30, 30)),
			); // eye
			let mut shoulders = shape(ShapeKind::Rectangle, 230, 385, 340, 215, (70, 100, 160));
			shoulders.corner = 40; // soft shoulders
			d.add_shape(front, shoulders);
		}
		"Landscape" => {
			// A landscape in depth: sky and far hills as the
			// background, near ground and a tree in the foreground.
			d.rename_layer(0, "Distance");
			d.set_layer_depth(0, DepthRole::Background);
			d.add_shape(
				0,
				shape(ShapeKind::Rectangle, 0, 0, 800, 400, (170, 205, 235)),
			); // sky
			d.add_shape(0, shape(ShapeKind::Ellipse, 640, 45, 95, 95, (255, 205, 0))); // sun
			d.add_shape(
				0,
				Shape::stroke(
					vec![(0, 400), (200, 300), (420, 385), (620, 315), (800, 395)],
					8,
					Srgb::new(120, 150, 110),
				),
			); // far hills
			let front = d.add_layer("Near ground");
			d.set_layer_depth(front, DepthRole::Foreground);
			d.add_shape(
				front,
				shape(ShapeKind::Rectangle, 0, 400, 800, 200, (105, 175, 85)),
			); // grass
			d.add_shape(
				front,
				shape(ShapeKind::Rectangle, 118, 330, 24, 130, (120, 80, 45)),
			); // trunk
			d.add_shape(
				front,
				shape(ShapeKind::Ellipse, 55, 190, 150, 160, (60, 140, 60)),
			); // canopy
			d.add_shape(
				front,
				Shape::stroke(vec![(430, 470), (760, 470)], 6, Srgb::new(140, 105, 70)),
			); // fence rail
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
			assert!(
				shape_count(&d) >= 4,
				"{name} is too empty: {}",
				shape_count(&d)
			);
			// A template is an ordinary design: it loads through the
			// normal project path and comes back unchanged.
			let text = Project {
				drawing: d.clone(),
				palette: Vec::new(),
				pinned_sliders: Vec::new(),
				slider_target: crate::sliders::AdjustTarget::BaseColor,
				page_jump_percent: crate::project::DEFAULT_PAGE_JUMP_PERCENT,
				goal: None,
				grayscale_start: false,
				interaction_mode: crate::project::InteractionMode::Type,
			}
			.to_text();
			let back = Project::from_text(&text).unwrap().drawing;
			assert_eq!(
				back.summary(),
				d.summary(),
				"{name} changed in the project round-trip"
			);
			assert_eq!(back.flatten().caption_sheet(), d.flatten().caption_sheet());
		}
		assert!(template("Castle").is_none());
	}

	#[test]
	fn depth_templates_tag_their_layers_by_role() {
		let portrait = template("Portrait").unwrap();
		assert_eq!(portrait.layers.len(), 3);
		assert_eq!(portrait.layers[0].depth, DepthRole::Background);
		assert_eq!(portrait.layers[1].depth, DepthRole::Midground);
		assert_eq!(portrait.layers[2].depth, DepthRole::Foreground);
		let landscape = template("Landscape").unwrap();
		assert_eq!(landscape.layers[0].depth, DepthRole::Background);
		assert_eq!(landscape.layers[1].depth, DepthRole::Foreground);
		// The older templates make no depth claim.
		assert_eq!(
			template("Face").unwrap().layers[0].depth,
			DepthRole::Unassigned
		);
	}

	#[test]
	fn every_pattern_builds_from_its_two_colors_and_round_trips() {
		let (a, b) = (Srgb::new(220, 50, 50), Srgb::new(253, 243, 216));
		for name in PATTERN_NAMES {
			let d = pattern_template(name, a, b).unwrap();
			// The ground is the second color; every mark is the
			// first; nothing else appears.
			assert_eq!(d.background, b, "{name}");
			assert_eq!(d.layers.len(), 1, "{name}");
			assert_eq!(d.layers[0].name, "Pattern", "{name}");
			assert_eq!(d.layers[0].depth, DepthRole::Background, "{name}");
			assert!(!d.layers[0].shapes.is_empty(), "{name} paints nothing");
			for s in &d.layers[0].shapes {
				assert_eq!(s.fill, a, "{name} used a third color");
				assert!(
					s.x >= 0 && s.y >= 0 && s.x + s.w as i32 <= 800 && s.y + s.h as i32 <= 600,
					"{name}: a mark escapes the canvas"
				);
			}
			// The render proves the weave: checkerboard and
			// stripes open with the first color at the top-left;
			// dots open with ground there and a dot centred at
			// (50, 50).
			let img = d.flatten().render();
			let first_px = if *name == "Dots" { (50, 50) } else { (10, 10) };
			assert_eq!(
				img.get_pixel(first_px.0, first_px.1),
				&image::Rgb([220, 50, 50]),
				"{name}: first color where the pattern starts"
			);
			// An ordinary design: it saves and loads unchanged.
			let text = Project {
				drawing: d.clone(),
				palette: Vec::new(),
				pinned_sliders: Vec::new(),
				slider_target: crate::sliders::AdjustTarget::BaseColor,
				page_jump_percent: crate::project::DEFAULT_PAGE_JUMP_PERCENT,
				goal: None,
				grayscale_start: false,
				interaction_mode: crate::project::InteractionMode::Type,
			}
			.to_text();
			let back = Project::from_text(&text).unwrap().drawing;
			assert_eq!(
				back.summary(),
				d.summary(),
				"{name} changed in the round-trip"
			);
		}
		assert!(pattern_template("Zigzag", a, b).is_none());
	}

	#[test]
	fn the_checkerboard_alternates_and_the_stripes_run_straight() {
		let (a, b) = (Srgb::BLACK, Srgb::WHITE);
		let checks = pattern_template("Checkerboard", a, b).unwrap();
		assert_eq!(
			checks.layers[0].shapes.len(),
			24,
			"half of the 8 by 6 cells"
		);
		let img = checks.flatten().render();
		assert_eq!(
			img.get_pixel(150, 10),
			&image::Rgb([255, 255, 255]),
			"the next cell is ground"
		);
		assert_eq!(
			img.get_pixel(150, 150),
			&image::Rgb([0, 0, 0]),
			"diagonal cells match"
		);
		let stripes = pattern_template("Stripes", a, b).unwrap();
		assert_eq!(stripes.layers[0].shapes.len(), 4);
		let img = stripes.flatten().render();
		assert_eq!(
			img.get_pixel(50, 590),
			&image::Rgb([0, 0, 0]),
			"a stripe runs the full height"
		);
		assert_eq!(
			img.get_pixel(150, 300),
			&image::Rgb([255, 255, 255]),
			"ground between stripes"
		);
		let dots = pattern_template("Dots", a, b).unwrap();
		assert_eq!(dots.layers[0].shapes.len(), 48);
		let img = dots.flatten().render();
		assert_eq!(
			img.get_pixel(50, 50),
			&image::Rgb([0, 0, 0]),
			"a dot's centre"
		);
		assert_eq!(
			img.get_pixel(10, 10),
			&image::Rgb([255, 255, 255]),
			"ground at the grid corner"
		);
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

use patois::t;

/// Extraction anchor: the template and pattern names are data
/// tables matched by their English names, so every name is listed
/// here once as a literal. A test keeps this list in step.
#[doc(hidden)]
pub fn template_name_anchors() -> Vec<String> {
	vec![
		t("Birthday card"),
		t("Simple house"),
		t("Face"),
		t("Chart"),
		t("Portrait"),
		t("Landscape"),
		t("Checkerboard"),
		t("Stripes"),
		t("Dots"),
	]
}

#[cfg(test)]
mod anchor_tests {
	use super::*;

	#[test]
	fn template_name_anchors_match_the_tables() {
		let anchors = template_name_anchors();
		for name in TEMPLATE_NAMES.iter().chain(PATTERN_NAMES.iter()) {
			assert!(anchors.iter().any(|a| a == name), "missing anchor: {name}");
		}
		assert_eq!(anchors.len(), TEMPLATE_NAMES.len() + PATTERN_NAMES.len());
	}
}
