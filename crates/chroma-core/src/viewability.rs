//! viewability: how a design reads from one viewpoint.
//!
//! Given a viewing angle (where the viewer stands: 0 degrees is
//! straight on from the front; an element's own rotation is
//! measured the same way) and a depth band, the report walks
//! every element on a visible layer, in paint order, and says
//! one of four things about it: it is viewable, it is partly
//! hidden or hidden behind a named element painted over it, it
//! is turned away at so many degrees, or it sits outside the
//! chosen depth.
//!
//! The maths is honest box maths: each element is its bounding
//! box, occlusion is the share of the box a later element's box
//! covers, and facing compares two angles. Curves, holes and
//! see-through parts are not taken into account — the report
//! says so, in its own header.

use crate::design::Shape;
use crate::drawing::{DepthRole, Drawing};

/// The depth a viewability report covers: every depth, or one
/// role's band of the scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthBand {
	All,
	Background,
	Midground,
	Foreground,
}

impl DepthBand {
	/// Every band, in chooser order.
	pub const ALL: [DepthBand; 4] =
		[DepthBand::All, DepthBand::Background, DepthBand::Midground, DepthBand::Foreground];

	/// The spoken name the chooser shows.
	pub fn name(&self) -> &'static str {
		match self {
			DepthBand::All => "All depths",
			DepthBand::Background => "Background only",
			DepthBand::Midground => "Midground only",
			DepthBand::Foreground => "Foreground only",
		}
	}

	/// The role this band is limited to; `None` for all depths.
	pub fn role(&self) -> Option<DepthRole> {
		match self {
			DepthBand::All => None,
			DepthBand::Background => Some(DepthRole::Background),
			DepthBand::Midground => Some(DepthRole::Midground),
			DepthBand::Foreground => Some(DepthRole::Foreground),
		}
	}

	/// Does this band cover a layer with this role?
	pub fn includes(&self, role: DepthRole) -> bool {
		self.role().map_or(true, |r| r == role)
	}
}

/// The smallest angle between two headings, in degrees
/// (0..=180): rotation wraps, so 350 and 10 are 20 apart.
pub fn angle_difference(a: f64, b: f64) -> f64 {
	let d = (a - b).rem_euclid(360.0);
	d.min(360.0 - d)
}

/// The box an element occupies for viewability: strokes use the
/// span of their points (at least the brush wide), everything
/// else its own x, y, w, h. (x0, y0, x1, y1).
fn element_box(shape: &Shape) -> (i32, i32, i32, i32) {
	let w = shape.w.max(shape.brush.max(1)) as i32;
	let h = shape.h.max(shape.brush.max(1)) as i32;
	(shape.x, shape.y, shape.x + w, shape.y + h)
}

/// The share of `covered`'s box that `coverer`'s box overlaps:
/// 0.0 when they are apart, 1.0 when the coverer contains it.
pub fn coverage_fraction(coverer: (i32, i32, i32, i32), covered: (i32, i32, i32, i32)) -> f64 {
	let area = (covered.2 - covered.0).max(0) as f64 * (covered.3 - covered.1).max(0) as f64;
	if area <= 0.0 {
		return 0.0;
	}
	let ix = (coverer.2.min(covered.2) - coverer.0.max(covered.0)).max(0) as f64;
	let iy = (coverer.3.min(covered.3) - coverer.1.max(covered.1)).max(0) as f64;
	ix * iy / area
}

/// At or above this share of its box covered by one element
/// painted over it, an element counts as partly hidden; at or
/// above the higher line it counts as hidden outright.
pub const PARTLY_HIDDEN_AT: f64 = 0.60;
pub const HIDDEN_AT: f64 = 0.95;
/// Turned more than this many degrees away from the viewer, an
/// element shows its back: it counts as turned away. Exactly
/// side-on (90) still counts as facing — an edge is something
/// to see.
pub const TURNED_AWAY_PAST: f64 = 90.0;

/// One element of the report, as the walk collects it: the
/// shape, the layer it sits on, and that layer's depth role.
struct Subject<'a> {
	shape: &'a Shape,
	layer_name: &'a str,
	role: DepthRole,
}

