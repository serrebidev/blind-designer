//! variables: replacement variables — {name} tokens written into
//! a design's words now, filled in with real values later.
//!
//! A designer building a template — a birthday card, a form, a
//! certificate — writes "{name}" in a text element, or in any
//! element's caption, and carries on designing. When the words
//! are needed for real, the tokens are filled in from a list of
//! name/value pairs: on a copy (the template stays a template),
//! or in the project itself (the template becomes the finished
//! piece). Either way the substitution is ONE step in the design's
//! log, like every other edit.
//!
//! A token is `{`, one or more of `a-z`, `0-9`, `-`, then `}` —
//! lowercase only, so "{Name}" is ordinary text, not a token, and
//! a lone "{" with no closing brace is ordinary text too. There
//! are no escapes: "{{name}}" reads as the text "{" followed by
//! the token {name} followed by "}". Substitution is a single
//! pass over the original words — a value that itself contains a
//! token is written as-is, never filled in a second time.
//!
//! Tokens live in two places: a text element's phrase, and any
//! element's hand-written caption (generated captions are computed
//! from the design, so they hold no words of the designer's to
//! substitute). Animation frames are snapshots of the design, so
//! they are scanned and substituted too — a design whose frames
//! still said "{name}" after its canvas was filled in would be
//! two different designs in one file.

use std::collections::HashMap;

use crate::design::ShapeKind;
use crate::drawing::{Drawing, Event, Layer};
use crate::project::Project;

/// What a substitution did: how many token occurrences were
/// replaced, how many elements changed (an element whose phrase
/// and caption both held tokens counts once), and the tokens
/// found in the design that no value was given for — left in the
/// words exactly as written, and named here so the caller can
/// say so. Both counts are zero, and `unknown` holds every
/// token, when no values matched at all.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SubstituteReport {
	pub replaced: usize,
	pub affected: usize,
	pub unknown: Vec<String>,
}

/// The token names in one string, in order of appearance,
/// repeats included.
fn tokens_in(text: &str) -> Vec<&str> {
	let bytes = text.as_bytes();
	let mut out = Vec::new();
	let mut i = 0;
	while i < bytes.len() {
		if bytes[i] == b'{' {
			let start = i + 1;
			let mut j = start;
			while j < bytes.len()
				&& (bytes[j].is_ascii_lowercase() || bytes[j].is_ascii_digit() || bytes[j] == b'-')
			{
				j += 1;
			}
			if j > start && j < bytes.len() && bytes[j] == b'}' {
				out.push(&text[start..j]);
				i = j + 1;
				continue;
			}
		}
		i += 1;
	}
	out
}

/// Every token name in the design, unique, in first-appearance
/// order: the canvas's layers in paint order (each element's
/// phrase before its caption), then each animation frame's
/// layers the same way.
fn scan_layers(layers: &[Layer], out: &mut Vec<String>) {
	for layer in layers {
		for shape in &layer.shapes {
			let mut texts: Vec<&str> = Vec::new();
			if shape.kind == ShapeKind::Text {
				texts.push(&shape.text);
			}
			if let Some(caption) = &shape.caption {
				texts.push(caption);
			}
			for text in texts {
				for name in tokens_in(text) {
					if !out.iter().any(|n| n == name) {
						out.push(name.to_string());
					}
				}
			}
		}
	}
}

/// Every replacement-variable name the project uses, unique, in
/// first-appearance order. This is the list the app's dialog
/// offers one value field per name.
pub fn scan(project: &Project) -> Vec<String> {
	let mut out = Vec::new();
	scan_layers(&project.drawing.layers, &mut out);
	for frame in &project.drawing.frames {
		scan_layers(&frame.layers, &mut out);
	}
	out
}

