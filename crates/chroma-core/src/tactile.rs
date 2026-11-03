//! tactile: the design (and any picture) as something to feel.
//!
//! A tactile reading replaces the picture with raised lines and
//! words: an SVG of stroked outlines (a different dash pattern per
//! layer, so layers can be told apart by touch), a text sheet that
//! walks the design element by element in reading order, braille
//! labels for the elements, and area descriptions that name only
//! what falls inside one part of the canvas. A picture someone
//! hands the app gets the same treatment as a grid of shade cells.

use crate::braille::to_braille;
use crate::design::{Design, Shape, ShapeKind};
use crate::drawing::Drawing;
use crate::{color_words, extract_palette, Srgb};

/// Line styles for the raised-line SVG, one per layer, in order.
const LINE_STYLES: &[(&str, &str)] = &[
	("solid", ""),
	("long dashes", "12 6"),
	("short dashes", "5 5"),
	("dots", "1 5"),
	("dash and dot", "12 5 2 5"),
	("long and short dashes", "16 5 5 5"),
];

fn line_style(layer: usize) -> (&'static str, &'static str) {
	LINE_STYLES[layer % LINE_STYLES.len()]
}

fn esc(s: &str) -> String {
	let mut out = String::with_capacity(s.len());
	for c in s.chars() {
		match c {
			'&' => out.push_str("&amp;"),
			'<' => out.push_str("&lt;"),
			'>' => out.push_str("&gt;"),
			'"' => out.push_str("&quot;"),
			_ => out.push(c),
		}
	}
	out
}

/// The box a shape occupies for area tests: strokes use the span of
/// their points (at least the brush wide), everything else its own
/// x, y, w, h.
fn shape_box(shape: &Shape) -> (i32, i32, i32, i32) {
	let w = shape.w.max(shape.brush.max(1)) as i32;
	let h = shape.h.max(shape.brush.max(1)) as i32;
	(shape.x, shape.y, shape.x + w, shape.y + h)
}

/// Elements in reading order: top to bottom, then left to right.
fn reading_order<'a>(shapes: &[&'a Shape]) -> Vec<&'a Shape> {
	let mut sorted: Vec<&Shape> = shapes.to_vec();
	sorted.sort_by_key(|s| (s.y, s.x));
	sorted
}

