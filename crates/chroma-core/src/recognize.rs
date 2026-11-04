//! recognize: what known shape a freehand stroke's points trace,
//! if any — Elshara's "match a stroke combination or pattern to
//! form a shape if known". Pure geometry over the sampled points
//! a stroke keeps: no image, no model, nothing learned. The
//! thresholds are conservative on purpose — a stroke that is not
//! clearly a line, a rectangle, an ellipse or a triangle is
//! Unknown, and the caller leaves it exactly as it was drawn.

/// The known shape a stroke's points trace, with the fitted
/// parameters the caller builds the clean shape from: a line's
/// two ends, a rectangle's or ellipse's bounding box (position
/// and size, in canvas pixels), or a triangle's three corners.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recognized {
	Line { from: (i32, i32), to: (i32, i32) },
	Rectangle { x: i32, y: i32, w: u32, h: u32 },
	Ellipse { x: i32, y: i32, w: u32, h: u32 },
	Triangle { corners: [(i32, i32); 3] },
	Unknown,
}

fn dist(a: (i32, i32), b: (i32, i32)) -> f64 {
	let dx = a.0 as f64 - b.0 as f64;
	let dy = a.1 as f64 - b.1 as f64;
	(dx * dx + dy * dy).sqrt()
}

/// Distance from a point to a segment (not the infinite line).
fn seg_dist(p: (i32, i32), a: (i32, i32), b: (i32, i32)) -> f64 {
	let (ax, ay) = (a.0 as f64, a.1 as f64);
	let (bx, by) = (b.0 as f64, b.1 as f64);
	let (px, py) = (p.0 as f64, p.1 as f64);
	let (dx, dy) = (bx - ax, by - ay);
	let len2 = dx * dx + dy * dy;
	let t = if len2 <= 0.0 {
		0.0
	} else {
		(((px - ax) * dx + (py - ay) * dy) / len2).clamp(0.0, 1.0)
	};
	let (qx, qy) = (ax + t * dx, ay + t * dy);
	((px - qx) * (px - qx) + (py - qy) * (py - qy)).sqrt()
}

/// Ramer–Douglas–Peucker simplification of a polyline: the
/// points that survive when every stretch of the path stays
/// within `eps` of the chord between its kept neighbours.
/// Iterative, so a long stroke cannot blow the stack.
fn simplify(pts: &[(i32, i32)], eps: f64) -> Vec<(i32, i32)> {
	if pts.len() < 3 {
		return pts.to_vec();
	}
	let mut keep = vec![false; pts.len()];
	keep[0] = true;
	keep[pts.len() - 1] = true;
	let mut stack = vec![(0, pts.len() - 1)];
	while let Some((a, b)) = stack.pop() {
		if b <= a + 1 {
			continue;
		}
		let (mut far, mut far_d) = (a, 0.0);
		for i in (a + 1)..b {
			let d = seg_dist(pts[i], pts[a], pts[b]);
			if d > far_d {
				far_d = d;
				far = i;
			}
		}
		if far_d > eps {
			keep[far] = true;
			stack.push((a, far));
			stack.push((far, b));
		}
	}
	pts.iter()
		.zip(&keep)
		.filter(|(_, &k)| k)
		.map(|(&p, _)| p)
		.collect()
}

