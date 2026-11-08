//! search: one search over everything a design can gain.
//!
//! Elshara's ask: type a word, find the thing — a palette color
//! by its name or its hex, an element by its caption, a symbol,
//! a template — and put it where the cursor is. This module is
//! the finding half: it searches the project's palette, the
//! starter colors, the project's own elements (so one can be
//! copied to the cursor), the symbol library and the templates,
//! and returns labeled hits in that order. The putting half —
//! what inserting each kind of hit means — lives in the app,
//! which owns the dialogs; the hit carries everything the app
//! needs to do it.
//!
//! Matching is a plain case-insensitive substring test over
//! each candidate's spoken text: a color matches its words
//! ("sky blue"), its hex with or without the # ("3a7c"), an
//! element matches its caption, its kind word and its plain
//! description. An empty query matches everything: the dialog
//! opens as a browser, not a blank stare.

use patois::t;

use crate::palette::starter_palette;
use crate::project::Project;
use crate::symbols::SYMBOL_NAMES;
use crate::templates::{PATTERN_NAMES, TEMPLATE_NAMES};
use crate::{color_words, describe, ColorFormat, Srgb};

/// One thing the search found. The label is the row the results
/// list speaks; the payload is what inserting the hit needs.
#[derive(Debug, Clone, PartialEq)]
pub enum CatalogHit {
	/// A color already in the project's palette, by index.
	PaletteColor { index: usize, label: String },
	/// A starter color the project does not necessarily hold.
	StarterColor { color: Srgb, label: String },
	/// An element already in the design, by (layer, shape):
	/// inserting it copies it to the cursor.
	Element {
		layer: usize,
		index: usize,
		label: String,
	},
	/// A symbol from the library, by name.
	Symbol { name: &'static str, label: String },
	/// A whole-design template, by name.
	Template { name: &'static str, label: String },
	/// A two-color pattern template, by name.
	Pattern { name: &'static str, label: String },
}

impl CatalogHit {
	pub fn label(&self) -> &str {
		match self {
			CatalogHit::PaletteColor { label, .. }
			| CatalogHit::StarterColor { label, .. }
			| CatalogHit::Element { label, .. }
			| CatalogHit::Symbol { label, .. }
			| CatalogHit::Template { label, .. }
			| CatalogHit::Pattern { label, .. } => label,
		}
	}
}

fn hits(query: &str, text: &str) -> bool {
	query.is_empty() || text.to_lowercase().contains(query)
}

/// Everything in the project and the libraries that matches
/// `query`, in catalog order: palette colors, starter colors,
/// elements, symbols, templates, patterns.
pub fn search_catalog(project: &Project, query: &str) -> Vec<CatalogHit> {
	let q = query.trim().to_lowercase();
	let mut out = Vec::new();
	// Palette colors: by their words, their hex in any dress,
	// and where they came from.
	for (i, pc) in project.palette.iter().enumerate() {
		let text = format!(
			"{} {} {} {}",
			describe(pc.color),
			color_words(pc.color),
			pc.color.to_hex(),
			pc.provenance.spoken()
		);
		if hits(&q, &text) {
			out.push(CatalogHit::PaletteColor {
				index: i,
				label: t("Palette color: %s").replacen(
					"%s",
					&(pc.spoken(ColorFormat::Hex)).to_string(),
					1,
				),
			});
		}
	}
	// The starter colors, by the same words.
	for pc in starter_palette() {
		let text = t("starter %s %s")
			.replacen("%s", &(describe(pc.color)).to_string(), 1)
			.replacen("%s", &(pc.color.to_hex()).to_string(), 1);
		if hits(&q, &text) {
			out.push(CatalogHit::StarterColor {
				color: pc.color,
				label: t("Starter color: %s").replacen(
					"%s",
					&(pc.spoken(ColorFormat::Hex)).to_string(),
					1,
				),
			});
		}
	}
	// The design's own elements: caption, kind, description,
	// layer.
	for (li, layer) in project.drawing.layers.iter().enumerate() {
		for (si, shape) in layer.shapes.iter().enumerate() {
			let plain = project.drawing.plain_of(shape);
			let text = format!(
				"{} {} {} {}",
				shape.kind.name(),
				plain,
				shape.caption.clone().unwrap_or_default(),
				layer.name
			);
			if hits(&q, &text) {
				let mut label = t("Element: %s: %s")
					.replacen("%s", &(layer.name).to_string(), 1)
					.replacen("%s", &(plain).to_string(), 1);
				if let Some(c) = &shape.caption {
					label.push_str(&t(" Caption: %s").replacen("%s", &(c).to_string(), 1));
				}
				out.push(CatalogHit::Element {
					layer: li,
					index: si,
					label,
				});
			}
		}
	}
	// Symbols, templates, patterns: by name.
	for name in SYMBOL_NAMES {
		if hits(&q, &t("%s symbol").replacen("%s", (name).as_ref(), 1)) {
			out.push(CatalogHit::Symbol {
				name,
				label: t("Symbol: %s").replacen("%s", (name).as_ref(), 1),
			});
		}
	}
	for name in TEMPLATE_NAMES {
		if hits(&q, &t("%s template").replacen("%s", (name).as_ref(), 1)) {
			out.push(CatalogHit::Template {
				name,
				label: t("Template: %s").replacen("%s", (name).as_ref(), 1),
			});
		}
	}
	for name in PATTERN_NAMES {
		if hits(&q, &t("%s pattern").replacen("%s", (name).as_ref(), 1)) {
			out.push(CatalogHit::Pattern {
				name,
				label: t("Pattern: %s").replacen("%s", (name).as_ref(), 1),
			});
		}
	}
	out
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::design::{Shape, ShapeKind};
	use crate::drawing::Drawing;
	use crate::palette::PaletteColor;
	use crate::project::DEFAULT_PAGE_JUMP_PERCENT;
	use crate::sliders::AdjustTarget;

	fn project() -> Project {
		let mut d = Drawing::new(800, 600, Srgb::WHITE);
		let mut lamp = Shape::new(ShapeKind::Ellipse, 100, 100, 40, 60, Srgb::new(255, 205, 0));
		lamp.caption = Some("the lighthouse lamp".to_string());
		d.add_shape(0, lamp);
		d.add_shape(
			0,
			Shape::new(
				ShapeKind::Rectangle,
				300,
				300,
				80,
				80,
				Srgb::new(58, 124, 165),
			),
		);
		Project {
			drawing: d,
			palette: vec![PaletteColor::typed(Srgb::new(58, 124, 165))],
			pinned_sliders: Vec::new(),
			slider_target: AdjustTarget::BaseColor,
			page_jump_percent: DEFAULT_PAGE_JUMP_PERCENT,
			goal: None,
			grayscale_start: false,
			interaction_mode: crate::project::InteractionMode::Type,
		}
	}

	#[test]
	fn finds_colors_by_word_and_hex_fragment() {
		let p = project();
		// By a word of the color's description.
		let words = color_words(Srgb::new(58, 124, 165));
		let first = words.split_whitespace().last().unwrap().to_lowercase();
		let found = search_catalog(&p, &first);
		assert!(
			found
				.iter()
				.any(|h| matches!(h, CatalogHit::PaletteColor { index: 0, .. })),
			"word {first:?} in {found:?}"
		);
		// By a hex fragment, without the # and in lower case.
		let found = search_catalog(&p, "3a7c");
		assert!(
			found
				.iter()
				.any(|h| matches!(h, CatalogHit::PaletteColor { index: 0, .. })),
			"{found:?}"
		);
		// The starter blue is the same color: it is found too,
		// as a starter color, and can still be inserted.
		assert!(
			found
				.iter()
				.any(|h| matches!(h, CatalogHit::StarterColor { .. })),
			"{found:?}"
		);
	}

	#[test]
	fn finds_elements_by_caption_kind_and_layer() {
		let p = project();
		let found = search_catalog(&p, "lighthouse");
		assert_eq!(found.len(), 1, "{found:?}");
		assert!(matches!(
			found[0],
			CatalogHit::Element {
				layer: 0,
				index: 0,
				..
			}
		));
		// By kind word: both shapes are found, the ellipse first.
		let found = search_catalog(&p, "rectangle");
		assert!(
			found
				.iter()
				.any(|h| matches!(h, CatalogHit::Element { index: 1, .. })),
			"{found:?}"
		);
		let found = search_catalog(&p, "layer 1");
		assert_eq!(
			found
				.iter()
				.filter(|h| matches!(h, CatalogHit::Element { .. }))
				.count(),
			2,
			"{found:?}"
		);
	}

	#[test]
	fn finds_symbols_templates_and_patterns_by_name() {
		let p = project();
		// "star" also matches the starter colors' text ("starter"
		// contains it) — substring matching is honest about that;
		// the symbol is among the hits either way.
		assert!(search_catalog(&p, "star")
			.iter()
			.any(|h| matches!(h, CatalogHit::Symbol { name: "star", .. })));
		assert!(matches!(
			search_catalog(&p, "star symbol").as_slice(),
			[CatalogHit::Symbol { name: "star", .. }]
		));
		assert!(matches!(
			search_catalog(&p, "portrait").as_slice(),
			[CatalogHit::Template {
				name: "Portrait",
				..
			}]
		));
		assert!(matches!(
			search_catalog(&p, "checker").as_slice(),
			[CatalogHit::Pattern {
				name: "Checkerboard",
				..
			}]
		));
	}

	#[test]
	fn an_empty_query_browses_the_whole_catalog() {
		let p = project();
		let found = search_catalog(&p, "  ");
		// 1 palette color + 1 starter + 2 elements + 7 symbols
		// + 6 templates + 3 patterns.
		assert_eq!(found.len(), 1 + 1 + 2 + 7 + 6 + 3, "{found:?}");
		assert!(matches!(found[0], CatalogHit::PaletteColor { .. }));
		assert!(search_catalog(&p, "no such thing anywhere").is_empty());
	}
}