impl Drawing {
	/// The raised-line SVG: every visible shape as a black stroked
	/// outline on white, unfilled, with a dash pattern per layer so
	/// layers can be told apart by touch. Text elements are drawn as
	/// their braille translation inside an outlined box (a swell-paper
	/// printer raises the braille dots, not the letterforms).
	pub fn to_tactile_svg(&self) -> String {
		let mut out = format!(
			"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" role=\"img\" aria-label=\"Raised-line drawing, {} by {} pixels\">\n",
			self.width,
			self.height,
			self.width,
			self.height,
			self.width,
			self.height
		);
		out.push_str(&format!(
			"<title>Raised-line drawing, {} by {} pixels, background {}</title>\n",
			self.width,
			self.height,
			esc(&color_words(self.background))
		));
		out.push_str(&format!(
			"<rect x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"white\" stroke=\"black\" stroke-width=\"2\"/>\n",
			self.width, self.height
		));
		for (li, layer) in self.layers.iter().enumerate() {
			if !layer.visible {
				continue;
			}
			let (_, dash) = line_style(li);
			let dash_attr = if dash.is_empty() {
				String::new()
			} else {
				format!(" stroke-dasharray=\"{dash}\"")
			};
			out.push_str(&format!("<g data-layer=\"{}\" fill=\"none\" stroke=\"black\"{dash_attr}>\n", esc(&layer.name)));
			for shape in &layer.shapes {
				let caption = esc(&self.description_of(shape));
				match shape.kind {
					ShapeKind::Rectangle => out.push_str(&format!(
						"<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" stroke-width=\"2\"><title>{caption}</title></rect>\n",
						shape.x, shape.y, shape.w, shape.h
					)),
					ShapeKind::Ellipse => out.push_str(&format!(
						"<ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" stroke-width=\"2\"><title>{caption}</title></ellipse>\n",
						shape.x as f64 + shape.w as f64 / 2.0,
						shape.y as f64 + shape.h as f64 / 2.0,
						shape.w as f64 / 2.0,
						shape.h as f64 / 2.0
					)),
					ShapeKind::Stroke => {
						if shape.points.len() == 1 {
							let (px, py) = shape.points[0];
							out.push_str(&format!(
								"<circle cx=\"{px}\" cy=\"{py}\" r=\"{}\" stroke-width=\"2\"><title>{caption}</title></circle>\n",
								(shape.brush / 2).max(1)
							));
						} else {
							let pts: Vec<String> =
								shape.points.iter().map(|(px, py)| format!("{px},{py}")).collect();
							out.push_str(&format!(
								"<polyline points=\"{}\" stroke-width=\"{}\"><title>{caption}</title></polyline>\n",
								pts.join(" "),
								shape.brush.max(2)
							));
						}
					}
					ShapeKind::Text => {
						out.push_str(&format!(
							"<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" stroke-width=\"2\"><title>{}</title></rect>\n",
							shape.x,
							shape.y,
							shape.w,
							shape.h,
							esc(&shape.text)
						));
						out.push_str(&format!(
							"<text x=\"{}\" y=\"{}\" font-size=\"{}\" fill=\"black\" stroke=\"none\">{}</text>\n",
							shape.x + 4,
							shape.y + (shape.h as f64 * 0.7) as i32,
							(shape.h as f64 * 0.6).max(10.0) as u32,
							esc(&to_braille(&shape.text))
						));
					}
					ShapeKind::Image => {
						out.push_str(&format!(
							"<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" stroke-width=\"2\"><title>{caption}</title></rect>\n",
							shape.x, shape.y, shape.w, shape.h
						));
						out.push_str(&format!(
							"<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke-width=\"1\"/>\n<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke-width=\"1\"/>\n",
							shape.x,
							shape.y,
							shape.x + shape.w as i32,
							shape.y + shape.h as i32,
							shape.x,
							shape.y + shape.h as i32,
							shape.x + shape.w as i32,
							shape.y
						));
					}
				}
			}
			out.push_str("</g>\n");
		}
		out.push_str("</svg>\n");
		out
	}

	/// The tactile text sheet: a walk through the design, layer by
	/// layer, elements in reading order, each with its plain-language
	/// description, its caption, and its braille label. Hidden layers
	/// are noted and skipped, as they are everywhere else.
	pub fn tactile_sheet(&self) -> String {
		let total: usize = self
			.layers
			.iter()
			.filter(|l| l.visible)
			.map(|l| l.shapes.len())
			.sum();
		let mut out = format!(
			"Tactile sheet for a design {} by {} pixels, background {} ({}).\n\
			 {total} elements. The walk goes layer by layer; inside a layer, \
			 top to bottom, left to right — the order a hand reads.\n",
			self.width,
			self.height,
			color_words(self.background),
			self.background.to_hex()
		);
		for (li, layer) in self.layers.iter().enumerate() {
			if !layer.visible {
				out.push_str(&format!(
					"\nLayer \"{}\" is hidden; it is not part of the tactile design.\n",
					layer.name
				));
				continue;
			}
			let (style_name, _) = line_style(li);
			out.push_str(&format!(
				"\nLayer \"{}\" ({} lines in the raised-line version), {} elements:\n",
				layer.name,
				style_name,
				layer.shapes.len()
			));
			let refs: Vec<&Shape> = layer.shapes.iter().collect();
			for (i, shape) in reading_order(&refs).iter().enumerate() {
				let caption = self.description_of(shape);
				out.push_str(&format!(
					"{}. {}. Caption: {}. Braille label: {}\n",
					i + 1,
					self.plain_of(shape),
					caption,
					to_braille(&caption)
				));
			}
		}
		out
	}

	/// Braille labels for the design's elements, one numbered line
	/// per element on a visible layer, ready to save as a .brf-style
	/// plain text file and emboss or read as-is.
	pub fn braille_labels(&self) -> String {
		let mut out = String::from("Braille labels (Grade 1, uncontracted)\n");
		let mut n = 0;
		for layer in &self.layers {
			if !layer.visible {
				continue;
			}
			let refs: Vec<&Shape> = layer.shapes.iter().collect();
			for shape in reading_order(&refs) {
				n += 1;
				out.push_str(&format!("{n}: {}\n", to_braille(&self.description_of(shape))));
			}
		}
		if n == 0 {
			out.push_str("No elements to label.\n");
		}
		out
	}