/// Classify a stroke's points. Fewer than three points, or a
/// trace too small to judge, is Unknown — there is nothing to
/// match against yet.
pub fn recognize(points: &[(i32, i32)]) -> Recognized {
	if points.len() < 3 {
		return Recognized::Unknown;
	}
	let (mut minx, mut miny) = (i32::MAX, i32::MAX);
	let (mut maxx, mut maxy) = (i32::MIN, i32::MIN);
	for &(x, y) in points {
		minx = minx.min(x);
		miny = miny.min(y);
		maxx = maxx.max(x);
		maxy = maxy.max(y);
	}
	let w = (maxx - minx) as f64;
	let h = (maxy - miny) as f64;
	let diag = (w * w + h * h).sqrt();
	if diag < 12.0 {
		return Recognized::Unknown;
	}
	let first = points[0];
	let last = points[points.len() - 1];
	// A drawn shape's pen lifts near where it came down; an open
	// stroke's ends are a shape's width apart.
	let closed = dist(first, last) <= (0.15 * diag).max(6.0);
	if !closed {
		// The one open shape: every point near the straight
		// segment between the ends. Anything else open — an
		// arc, a hook, a squiggle — is honestly not known.
		let tol = (0.05 * dist(first, last)).max(3.0);
		if points.iter().all(|&p| seg_dist(p, first, last) <= tol) {
			return Recognized::Line { from: first, to: last };
		}
		return Recognized::Unknown;
	}
	// Closed shapes need a real area in both directions: a line
	// retraced back on itself is thin in one of them.
	if w < 10.0 || h < 10.0 {
		return Recognized::Unknown;
	}
	let tol = (0.06 * diag).max(3.0);
	// Rectangle: nearly every point hugs one of the bounding
	// box's four edges, and every edge is visited — a C or a U
	// leaves an edge bare and fails the second half.
	let edges = [
		((minx, miny), (maxx, miny)),
		((maxx, miny), (maxx, maxy)),
		((maxx, maxy), (minx, maxy)),
		((minx, maxy), (minx, miny)),
	];
	let hugging = points
		.iter()
		.filter(|&&p| edges.iter().any(|&(a, b)| seg_dist(p, a, b) <= tol))
		.count();
	let every_edge_visited = edges
		.iter()
		.all(|&(a, b)| points.iter().any(|&p| seg_dist(p, a, b) <= tol));
	if every_edge_visited && hugging as f64 >= 0.9 * points.len() as f64 {
		return Recognized::Rectangle {
			x: minx,
			y: miny,
			w: w as u32,
			h: h as u32,
		};
	}
	// Ellipse: nearly every point rides the bounding box's
	// ellipse rim — its normalized radius stays near one — and
	// the rim is ridden in all four quadrants, so a half-drawn
	// arc that happens to close does not pass.
	let (cx, cy) = (minx as f64 + w / 2.0, miny as f64 + h / 2.0);
	let (rx, ry) = (w / 2.0, h / 2.0);
	let rim = |p: (i32, i32)| {
		let nx = (p.0 as f64 - cx) / rx;
		let ny = (p.1 as f64 - cy) / ry;
		(nx * nx + ny * ny).sqrt()
	};
	let on_rim = |p: (i32, i32)| (0.78..=1.25).contains(&rim(p));
	let riding = points.iter().filter(|&&p| on_rim(p)).count();
	let quadrants_covered = [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)]
		.iter()
		.all(|&(sx, sy)| {
			points.iter().any(|&p| {
				(p.0 as f64 - cx) * sx > 0.0 && (p.1 as f64 - cy) * sy > 0.0 && on_rim(p)
			})
		});
	if quadrants_covered && riding as f64 >= 0.85 * points.len() as f64 {
		return Recognized::Ellipse {
			x: minx,
			y: miny,
			w: w as u32,
			h: h as u32,
		};
	}
	// Triangle: simplify the closed loop (first point repeated
	// at the end, so the simplifier sees the whole circuit);
	// exactly three corners survive, and no side is a sliver —
	// a retraced line simplifies to a shape with a side near
	// zero and fails here.
	let mut looped: Vec<(i32, i32)> = points.to_vec();
	looped.push(first);
	let mut corners = simplify(&looped, 0.05 * diag);
	corners.pop(); // the repeated first point
	if corners.len() == 3 {
		let c = [corners[0], corners[1], corners[2]];
		let sides = [dist(c[0], c[1]), dist(c[1], c[2]), dist(c[2], c[0])];
		if sides.iter().all(|&s| s >= 0.3 * diag) {
			return Recognized::Triangle { corners: c };
		}
	}
	Recognized::Unknown
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Deterministic hand-jitter: -2..=2 pixels, no randomness,
	/// so a test that passes passes every run.
	fn jitter(i: usize) -> i32 {
		(((i * 37 + 11) % 5) as i32) - 2
	}

	fn jittered(pts: &[(i32, i32)]) -> Vec<(i32, i32)> {
		pts.iter()
			.enumerate()
			.map(|(i, &(x, y))| (x + jitter(i), y + jitter(i + 7)))
			.collect()
	}

	/// Points along the segment a→b, `n` of them, ends included.
	fn along(a: (i32, i32), b: (i32, i32), n: usize) -> Vec<(i32, i32)> {
		(0..n)
			.map(|i| {
				let t = i as f64 / (n - 1) as f64;
				(
					(a.0 as f64 + (b.0 - a.0) as f64 * t).round() as i32,
					(a.1 as f64 + (b.1 - a.1) as f64 * t).round() as i32,
				)
			})
			.collect()
	}

	/// A closed walk of a rectangle's perimeter, starting at its
	/// top-left corner.
	fn rect_points(x: i32, y: i32, w: i32, h: i32) -> Vec<(i32, i32)> {
		let mut pts = along((x, y), (x + w, y), 9);
		pts.extend(along((x + w, y), (x + w, y + h), 7));
		pts.extend(along((x + w, y + h), (x, y + h), 9));
		pts.extend(along((x, y + h), (x, y), 7));
		pts
	}

	/// A closed walk of an ellipse's rim.
	fn ellipse_points(cx: i32, cy: i32, rx: i32, ry: i32) -> Vec<(i32, i32)> {
		let n = 40;
		let mut pts: Vec<(i32, i32)> = (0..n)
			.map(|i| {
				let t = std::f64::consts::TAU * i as f64 / n as f64;
				(
					cx + (rx as f64 * t.cos()).round() as i32,
					cy + (ry as f64 * t.sin()).round() as i32,
				)
			})
			.collect();
		pts.push(pts[0]);
		pts
	}

	/// A closed walk of a triangle's three sides, starting at a
	/// corner.
	fn triangle_points(a: (i32, i32), b: (i32, i32), c: (i32, i32)) -> Vec<(i32, i32)> {
		let mut pts = along(a, b, 10);
		pts.extend(along(b, c, 10));
		pts.extend(along(c, a, 10));
		pts
	}

	#[test]
	fn a_straight_open_stroke_is_a_line() {
		let pts = along((10, 20), (190, 60), 20);
		assert_eq!(
			recognize(&pts),
			Recognized::Line { from: (10, 20), to: (190, 60) }
		);
		// A hand's wobble does not unmake the line.
		assert!(matches!(recognize(&jittered(&pts)), Recognized::Line { .. }));
	}

	#[test]
	fn a_rectangle_is_its_bounding_box() {
		let pts = rect_points(20, 30, 120, 80);
		assert_eq!(
			recognize(&pts),
			Recognized::Rectangle { x: 20, y: 30, w: 120, h: 80 }
		);
		match recognize(&jittered(&pts)) {
			Recognized::Rectangle { x, y, w, h } => {
				assert!((x - 20).abs() <= 2 && (y - 30).abs() <= 2, "{x},{y}");
				assert!((w as i32 - 120).abs() <= 4 && (h as i32 - 80).abs() <= 4, "{w}x{h}");
			}
			other => panic!("jittered rectangle read as {other:?}"),
		}
	}

	#[test]
	fn an_ellipse_is_its_bounding_box() {
		let pts = ellipse_points(100, 90, 70, 45);
		assert_eq!(
			recognize(&pts),
			Recognized::Ellipse { x: 30, y: 45, w: 140, h: 90 }
		);
		match recognize(&jittered(&pts)) {
			Recognized::Ellipse { x, y, w, h } => {
				assert!((x - 30).abs() <= 2 && (y - 45).abs() <= 2, "{x},{y}");
				assert!((w as i32 - 140).abs() <= 4 && (h as i32 - 90).abs() <= 4, "{w}x{h}");
			}
			other => panic!("jittered ellipse read as {other:?}"),
		}
	}

	#[test]
	fn a_triangle_is_its_three_corners() {
		let (a, b, c) = ((100, 15), (15, 130), (185, 130));
		let pts = triangle_points(a, b, c);
		match recognize(&pts) {
			Recognized::Triangle { corners } => {
				for want in [a, b, c] {
					assert!(
						corners.iter().any(|&got| (got.0 - want.0).abs() <= 1 && (got.1 - want.1).abs() <= 1),
						"corner {want:?} missing from {corners:?}"
					);
				}
			}
			other => panic!("triangle read as {other:?}"),
		}
		assert!(matches!(recognize(&jittered(&pts)), Recognized::Triangle { .. }));
	}

	#[test]
	fn the_unknowns_stay_unknown() {
		// An open squiggle: neither straight nor closed.
		let squiggle: Vec<(i32, i32)> = (0..30)
			.map(|i| (10 + i * 6, 80 + ((i as f64 * 0.7).sin() * 30.0).round() as i32))
			.collect();
		assert_eq!(recognize(&squiggle), Recognized::Unknown);
		// Two points are not a shape yet.
		assert_eq!(recognize(&[(5, 5), (90, 90)]), Recognized::Unknown);
		// Nothing at all.
		assert_eq!(recognize(&[]), Recognized::Unknown);
		// An open arc: curved, and its ends are far apart.
		let arc: Vec<(i32, i32)> = (0..20)
			.map(|i| {
				let t = std::f64::consts::PI * i as f64 / 19.0;
				(
					100 + (70.0 * t.cos()).round() as i32,
					100 + (70.0 * t.sin()).round() as i32,
				)
			})
			.collect();
		assert_eq!(recognize(&arc), Recognized::Unknown);
		// A C leaves the right edge of its box bare: not a
		// rectangle, however neatly its three sides are drawn.
		let mut c_shape = along((140, 20), (20, 20), 9);
		c_shape.extend(along((20, 20), (20, 120), 8));
		c_shape.extend(along((20, 120), (140, 120), 9));
		assert_eq!(recognize(&c_shape), Recognized::Unknown);
		// A retraced line — out and straight back — is closed
		// but has no area; it is not a shape either.
		let mut there = along((10, 50), (180, 52), 16);
		there.extend(along((180, 52), (10, 50), 16));
		assert_eq!(recognize(&there), Recognized::Unknown);
	}
}
