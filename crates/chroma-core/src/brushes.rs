//! brushes: the saved-brush library.
//!
//! A stroke's brush is three things: its width in pixels, its color,
//! and the name of its stamp pattern. A brush the designer likes can
//! be saved under a name and applied to new strokes, so a favorite
//! pen is one choice instead of three settings re-typed every time.
//! The library persists as a small text file in the same hand-rolled
//! line format as the project file; the app keeps it beside the
//! executable.

use patois::t;

use crate::project::{quote, tokens};
use crate::Srgb;

/// The stamp patterns a stroke can be painted with.
pub const PATTERNS: [&str; 3] = ["solid", "dashed", "dotted"];

/// One saved brush: width, color and stamp pattern under a name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Brush {
	pub name: String,
	pub width: u32,
	pub color: Srgb,
	pub pattern: String,
}

impl Brush {
	pub fn new(name: &str, width: u32, color: Srgb, pattern: &str) -> Brush {
		Brush {
			name: name.to_string(),
			width: width.max(1),
			color,
			pattern: if PATTERNS.contains(&pattern) {
				pattern.to_string()
			} else {
				"solid".to_string()
			},
		}
	}

	/// The brush in one spoken line, for the library list.
	pub fn spoken(&self) -> String {
		t("%s: %s pixel brush, %s, %s pattern")
			.replacen("%s", &(self.name).to_string(), 1)
			.replacen("%s", &(self.width).to_string(), 1)
			.replacen("%s", &(crate::color_words(self.color)).to_string(), 1)
			.replacen("%s", &(self.pattern).to_string(), 1)
	}
}

/// The named brushes, in the order they were saved.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BrushLibrary {
	pub brushes: Vec<Brush>,
}

impl BrushLibrary {
	/// Save a brush: an existing brush with the same name (any case)
	/// is replaced, so re-saving updates a favorite in place.
	pub fn save(&mut self, brush: Brush) {
		if let Some(old) = self
			.brushes
			.iter_mut()
			.find(|b| b.name.eq_ignore_ascii_case(&brush.name))
		{
			*old = brush;
			return;
		}
		self.brushes.push(brush);
	}

	pub fn get(&self, name: &str) -> Option<&Brush> {
		self.brushes
			.iter()
			.find(|b| b.name.eq_ignore_ascii_case(name))
	}

	pub fn remove(&mut self, name: &str) -> bool {
		let before = self.brushes.len();
		self.brushes.retain(|b| !b.name.eq_ignore_ascii_case(name));
		self.brushes.len() != before
	}

	/// The whole library as file text.
	pub fn to_text(&self) -> String {
		let mut out = String::from("blind-designer-brushes 1\n");
		for b in &self.brushes {
			out.push_str(
				&t("brush %s %s %s %s\n")
					.replacen("%s", &(quote(&b.name)).to_string(), 1)
					.replacen("%s", &(b.width).to_string(), 1)
					.replacen("%s", &(b.color.to_hex()).to_string(), 1)
					.replacen("%s", &(b.pattern).to_string(), 1),
			);
		}
		out.push_str("end\n");
		out
	}

	/// Read a library file back. Malformed input is an `Err`, never a
	/// half-loaded library.
	pub fn from_text(text: &str) -> Result<BrushLibrary, String> {
		let mut lines = text.lines();
		let head = lines.next().ok_or(t("empty brush file"))?;
		if tokens(head)?.join(" ") != "blind-designer-brushes 1" {
			return Err(t("not a Blind Designer brush file"));
		}
		let mut lib = BrushLibrary::default();
		for line in lines {
			if line.trim().is_empty() {
				continue;
			}
			let toks = tokens(line)?;
			match toks.first().map(String::as_str) {
				Some("end") => return Ok(lib),
				Some("brush") => {
					if toks.len() != 5 {
						return Err(t("a brush needs 5 fields, got %s").replacen(
							"%s",
							&(toks.len()).to_string(),
							1,
						));
					}
					let width: u32 = toks[2].parse().map_err(|_| {
						t("bad brush width: %s").replacen("%s", &(toks[2]).to_string(), 1)
					})?;
					let color = Srgb::from_hex(&toks[3]).ok_or_else(|| {
						t("bad brush color: %s").replacen("%s", &(toks[3]).to_string(), 1)
					})?;
					lib.save(Brush::new(&toks[1], width, color, &toks[4]));
				}
				_ => {
					return Err(t("unknown line in brush file: %s").replacen(
						"%s",
						line,
						1,
					))
				}
			}
		}
		Err(t("brush file ends before the end marker"))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn ink() -> Srgb {
		Srgb::new(15, 94, 104)
	}

	#[test]
	fn save_get_remove_and_replace() {
		let mut lib = BrushLibrary::default();
		lib.save(Brush::new("Fine pen", 2, ink(), "solid"));
		lib.save(Brush::new(
			"Sky writer",
			9,
			Srgb::new(135, 206, 235),
			"dashed",
		));
		assert_eq!(lib.brushes.len(), 2);
		assert_eq!(lib.get("fine PEN").unwrap().width, 2);
		assert_eq!(lib.get("Sky writer").unwrap().pattern, "dashed");
		// Re-saving under the same name updates in place, no duplicate.
		lib.save(Brush::new("Fine pen", 4, ink(), "dotted"));
		assert_eq!(lib.brushes.len(), 2);
		assert_eq!(lib.get("Fine pen").unwrap().width, 4);
		assert!(lib.remove("sky writer"));
		assert!(!lib.remove("sky writer"));
		assert_eq!(lib.brushes.len(), 1);
	}

	#[test]
	fn unknown_patterns_fall_back_to_solid() {
		assert_eq!(Brush::new("x", 3, ink(), "zigzag").pattern, "solid");
		assert_eq!(Brush::new("x", 0, ink(), "solid").width, 1);
	}

	#[test]
	fn the_library_round_trips_as_text() {
		let mut lib = BrushLibrary::default();
		lib.save(Brush::new("Fine \"quoted\" pen", 2, ink(), "solid"));
		lib.save(Brush::new(
			"Sky writer",
			9,
			Srgb::new(135, 206, 235),
			"dashed",
		));
		let back = BrushLibrary::from_text(&lib.to_text()).unwrap();
		assert_eq!(back, lib);
		assert_eq!(back.to_text(), lib.to_text());
		assert!(BrushLibrary::from_text("hello").is_err());
		assert!(BrushLibrary::from_text(
			"blind-designer-brushes 1\nbrush \"x\" wide #000000 solid\nend\n"
		)
		.is_err());
		assert!(BrushLibrary::from_text("blind-designer-brushes 1\n").is_err());
		// An empty library is a valid file.
		assert_eq!(
			BrushLibrary::from_text(&BrushLibrary::default().to_text())
				.unwrap()
				.brushes
				.len(),
			0
		);
	}

	#[test]
	fn a_saved_brush_applies_its_three_settings() {
		// Apply = take the brush's width, color and pattern as the
		// current stroke settings, exactly as stored.
		let mut lib = BrushLibrary::default();
		lib.save(Brush::new(
			"Sky writer",
			9,
			Srgb::new(135, 206, 235),
			"dashed",
		));
		let applied = lib.get("Sky writer").unwrap().clone();
		assert_eq!(
			(applied.width, applied.color, applied.pattern.as_str()),
			(9, Srgb::new(135, 206, 235), "dashed")
		);
		assert!(applied.spoken().contains("Sky writer"));
		assert!(applied.spoken().contains("9 pixel brush"));
		assert!(applied.spoken().contains("dashed pattern"));
	}
}