	/// Describe one part of the canvas: a named area ("top left",
	/// "center", "bottom right", or a single edge like "top") or a
	/// typed box ("100, 100, 300, 200" — across, down, width,
	/// height). Only elements touching the area are named, in
	/// reading order; elements only partly inside are marked.
	/// `None` when the words name no area.
	pub fn describe_area_text(&self, text: &str) -> Option<String> {
		let (label, bx, by, bw, bh) = parse_area(text, self.width, self.height)?;
		let (x1, y1) = (bx + bw as i32, by + bh as i32);
		let mut hits: Vec<(&Shape, bool)> = Vec::new();
		for layer in &self.layers {
			if !layer.visible {
				continue;
			}
			for shape in &layer.shapes {
				let (sx0, sy0, sx1, sy1) = shape_box(shape);
				if sx0 < x1 && bx < sx1 && sy0 < y1 && by < sy1 {
					let inside = sx0 >= bx && sy0 >= by && sx1 <= x1 && sy1 <= y1;
					hits.push((shape, inside));
				}
			}
		}
		let refs: Vec<&Shape> = hits.iter().map(|(s, _)| *s).collect();
		let ordered = reading_order(&refs);
		let mut out = format!(
			"The {label} area ({} to {} across, {} to {} down) holds {} element{}.\n",
			bx,
			x1,
			by,
			y1,
			hits.len(),
			if hits.len() == 1 { "" } else { "s" }
		);
		for (i, shape) in ordered.iter().enumerate() {
			let inside = hits.iter().find(|(s, _)| std::ptr::eq(*s, *shape)).map(|(_, b)| *b);
			out.push_str(&format!("{}. {}", i + 1, self.plain_of(shape)));
			if inside == Some(false) {
				out.push_str(" (partly in this area)");
			}
			out.push('\n');
		}
		Some(out)
	}
}

/// Parse an area: four numbers are a box; otherwise the words name
/// one cell of the 3 by 3 grid the plain descriptions use.
fn parse_area(text: &str, canvas_w: u32, canvas_h: u32) -> Option<(String, i32, i32, u32, u32)> {
	let t = text.trim().to_lowercase();
	if t.is_empty() {
		return None;
	}
	let nums: Vec<&str> =
		t.split(|c: char| c == ',' || c.is_whitespace()).filter(|s| !s.is_empty()).collect();
	if nums.len() == 4 {
		let v: Vec<i32> = nums.iter().filter_map(|n| n.parse::<i32>().ok()).collect();
		if v.len() == 4 && v[2] >= 1 && v[3] >= 1 {
			return Some((
				format!("box at {}, {}", v[0], v[1]),
				v[0],
				v[1],
				v[2] as u32,
				v[3] as u32,
			));
		}
		return None;
	}
	let has = |words: &[&str]| t.split_whitespace().any(|w| words.contains(&w));
	if !has(&["left", "right", "top", "bottom", "center", "centre", "middle"]) {
		return None;
	}
	let col = if has(&["left"]) {
		0
	} else if has(&["right"]) {
		2
	} else {
		1
	};
	let row = if has(&["top"]) {
		0
	} else if has(&["bottom"]) {
		2
	} else {
		1
	};
	let label = match (row, col) {
		(1, 1) => "center".to_string(),
		(r, c) => format!(
			"{} {}",
			["top", "middle", "bottom"][r],
			["left", "center", "right"][c]
		),
	};
	let (w, h) = (canvas_w as i32, canvas_h as i32);
	let (col, row) = (col as i32, row as i32);
	let x = col * w / 3;
	let x1 = if col == 2 { w } else { (col + 1) * w / 3 };
	let y = row * h / 3;
	let y1 = if row == 2 { h } else { (row + 1) * h / 3 };
	Some((label, x, y, (x1 - x) as u32, (y1 - y) as u32))
}

