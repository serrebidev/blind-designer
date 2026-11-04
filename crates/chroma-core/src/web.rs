//! web: the design as a web page, and web files back into designs.
//!
//! Export writes one standalone HTML page: the palette as CSS custom
//! properties (the same variables `Design::to_css` produces), then one
//! absolutely positioned element per shape, its caption carried as the
//! text a screen reader announces (aria-label). Import reads the two
//! things a hand-written page actually declares: CSS colors (custom
//! properties and color declarations, each with a note saying where it
//! came from) and simple HTML elements with inline styles (their boxes
//! become shapes, captioned with the element's name).
//!
//! The parsers are deliberately minimal and hand-rolled: hex, rgb() and
//! a table of named colors; px lengths only; inline styles only.
//! Stylesheet rules are harvested for their colors but are NOT matched
//! against elements, and layout is a vertical stack unless left/top say
//! otherwise.
//! ponytail: minimal parsers by design — upgrade path is a real CSS/HTML
//! parser crate if imports ever need cascade, classes or relative units.

use crate::design::{Design, Shape, ShapeKind};
use crate::drawing::Drawing;
use crate::{describe, Srgb};

/// One color found in a CSS or HTML source, with where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssColor {
	pub color: Srgb,
	/// Provenance note, e.g. `custom property --brand-1` or
	/// `background-color in rule "body"`.
	pub source: String,
}