impl Drawing {
	/// The viewability report: from `view_angle` degrees (0 is
	/// straight on from the front; elements turned more than 90
	/// degrees away from it show their backs), covering `band`.
	/// Every element on a visible layer gets one line, in paint
	/// order, in plain words.
	pub fn viewability_report(&self, view_angle: f64, band: DepthBand) -> String {
		let view_angle = view_angle.rem_euclid(360.0);
		let band_words = match band.role() {
			None => "all depths".to_string(),
			Some(role) => format!("the {} only", role.name()),
		};
		let mut out = format!(
			"Viewability from {view_angle:.0} degrees, {band_words}.\n\
			 Judged by each element's box and rotation: elements count as their \
			 boxes, so curves, holes and see-through parts are not taken into \
			 account.\n"
		);
		let mut subjects: Vec<Subject> = Vec::new();
		for layer in &self.layers {
			if !layer.visible {
				continue;
			}
			for shape in &layer.shapes {
				subjects.push(Subject {
					shape,
					layer_name: &layer.name,
					role: layer.depth,
				});
			}
		}
		if subjects.is_empty() {
			out.push_str("There are no elements on visible layers to check.\n");
			return out;
		}
		let (mut viewable, mut hidden_count, mut turned, mut outside) = (0, 0, 0, 0);
		for (i, subject) in subjects.iter().enumerate() {
			let shape = subject.shape;
			let depth_note = if subject.role == DepthRole::Unassigned {
				"no depth role".to_string()
			} else {
				subject.role.name().to_string()
			};
			let head = format!(
				"- {} (layer \"{}\", {})",
				self.plain_of(shape),
				subject.layer_name,
				depth_note
			);
			if !band.includes(subject.role) {
				outside += 1;
				out.push_str(&format!("{head}: outside the chosen depth.\n"));
				continue;
			}
			let turned_by = angle_difference(shape.angle, view_angle);
			if turned_by > TURNED_AWAY_PAST {
				turned += 1;
				out.push_str(&format!(
					"{head}: turned away at {turned_by:.0} degrees.\n"
				));
				continue;
			}
			// The strongest single cover among the elements
			// painted after this one.
			let my_box = element_box(shape);
			let mut strongest: Option<(&Shape, f64)> = None;
			for later in &subjects[i + 1..] {
				let cover = coverage_fraction(element_box(later.shape), my_box);
				if strongest.map_or(true, |(_, best)| cover > best) {
					strongest = Some((later.shape, cover));
				}
			}
			match strongest {
				Some((coverer, cover)) if cover >= HIDDEN_AT => {
					hidden_count += 1;
					out.push_str(&format!(
						"{head}: hidden behind {}.\n",
						self.plain_of(coverer)
					));
				}
				Some((coverer, cover)) if cover >= PARTLY_HIDDEN_AT => {
					hidden_count += 1;
					out.push_str(&format!(
						"{head}: partly hidden behind {} (about {:.0} percent covered).\n",
						self.plain_of(coverer),
						cover * 100.0
					));
				}
				_ => {
					viewable += 1;
					out.push_str(&format!("{head}: viewable.\n"));
				}
			}
		}
		out.push_str(&format!(
			"{} elements: {} viewable, {} hidden or partly hidden, {} turned away, {} outside the chosen depth.\n",
			subjects.len(),
			viewable,
			hidden_count,
			turned,
			outside
		));
		out
	}

