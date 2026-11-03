//! symbols: a starter library of named symbol stamps.
//!
//! Each symbol is a parametric point set in a unit square, scaled
//! into whatever box the user places it in, and drawn with the
//! stroke machinery the app already has — a symbol is just one or
//! more stroke paths with a shared caption. The geometry lives here
//! so it can be unit-tested: every path stays inside its box.

/// The symbols on offer, in menu order.
pub const SYMBOL_NAMES: &[&str] =
	&["star", "heart", "arrow", "check", "cross", "sun", "moon"];

fn scale(paths: Vec<Vec<(f64, f64)>>, x: i32, y: i32, w: u32, h: u32) -> Vec<Vec<(i32, i32)>> {
	paths
		.iter()
		.map(|path| {
			path.iter()
				.map(|&(nx, ny)| {
					(x + (nx * w as f64).round() as i32, y + (ny * h as f64).round() as i32)
				})
				.collect()
		})
		.collect()
}

fn unit_paths(name: &str) -> Option<Vec<Vec<(f64, f64)>>> {
	let paths: Vec<Vec<(f64, f64)>> = match name {
		"star" => {
			let mut pts = Vec::new();
			for i in 0..10 {
				let (r, a) = if i % 2 == 0 {
					(0.5, -90.0 + i as f64 * 36.0)
				} else {
					(0.22, -90.0 + i as f64 * 36.0)
				};
				let rad = a.to_radians();
				pts.push((0.5 + r * rad.cos(), 0.5 + r * rad.sin()));
			}
			pts.push(pts[0]);
			vec![pts]
		}
		"heart" => {
			// The classic parametric heart curve, mapped from its
			// native x in [-16, 16], y in [-17, 12] into the unit
			// square (y flipped: the curve's up is the box's top).
			let mut pts = Vec::new();
			for i in 0..=48 {
				let t = i as f64 / 48.0 * std::f64::consts::TAU;
				let hx = 16.0 * t.sin().powi(3);
				let hy =
					13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos();
				pts.push((hx / 32.0 + 0.5, (12.0 - hy) / 30.0));
			}
			vec![pts]
		}
		"arrow" => vec![vec![
			(0.0, 0.35),
			(0.58, 0.35),
			(0.58, 0.12),
			(1.0, 0.5),
			(0.58, 0.88),
			(0.58, 0.65),
			(0.0, 0.65),
			(0.0, 0.35),
		]],
		"check" => vec![vec![(0.05, 0.52), (0.38, 0.84), (0.95, 0.16)]],
		"cross" => vec![vec![
			(0.34, 0.0),
			(0.66, 0.0),
			(0.66, 0.34),
			(1.0, 0.34),
			(1.0, 0.66),
			(0.66, 0.66),
			(0.66, 1.0),
			(0.34, 1.0),
			(0.34, 0.66),
			(0.0, 0.66),
			(0.0, 0.34),
			(0.34, 0.34),
			(0.34, 0.0),
		]],
		"sun" => {
			let mut paths = Vec::new();
			let mut disc = Vec::new();
			for i in 0..=24 {
				let a = i as f64 / 24.0 * std::f64::consts::TAU;
				disc.push((0.5 + 0.26 * a.cos(), 0.5 + 0.26 * a.sin()));
			}
			paths.push(disc);
			for k in 0..8 {
				let a = k as f64 * std::f64::consts::FRAC_PI_4;
				paths.push(vec![
					(0.5 + 0.35 * a.cos(), 0.5 + 0.35 * a.sin()),
					(0.5 + 0.49 * a.cos(), 0.5 + 0.49 * a.sin()),
				]);
			}
			paths
		}
		"moon" => {
			// A crescent: the left rim of the outer circle, closed by
			// the rim of a smaller circle offset to the right.
			let mut pts = Vec::new();
			for i in 0..=20 {
				let a = (110.0 + i as f64 * 7.0).to_radians();
				pts.push((0.44 + 0.44 * a.cos(), 0.5 + 0.44 * a.sin()));
			}
			for i in 0..=20 {
				let a = (242.0 - i as f64 * 5.7).to_radians();
				pts.push((0.64 + 0.36 * a.cos(), 0.5 + 0.38 * a.sin()));
			}
			pts.push(pts[0]);
			vec![pts]
		}
		_ => return None,
	};
	Some(paths)
}

/// The stroke paths for a symbol placed in a box: one path per pen
/// stroke (a sun is a disc plus eight rays, so nine paths), every
/// point inside the box. `None` for an unknown symbol name.
pub fn symbol_paths(name: &str, x: i32, y: i32, w: u32, h: u32) -> Option<Vec<Vec<(i32, i32)>>> {
	Some(scale(unit_paths(name)?, x, y, w, h))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn every_symbol_draws_inside_its_box() {
		for name in SYMBOL_NAMES {
			let paths = symbol_paths(name, 100, 50, 80, 60).unwrap();
			assert!(!paths.is_empty(), "{name} has no paths");
			for path in &paths {
				assert!(path.len() >= 2, "{name} has a path shorter than 2 points");
				for &(px, py) in path {
					assert!(
						(100..=180).contains(&px) && (50..=110).contains(&py),
						"{name} point ({px}, {py}) escapes its box"
					);
				}
			}
		}
		assert!(symbol_paths("dragon", 0, 0, 10, 10).is_none());
	}

	#[test]
	fn star_is_a_closed_ten_point_outline() {
		let paths = symbol_paths("star", 0, 0, 100, 100).unwrap();
		assert_eq!(paths.len(), 1);
		let pts = &paths[0];
		assert_eq!(pts.len(), 11); // 10 corners plus the closing point
		assert_eq!(pts.first(), pts.last());
		// The first point is the top spike: centered across, at the top.
		assert_eq!(pts[0], (50, 0));
	}

	#[test]
	fn check_runs_from_low_left_up_to_the_right() {
		let pts = &symbol_paths("check", 0, 0, 100, 100).unwrap()[0];
		assert_eq!(pts.len(), 3);
		assert!(pts[0].0 < pts[1].0 && pts[1].0 < pts[2].0);
		assert!(pts[1].1 > pts[0].1 && pts[1].1 > pts[2].1); // the dip
		assert!(pts[2].1 < pts[0].1); // ends highest
	}

	#[test]
	fn sun_is_a_disc_plus_eight_rays() {
		let paths = symbol_paths("sun", 0, 0, 100, 100).unwrap();
		assert_eq!(paths.len(), 9);
		assert_eq!(paths[0].len(), 25); // closed 24-gon
		assert_eq!(paths[0].first(), paths[0].last());
		assert!(paths[1..].iter().all(|p| p.len() == 2));
	}

	#[test]
	fn heart_is_symmetric_and_taller_than_its_middle() {
		let pts = &symbol_paths("heart", 0, 0, 200, 200).unwrap()[0];
		assert!(pts.len() > 40);
		// The curve's first point is the top-center dip.
		assert!((pts[0].0 - 100).abs() <= 2, "dip at {}", pts[0].0);
		assert!(pts[0].1 > 20 && pts[0].1 < 80, "dip depth {}", pts[0].1);
		// The bottom tip is the lowest point, near the horizontal middle.
		let tip = pts.iter().max_by_key(|p| p.1).unwrap();
		assert!((tip.0 - 100).abs() <= 4, "tip at {}", tip.0);
		assert!(tip.1 > 180, "tip depth {}", tip.1);
	}
}
