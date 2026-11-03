//! plain: plain-language input for the add and move dialogs.
//!
//! Position, size and color fields accept words as well as numbers:
//! a position can be "top left", "center" or "a bit right" (a nudge
//! from where the shape is now); a size can be "a bit wider" as well
//! as "200 by 100"; a color can be "sky blue" as well as "#87CEEB".
//! Every parser returns `None` on words it does not understand, so the
//! dialog can say so instead of guessing.

use crate::web::named_color;
use crate::Srgb;

/// How far "a bit" moves a shape, in pixels.
pub const NUDGE: i32 = 10;
/// How much "a bit wider" and friends change a size, in pixels.
pub const GROW: i32 = 10;

/// Multi-word and everyday color names beyond the CSS table in
/// `web::named_color`.
fn extra_color(word: &str) -> Option<Srgb> {
	let hex = match word {
		"sky blue" => "#87CEEB",
		"light blue" => "#ADD8E6",
		"dark blue" => "#00008B",
		"midnight blue" => "#191970",
		"royal blue" => "#4169E1",
		"light green" => "#90EE90",
		"dark green" => "#006400",
		"forest green" => "#228B22",
		"sea green" => "#2E8B57",
		"dark red" => "#8B0000",
		"hot pink" => "#FF69B4",
		"deep pink" => "#FF1493",
		"light gray" | "light grey" => "#D3D3D3",
		"dark gray" | "dark grey" => "#A9A9A9",
		"slate gray" | "slate grey" => "#708090",
		"gold" => "#FFD700",
		"beige" => "#F5F5DC",
		"ivory" => "#FFFFF0",
		"coral" => "#FF7F50",
		"salmon" => "#FA8072",
		"khaki" => "#F0E68C",
		"lavender" => "#E6E6FA",
		"violet" => "#EE82EE",
		"indigo" => "#4B0082",
		"turquoise" => "#40E0D0",
		"crimson" => "#DC143C",
		"chocolate" => "#D2691E",
		"tomato" => "#FF6347",
		_ => return None,
	};
	Srgb::from_hex(hex)
}

/// A color typed in plain words: a hex code, an rgb(r, g, b) triple,
/// or a color name ("red", "sky blue", "dark gray").
pub fn parse_color(text: &str) -> Option<Srgb> {
	let t = text.trim().to_lowercase();
	if t.is_empty() {
		return None;
	}
	if let Some(c) = Srgb::from_hex(&t) {
		return Some(c);
	}
	if let Some(inner) = t.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
		let nums: Vec<&str> = inner.split(',').map(|n| n.trim()).collect();
		if nums.len() == 3 {
			let (Ok(r), Ok(g), Ok(b)) =
				(nums[0].parse::<u8>(), nums[1].parse::<u8>(), nums[2].parse::<u8>())
			else {
				return None;
			};
			return Some(Srgb::new(r, g, b));
		}
		return None;
	}
	extra_color(&t).or_else(|| named_color(&t))
}

/// A named spot on the canvas, as the point a shape's top-left corner
/// goes to. `None` when the words name no spot.
fn named_spot(t: &str, canvas_w: u32, canvas_h: u32) -> Option<(i32, i32)> {
	let (w, h) = (canvas_w as i32, canvas_h as i32);
	let col = |words: &[&str]| words.iter().any(|x| t.split_whitespace().any(|w| w == *x));
	let spot = match t {
		"center" | "centre" | "middle" | "center of the canvas" | "centre of the canvas" => {
			(w / 2, h / 2)
		}
		_ if col(&["left"]) && col(&["top"]) => (0, 0),
		_ if col(&["right"]) && col(&["top"]) => (w, 0),
		_ if col(&["left"]) && col(&["bottom"]) => (0, h),
		_ if col(&["right"]) && col(&["bottom"]) => (w, h),
		_ if col(&["top"]) => (w / 2, 0),
		_ if col(&["bottom"]) => (w / 2, h),
		_ if col(&["left"]) => (0, h / 2),
		_ if col(&["right"]) => (w, h / 2),
		_ => return None,
	};
	Some(spot)
}

/// A position typed in the dialogs: "120, 80" (or "120 80"), a named
/// spot ("top left", "center", "bottom right"), or — with a current
/// position to start from — a nudge ("a bit left", "a bit down").
pub fn parse_position(
	text: &str,
	canvas_w: u32,
	canvas_h: u32,
	current: Option<(i32, i32)>,
) -> Option<(i32, i32)> {
	let t = text.trim().to_lowercase();
	if t.is_empty() {
		return None;
	}
	// Numbers first: two whole numbers separated by a comma or space.
	let nums: Vec<&str> =
		t.split(|c: char| c == ',' || c.is_whitespace()).filter(|s| !s.is_empty()).collect();
	if nums.len() == 2 {
		if let (Ok(x), Ok(y)) = (nums[0].parse::<i32>(), nums[1].parse::<i32>()) {
			return Some((x, y));
		}
	}
	// Nudge words ("a bit right") outrank named spots: the phrase
	// names a direction AND a spot word, and the nudge is the meaning.
	let nudgy = t.contains("bit") || t.contains("little") || t.contains("slightly");
	if nudgy {
		let (cx, cy) = current?;
		let (mut dx, mut dy) = (0, 0);
		if t.contains("left") {
			dx = -NUDGE;
		}
		if t.contains("right") {
			dx = NUDGE;
		}
		if t.contains("up") || t.contains("top") {
			dy = -NUDGE;
		}
		if t.contains("down") || t.contains("bottom") {
			dy = NUDGE;
		}
		if dx != 0 || dy != 0 {
			return Some((cx + dx, cy + dy));
		}
		return None;
	}
	named_spot(&t, canvas_w, canvas_h)
}