fn mean_luminance(img: &image::DynamicImage) -> f32 {
	let small = img.thumbnail(64, 64).to_rgb8();
	let n = (small.width() * small.height()).max(1) as f32;
	small.pixels().map(|p| Srgb::new(p[0], p[1], p[2]).luminance()).sum::<f32>() / n
}

/// Sharpness as the root-mean-square neighbor difference: a hard
/// edge counts far more than the same total change spread out as a
/// gentle gradient, which is exactly the sharp/blurry distinction.
fn mean_sharpness(img: &image::DynamicImage) -> f32 {
	let gray = img.thumbnail(96, 96).to_luma8();
	let (w, h) = (gray.width(), gray.height());
	if w < 2 || h < 2 {
		return 0.0;
	}
	let mut sum = 0f32;
	let mut count = 0u32;
	for y in 0..h {
		for x in 0..w - 1 {
			let d = gray.get_pixel(x, y)[0] as f32 - gray.get_pixel(x + 1, y)[0] as f32;
			sum += d * d;
			count += 1;
		}
	}
	for y in 0..h - 1 {
		for x in 0..w {
			let d = gray.get_pixel(x, y)[0] as f32 - gray.get_pixel(x, y + 1)[0] as f32;
			sum += d * d;
			count += 1;
		}
	}
	(sum / count.max(1) as f32).sqrt()
}

fn brightness_word(lum: f32) -> &'static str {
	match lum {
		x if x < 0.03 => "near black",
		x if x < 0.12 => "dark",
		x if x < 0.35 => "medium",
		x if x < 0.65 => "light",
		_ => "near white",
	}
}

fn same_color(a: Srgb, b: Srgb) -> bool {
	let (oa, ob) = (a.to_oklch(), b.to_oklch());
	let to_ab = |o: crate::Oklch| (o.c * o.h.to_radians().cos(), o.c * o.h.to_radians().sin());
	let (aa, ab) = to_ab(oa);
	let (ba, bb) = to_ab(ob);
	((oa.l - ob.l).powi(2) + (aa - ba).powi(2) + (ab - bb).powi(2)).sqrt() < 0.05
}

/// The measured facts of a comparison between two images.
#[derive(Debug, Clone)]
pub struct ImageComparison {
	pub first_size: (u32, u32),
	pub second_size: (u32, u32),
	pub shared: Vec<Srgb>,
	pub only_first: Vec<Srgb>,
	pub only_second: Vec<Srgb>,
	pub brightness_first: f32,
	pub brightness_second: f32,
	pub sharpness_first: f32,
	pub sharpness_second: f32,
}

/// Compare two images: sizes, palette overlap and differences,
/// average brightness, and sharpness (root-mean-square neighbor
/// difference — a blurry picture changes gently from pixel to
/// pixel, a sharp one jumps).
pub fn compare_images(a: &image::DynamicImage, b: &image::DynamicImage) -> ImageComparison {
	let pal_a: Vec<Srgb> = extract_palette(a, 8).iter().map(|e| e.color).collect();
	let pal_b: Vec<Srgb> = extract_palette(b, 8).iter().map(|e| e.color).collect();
	let mut shared = Vec::new();
	let mut only_first = Vec::new();
	for c in &pal_a {
		if pal_b.iter().any(|o| same_color(*c, *o)) {
			if !shared.iter().any(|s| same_color(*s, *c)) {
				shared.push(*c);
			}
		} else {
			only_first.push(*c);
		}
	}
	let only_second: Vec<Srgb> = pal_b
		.iter()
		.filter(|c| !pal_a.iter().any(|o| same_color(**c, *o)))
		.copied()
		.collect();
	ImageComparison {
		first_size: (a.width(), a.height()),
		second_size: (b.width(), b.height()),
		shared,
		only_first,
		only_second,
		brightness_first: mean_luminance(a),
		brightness_second: mean_luminance(b),
		sharpness_first: mean_sharpness(a),
		sharpness_second: mean_sharpness(b),
	}
}