/// One string, substituted: known tokens replaced by their
/// values, unknown tokens left exactly as written and recorded
/// in the report. Returns how many occurrences were replaced.
fn substitute_str(
	text: &str,
	values: &HashMap<&str, &str>,
	unknown: &mut Vec<String>,
) -> (String, usize) {
	if !text.contains('{') {
		return (text.to_string(), 0);
	}
	let mut out = String::with_capacity(text.len());
	let mut rest = text;
	let mut replaced = 0;
	// Walk token by token: copy the plain run before each token,
	// then the value (or the token itself, when unfilled).
	loop {
		let bytes = rest.as_bytes();
		let mut found: Option<(usize, usize, &str)> = None; // (open, after close, name)
		let mut i = 0;
		while i < bytes.len() {
			if bytes[i] == b'{' {
				let start = i + 1;
				let mut j = start;
				while j < bytes.len()
					&& (bytes[j].is_ascii_lowercase()
						|| bytes[j].is_ascii_digit()
						|| bytes[j] == b'-')
				{
					j += 1;
				}
				if j > start && j < bytes.len() && bytes[j] == b'}' {
					found = Some((i, j + 1, &rest[start..j]));
					break;
				}
			}
			i += 1;
		}
		let Some((open, after, name)) = found else {
			out.push_str(rest);
			break;
		};
		out.push_str(&rest[..open]);
		match values.get(name) {
			Some(value) => {
				out.push_str(value);
				replaced += 1;
			}
			None => {
				out.push_str(&rest[open..after]);
				if !unknown.iter().any(|n| n == name) {
					unknown.push(name.to_string());
				}
			}
		}
		rest = &rest[after..];
	}
	(out, replaced)
}

/// Substitute one set of layers in place, counting into the
/// report. An element counts as affected when its phrase or its
/// caption changed.
fn substitute_layers(
	layers: &mut [Layer],
	values: &HashMap<&str, &str>,
	report: &mut SubstituteReport,
) {
	for layer in layers {
		for shape in &mut layer.shapes {
			let mut changed = false;
			if shape.kind == ShapeKind::Text {
				let (new, n) = substitute_str(&shape.text, values, &mut report.unknown);
				if n > 0 {
					shape.text = new;
					report.replaced += n;
					changed = true;
				}
			}
			if let Some(caption) = &shape.caption {
				let (new, n) = substitute_str(caption, values, &mut report.unknown);
				if n > 0 {
					shape.caption = Some(new);
					report.replaced += n;
					changed = true;
				}
			}
			if changed {
				report.affected += 1;
			}
		}
	}
}

impl Drawing {
	/// Fill in this drawing's replacement variables in place:
	/// every token with a value is replaced, in phrases and
	/// captions, on the canvas and in every animation frame;
	/// tokens without a value stay as written and are named in
	/// the report. One event is logged for the whole
	/// substitution — and only when something was replaced, so
	/// a values list that matches nothing leaves the design
	/// (and its history) untouched. If a name is given twice,
	/// the last value wins.
	pub fn substitute_variables(&mut self, values: &[(String, String)]) -> SubstituteReport {
		let map: HashMap<&str, &str> =
			values.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
		let mut report = SubstituteReport::default();
		substitute_layers(&mut self.layers, &map, &mut report);
		for frame in &mut self.frames {
			substitute_layers(&mut frame.layers, &map, &mut report);
		}
		if report.replaced > 0 {
			self.log.push(Event::VariablesSubstituted { count: report.replaced });
		}
		report
	}
}