/// A size typed in the dialogs: "200 by 100" (also "200x100",
/// "200, 100" or "200 100"), or — with a current size to start from —
/// a relative change: "a bit wider", "a bit narrower", "a bit
/// taller", "a bit shorter", "a bit bigger", "a bit smaller".
pub fn parse_size(text: &str, current: Option<(u32, u32)>) -> Option<(u32, u32)> {
	let t = text.trim().to_lowercase();
	if t.is_empty() {
		return None;
	}
	let nums: Vec<&str> = t
		.split(|c: char| c == ',' || c == 'x' || c.is_whitespace())
		.filter(|s| !s.is_empty() && *s != "by")
		.collect();
	if nums.len() == 2 {
		if let (Ok(w), Ok(h)) = (nums[0].parse::<u32>(), nums[1].parse::<u32>()) {
			if w >= 1 && h >= 1 {
				return Some((w, h));
			}
			return None;
		}
	}
	let (cw, ch) = current?;
	let grow = |v: u32, d: i32| -> u32 { (v as i32 + d).max(1) as u32 };
	if t.contains("wider") {
		return Some((grow(cw, GROW), ch));
	}
	if t.contains("narrower") || t.contains("thinner") {
		return Some((grow(cw, -GROW), ch));
	}
	if t.contains("taller") {
		return Some((cw, grow(ch, GROW)));
	}
	if t.contains("shorter") {
		return Some((cw, grow(ch, -GROW)));
	}
	if t.contains("bigger") || t.contains("larger") {
		return Some((grow(cw, GROW), grow(ch, GROW)));
	}
	if t.contains("smaller") {
		return Some((grow(cw, -GROW), grow(ch, -GROW)));
	}
	None
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn positions_accept_numbers_spots_and_nudges() {
		assert_eq!(parse_position("120, 80", 800, 600, None), Some((120, 80)));
		assert_eq!(parse_position("120 80", 800, 600, None), Some((120, 80)));
		assert_eq!(parse_position("-5, -10", 800, 600, None), Some((-5, -10)));
		assert_eq!(parse_position("top left", 800, 600, None), Some((0, 0)));
		assert_eq!(parse_position("Top Right", 800, 600, None), Some((800, 0)));
		assert_eq!(parse_position("bottom right", 800, 600, None), Some((800, 600)));
		assert_eq!(parse_position("bottom left", 800, 600, None), Some((0, 600)));
		assert_eq!(parse_position("center", 800, 600, None), Some((400, 300)));
		assert_eq!(parse_position("centre", 800, 600, None), Some((400, 300)));
		// A nudge moves from the current position by one NUDGE step.
		assert_eq!(parse_position("a bit right", 800, 600, Some((50, 60))), Some((60, 60)));
		assert_eq!(parse_position("a bit down", 800, 600, Some((50, 60))), Some((50, 70)));
		assert_eq!(parse_position("a little up and left", 800, 600, Some((50, 60))), Some((40, 50)));
		// No current position, no nudge; nonsense is refused, not guessed.
		assert_eq!(parse_position("a bit right", 800, 600, None), None);
		assert_eq!(parse_position("over there", 800, 600, Some((0, 0))), None);
		assert_eq!(parse_position("", 800, 600, None), None);
	}

	#[test]
	fn sizes_accept_numbers_and_relative_words() {
		assert_eq!(parse_size("200 by 100", None), Some((200, 100)));
		assert_eq!(parse_size("200x100", None), Some((200, 100)));
		assert_eq!(parse_size("200, 100", None), Some((200, 100)));
		assert_eq!(parse_size("a bit wider", Some((200, 100))), Some((210, 100)));
		assert_eq!(parse_size("a bit narrower", Some((200, 100))), Some((190, 100)));
		assert_eq!(parse_size("a bit taller", Some((200, 100))), Some((200, 110)));
		assert_eq!(parse_size("a bit shorter", Some((200, 100))), Some((200, 90)));
		assert_eq!(parse_size("a bit bigger", Some((200, 100))), Some((210, 110)));
		assert_eq!(parse_size("a bit smaller", Some((15, 8))), Some((5, 1))); // never below 1
		assert_eq!(parse_size("a bit wider", None), None);
		assert_eq!(parse_size("huge", Some((200, 100))), None);
		assert_eq!(parse_size("0 by 10", None), None);
	}

	#[test]
	fn colors_accept_hex_rgb_and_names() {
		assert_eq!(parse_color("#87CEEB"), Some(Srgb::new(135, 206, 235)));
		assert_eq!(parse_color("FF0000"), Some(Srgb::new(255, 0, 0)));
		assert_eq!(parse_color("rgb(10, 20, 30)"), Some(Srgb::new(10, 20, 30)));
		assert_eq!(parse_color("red"), Some(Srgb::new(255, 0, 0)));
		assert_eq!(parse_color("Sky Blue"), Some(Srgb::new(135, 206, 235)));
		assert_eq!(parse_color("dark gray"), Some(Srgb::new(169, 169, 169)));
		assert_eq!(parse_color("dark grey"), Some(Srgb::new(169, 169, 169)));
		assert_eq!(parse_color("teal"), Some(Srgb::new(0, 128, 128)));
		assert_eq!(parse_color("not a color"), None);
		assert_eq!(parse_color(""), None);
	}
}
