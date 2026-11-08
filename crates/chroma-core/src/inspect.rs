//! inspect: the inspector's reports — one element, layer or
//! palette color, explained in plain words.
//!
//! Enter on a list in the app asks three questions about the
//! thing under the cursor: what is it, where is it used, and
//! what touches it — including the honest answer to "what
//! happens if I leave it alone", which is that nothing changes
//! it on its own. The text is generated here, from the drawing
//! itself, so it can be unit-tested and can never describe a
//! design that is not the one on screen.
//!
//! Elements are addressed by their flat index — the numbering
//! of the elements list: layer 1's shapes first, then layer
//! 2's, in paint order.

use patois::t;

use crate::design::ShapeKind;
use crate::drawing::{DepthRole, Drawing};
use crate::palette::PaletteColor;
use crate::{describe, Srgb};

/// Flat element index -> (layer index, shape index within it).
fn locate(drawing: &Drawing, mut flat: usize) -> Option<(usize, usize)> {
	for (li, layer) in drawing.layers.iter().enumerate() {
		if flat < layer.shapes.len() {
			return Some((li, flat));
		}
		flat -= layer.shapes.len();
	}
	None
}

/// Is the frame's copy of this element the same element? Frames
/// are snapshots, so identity is positional: the frame's layer
/// at the same place holds a shape at the same place with the
/// same kind, box, color, angle and words.
fn frame_holds(
	frame: &crate::animation::Frame,
	li: usize,
	si: usize,
	shape: &crate::design::Shape,
) -> bool {
	let Some(layer) = frame.layers.get(li) else {
		return false;
	};
	let Some(copy) = layer.shapes.get(si) else {
		return false;
	};
	copy.kind == shape.kind
		&& copy.x == shape.x
		&& copy.y == shape.y
		&& copy.w == shape.w
		&& copy.h == shape.h
		&& copy.fill == shape.fill
		&& copy.angle == shape.angle
		&& copy.text == shape.text
		&& copy.caption == shape.caption
}