fn escape_html(s: &str) -> String {
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

impl Design {
	/// The design as a standalone HTML page with inline CSS: palette as
	/// CSS variables, canvas as a positioned box, one element per shape
	/// with its caption as the accessible name.
	pub fn to_html(&self, title: &str) -> String {
		let mut out = String::new();
		out.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
		out.push_str(&format!("<title>{}</title>\n<style>\n", escape_html(title)));
		out.push_str(&self.to_css("design"));
		out.push_str(".design { position: relative; }\n.shape { position: absolute; }\n</style>\n</head>\n<body>\n");
		out.push_str(&format!(
			"<main class=\"design\" role=\"img\" aria-label=\"{}\" style=\"width: {}px; height: {}px; background: {};\">\n",
			escape_html(&format!(
				"Design {} by {} pixels, background {}",
				self.width,
				self.height,
				describe(self.background)
			)),
			self.width,
			self.height,
			self.background.to_hex()
		));
		out.push_str(&self.shapes_markup());
		out.push_str("</main>\n</body>\n</html>\n");
		out
	}

	/// One absolutely positioned element per shape, each carrying
	/// its caption as its accessible name — the markup `to_html`
	/// puts inside the page, shared with the animated export, which
	/// stacks one such group per frame.
	pub(crate) fn shapes_markup(&self) -> String {
		let mut out = String::new();
		for shape in &self.shapes {
			let caption = escape_html(&self.caption_for(shape));
			let hex = shape.fill.to_hex();
			let rotate = if shape.angle != 0.0 {
				format!(" transform: rotate({}deg);", shape.angle)
			} else {
				String::new()
			};
			match shape.kind {
				ShapeKind::Stroke => {
					// A stroke is an inline SVG polyline over its own box.
					let points: Vec<String> = shape
						.points
						.iter()
						.map(|(x, y)| format!("{x},{y}"))
						.collect();
					out.push_str(&format!(
						"<svg class=\"shape\" role=\"img\" aria-label=\"{caption}\" style=\"left: {}px; top: {}px; width: {}px; height: {}px;{rotate}\" viewBox=\"{} {} {} {}\" preserveAspectRatio=\"none\" width=\"{}\" height=\"{}\"><polyline points=\"{}\" fill=\"none\" stroke=\"{hex}\" stroke-width=\"{}\"/></svg>\n",
						shape.x,
						shape.y,
						shape.w,
						shape.h,
						shape.x,
						shape.y,
						shape.w.max(1),
						shape.h.max(1),
						shape.w.max(1),
						shape.h.max(1),
						points.join(" "),
						shape.brush
					));
				}
				ShapeKind::Text => {
					// The phrase is the element's content, so a screen
					// reader reads the text itself; the font name and
					// size become real CSS.
					out.push_str(&format!(
						"<div class=\"shape\" style=\"left: {}px; top: {}px; width: {}px; height: {}px; color: {hex}; font-family: {}; font-size: {}px;{rotate}\">{}</div>\n",
						shape.x,
						shape.y,
						shape.w,
						shape.h,
						escape_html(&shape.font),
						shape.font_size,
						escape_html(&shape.text)
					));
				}
				ShapeKind::Image => {
					out.push_str(&format!(
						"<img class=\"shape\" src=\"{}\" alt=\"{caption}\" style=\"left: {}px; top: {}px; width: {}px; height: {}px;{rotate}\">\n",
						escape_html(&shape.image_path),
						shape.x,
						shape.y,
						shape.w,
						shape.h
					));
				}
				kind => {
					let radius = if kind == ShapeKind::Ellipse {
						" border-radius: 50%;".to_string()
					} else if kind == ShapeKind::Rectangle && shape.corner > 0 {
						format!(" border-radius: {}%;", shape.corner)
					} else {
						String::new()
					};
					out.push_str(&format!(
						"<div class=\"shape\" role=\"img\" aria-label=\"{caption}\" style=\"left: {}px; top: {}px; width: {}px; height: {}px; background: {hex};{radius}{rotate}\"></div>\n",
						shape.x, shape.y, shape.w, shape.h
					));
				}
			}
		}

		out
	}

}

impl Drawing {
	/// The animation as a standalone HTML page: the frames stacked
	/// on one stage, stepped through by a generated CSS animation —
	/// each frame is visible only during its own slice of the total
	/// running time, then the film loops. Every frame's shapes are
	/// the same accessible elements the static export writes, and
	/// the page also carries the frame-by-frame description as a
	/// readable list, so the film can be followed by ear as well as
	/// by eye. (No sprite sheet and no script: stacked elements and
	/// keyframes are all a browser needs.) A static design exports
	/// as a one-frame page with no animation rules. Readers who
	/// prefer reduced motion get the frames stacked statically.
	pub fn to_html_animated(&self, title: &str) -> String {
		let designs = self.frame_designs();
		let durations = self.frame_durations();
		let total: u32 = durations.iter().sum();
		let mut out = String::new();
		out.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
		out.push_str(&format!("<title>{}</title>\n<style>\n", escape_html(title)));
		out.push_str(&self.flatten().to_css("design"));
		out.push_str(".stage { position: relative; }\n.shape { position: absolute; }\n.frame { position: absolute; left: 0; top: 0; width: 100%; height: 100%; opacity: 0; }\n");
		if designs.len() > 1 {
			// Each frame gets its own keyframes: hidden until its
			// slice starts, visible through it, hidden after. The
			// small epsilon step (a hundredth of a percent, or half
			// the slice when the slice is thinner) makes the switch
			// instant rather than a fade.
			let mut start = 0u32;
			for (i, ms) in durations.iter().enumerate() {
				let s = start as f64 * 100.0 / total.max(1) as f64;
				let e = (start + ms) as f64 * 100.0 / total.max(1) as f64;
				let eps = ((e - s) / 2.0).min(0.01);
				out.push_str(&format!("@keyframes frame{i} {{\n"));
				if s <= 0.0 {
					out.push_str("0% { opacity: 1; }\n");
				} else {
					out.push_str("0% { opacity: 0; }\n");
					out.push_str(&format!("{s:.2}% {{ opacity: 0; }}\n"));
					out.push_str(&format!("{:.2}% {{ opacity: 1; }}\n", s + eps));
				}
				if e >= 100.0 {
					out.push_str("100% { opacity: 1; }\n");
				} else {
					out.push_str(&format!("{e:.2}% {{ opacity: 1; }}\n"));
					out.push_str(&format!("{:.2}% {{ opacity: 0; }}\n", e + eps));
					out.push_str("100% { opacity: 0; }\n");
				}
				out.push_str("}\n");
				out.push_str(&format!(
					".frame{i} {{ animation: frame{i} {total}ms linear infinite; }}\n"
				));
				start += ms;
			}
		} else {
			out.push_str(".frame0 { opacity: 1; }\n");
		}
		out.push_str("@media (prefers-reduced-motion: reduce) {\n.frame { animation: none; position: static; opacity: 1; }\n}\n</style>\n</head>\n<body>\n");
		out.push_str(&format!(
			"<main>\n<div class=\"stage\" role=\"img\" aria-label=\"{}\" style=\"width: {}px; height: {}px;\">\n",
			escape_html(&format!(
				"Animated design, {} frames, total running time {} milliseconds",
				designs.len(),
				total
			)),
			self.width,
			self.height
		));
		for (i, design) in designs.iter().enumerate() {
			out.push_str(&format!(
				"<div class=\"frame frame{i}\" style=\"background: {};\">",
				design.background.to_hex()
			));
			out.push_str(&design.shapes_markup());
			out.push_str("</div>\n");
		}
		out.push_str("</div>\n<section>\n<h2>Frame by frame description</h2>\n<ol>\n");
		for i in 0..designs.len() {
			let what = self.frame_change_description(i).unwrap_or_default();
			out.push_str(&format!(
				"<li>Frame {} ({} milliseconds): {}</li>\n",
				i + 1,
				durations[i],
				escape_html(&what)
			));
		}
		out.push_str("</ol>\n</section>\n</main>\n</body>\n</html>\n");
		out
	}
}

/// The CSS named colors this parser knows (the common, unambiguous set).
/// Also the name table plain-language color input falls back to.
pub fn named_color(word: &str) -> Option<Srgb> {
	let hex = match word.to_ascii_lowercase().as_str() {
		"black" => "#000000",
		"white" => "#FFFFFF",
		"red" => "#FF0000",
		"green" => "#008000",
		"blue" => "#0000FF",
		"yellow" => "#FFFF00",
		"orange" => "#FFA500",
		"purple" => "#800080",
		"pink" => "#FFC0CB",
		"brown" => "#A52A2A",
		"gray" | "grey" => "#808080",
		"silver" => "#C0C0C0",
		"maroon" => "#800000",
		"olive" => "#808000",
		"lime" => "#00FF00",
		"aqua" | "cyan" => "#00FFFF",
		"teal" => "#008080",
		"navy" => "#000080",
		"fuchsia" | "magenta" => "#FF00FF",
		_ => return None,
	};
	Srgb::from_hex(hex)
}

/// Every color named in one declaration value: hex codes, rgb()/rgba()
/// calls and named colors, in order of appearance.
fn colors_in_value(value: &str) -> Vec<Srgb> {
	let bytes = value.as_bytes();
	let mut out = Vec::new();
	let mut i = 0;
	while i < bytes.len() {
		let c = bytes[i] as char;
		if c == '#' {
			let start = i + 1;
			let mut end = start;
			while end < bytes.len() && (bytes[end] as char).is_ascii_hexdigit() {
				end += 1;
			}
			let digits = &value[start..end];
			if digits.len() == 3 || digits.len() == 6 {
				if let Some(color) = Srgb::from_hex(digits) {
					out.push(color);
				}
			}
			i = end.max(i + 1);
		} else if c.is_ascii_alphabetic() {
			let start = i;
			while i < bytes.len() {
				let ch = bytes[i] as char;
				if ch.is_ascii_alphanumeric() || ch == '-' {
					i += 1;
				} else {
					break;
				}
			}
			let word = &value[start..i];
			if word.eq_ignore_ascii_case("rgb") || word.eq_ignore_ascii_case("rgba") {
				// Skip to the closing paren and read the first three numbers.
				if let Some(open) = value[i..].find('(') {
					let inner_start = i + open + 1;
					if let Some(close) = value[inner_start..].find(')') {
						let inner = &value[inner_start..inner_start + close];
						let nums: Vec<f32> = inner
							.split(|ch: char| ch == ',' || ch == '/' || ch.is_whitespace())
							.filter(|t| !t.is_empty())
							.filter_map(|t| {
								let t = t.trim_end_matches('%');
								t.parse::<f32>().ok()
							})
							.collect();
						if nums.len() >= 3 {
							// Percent components only when the source used them:
							// rgb(100%, 0%, 0%) — detect by a '%' in the source.
							let pct = inner.contains('%');
							let ch = |v: f32| {
								if pct {
									(v * 255.0 / 100.0).round().clamp(0.0, 255.0) as u8
								} else {
									v.round().clamp(0.0, 255.0) as u8
								}
							};
							out.push(Srgb::new(ch(nums[0]), ch(nums[1]), ch(nums[2])));
						}
						i = inner_start + close + 1;
					}
				}
			} else if let Some(color) = named_color(word) {
				out.push(color);
			}
		} else {
			i += 1;
		}
	}
	out
}

/// `prop: value` pairs from a declaration block or an inline style
/// attribute. Property names come back lowercased.
fn declarations(body: &str) -> Vec<(String, String)> {
	body.split(';')
		.filter_map(|part| {
			let (prop, value) = part.split_once(':')?;
			let prop = prop.trim().to_ascii_lowercase();
			if prop.is_empty() {
				return None;
			}
			Some((prop, value.trim().to_string()))
		})
		.collect()
}

fn push_color(found: &mut Vec<CssColor>, color: Srgb, source: &str) {
	if !found.iter().any(|c| c.color == color) {
		found.push(CssColor { color, source: source.to_string() });
	}
}

/// Parse a CSS stylesheet's colors: custom properties and ordinary
/// color declarations, de-duplicated, each with a provenance note.
pub fn parse_css_colors(css: &str) -> Vec<CssColor> {
	// Strip comments so a commented-out color never imports.
	let mut clean = String::with_capacity(css.len());
	let mut rest = css;
	while let Some(start) = rest.find("/*") {
		clean.push_str(&rest[..start]);
		match rest[start..].find("*/") {
			Some(end) => rest = &rest[start + end + 2..],
			None => {
				rest = "";
				break;
			}
		}
	}
	clean.push_str(rest);

	let mut found: Vec<CssColor> = Vec::new();
	// Walk rule blocks: selector is the text before each '{'.
	let mut rest = clean.as_str();
	loop {
		let Some(open) = rest.find('{') else { break };
		let selector = rest[..open].trim();
		let selector = selector.rsplit(['}', ';']).next().unwrap_or(selector).trim();
		let Some(close) = rest[open..].find('}') else { break };
		let body = &rest[open + 1..open + close];
		for (prop, value) in declarations(body) {
			let source = if let Some(name) = prop.strip_prefix("--") {
				format!("custom property --{name}")
			} else {
				format!("{prop} in rule \"{selector}\"")
			};
			for color in colors_in_value(&value) {
				push_color(&mut found, color, &source);
			}
		}
		rest = &rest[open + close + 1..];
	}
	found
}

/// A pixel length: "120px", "120" or "120.5px". Anything else (percent,
/// em, auto) is not a pixel box and returns None.
fn px(value: &str) -> Option<i32> {
	let v = value.trim().trim_end_matches("px").trim();
	v.parse::<f32>().ok().map(|n| n.round() as i32)
}

fn style_get<'a>(style: &'a [(String, String)], prop: &str) -> Option<&'a str> {
	style.iter().find(|(p, _)| p == prop).map(|(_, v)| v.as_str())
}