impl ImageComparison {
	/// The comparison as a spoken-style report. `first` and `second`
	/// name the pictures ("the first image", or a file name).
	pub fn report(&self, first: &str, second: &str) -> String {
		let mut out = format!("Comparing {first} with {second}.\n");
		if self.first_size == self.second_size {
			out.push_str(&format!(
				"Same size: {} by {} pixels.\n",
				self.first_size.0, self.first_size.1
			));
		} else {
			out.push_str(&format!(
				"Different sizes: {first} is {} by {} pixels; {second} is {} by {} pixels.\n",
				self.first_size.0, self.first_size.1, self.second_size.0, self.second_size.1
			));
		}
		let words = |list: &[Srgb]| -> String {
			list.iter().map(|c| color_words(*c)).collect::<Vec<_>>().join(", ")
		};
		if self.shared.is_empty() {
			out.push_str("They share no palette colors.\n");
		} else {
			out.push_str(&format!(
				"They share {} palette color{}: {}.\n",
				self.shared.len(),
				if self.shared.len() == 1 { "" } else { "s" },
				words(&self.shared)
			));
		}
		if !self.only_first.is_empty() {
			out.push_str(&format!("Only in {first}: {}.\n", words(&self.only_first)));
		}
		if !self.only_second.is_empty() {
			out.push_str(&format!("Only in {second}: {}.\n", words(&self.only_second)));
		}
		let (ba, bb) = (self.brightness_first, self.brightness_second);
		if (ba - bb).abs() < 0.02 {
			out.push_str(&format!(
				"About the same brightness overall (both {}).\n",
				brightness_word((ba + bb) / 2.0)
			));
		} else if ba > bb {
			out.push_str(&format!(
				"{first} is brighter overall ({} against {}).\n",
				brightness_word(ba),
				brightness_word(bb)
			));
		} else {
			out.push_str(&format!(
				"{second} is brighter overall ({} against {}).\n",
				brightness_word(bb),
				brightness_word(ba)
			));
		}
		let (sa, sb) = (self.sharpness_first, self.sharpness_second);
		let ratio = if sa.max(sb) <= 0.0 {
			1.0
		} else {
			sa.min(sb) / sa.max(sb)
		};
		if ratio > 0.85 {
			out.push_str("About equally sharp — neither is noticeably blurrier.\n");
		} else if sa > sb {
			out.push_str(&format!(
				"{first} is sharper and clearer; {second} is blurrier.\n"
			));
		} else {
			out.push_str(&format!(
				"{second} is sharper and clearer; {first} is blurrier.\n"
			));
		}
		out
	}
}

/// Braille shade patterns, least dots (brightest) to most (darkest).
const SHADE_CELLS: [char; 7] = ['\u{2800}', '⠂', '⠅', '⠇', '⠏', '⠟', '⠿'];
const SHADE_WORDS: [&str; 7] = ["bright", "pale", "light", "medium", "dark gray", "dark", "black"];

/// A picture as a coarse grid of shade cells — the screenshot-to-
/// tactile treatment. Each cell holds the average darkness of that
/// part of the picture, 0.0 (white) to 1.0 (black).
#[derive(Debug, Clone)]
pub struct TactileGrid {
	pub cols: u32,
	pub rows: u32,
	/// Darkness per cell, row-major, top row first.
	pub cells: Vec<f32>,
}

pub fn tactile_grid(img: &image::DynamicImage, cols: u32, rows: u32) -> TactileGrid {
	let cols = cols.clamp(1, 80);
	let rows = rows.clamp(1, 60);
	let small = img.resize_exact(cols, rows, image::imageops::FilterType::Triangle).to_luma8();
	let cells = small.pixels().map(|p| 1.0 - p[0] as f32 / 255.0).collect();
	TactileGrid { cols, rows, cells }
}

impl TactileGrid {
	fn level(&self, index: usize) -> usize {
		(self.cells.get(index).copied().unwrap_or(0.0) * 6.0).round().clamp(0.0, 6.0) as usize
	}

	/// The grid as braille shade rows, top row first.
	pub fn braille_rows(&self) -> Vec<String> {
		(0..self.rows as usize)
			.map(|r| {
				(0..self.cols as usize)
					.map(|c| SHADE_CELLS[self.level(r * self.cols as usize + c)])
					.collect()
			})
			.collect()
	}