/// The full inspector report for one element, or `None` when
/// the flat index names no element.
pub fn inspect_shape(drawing: &Drawing, flat: usize) -> Option<String> {
	let (li, si) = locate(drawing, flat)?;
	let layer = &drawing.layers[li];
	let shape = &layer.shapes[si];
	let mut out = String::new();
	// What it is.
	out.push_str(&format!("{}\n", drawing.plain_of(shape)));
	if let Some(caption) = &shape.caption {
		out.push_str(&t("Its caption: \"%s\".\n").replacen("%s", &(caption).to_string(), 1));
	}
	out.push_str(
		&t("It is element %s of %s on layer \"%s\", %s by %s pixels at %s, %s.\n")
			.replacen("%s", &(si + 1).to_string(), 1)
			.replacen("%s", &(layer.shapes.len()).to_string(), 1)
			.replacen("%s", &(layer.name).to_string(), 1)
			.replacen("%s", &(shape.w).to_string(), 1)
			.replacen("%s", &(shape.h).to_string(), 1)
			.replacen("%s", &(shape.x).to_string(), 1)
			.replacen("%s", &(shape.y).to_string(), 1),
	);
	if shape.angle != 0.0 {
		out.push_str(&t("It is turned %s degrees.\n").replacen(
			"%s",
			&(shape.angle).to_string(),
			1,
		));
	}
	match shape.kind {
		ShapeKind::Rectangle if shape.corner > 0 => {
			out.push_str(
				&t("Its corners are %s percent rounded (0 is sharp, 50 is fully rounded).\n")
					.replacen("%s", &(shape.corner).to_string(), 1),
			);
		}
		ShapeKind::Stroke => {
			out.push_str(
				&t("It is drawn with a %s pixel brush%s.")
					.replacen("%s", &(shape.brush).to_string(), 1)
					.replacen(
						"%s",
						&(if shape.pattern.is_empty() || shape.pattern == "solid" {
							String::new()
						} else {
							t(", %s pattern").replacen("%s", &shape.pattern, 1)
						})
						.to_string(),
						1,
					),
			);
			out.push('\n');
		}
		ShapeKind::Text => {
			out.push_str(
				&t("The words are set in %s at %s points.\n")
					.replacen(
						"%s",
						if shape.font.is_empty() {
							"the built-in font"
						} else {
							&shape.font
						},
						1,
					)
					.replacen("%s", &(shape.font_size).to_string(), 1),
			);
		}
		ShapeKind::Image => {
			out.push_str(&t("The picture comes from the file %s.\n").replacen(
				"%s",
				&(shape.image_path).to_string(),
				1,
			));
		}
		_ => {}
	}
	if shape.opacity < 100 {
		out.push_str(
			&t("It paints %s percent opaque, so what is beneath it shows through.\n").replacen(
				"%s",
				&(shape.opacity).to_string(),
				1,
			),
		);
	}
	// Where it is used.
	out.push_str(
		&t("Layer \"%s\" is layer %s of %s%s.\n")
			.replacen("%s", &(layer.name).to_string(), 1)
			.replacen("%s", &(li + 1).to_string(), 1)
			.replacen("%s", &(drawing.layers.len()).to_string(), 1)
			.replacen(
				"%s",
				&(if layer.depth == DepthRole::Unassigned {
					String::new()
				} else {
					t(", the %s of the design").replacen("%s", &layer.depth.name(), 1)
				})
				.to_string(),
				1,
			),
	);
	if layer.visible {
		out.push_str(&t(
			"The layer is shown, so every export includes this element.\n",
		));
	} else {
		out.push_str(&t("The layer is hidden, so the exports leave this element out until the layer is shown.\n"));
	}
	if drawing.frames.is_empty() {
		out.push_str(&t(
			"The design is a still picture; it has no animation frames.\n",
		));
	} else {
		let held = drawing
			.frames
			.iter()
			.filter(|f| frame_holds(f, li, si, shape))
			.count();
		out.push_str(&t("It appears in %s of the animation's %s %s (frames are snapshots: a frame counts when its copy of this layer holds the same element in the same place).\n").replacen("%s", &(held).to_string(), 1).replacen("%s", &(drawing.frames.len()).to_string(), 1).replacen("%s", &(plural("frame", drawing.frames.len())).to_string(), 1));
	}
	// What touches it.
	if shape.locked {
		out.push_str(&t("It is LOCKED: the layer-wide and depth-wide color adjustments skip it, and bulk changes count it as stayed-the-same. Direct edits to it alone — move, resize, recolor, rename — still work.\n"));
	} else {
		out.push_str(&t("It is dynamic (not locked): a color adjustment aimed at layer \"%s\" changes its color along with the layer's other unlocked elements").replacen("%s", &(layer.name).to_string(), 1));
		if layer.depth != DepthRole::Unassigned {
			out.push_str(
				&t(", and an adjustment aimed at the %s depth reaches it the same way").replacen(
					"%s",
					&(layer.depth.name()).to_string(),
					1,
				),
			);
		}
		out.push_str(".\n");
	}
	out.push_str(&t("Left alone, it stays exactly as it is: nothing in Blind Designer changes an element by itself.\n"));
	Some(out.trim_end().to_string())
}