/// The fill an element paints with: background-color, else a plain
/// color `background`, else its text `color`.
fn element_fill(style: &[(String, String)]) -> Option<Srgb> {
	for prop in ["background-color", "background", "color"] {
		if let Some(value) = style_get(style, prop) {
			if let Some(color) = colors_in_value(value).into_iter().next() {
				return Some(color);
			}
		}
	}
	None
}

/// The result of importing an HTML file: the design it describes plus
/// every color found anywhere in it (inline styles and `<style>`
/// blocks), each with its provenance.
#[derive(Debug, Clone)]
pub struct HtmlImport {
	pub design: Design,
	pub colors: Vec<CssColor>,
}

struct OpenElement {
	tag: String,
	style: Vec<(String, String)>,
	text: String,
}

/// Parse one tag's inner text (no angle brackets) into name + attributes.
fn parse_tag(inner: &str) -> (String, Vec<(String, String)>) {
	let inner = inner.trim().trim_end_matches('/');
	let mut parts = inner.splitn(2, char::is_whitespace);
	let name = parts.next().unwrap_or("").to_ascii_lowercase();
	let mut attrs = Vec::new();
	let mut rest = parts.next().unwrap_or("");
	while !rest.trim().is_empty() {
		rest = rest.trim_start();
		let Some(eq) = rest.find('=') else { break };
		let key = rest[..eq].trim().to_ascii_lowercase();
		rest = rest[eq + 1..].trim_start();
		let value = if let Some(q) = rest.chars().next().filter(|c| *c == '"' || *c == '\'') {
			match rest[1..].find(q) {
				Some(end) => {
					let v = &rest[1..1 + end];
					rest = &rest[1 + end + 1..];
					v.to_string()
				}
				None => break,
			}
		} else {
			let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
			let v = &rest[..end];
			rest = &rest[end..];
			v.to_string()
		};
		if !key.is_empty() {
			attrs.push((key, value));
		}
	}
	(name, attrs)
}