	/// The full sheet: the shade scale, the braille grid, the same
	/// grid in words (runs of columns per row), and where the darkest
	/// and brightest parts sit.
	pub fn sheet(&self, name: &str) -> String {
		let mut out = format!(
			"Tactile grid of {name}: {} columns by {} rows. Each cell is the average shade of that part of the picture.\n\
			 Shade scale, lightest to darkest: {}\n\n",
			self.cols,
			self.rows,
			SHADE_CELLS.iter().collect::<String>()
		);
		for row in self.braille_rows() {
			out.push_str(&row);
			out.push('\n');
		}
		out.push_str("\nIn words:\n");
		for r in 0..self.rows as usize {
			let mut runs: Vec<String> = Vec::new();
			let mut start = 0usize;
			for c in 1..=self.cols as usize {
				let level = self.level(r * self.cols as usize + c.min(self.cols as usize - 1));
				let prev = self.level(r * self.cols as usize + c - 1);
				if c == self.cols as usize || level != prev {
					let word = SHADE_WORDS[prev];
					runs.push(if start == c - 1 {
						format!("column {} {word}", start + 1)
					} else {
						format!("columns {} to {} {word}", start + 1, c)
					});
					start = c;
				}
			}
			out.push_str(&format!("Row {}: {}.\n", r + 1, runs.join("; ")));
		}
		if !self.cells.is_empty() {
			// First cell wins ties, so the spots read in reading order.
			let (mut dark, mut light) = (0usize, 0usize);
			for (i, v) in self.cells.iter().enumerate() {
				if *v > self.cells[dark] {
					dark = i;
				}
				if *v < self.cells[light] {
					light = i;
				}
			}
			let spot = |i: usize| {
				let (c, r) = (i as u32 % self.cols, i as u32 / self.cols);
				Design::position_word(
					(c as f32 + 0.5) / self.cols as f32,
					(r as f32 + 0.5) / self.rows as f32,
				)
			};
			out.push_str(&format!(
				"Darkest part: {}. Brightest part: {}.\n",
				spot(dark),
				spot(light)
			));
		}
		out
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::design::ShapeKind;
	use crate::drawing::Drawing;

	fn sample() -> Drawing {
		let mut d = Drawing::new(300, 300, Srgb::WHITE);
		// Top-left red square; a caption on it.
		let mut sq = Shape::new(ShapeKind::Rectangle, 0, 0, 60, 60, Srgb::new(255, 0, 0));
		sq.caption = Some("red square".to_string());
		d.add_shape(0, sq);
		// Bottom-right blue circle, partly outside the bottom-right cell.
		d.add_shape(0, Shape::new(ShapeKind::Ellipse, 240, 240, 60, 60, Srgb::new(0, 0, 255)));
		d
	}

	#[test]
	fn tactile_sheet_walks_in_reading_order_with_braille() {
		let sheet = sample().tactile_sheet();
		assert!(sheet.contains("background white"), "{sheet}");
		let red = sheet.find("red square").unwrap();
		let blue = sheet.find("blue").unwrap();
		assert!(red < blue, "reading order: red (top) before blue (bottom):\n{sheet}");
		// The caption's braille label rides along.
		assert!(sheet.contains(&to_braille("red square")), "{sheet}");
	}

	#[test]
	fn tactile_svg_outlines_shapes_with_layer_styles() {
		let mut d = sample();
		let top = d.add_layer("Sky");
		d.add_shape(top, Shape::new(ShapeKind::Rectangle, 10, 10, 20, 20, Srgb::BLACK));
		let svg = d.to_tactile_svg();
		assert!(svg.contains("<rect"), "{svg}");
		assert!(svg.contains("<ellipse"), "{svg}");
		assert!(svg.contains("data-layer=\"Sky\""), "{svg}");
		assert!(svg.contains("stroke-dasharray=\"12 6\""), "second layer dashes:\n{svg}");
		assert!(svg.contains("fill=\"none\""), "{svg}");
		// A text element appears as its braille translation.
		d.add_shape(0, Shape::text("Hi", "Arial", 24, 0, 100, 60, 30, Srgb::BLACK));
		let svg = d.to_tactile_svg();
		assert!(svg.contains(&to_braille("Hi")), "{svg}");
	}

	#[test]
	fn braille_labels_are_numbered_per_element() {
		let labels = sample().braille_labels();
		assert!(labels.contains(&format!("1: {}", to_braille("red square"))), "{labels}");
		assert!(labels.contains("2: "), "{labels}");
	}

	#[test]
	fn describe_area_filters_by_name_and_box() {
		let d = sample();
		let top_left = d.describe_area_text("top left").unwrap();
		assert!(top_left.contains("holds 1 element"), "{top_left}");
		assert!(top_left.contains("rectangle"), "{top_left}");
		let bottom_right = d.describe_area_text("bottom right").unwrap();
		assert!(bottom_right.contains("ellipse"), "{bottom_right}");
		// A box over the middle holds nothing; the square is outside it.
		let mid = d.describe_area_text("120, 120, 60, 60").unwrap();
		assert!(mid.contains("holds 0 elements"), "{mid}");
		// A box clipping the square's corner marks it partial.
		let clip = d.describe_area_text("40, 40, 100, 100").unwrap();
		assert!(clip.contains("(partly in this area)"), "{clip}");
		assert!(d.describe_area_text("over there").is_none());
		assert!(d.describe_area_text("").is_none());
	}

	#[test]
	fn comparison_reports_size_palette_and_sharpness() {
		// First: left half black, right half white (sharp edge).
		let mut raw_a = image::RgbImage::new(64, 64);
		for (x, _y, p) in raw_a.enumerate_pixels_mut() {
			*p = if x < 32 { image::Rgb([0, 0, 0]) } else { image::Rgb([255, 255, 255]) };
		}
		let a = image::DynamicImage::ImageRgb8(raw_a);
		// Second: a smooth gray gradient (blurry by construction), smaller.
		let mut raw_b = image::RgbImage::new(32, 32);
		for (x, _y, p) in raw_b.enumerate_pixels_mut() {
			let v = (x * 255 / 31) as u8;
			*p = image::Rgb([v, v, v]);
		}
		let b = image::DynamicImage::ImageRgb8(raw_b);
		let cmp = compare_images(&a, &b);
		assert_eq!(cmp.first_size, (64, 64));
		assert_eq!(cmp.second_size, (32, 32));
		assert!(cmp.sharpness_first > cmp.sharpness_second * 3.0,
			"edge {} vs gradient {}", cmp.sharpness_first, cmp.sharpness_second);
		let report = cmp.report("the first image", "the second image");
		assert!(report.contains("Different sizes"), "{report}");
		assert!(report.contains("sharper and clearer"), "{report}");
		assert!(report.contains("blurrier"), "{report}");
		// Identical images share their whole palette.
		let same = compare_images(&a, &a);
		assert!(same.only_first.is_empty() && same.only_second.is_empty());
		assert!(!same.shared.is_empty());
		assert!(same.report("one", "two").contains("Same size"), "{}", same.report("one", "two"));
	}

	#[test]
	fn tactile_grid_maps_darkness_to_shade_cells() {
		// Left half black, right half white, 4 by 2 grid.
		let mut raw = image::RgbImage::new(8, 4);
		for (x, _y, p) in raw.enumerate_pixels_mut() {
			*p = if x < 4 { image::Rgb([0, 0, 0]) } else { image::Rgb([255, 255, 255]) };
		}
		let grid = tactile_grid(&image::DynamicImage::ImageRgb8(raw), 4, 2);
		assert_eq!(grid.cells.len(), 8);
		// Darkness falls monotonically across the row; the boundary
		// cells blend, because each cell is an average.
		assert!(grid.cells[0] >= grid.cells[1] && grid.cells[1] >= grid.cells[2]);
		assert!(grid.cells[2] >= grid.cells[3]);
		assert_eq!(grid.cells[0], 1.0);
		assert_eq!(grid.cells[3], 0.0);
		let rows = grid.braille_rows();
		assert_eq!(rows.len(), 2);
		assert!(rows[0].starts_with('⠿'), "dark cells first: {}", rows[0]);
		assert!(rows[0].ends_with('\u{2800}'), "bright cells last: {}", rows[0]);
		let sheet = grid.sheet("test picture");
		assert!(sheet.contains("column 1 black"), "{sheet}");
		assert!(sheet.contains("column 4 bright"), "{sheet}");
		assert!(sheet.contains("Darkest part: top left"), "{sheet}");
		assert!(sheet.contains("Brightest part: top right"), "{sheet}");
	}
}