/// The full inspector report for one layer, or `None` when the
/// index names no layer.
pub fn inspect_layer(drawing: &Drawing, index: usize) -> Option<String> {
	let layer = drawing.layers.get(index)?;
	let mut out = String::new();
	out.push_str(
		&t("Layer %s of %s: \"%s\", %s element%s.\n")
			.replacen("%s", &(index + 1).to_string(), 1)
			.replacen("%s", &(drawing.layers.len()).to_string(), 1)
			.replacen("%s", &(layer.name).to_string(), 1)
			.replacen("%s", &(layer.shapes.len()).to_string(), 1)
			.replacen(
				"%s",
				if layer.shapes.len() == 1 { "" } else { "s" },
				1,
			),
	);
	// Its contents, by kind.
	let mut kinds: Vec<(ShapeKind, usize)> = Vec::new();
	for s in &layer.shapes {
		match kinds.iter_mut().find(|(k, _)| *k == s.kind) {
			Some((_, n)) => *n += 1,
			None => kinds.push((s.kind, 1)),
		}
	}
	if !kinds.is_empty() {
		let parts: Vec<String> = kinds
			.iter()
			.map(|(k, n)| format!("{n} {}", plural(&k.name(), *n)))
			.collect();
		out.push_str(&t("It holds %s.\n").replacen("%s", &(parts.join(", ")).to_string(), 1));
	}
	// Where it sits: paint order is the layer order.
	match (index, drawing.layers.len()) {
		(0, 1) => out.push_str(&t("It is the only layer; everything paints on it.\n")),
		(0, _) => out.push_str(&t("It paints first: everything else stands on it.\n")),
		(i, n) if i == n - 1 => out.push_str(&t("It paints last: nothing covers it.\n")),
		_ => out.push_str(
			&t("It paints over layer \"%s\" and under layer \"%s\".\n")
				.replacen("%s", &(drawing.layers[index - 1].name).to_string(), 1)
				.replacen("%s", &(drawing.layers[index + 1].name).to_string(), 1),
		),
	}
	if layer.depth == DepthRole::Unassigned {
		out.push_str(&t(
			"It has no depth role, so the depth-wide adjustments do not reach it.\n",
		));
	} else {
		out.push_str(&t("It is the %s of the design: an adjustment aimed at the %s depth changes this layer's unlocked elements along with every other %s layer's.\n").replacen("%s", &(layer.depth.name()).to_string(), 1).replacen("%s", &(layer.depth.name()).to_string(), 1).replacen("%s", &(layer.depth.name()).to_string(), 1));
	}
	let locked = layer.shapes.iter().filter(|s| s.locked).count();
	if locked > 0 {
		out.push_str(&t("%s of its elements are locked: bulk color changes skip them and count them as stayed-the-same.\n").replacen("%s", &(locked).to_string(), 1));
	}
	if layer.visible {
		out.push_str(&t("It is shown, so its elements are in every export.\n"));
	} else {
		out.push_str(&t(
			"It is hidden, so its elements are left out of every export until it is shown.\n",
		));
	}
	out.push_str(&t("Left alone, it stays exactly as it is: hiding, renaming, a depth role or a bulk color change are the things that can touch a whole layer, and each one is a step you take.\n"));
	Some(out.trim_end().to_string())
}

fn plural(kind: &str, n: usize) -> String {
	if n == 1 {
		kind.to_string()
	} else {
		format!("{kind}s")
	}
}