const VOID_TAGS: &[&str] = &[
	"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source",
	"track", "wbr",
];

/// Import a simple HTML file: elements with inline styles become
/// shapes (positioned by left/top, stacked top to bottom when the page
/// does not position them), captioned with the element's tag and text.
pub fn import_html(html: &str) -> HtmlImport {
	let mut colors: Vec<CssColor> = Vec::new();
	let mut shapes: Vec<Shape> = Vec::new();
	let mut background: Option<Srgb> = None;
	let mut canvas: Option<(u32, u32)> = None;
	let mut stack: Vec<OpenElement> = Vec::new();
	let mut cursor_y: i32 = 0; // stack-layout position for unpositioned elements

	let finish = |el: &OpenElement,
	              shapes: &mut Vec<Shape>,
	              colors: &mut Vec<CssColor>,
	              background: &mut Option<Srgb>,
	              canvas: &mut Option<(u32, u32)>,
	              cursor_y: &mut i32| {
		for (prop, value) in &el.style {
			for color in colors_in_value(value) {
				push_color(colors, color, &format!("{prop} on <{}>", el.tag));
			}
		}
		if el.tag == "body" || el.tag == "html" || el.tag == "main" {
			if let Some(bg) = element_fill(&el.style) {
				if background.is_none() {
					*background = Some(bg);
				}
			}
			let w = style_get(&el.style, "width").and_then(px);
			let h = style_get(&el.style, "height").and_then(px);
			if let (Some(w), Some(h)) = (w, h) {
				*canvas = Some((w.max(1) as u32, h.max(1) as u32));
			}
			return;
		}
		if matches!(el.tag.as_str(), "head" | "title" | "meta" | "link" | "script" | "style") {
			return;
		}
		let Some(fill) = element_fill(&el.style) else { return };
		let w = style_get(&el.style, "width").and_then(px).unwrap_or(100).max(1);
		let h = style_get(&el.style, "height").and_then(px).unwrap_or(20).max(1);
		let x = style_get(&el.style, "left").and_then(px).unwrap_or(0);
		let y = style_get(&el.style, "top").and_then(px).unwrap_or(*cursor_y);
		*cursor_y = (*cursor_y).max(y + h);
		let kind = match style_get(&el.style, "border-radius") {
			Some(r) if r.trim().trim_end_matches('%').trim().parse::<f32>().is_ok_and(|v| v >= 50.0) => {
				ShapeKind::Ellipse
			}
			_ => ShapeKind::Rectangle,
		};
		let text = el.text.split_whitespace().collect::<Vec<_>>().join(" ");
		let mut caption = format!("{} element", el.tag);
		if !text.is_empty() {
			let short: String = text.chars().take(60).collect();
			caption.push_str(&format!(": \"{short}\""));
		}
		let mut shape = Shape::new(kind, x, y, w as u32, h as u32, fill);
		shape.caption = Some(caption);
		shapes.push(shape);
	};

	let mut rest = html;
	while let Some(lt) = rest.find('<') {
		// Text before the tag belongs to the innermost open element.
		if lt > 0 {
			if let Some(top) = stack.last_mut() {
				top.text.push_str(&rest[..lt]);
				top.text.push(' ');
			}
			rest = &rest[lt..];
			continue;
		}
		if rest.starts_with("<!--") {
			match rest.find("-->") {
				Some(end) => rest = &rest[end + 3..],
				None => break,
			}
			continue;
		}
		let Some(gt) = rest.find('>') else { break };
		let inner = &rest[1..gt];
		rest = &rest[gt + 1..];
		if inner.starts_with('!') || inner.starts_with('?') {
			continue; // doctype, declarations
		}
		if let Some(name) = inner.strip_prefix('/') {
			let name = name.trim().to_ascii_lowercase();
			// Close the matching element (and anything left open inside it).
			if let Some(pos) = stack.iter().rposition(|el| el.tag == name) {
				while stack.len() > pos {
					let el = stack.pop().unwrap();
					finish(&el, &mut shapes, &mut colors, &mut background, &mut canvas, &mut cursor_y);
				}
			}
			continue;
		}
		let self_closing = inner.trim_end().ends_with('/');
		let (name, attrs) = parse_tag(inner);
		if name.is_empty() {
			continue;
		}
		if name == "style" || name == "script" {
			// Harvest stylesheet colors; never treat the text as markup.
			let close = format!("</{name}");
			match rest.to_ascii_lowercase().find(&close) {
				Some(end) => {
					if name == "style" {
						for c in parse_css_colors(&rest[..end]) {
							push_color(&mut colors, c.color, &c.source);
						}
					}
					rest = &rest[end..];
				}
				None => break,
			}
			continue;
		}
		let style = attrs
			.iter()
			.find(|(k, _)| k == "style")
			.map(|(_, v)| declarations(v))
			.unwrap_or_default();
		let el = OpenElement { tag: name.clone(), style, text: String::new() };
		if self_closing || VOID_TAGS.contains(&name.as_str()) {
			finish(&el, &mut shapes, &mut colors, &mut background, &mut canvas, &mut cursor_y);
		} else {
			stack.push(el);
		}
	}
	// Anything still open at end of file still counts.
	while let Some(el) = stack.pop() {
		finish(&el, &mut shapes, &mut colors, &mut background, &mut canvas, &mut cursor_y);
	}

	let (width, height) = canvas.unwrap_or_else(|| {
		let w = shapes.iter().map(|s| (s.x + s.w as i32).max(0) as u32).max().unwrap_or(0);
		let h = shapes.iter().map(|s| (s.y + s.h as i32).max(0) as u32).max().unwrap_or(0);
		(w.max(100), h.max(100))
	});
	let mut design = Design::new(width, height, background.unwrap_or(Srgb::WHITE));
	design.shapes = shapes;
	HtmlImport { design, colors }
}