	/// One element's stack: what is painted over it, what it is
	/// painted over, and what sits beneath it all — the element
	/// wizard's first step. The element is named by its flat
	/// index (the elements list's numbering, over every layer).
	/// Only visible layers paint, so only they stack; an element
	/// on a hidden layer gets that said about it instead. The
	/// maths is the same honest box maths as the report above:
	/// overlaps are box overlaps, in paint order.
	pub fn stack_report(&self, flat: usize) -> Option<String> {
		// Locate the element over every layer, hidden included.
		let (mut rest, mut target) = (flat, None);
		for (li, layer) in self.layers.iter().enumerate() {
			if rest < layer.shapes.len() {
				target = Some((li, rest));
				break;
			}
			rest -= layer.shapes.len();
		}
		let (tli, tsi) = target?;
		let tlayer = &self.layers[tli];
		let tshape = &tlayer.shapes[tsi];
		let head = format!(
			"{} on layer \"{}\".\n",
			self.plain_of(tshape),
			tlayer.name
		);
		if !tlayer.visible {
			return Some(format!(
				"{head}Layer \"{}\" is hidden, so this element does not paint: nothing is stacked over or under it until the layer is shown.\n",
				tlayer.name
			));
		}
		// The painting stack: visible layers' elements in paint
		// order, with the target's place in it.
		let mut stack: Vec<&Shape> = Vec::new();
		let mut at = 0usize;
		for layer in &self.layers {
			if !layer.visible {
				continue;
			}
			for shape in &layer.shapes {
				if std::ptr::eq(shape, tshape) {
					at = stack.len();
				}
				stack.push(shape);
			}
		}
		let my_box = element_box(tshape);
		let mut over: Vec<String> = Vec::new();
		for later in &stack[at + 1..] {
			let cover = coverage_fraction(element_box(later), my_box);
			if cover > 0.0 {
				over.push(format!(
					"{} (covers about {:.0} percent of it)",
					self.plain_of(later),
					cover * 100.0
				));
			}
		}
		let mut under: Vec<String> = Vec::new();
		for earlier in &stack[..at] {
			let covered = coverage_fraction(my_box, element_box(earlier));
			if covered > 0.0 {
				under.push(format!(
					"{} (it covers about {:.0} percent of that)",
					self.plain_of(earlier),
					covered * 100.0
				));
			}
		}
		let mut out = head;
		if over.is_empty() {
			out.push_str("Painted over it: nothing — it is on top of everything it touches.\n");
		} else {
			out.push_str(&format!("Painted over it: {}.\n", over.join("; ")));
		}
		if under.is_empty() {
			out.push_str(
				"Painted under it: nothing — beneath it is the design background itself.\n",
			);
		} else {
			out.push_str(&format!("Painted under it: {}.\n", under.join("; ")));
		}
		out.push_str(&format!(
			"It is element {} of {} in the painting order.\n",
			at + 1,
			stack.len()
		));
		Some(out)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::design::ShapeKind;
	use crate::Srgb;

	fn rect(x: i32, y: i32, w: u32, h: u32) -> Shape {
		Shape::new(ShapeKind::Rectangle, x, y, w, h, Srgb::new(200, 40, 30))
	}

	#[test]
	fn angle_difference_wraps_and_caps() {
		assert_eq!(angle_difference(0.0, 0.0), 0.0);
		assert_eq!(angle_difference(350.0, 10.0), 20.0);
		assert_eq!(angle_difference(10.0, 350.0), 20.0);
		assert_eq!(angle_difference(0.0, 180.0), 180.0);
		assert_eq!(angle_difference(-45.0, 45.0), 90.0);
		assert_eq!(angle_difference(720.0, 0.0), 0.0);
	}

	#[test]
	fn coverage_counts_the_covered_share() {
		let big = (0, 0, 100, 100);
		assert_eq!(coverage_fraction(big, big), 1.0);
		assert_eq!(coverage_fraction((200, 200, 300, 300), big), 0.0);
		// A quarter of the box overlapped.
		assert!((coverage_fraction((50, 50, 150, 150), big) - 0.25).abs() < 1e-9);
		// A coverer smaller than the covered: only its own area counts.
		assert!((coverage_fraction((25, 25, 75, 75), big) - 0.25).abs() < 1e-9);
		assert_eq!(coverage_fraction(big, (10, 10, 10, 10)), 0.0, "a zero-area box is uncoverable");
	}

	#[test]
	fn facing_thresholds_and_wrap() {
		let mut d = Drawing::new(200, 200, Srgb::WHITE);
		let front = d.add_layer("Front");
		d.add_shape(front, rect(10, 10, 20, 20));
		d.add_shape(front, rect(100, 10, 20, 20));
		d.rotate_shape(front, 1, 95.0);
		d.add_shape(front, rect(10, 100, 20, 20));
		d.rotate_shape(front, 2, 350.0);
		d.add_shape(front, rect(100, 100, 20, 20));
		d.rotate_shape(front, 3, 90.0);
		let report = d.viewability_report(0.0, DepthBand::All);
		// Straight on: viewable. 95 degrees: turned away at 95.
		// 350 degrees is only 10 off straight on: viewable.
		// Exactly 90 (side-on) still counts as facing.
		assert_eq!(report.matches("viewable.").count(), 3, "{report}");
		assert!(report.contains("turned away at 95 degrees"), "{report}");
		// From the back (180), the straight-on element is the
		// turned-away one and the 95-degree one nearly faces.
		let behind = d.viewability_report(180.0, DepthBand::All);
		assert!(behind.contains("turned away at 180 degrees"), "{behind}");
	}

	#[test]
	fn occlusion_needs_sixty_percent_from_what_paints_after() {
		let mut d = Drawing::new(200, 200, Srgb::WHITE);
		// Buried: covered exactly by a twin on the layer above.
		d.add_shape(0, rect(0, 0, 100, 100));
		let top = d.add_layer("Top");
		d.add_shape(top, rect(0, 0, 100, 100));
		// Partly covered: 70 of its 100 columns lie under the twin.
		d.add_shape(0, rect(0, 110, 100, 50));
		d.add_shape(top, rect(30, 110, 100, 50));
		// Barely touched: only half under a later shape on its own
		// layer — still viewable.
		d.add_shape(0, rect(110, 0, 80, 40));
		d.add_shape(0, rect(150, 0, 80, 40));
		let report = d.viewability_report(0.0, DepthBand::All);
		assert!(report.contains("hidden behind"), "{report}");
		assert!(report.contains("partly hidden behind"), "{report}");
		assert!(report.contains("(about 70 percent covered)"), "{report}");
		// Both top-layer shapes and both barely-touched shapes
		// read viewable; nothing that paints earlier hides anything.
		assert_eq!(report.matches("viewable.").count(), 4, "{report}");
	}

	#[test]
	fn the_depth_band_leaves_other_depths_outside() {
		let mut d = Drawing::new(200, 200, Srgb::WHITE);
		d.set_layer_depth(0, DepthRole::Background);
		d.add_shape(0, rect(0, 0, 50, 50));
		let front = d.add_layer("Front");
		d.set_layer_depth(front, DepthRole::Foreground);
		d.add_shape(front, rect(100, 100, 50, 50));
		let report = d.viewability_report(0.0, DepthBand::Foreground);
		assert!(report.contains("outside the chosen depth."), "{report}");
		assert!(report.contains("1 viewable"), "{report}");
		assert!(report.contains("the foreground only"), "{report}");
		// All depths covers both.
		let all = d.viewability_report(0.0, DepthBand::All);
		assert_eq!(all.matches("viewable.").count(), 2, "{all}");
		// Band membership is by role: unassigned is no band's member.
		assert!(!DepthBand::Background.includes(DepthRole::Unassigned));
		assert!(DepthBand::All.includes(DepthRole::Unassigned));
	}

	#[test]
	fn stack_report_names_over_and_under() {
		let mut d = Drawing::new(200, 200, Srgb::WHITE);
		d.add_shape(0, rect(0, 0, 100, 100)); // bottom
		d.add_shape(0, rect(0, 0, 100, 100)); // middle: fully covered, fully covering
		d.add_shape(0, rect(0, 0, 50, 100)); // top: covers half of those below
		let mid = d.stack_report(1).unwrap();
		assert!(mid.contains("covers about 50 percent of it"), "{mid}");
		assert!(mid.contains("it covers about 100 percent of that"), "{mid}");
		assert!(mid.contains("element 2 of 3"), "{mid}");
		let bottom = d.stack_report(0).unwrap();
		assert!(bottom.contains("Painted under it: nothing"), "{bottom}");
		let top = d.stack_report(2).unwrap();
		assert!(top.contains("Painted over it: nothing"), "{top}");
		assert!(d.stack_report(3).is_none());
		// A hidden layer's element does not stack at all.
		d.set_layer_visible(0, false);
		let hidden = d.stack_report(0).unwrap();
		assert!(hidden.contains("does not paint"), "{hidden}");
	}

	#[test]
	fn hidden_layers_and_empty_designs_are_honest() {
		let mut d = Drawing::new(200, 200, Srgb::WHITE);
		let report = d.viewability_report(45.0, DepthBand::All);
		assert!(report.contains("no elements on visible layers"), "{report}");
		d.add_shape(0, rect(0, 0, 50, 50));
		d.add_shape(0, rect(0, 0, 50, 50));
		d.set_layer_visible(0, false);
		let report = d.viewability_report(0.0, DepthBand::All);
		assert!(report.contains("no elements on visible layers"), "{report}");
	}
}