/// The full inspector report for one palette color, or `None`
/// when the index names no palette color.
pub fn inspect_color(drawing: &Drawing, palette: &[PaletteColor], index: usize) -> Option<String> {
	let pc = palette.get(index)?;
	let color: Srgb = pc.color;
	let mut out = String::new();
	out.push_str(&format!("{} ({}).\n", describe(color), color.to_hex()));
	out.push_str(&t("It came from %s.\n").replacen("%s", &(pc.provenance.spoken()).to_string(), 1));
	if let Some(cov) = pc.coverage {
		out.push_str(
			&t("It covered %s percent of the image it was extracted from.\n").replacen(
				"%s",
				&format!("{:.0}", cov * 100.0),
				1,
			),
		);
	}
	// Which elements use it, by their own fill: elements keep
	// their color, the palette is a working set.
	let mut uses: Vec<String> = Vec::new();
	let mut total = 0;
	for layer in &drawing.layers {
		for shape in &layer.shapes {
			if shape.fill == color {
				total += 1;
				if uses.len() < 5 {
					uses.push(format!(
						"\"{}\": {}",
						layer.name,
						drawing.description_of(shape)
					));
				}
			}
		}
	}
	let is_background = drawing.background == color;
	if total == 0 && !is_background {
		out.push_str(&t(
			"No element uses it yet, and it is not the background.\n",
		));
	} else {
		if total > 0 {
			out.push_str(
				&t("%s element%s use%s it:\n")
					.replacen("%s", &(total).to_string(), 1)
					.replacen("%s", if total == 1 { "" } else { "s" }, 1)
					.replacen("%s", if total == 1 { "s" } else { "" }, 1),
			);
			for u in &uses {
				out.push_str(&format!("  {u}\n"));
			}
			if total > uses.len() {
				out.push_str(&t("  and %s more.\n").replacen(
					"%s",
					&(total - uses.len()).to_string(),
					1,
				));
			}
		}
		if is_background {
			out.push_str(&t("It is the design's background color.\n"));
		}
	}
	out.push_str(&t("Left alone, it stays in the palette. Removing a palette color never recolors an element: elements keep their own colors, and the palette is a working set you pick from.\n"));
	Some(out.trim_end().to_string())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::design::{Shape, ShapeKind};
	use crate::palette::Provenance;

	fn red() -> Srgb {
		Srgb::new(200, 40, 30)
	}

	/// A drawing: layer "Back" (background depth) with a locked
	/// red rectangle; layer "Front" with an ellipse in the
	/// palette's blue and a caption; one animation frame captured
	/// before the caption existed... frames snapshot whole
	/// layers, so the test captures after building.
	fn project() -> (Drawing, Vec<PaletteColor>) {
		let mut d = Drawing::new(800, 600, Srgb::WHITE);
		d.rename_layer(0, "Back");
		d.set_layer_depth(0, DepthRole::Background);
		d.add_shape(0, Shape::new(ShapeKind::Rectangle, 10, 10, 100, 50, red()));
		d.set_shape_locked(0, 0, true);
		let front = d.add_layer("Front");
		let mut cap = Shape::new(
			ShapeKind::Ellipse,
			200,
			100,
			40,
			40,
			Srgb::new(58, 124, 165),
		);
		cap.opacity = 60;
		d.add_shape(front, cap);
		d.caption_shape(front, 0, "the moon".to_string());
		d.add_frame();
		let palette = vec![
			PaletteColor::typed(Srgb::new(58, 124, 165)),
			PaletteColor::new(red(), Provenance::Starter),
			PaletteColor::typed(Srgb::new(1, 2, 3)),
		];
		(d, palette)
	}

	#[test]
	fn shape_report_says_what_where_and_what_touches_it() {
		let (d, _) = project();
		// The locked rectangle is flat element 0.
		let text = inspect_shape(&d, 0).unwrap();
		assert!(text.contains("rectangle"), "{text}");
		assert!(text.contains("element 1 of 1 on layer \"Back\""), "{text}");
		assert!(text.contains("the background of the design"), "{text}");
		assert!(
			text.contains("every export includes this element"),
			"{text}"
		);
		assert!(
			text.contains("appears in 1 of the animation's 1 frame"),
			"{text}"
		);
		assert!(text.contains("LOCKED"), "{text}");
		assert!(text.contains("skip it"), "{text}");
		assert!(
			text.contains("Left alone, it stays exactly as it is"),
			"{text}"
		);
		// The ellipse is flat element 1: dynamic, translucent,
		// captioned, on an unassigned layer.
		let text = inspect_shape(&d, 1).unwrap();
		assert!(text.contains("Its caption: \"the moon\""), "{text}");
		assert!(text.contains("60 percent opaque"), "{text}");
		assert!(text.contains("dynamic (not locked)"), "{text}");
		assert!(text.contains("layer \"Front\""), "{text}");
		assert!(inspect_shape(&d, 2).is_none());
	}

	#[test]
	fn layer_report_covers_contents_depth_and_locks() {
		let (d, _) = project();
		let text = inspect_layer(&d, 0).unwrap();
		assert!(text.contains("Layer 1 of 2: \"Back\", 1 element"), "{text}");
		assert!(text.contains("It holds 1 rectangle"), "{text}");
		assert!(text.contains("paints first"), "{text}");
		assert!(
			text.contains("adjustment aimed at the background depth"),
			"{text}"
		);
		assert!(text.contains("1 of its elements are locked"), "{text}");
		let text = inspect_layer(&d, 1).unwrap();
		assert!(text.contains("paints last"), "{text}");
		assert!(text.contains("no depth role"), "{text}");
		assert!(inspect_layer(&d, 2).is_none());
	}

	#[test]
	fn color_report_lists_its_users_and_the_unused() {
		let (d, palette) = project();
		// The blue: used by the ellipse.
		let text = inspect_color(&d, &palette, 0).unwrap();
		assert!(text.contains("typed in by hand"), "{text}");
		assert!(text.contains("1 element uses it"), "{text}");
		assert!(text.contains("\"Front\""), "{text}");
		// The red: used by the rectangle, from the starter set.
		let text = inspect_color(&d, &palette, 1).unwrap();
		assert!(text.contains("1 element uses it"), "{text}");
		// The near-black: used by nothing.
		let text = inspect_color(&d, &palette, 2).unwrap();
		assert!(text.contains("No element uses it yet"), "{text}");
		assert!(text.contains("never recolors an element"), "{text}");
		assert!(inspect_color(&d, &palette, 3).is_none());
		// The background is a use too.
		let mut d2 = Drawing::new(10, 10, red());
		d2.add_shape(0, Shape::new(ShapeKind::Rectangle, 0, 0, 5, 5, Srgb::BLACK));
		let text = inspect_color(&d2, &palette, 1).unwrap();
		assert!(
			text.contains("It is the design's background color"),
			"{text}"
		);
	}
}