#[cfg(test)]
mod tests {
	use super::*;

	const CSS_FIXTURE: &str = "
		/* --ignored: #123456; */
		:root {
			--brand-1: #0F5E68;
			--accent: rgb(255, 128, 0);
		}
		body { background-color: #FFFFFF; color: #222; }
		.card { border: 1px solid teal; background: rgba(15, 94, 104, 0.5); }
	";

	#[test]
	fn css_import_reads_custom_properties_and_declarations() {
		let found = parse_css_colors(CSS_FIXTURE);
		let hexes: Vec<String> = found.iter().map(|c| c.color.to_hex()).collect();
		assert!(hexes.contains(&"#0F5E68".to_string()), "{hexes:?}");
		assert!(hexes.contains(&"#FF8000".to_string()), "{hexes:?}");
		assert!(hexes.contains(&"#FFFFFF".to_string()), "{hexes:?}");
		assert!(hexes.contains(&"#222222".to_string()), "{hexes:?}");
		assert!(hexes.contains(&"#008080".to_string()), "{hexes:?}"); // teal, named
		assert!(!hexes.contains(&"#123456".to_string()), "comment color leaked: {hexes:?}");
		// #0F5E68 appears twice in the fixture (custom property + rgba) but imports once.
		assert_eq!(found.iter().filter(|c| c.color.to_hex() == "#0F5E68").count(), 1);
		let brand = found.iter().find(|c| c.color.to_hex() == "#0F5E68").unwrap();
		assert_eq!(brand.source, "custom property --brand-1");
		let body = found.iter().find(|c| c.color.to_hex() == "#FFFFFF").unwrap();
		assert!(body.source.contains("background-color"), "{}", body.source);
	}

	const HTML_FIXTURE: &str = r#"
		<!DOCTYPE html>
		<html><head><title>Card</title>
		<style>:root { --brand-1: #0F5E68; }</style></head>
		<body style="background-color: #EEEEEE">
			<header style="background-color: #0F5E68; width: 600px; height: 80px">Welcome</header>
			<div style="background-color: #FF8000; width: 200px; height: 100px; border-radius: 50%">Badge</div>
			<p style="color: #222222">Some text without a box size</p>
		</body></html>
	"#;

	#[test]
	fn html_import_builds_shapes_with_element_captions() {
		let imp = import_html(HTML_FIXTURE);
		assert_eq!(imp.design.background.to_hex(), "#EEEEEE");
		assert_eq!(imp.design.shapes.len(), 3, "{:?}", imp.design.shapes);
		let header = &imp.design.shapes[0];
		assert_eq!(header.kind, ShapeKind::Rectangle);
		assert_eq!((header.x, header.y, header.w, header.h), (0, 0, 600, 80));
		assert_eq!(header.fill.to_hex(), "#0F5E68");
		assert_eq!(header.caption.as_deref(), Some("header element: \"Welcome\""));
		let badge = &imp.design.shapes[1];
		assert_eq!(badge.kind, ShapeKind::Ellipse); // border-radius 50%
		assert_eq!(badge.y, 80); // stacked below the header
		let para = &imp.design.shapes[2];
		assert_eq!(para.fill.to_hex(), "#222222");
		// The caption sheet names the HTML elements.
		let sheet = imp.design.caption_sheet();
		assert!(sheet.contains("div element"), "{sheet}");
		// Colors include the <style> block's custom property.
		assert!(imp.colors.iter().any(|c| c.source == "custom property --brand-1"));
		// Canvas hugs the content.
		assert_eq!((imp.design.width, imp.design.height), (600, 200));
	}

	#[test]
	fn html_export_is_a_standalone_page_with_captions() {
		let mut d = Design::new(200, 100, Srgb::WHITE);
		let mut s = Shape::new(ShapeKind::Ellipse, 10, 20, 50, 40, Srgb::new(15, 94, 104));
		s.caption = Some("logo dot".to_string());
		d.shapes.push(s);
		d.shapes
			.push(Shape::stroke(vec![(0, 0), (100, 50)], 3, Srgb::new(255, 128, 0)));
		let html = d.to_html("My <Design>");
		assert!(html.starts_with("<!DOCTYPE html>"), "{html}");
		assert!(html.contains("<title>My &lt;Design&gt;</title>"), "{html}");
		assert!(html.contains("--design-1: #FFFFFF"), "{html}"); // palette as CSS variables
		assert!(html.contains("aria-label=\"logo dot\""), "{html}");
		assert!(html.contains("left: 10px; top: 20px; width: 50px; height: 40px"), "{html}");
		assert!(html.contains("border-radius: 50%"), "{html}");
		assert!(html.contains("<polyline"), "{html}");
		assert!(html.contains("stroke=\"#FF8000\""), "{html}");
	}

	#[test]
	fn html_export_rounds_rectangle_corners_with_border_radius() {
		let mut d = Design::new(200, 100, Srgb::WHITE);
		let mut s = Shape::new(ShapeKind::Rectangle, 10, 20, 50, 40, Srgb::BLACK);
		s.corner = 30;
		d.shapes.push(s);
		d.shapes.push(Shape::new(ShapeKind::Rectangle, 100, 20, 50, 40, Srgb::BLACK));
		let html = d.to_html("page");
		assert!(html.contains("border-radius: 30%"), "{html}");
		// The sharp rectangle carries no radius at all.
		assert_eq!(html.matches("border-radius").count(), 1, "{html}");
	}

	#[test]
	fn html_export_carries_text_and_image_elements() {
		let mut d = Design::new(300, 200, Srgb::WHITE);
		d.shapes.push(Shape::text("Big <Sale>", "Verdana", 32, 10, 10, 200, 40, Srgb::new(200, 0, 0)));
		d.shapes.push(Shape::image("photos/team day.png", 10, 60, 100, 80, Srgb::new(0, 0, 255)));
		let html = d.to_html("page");
		// The phrase is real text content (escaped), with real font CSS.
		assert!(html.contains("font-family: Verdana; font-size: 32px"), "{html}");
		assert!(html.contains(">Big &lt;Sale&gt;</div>"), "{html}");
		assert!(html.contains("color: #C80000"), "{html}");
		// The image references its file, captioned by its alt text.
		assert!(html.contains("<img class=\"shape\" src=\"photos/team day.png\""), "{html}");
		assert!(html.contains("alt=\"image from file team day.png"), "{html}");
	}

	#[test]
	fn animated_html_steps_through_the_frames() {
		let mut d = Drawing::new(100, 100, Srgb::WHITE);
		d.add_shape(
			0,
			Shape::new(ShapeKind::Ellipse, 40, 40, 20, 20, Srgb::new(200, 40, 30)),
		);
		d.add_frame();
		d.set_frame_duration(0, 400);
		d.shift_shape(0, 0, 20, 0);
		d.add_frame();
		d.set_frame_duration(1, 600);
		let html = d.to_html_animated("Film");
		assert!(html.starts_with("<!DOCTYPE html>"), "{html}");
		// One keyframed class per frame, timed on the total (1000 ms).
		assert!(html.contains("@keyframes frame0"), "{html}");
		assert!(html.contains("@keyframes frame1"), "{html}");
		assert!(html.contains("animation: frame0 1000ms linear infinite"), "{html}");
		assert!(html.contains("animation: frame1 1000ms linear infinite"), "{html}");
		// Frame 2 starts where frame 1 ends: 40 percent of the film.
		assert!(html.contains("40.00% { opacity: 0; }"), "{html}");
		// Both frames are stacked on the stage, shapes included.
		assert_eq!(html.matches("class=\"frame frame").count(), 2, "{html}");
		assert!(html.contains("Animated design, 2 frames, total running time 1000 milliseconds"), "{html}");
		// The film can be followed by ear: the description list is in
		// the page, with the diff wording for frame 2.
		assert!(html.contains("<h2>Frame by frame description</h2>"), "{html}");
		assert!(html.contains("Frame 2 (600 milliseconds):"), "{html}");
		assert!(html.contains("moves right 20 pixels"), "{html}");
		// Reduced motion stacks the frames statically instead.
		assert!(html.contains("prefers-reduced-motion"), "{html}");
		// A static design exports as one still frame, no keyframes.
		let still = Drawing::new(10, 10, Srgb::WHITE).to_html_animated("Still");
		assert!(!still.contains("@keyframes"), "{still}");
		assert!(still.contains(".frame0 { opacity: 1; }"), "{still}");
	}

	#[test]
	fn exported_html_imports_back() {
		let mut d = Design::new(300, 200, Srgb::new(238, 238, 238));
		d.shapes
			.push(Shape::new(ShapeKind::Rectangle, 5, 6, 100, 50, Srgb::new(15, 94, 104)));
		let imp = import_html(&d.to_html("round trip"));
		assert_eq!(imp.design.shapes.len(), 1);
		let s = &imp.design.shapes[0];
		assert_eq!((s.x, s.y, s.w, s.h), (5, 6, 100, 50));
		assert_eq!(s.fill.to_hex(), "#0F5E68");
		assert_eq!(imp.design.background.to_hex(), "#EEEEEE");
	}
}