/// Fill in a project's replacement variables on a COPY: the
/// project handed in is untouched, the one returned has every
/// substitution applied — and the substitution in its log,
/// like any other edit, so the copy's history says how it was
/// finished. This is the template workflow: design once with
/// tokens, print many finished copies.
pub fn substitute(project: &Project, values: &[(String, String)]) -> (Project, SubstituteReport) {
	let mut copy = project.clone();
	let report = copy.drawing.substitute_variables(values);
	(copy, report)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::design::Shape;
	use crate::drawing::Drawing;
	use crate::palette::PaletteColor;
	use crate::project::Project;
	use crate::Srgb;

	fn project_with(words: &[(&str, Option<&str>)]) -> Project {
		// (phrase, caption) per text element, on one layer.
		let mut drawing = Drawing::new(400, 300, Srgb::WHITE);
		for (phrase, caption) in words {
			let mut s = Shape::text(phrase, "Serif", 12, 0, 0, 100, 20, Srgb::BLACK);
			s.caption = caption.map(str::to_string);
			drawing.add_shape(0, s);
		}
		Project {
			drawing,
			palette: vec![PaletteColor::typed(Srgb::BLACK)],
			pinned_sliders: Vec::new(),
			slider_target: crate::sliders::AdjustTarget::BaseColor,
			page_jump_percent: 10,
			goal: None,
			grayscale_start: false,
			interaction_mode: crate::project::InteractionMode::Type,
		}
	}

	fn pairs(values: &[(&str, &str)]) -> Vec<(String, String)> {
		values.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
	}

	#[test]
	fn scan_orders_by_first_appearance_and_dedups() {
		let p = project_with(&[
			("Dear {name}, happy {day}!", None),
			("For {name} again", Some("Card for {name} on {day}")),
			("No tokens here", None),
		]);
		assert_eq!(scan(&p), vec!["name".to_string(), "day".to_string()]);
	}

	#[test]
	fn scan_ignores_non_tokens() {
		// Uppercase, spaces, empty braces and a lone brace are
		// ordinary text, not tokens.
		let p = project_with(&[("{Name} {two words} {} {unclosed and {ok-1}", None)]);
		assert_eq!(scan(&p), vec!["ok-1".to_string()]);
	}

	#[test]
	fn scan_of_tokenless_project_is_empty() {
		let p = project_with(&[("Plain words.", Some("A plain caption."))]);
		assert!(scan(&p).is_empty());
	}

	#[test]
	fn substitute_fills_phrases_and_captions_and_reports_unknowns() {
		let p = project_with(&[
			("Dear {name}, happy {day}!", None),
			("Static", Some("Card for {name}, ref {code}")),
		]);
		let (done, report) = substitute(&p, &pairs(&[("name", "Elshara"), ("day", "Tuesday")]));
		assert_eq!(report.replaced, 3);
		assert_eq!(report.affected, 2);
		assert_eq!(report.unknown, vec!["code".to_string()]);
		let shapes = &done.drawing.layers[0].shapes;
		assert_eq!(shapes[0].text, "Dear Elshara, happy Tuesday!");
		assert_eq!(shapes[1].caption.as_deref(), Some("Card for Elshara, ref {code}"));
		// The original project is untouched, and the copy's log
		// records the substitution as one step.
		assert_eq!(p.drawing.layers[0].shapes[0].text, "Dear {name}, happy {day}!");
		assert!(matches!(
			done.drawing.log.last(),
			Some(Event::VariablesSubstituted { count: 3 })
		));
	}

	#[test]
	fn substitute_with_no_matching_values_changes_nothing() {
		let p = project_with(&[("Dear {name}!", None)]);
		let log_len = p.drawing.log.len();
		let (done, report) = substitute(&p, &pairs(&[("other", "x")]));
		assert_eq!(report.replaced, 0);
		assert_eq!(report.affected, 0);
		assert_eq!(report.unknown, vec!["name".to_string()]);
		assert_eq!(done.drawing.layers[0].shapes[0].text, "Dear {name}!");
		// No event: a substitution that substituted nothing
		// is not a step in the design's history.
		assert_eq!(done.drawing.log.len(), log_len);
	}

	#[test]
	fn substitute_is_a_single_pass() {
		// A value containing a token is written as-is, not
		// filled in a second time.
		let p = project_with(&[("{a}", None)]);
		let (done, report) = substitute(&p, &pairs(&[("a", "{b}"), ("b", "second")]));
		assert_eq!(report.replaced, 1);
		assert_eq!(done.drawing.layers[0].shapes[0].text, "{b}");
	}

	#[test]
	fn substitute_reaches_animation_frames() {
		let mut p = project_with(&[("Hi {name}", None)]);
		p.drawing.add_frame();
		let (done, report) = substitute(&p, &pairs(&[("name", "Jo")]));
		assert_eq!(report.replaced, 2); // canvas + frame snapshot
		assert_eq!(done.drawing.frames[0].layers[0].shapes[0].text, "Hi Jo");
	}

	#[test]
	fn double_brace_is_text_around_a_token() {
		// "{{name}}" = "{" + the token {name} + "}".
		let p = project_with(&[("{{name}}", None)]);
		assert_eq!(scan(&p), vec!["name".to_string()]);
		let (done, _) = substitute(&p, &pairs(&[("name", "Jo")]));
		assert_eq!(done.drawing.layers[0].shapes[0].text, "{Jo}");
	}
}
