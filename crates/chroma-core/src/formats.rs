//! formats: the colorhexa-style catalogue of color systems, plus the
//! color tools built on them (blend, subtract, gradient, contrast,
//! text readability, harmony wheels, sound frequency to color).
//!
//! Every conversion here goes through CIE XYZ with the D65 white
//! point: sRGB is linearized first (the same gamma handling as
//! `Srgb::luminance`), then the standard sRGB-to-XYZ matrix. Nothing
//! in this module "mixes" by eye: each system is a formula, and the
//! tests pin the formulas to known anchor values and round-trips.
//!
//! Two honest limits, stated wherever the app repeats them:
//! - Dominant wavelength exists only for colors on the spectrum side
//!   of the white point. Purples and magentas have none; they get a
//!   *complementary* wavelength instead (reported as such, never
//!   silently swapped).
//! - The frequency-to-color mapping is an artistic convention
//!   (octave-folding sound up into the visible band), not physics:
//!   sound and light are different phenomena.

use crate::design::{Design, ShapeKind};
use crate::{ColorFormat, Srgb};

/// The D65 reference white, XYZ on the 0..100 scale.
pub const D65: Xyz = Xyz { x: 95.047, y: 100.0, z: 108.883 };
/// The D65 white point in chromaticity coordinates.
const D65_XY: (f64, f64) = (0.31271, 0.32902);

/// A color in CIE XYZ, each component on the 0..100 scale (Y is the
/// luminance; D65 white is (95.047, 100, 108.883)).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Xyz {
	pub x: f64,
	pub y: f64,
	pub z: f64,
}

impl Srgb {
	/// Linear sRGB to CIE XYZ (D65), components scaled 0..100.
	pub fn to_xyz(self) -> Xyz {
		let [r, g, b] = self.to_linear();
		let (r, g, b) = (r as f64, g as f64, b as f64);
		Xyz {
			x: (0.41239080 * r + 0.35758434 * g + 0.18048079 * b) * 100.0,
			y: (0.21263901 * r + 0.71516868 * g + 0.07219232 * b) * 100.0,
			z: (0.01933082 * r + 0.11919478 * g + 0.95053215 * b) * 100.0,
		}
	}

	/// CIE XYZ (0..100 scale, D65) back to sRGB. Out-of-gamut results
	/// clamp per channel inside the gamma step — conversions are
	/// reporting tools, so a clamp here is honest arithmetic, unlike
	/// the scheme transforms which remap to preserve hue.
	pub fn from_xyz(xyz: Xyz) -> Srgb {
		let (x, y, z) = (xyz.x / 100.0, xyz.y / 100.0, xyz.z / 100.0);
		Srgb::from_linear([
			(3.2404542 * x - 1.5371385 * y - 0.4985314 * z) as f32,
			(-0.9692660 * x + 1.8760108 * y + 0.0415175 * z) as f32,
			(0.0556434 * x - 0.2040259 * y + 1.0572252 * z) as f32,
		])
	}

	/// Chromaticity plus luminance: (x, y, Y), Y on the 0..100 scale.
	/// Black has no chromaticity and reports (0, 0, 0).
	pub fn to_xyy(self) -> (f64, f64, f64) {
		let xyz = self.to_xyz();
		let sum = xyz.x + xyz.y + xyz.z;
		if sum == 0.0 {
			return (0.0, 0.0, 0.0);
		}
		(xyz.x / sum, xyz.y / sum, xyz.y)
	}

	/// Build a color from chromaticity (x, y) and luminance Y (0..100).
	pub fn from_xyy(x: f64, y: f64, lum: f64) -> Srgb {
		if y <= 0.0 || lum <= 0.0 {
			return Srgb::BLACK;
		}
		Srgb::from_xyz(Xyz { x: x * lum / y, y: lum, z: (1.0 - x - y) * lum / y })
	}

	/// CMYK as percents 0..100: (cyan, magenta, yellow, key/black).
	pub fn to_cmyk(self) -> (f64, f64, f64, f64) {
		let (r, g, b) = (
			self.r as f64 / 255.0,
			self.g as f64 / 255.0,
			self.b as f64 / 255.0,
		);
		let k = 1.0 - r.max(g).max(b);
		if k >= 1.0 {
			return (0.0, 0.0, 0.0, 100.0);
		}
		let d = 1.0 - k;
		(
			(1.0 - r - k) / d * 100.0,
			(1.0 - g - k) / d * 100.0,
			(1.0 - b - k) / d * 100.0,
			k * 100.0,
		)
	}

	/// CMYK percents back to sRGB.
	pub fn from_cmyk(c: f64, m: f64, y: f64, k: f64) -> Srgb {
		let ch = |v: f64, k: f64| {
			(255.0 * (1.0 - (v / 100.0).clamp(0.0, 1.0)) * (1.0 - (k / 100.0).clamp(0.0, 1.0)))
				.round() as u8
		};
		Srgb::new(ch(c, k), ch(m, k), ch(y, k))
	}

	/// The color as one decimal number, 0 (black) to 16777215 (white):
	/// red * 65536 + green * 256 + blue.
	pub fn to_decimal(self) -> u32 {
		(self.r as u32) << 16 | (self.g as u32) << 8 | self.b as u32
	}

	/// A decimal color number back to sRGB; values above 16777215
	/// wrap modulo the 24-bit range, the way color pickers treat them.
	pub fn from_decimal(n: u32) -> Srgb {
		let n = n & 0x00FF_FFFF;
		Srgb::new((n >> 16) as u8, (n >> 8) as u8, n as u8)
	}

	/// The nearest web-safe color: each channel snapped to a multiple
	/// of 51 (the 216-color web palette grid).
	pub fn to_web_safe(self) -> Srgb {
		let snap = |v: u8| (v as u16 + 25) / 51 * 51;
		Srgb::new(snap(self.r) as u8, snap(self.g) as u8, snap(self.b) as u8)
	}

	/// The color as a 24-bit binary string, one byte per channel,
	/// bytes separated by spaces: "00001111 01011110 01101000".
	pub fn to_binary(self) -> String {
		format!("{:08b} {:08b} {:08b}", self.r, self.g, self.b)
	}

	/// Parse a 24-bit binary string (spaces optional) back to a color.
	pub fn from_binary(s: &str) -> Option<Srgb> {
		let bits: String = s.chars().filter(|c| *c == '0' || *c == '1').collect();
		if bits.len() != 24 || s.chars().any(|c| !matches!(c, '0' | '1' | ' ')) {
			return None;
		}
		let v = u32::from_str_radix(&bits, 2).ok()?;
		Some(Srgb::from_decimal(v))
	}
}

/// The Lab cube-root helper function and its inverse.
fn lab_f(t: f64) -> f64 {
	const D: f64 = 6.0 / 29.0;
	if t > D * D * D {
		t.cbrt()
	} else {
		t / (3.0 * D * D) + 4.0 / 29.0
	}
}

fn lab_f_inv(f: f64) -> f64 {
	const D: f64 = 6.0 / 29.0;
	if f > D {
		f * f * f
	} else {
		3.0 * D * D * (f - 4.0 / 29.0)
	}
}

/// A color in CIE L*a*b*: L 0..100, a and b signed (roughly ±128).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lab {
	pub l: f64,
	pub a: f64,
	pub b: f64,
}

impl Srgb {
	pub fn to_lab(self) -> Lab {
		let xyz = self.to_xyz();
		let (fx, fy, fz) = (
			lab_f(xyz.x / D65.x),
			lab_f(xyz.y / D65.y),
			lab_f(xyz.z / D65.z),
		);
		Lab { l: 116.0 * fy - 16.0, a: 500.0 * (fx - fy), b: 200.0 * (fy - fz) }
	}

	pub fn from_lab(lab: Lab) -> Srgb {
		let fy = (lab.l + 16.0) / 116.0;
		let fx = fy + lab.a / 500.0;
		let fz = fy - lab.b / 200.0;
		Srgb::from_xyz(Xyz {
			x: D65.x * lab_f_inv(fx),
			y: D65.y * if lab.l > 8.0 { fy * fy * fy } else { lab.l / 903.3 },
			z: D65.z * lab_f_inv(fz),
		})
	}

	/// CIE LCH: (lightness 0..100, chroma 0..~150, hue degrees 0..360)
	/// — Lab in polar form, hue measured like every other hue here.
	pub fn to_lch(self) -> (f64, f64, f64) {
		let lab = self.to_lab();
		let c = (lab.a * lab.a + lab.b * lab.b).sqrt();
		let h = lab.b.atan2(lab.a).to_degrees();
		(lab.l, c, if h < 0.0 { h + 360.0 } else { h })
	}

	pub fn from_lch(l: f64, c: f64, h: f64) -> Srgb {
		let hr = h.to_radians();
		Srgb::from_lab(Lab { l, a: c * hr.cos(), b: c * hr.sin() })
	}

	/// CIE L*u*v* against D65: L 0..100, u and v signed.
	pub fn to_luv(self) -> (f64, f64, f64) {
		let xyz = self.to_xyz();
		let denom = xyz.x + 15.0 * xyz.y + 3.0 * xyz.z;
		if denom == 0.0 {
			return (0.0, 0.0, 0.0);
		}
		let (up, vp) = (4.0 * xyz.x / denom, 9.0 * xyz.y / denom);
		let (un, vn) = white_uv();
		let l = self.to_lab().l;
		(l, 13.0 * l * (up - un), 13.0 * l * (vp - vn))
	}

	pub fn from_luv(l: f64, u: f64, v: f64) -> Srgb {
		if l <= 0.0 {
			return Srgb::BLACK;
		}
		let (un, vn) = white_uv();
		let up = u / (13.0 * l) + un;
		let vp = v / (13.0 * l) + vn;
		if vp == 0.0 {
			return Srgb::BLACK;
		}
		let y = D65.y * if l > 8.0 { ((l + 16.0) / 116.0).powi(3) } else { l / 903.3 };
		let x = y * 9.0 * up / (4.0 * vp);
		let z = y * (12.0 - 3.0 * up - 20.0 * vp) / (4.0 * vp);
		Srgb::from_xyz(Xyz { x, y, z })
	}

	/// Hunter Lab (the 1948 system, D65 constants): L 0..100,
	/// a and b signed. Not the same numbers as CIE Lab — the two
	/// systems share names and nothing else.
	pub fn to_hunter_lab(self) -> (f64, f64, f64) {
		let xyz = self.to_xyz();
		if xyz.y <= 0.0 {
			return (0.0, 0.0, 0.0);
		}
		let (ka, kb) = hunter_constants();
		let s = (xyz.y / D65.y).sqrt();
		(
			10.0 * xyz.y.sqrt(),
			ka * ((xyz.x / D65.x - xyz.y / D65.y) / s),
			kb * ((xyz.y / D65.y - xyz.z / D65.z) / s),
		)
	}

	pub fn from_hunter_lab(l: f64, a: f64, b: f64) -> Srgb {
		if l <= 0.0 {
			return Srgb::BLACK;
		}
		let (ka, kb) = hunter_constants();
		let y = (l / 10.0) * (l / 10.0);
		let s = (y / D65.y).sqrt();
		Srgb::from_xyz(Xyz {
			x: D65.x * (y / D65.y + a / ka * s),
			y,
			z: D65.z * (y / D65.y - b / kb * s),
		})
	}

	/// The dominant wavelength of this color, when it has one: the
	/// spectrum wavelength whose chromaticity lies on the line from
	/// the D65 white point through this color's chromaticity. Purple
	/// family colors have no dominant wavelength; theirs resolves to
	/// the complementary wavelength (the spectrum color that, mixed
	/// with this one, makes white). Achromatic colors return None.
	pub fn dominant_wavelength(self) -> Option<Wavelength> {
		let xyz = self.to_xyz();
		if xyz.x + xyz.y + xyz.z == 0.0 {
			return None; // black has no chromaticity to project
		}
		let (x, y, _) = self.to_xyy();
		let (wx, wy) = D65_XY;
		let (dx, dy) = (x - wx, y - wy);
		if (dx * dx + dy * dy).sqrt() < 0.002 {
			return None; // at the white point: no hue direction at all
		}
		if let Some((nm, t)) = locus_hit(wx, wy, dx, dy) {
			return Some(Wavelength {
				nanometres: nm,
				complementary: false,
				purity: (1.0 / t * 100.0).clamp(0.0, 100.0),
			});
		}
		let (nm, _) = locus_hit(wx, wy, -dx, -dy)?;
		Some(Wavelength { nanometres: nm, complementary: true, purity: 0.0 })
	}
}

fn white_uv() -> (f64, f64) {
	let denom = D65.x + 15.0 * D65.y + 3.0 * D65.z;
	(4.0 * D65.x / denom, 9.0 * D65.y / denom)
}

fn hunter_constants() -> (f64, f64) {
	let ka = (175.0 / 198.04) * (D65.x + D65.y);
	let kb = (70.0 / 218.11) * (D65.y + D65.z);
	(ka, kb)
}

/// The result of a dominant-wavelength lookup.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Wavelength {
	/// The wavelength in nanometres (380..780).
	pub nanometres: f64,
	/// True when this is the *complementary* wavelength: the color
	/// itself (a purple) has no dominant wavelength.
	pub complementary: bool,
	/// Excitation purity in percent: how far the color sits from
	/// white toward the spectrum color, 0 at white, 100 at the
	/// spectrum locus. Zero for complementary results.
	pub purity: f64,
}

impl Wavelength {
	/// The result in words, with the honesty built in: a purple says
	/// "complementary wavelength", never plain "wavelength".
	pub fn spoken(&self) -> String {
		if self.complementary {
			format!(
				"no dominant wavelength (purple family); complementary wavelength {:.0} nanometres",
				self.nanometres
			)
		} else {
			format!(
				"dominant wavelength {:.0} nanometres, excitation purity {:.0} percent",
				self.nanometres, self.purity
			)
		}
	}
}

/// The CIE 1931 spectrum locus, chromaticity at every 10 nm from
/// 380 to 780. The far-red tail (690 on) is effectively a point.
const LOCUS: &[(f64, f64, f64)] = &[
	(380.0, 0.1741, 0.0049),
	(390.0, 0.1738, 0.0049),
	(400.0, 0.1733, 0.0048),
	(410.0, 0.1726, 0.0048),
	(420.0, 0.1714, 0.0051),
	(430.0, 0.1689, 0.0069),
	(440.0, 0.1644, 0.0109),
	(450.0, 0.1566, 0.0177),
	(460.0, 0.1440, 0.0297),
	(470.0, 0.1241, 0.0578),
	(480.0, 0.0913, 0.1327),
	(490.0, 0.0454, 0.2950),
	(500.0, 0.0082, 0.5384),
	(510.0, 0.0139, 0.7502),
	(520.0, 0.0743, 0.8338),
	(530.0, 0.1547, 0.8058),
	(540.0, 0.2296, 0.7543),
	(550.0, 0.3016, 0.6923),
	(560.0, 0.3731, 0.6245),
	(570.0, 0.4441, 0.5547),
	(580.0, 0.5125, 0.4866),
	(590.0, 0.5752, 0.4242),
	(600.0, 0.6270, 0.3725),
	(610.0, 0.6658, 0.3340),
	(620.0, 0.6915, 0.3083),
	(630.0, 0.7079, 0.2920),
	(640.0, 0.7190, 0.2809),
	(650.0, 0.7260, 0.2740),
	(660.0, 0.7300, 0.2700),
	(670.0, 0.7334, 0.2666),
	(680.0, 0.7347, 0.2653),
	(690.0, 0.7347, 0.2653),
	(700.0, 0.7347, 0.2653),
	(710.0, 0.7347, 0.2653),
	(720.0, 0.7347, 0.2653),
	(730.0, 0.7347, 0.2653),
	(740.0, 0.7347, 0.2653),
	(750.0, 0.7347, 0.2653),
	(760.0, 0.7347, 0.2653),
	(770.0, 0.7347, 0.2653),
	(780.0, 0.7347, 0.2653),
];

/// Cast a ray from (wx, wy) along (dx, dy) and find where it first
/// crosses the spectrum locus. Returns the interpolated wavelength
/// and the ray parameter t (the crossing point is at w + t*d, so the
/// color itself sits at t = 1 and purity is 1/t).
fn locus_hit(wx: f64, wy: f64, dx: f64, dy: f64) -> Option<(f64, f64)> {
	let mut best: Option<(f64, f64)> = None;
	for seg in LOCUS.windows(2) {
		let (n0, ax, ay) = seg[0];
		let (n1, bx, by) = seg[1];
		let (ex, ey) = (bx - ax, by - ay);
		// Solve w + t*d = a + s*e (Cramer's rule).
		let det = dy * ex - dx * ey;
		if det.abs() < 1e-12 {
			continue;
		}
		let (rx, ry) = (ax - wx, ay - wy);
		let t = (ex * ry - ey * rx) / det;
		let s = (dx * ry - dy * rx) / det;
		if t > 0.0 && (0.0..=1.0).contains(&s) {
			if best.map_or(true, |(_, bt)| t < bt) {
				best = Some((n0 + s * (n1 - n0), t));
			}
		}
	}
	best
}

/// An approximation of the sRGB color of monochromatic light at
/// `nm` nanometres (380..780), the standard piecewise fit (Bruton)
/// with the usual intensity falloff at the band edges. Wavelengths
/// outside the visible band clamp to the nearest end.
pub fn wavelength_to_srgb(nm: f64) -> Srgb {
	let nm = nm.clamp(380.0, 780.0);
	let (r, g, b) = match nm {
		x if x < 440.0 => ((440.0 - x) / 60.0, 0.0, 1.0),
		x if x < 490.0 => (0.0, (x - 440.0) / 50.0, 1.0),
		x if x < 510.0 => (0.0, 1.0, (510.0 - x) / 20.0),
		x if x < 580.0 => ((x - 510.0) / 70.0, 1.0, 0.0),
		x if x < 645.0 => (1.0, (645.0 - x) / 65.0, 0.0),
		_ => (1.0, 0.0, 0.0),
	};
	let fall = match nm {
		x if x < 420.0 => 0.3 + 0.7 * (x - 380.0) / 40.0,
		x if x <= 700.0 => 1.0,
		x => 0.3 + 0.7 * (780.0 - x) / 80.0,
	};
	let ch = |v: f64| ((v * fall).clamp(0.0, 1.0).powf(0.8) * 255.0).round() as u8;
	Srgb::new(ch(r), ch(g), ch(b))
}

/// EXPERIMENTAL: the color of a sound frequency. The frequency is
/// folded by octaves (doubled or halved) until it lands in the
/// visible band — light at 384 to 789 terahertz, wavelengths 780
/// down to 380 nanometres — and that wavelength's color is returned
/// with the wavelength itself. An artistic convention, plainly: an
/// octave of sound maps to an octave of light because both are
/// waves, not because a note *is* a color.
pub fn frequency_to_color(hz: f64) -> Option<(Srgb, f64)> {
	if !hz.is_finite() || hz <= 0.0 {
		return None;
	}
	const C: f64 = 299_792_458.0; // speed of light, m/s
	const F_LO: f64 = C / 780e-9; // red end, ~384 THz
	const F_HI: f64 = C / 380e-9; // violet end, ~789 THz
	let mut f = hz;
	let mut guard = 0;
	while f < F_LO && guard < 200 {
		f *= 2.0;
		guard += 1;
	}
	while f >= F_HI && guard < 400 {
		f /= 2.0;
		guard += 1;
	}
	if !(F_LO..F_HI).contains(&f) {
		return None;
	}
	let nm = C / f * 1e9;
	Some((wavelength_to_srgb(nm), nm))
}

/// Blend two colors in OKLab space (perceptual mixing: the midpoint
/// looks halfway, not just averages the channels). `percent` is how
/// much of the second color is in the mix, 0..100.
pub fn blend(a: Srgb, b: Srgb, percent: u8) -> Srgb {
	let t = percent.min(100) as f32 / 100.0;
	let (oa, ob) = (a.to_oklch(), b.to_oklch());
	let mix = |x: f32, y: f32| x + (y - x) * t;
	let (_, aa, ab) = oa.to_oklab();
	let (_, ba, bb) = ob.to_oklab();
	crate::Oklch::from_oklab(mix(oa.l, ob.l), mix(aa, ba), mix(ab, bb)).to_srgb()
}

/// Subtract one color from another: each channel of the second is
/// taken away from the first, floored at zero. Subtracting black
/// changes nothing; subtracting white gives black.
pub fn subtract(a: Srgb, b: Srgb) -> Srgb {
	Srgb::new(
		a.r.saturating_sub(b.r),
		a.g.saturating_sub(b.g),
		a.b.saturating_sub(b.b),
	)
}

/// A gradient ramp of `steps` colors from `a` to `b`, both ends
/// included, interpolated in OKLab so the middle looks midway.
/// `steps` clamps to 2..=64.
pub fn gradient(a: Srgb, b: Srgb, steps: usize) -> Vec<Srgb> {
	let steps = steps.clamp(2, 64);
	(0..steps)
		.map(|i| blend(a, b, (i * 100 / (steps - 1)) as u8))
		.collect()
}

/// A tiny deterministic random source (xorshift64) — enough for a
/// "surprise me" color, no dependency, reproducible from its seed.
pub struct XorShift(u64);

impl XorShift {
	pub fn new(seed: u64) -> XorShift {
		// Zero is xorshift's fixed point; nudge it off.
		XorShift(if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed })
	}

	pub fn next_u64(&mut self) -> u64 {
		let mut x = self.0;
		x ^= x << 13;
		x ^= x >> 7;
		x ^= x << 17;
		self.0 = x;
		x
	}

	/// A random color from this seed stream.
	pub fn color(&mut self) -> Srgb {
		let v = self.next_u64();
		Srgb::new((v >> 16) as u8, (v >> 8) as u8, v as u8)
	}
}

/// A random color from a seed: same seed, same color, every run.
pub fn random_color(seed: u64) -> Srgb {
	XorShift::new(seed).color()
}

/// The WCAG verdict for a contrast ratio, in words a designer can
/// act on: the thresholds are the WCAG AA lines (4.5 for normal
/// text, 3 for large text) and the AAA line (7).
pub fn contrast_verdict(ratio: f32) -> &'static str {
	match ratio {
		x if x >= 7.0 => "excellent contrast, readable at any text size",
		x if x >= 4.5 => "good contrast, readable as normal text",
		x if x >= 3.0 => "fair contrast, readable only as large text",
		_ => "poor contrast, hard to read at any size",
	}
}

/// How readable one text element is, in one word group.
fn readability_word(ratio: f32) -> &'static str {
	match ratio {
		x if x >= 4.5 => "easy to read",
		x if x >= 3.0 => "hard to read",
		_ => "unreadable",
	}
}

/// A spoken-style report on every text element in a design: each
/// phrase, its color in words, its contrast against the design
/// background, and a readability verdict. Contrast is measured
/// against the background only — a shape painted directly behind a
/// text element is not accounted for (stated in the report itself).
pub fn text_readability(design: &Design) -> String {
	let texts: Vec<_> = design.shapes.iter().filter(|s| s.kind == ShapeKind::Text).collect();
	if texts.is_empty() {
		return "This design has no text elements to check.".to_string();
	}
	let mut out = format!(
		"Text readability, measured against the design background ({}). Shapes painted behind text are not accounted for.\n",
		crate::describe(design.background)
	);
	for (i, s) in texts.iter().enumerate() {
		let ratio = s.fill.contrast(design.background);
		out.push_str(&format!(
			"{}. \"{}\" in {}: contrast {:.1} to 1 against the background, {}.\n",
			i + 1,
			s.text,
			crate::color_words(s.fill),
			ratio,
			readability_word(ratio)
		));
	}
	out
}

// ---- The RYB (paint) wheel -------------------------------------------
//
// The artists' wheel: red, yellow and blue are the primaries, and
// mixing paint is not rotating through screen hues — green sits
// opposite red, violet opposite yellow. Scheme rotations done on
// this wheel go: screen hue -> RYB hue (the table below), rotate,
// RYB hue -> screen hue. The table ties the twelve traditional RYB
// hues to their screen-hue equivalents and interpolates linearly
// between anchors; both directions use the same table.

/// (RYB hue, screen hue) anchors, degrees, both ascending 0..360.
const RYB_ANCHORS: &[(f32, f32)] = &[
	(0.0, 0.0),     // red
	(30.0, 15.0),   // red-orange
	(60.0, 30.0),   // orange
	(90.0, 45.0),   // yellow-orange
	(120.0, 60.0),  // yellow
	(150.0, 90.0),  // yellow-green
	(180.0, 120.0), // green
	(210.0, 175.0), // blue-green
	(240.0, 240.0), // blue
	(270.0, 260.0), // blue-violet
	(300.0, 270.0), // violet
	(330.0, 315.0), // red-violet
	(360.0, 360.0), // red again
];

fn anchor_map(h: f32, from_ryb: bool) -> f32 {
	let h = crate::wrap_hue(h);
	for pair in RYB_ANCHORS.windows(2) {
		let (a, b) = (pair[0], pair[1]);
		let (lo, hi, out_lo, out_hi) = if from_ryb {
			(a.0, b.0, a.1, b.1)
		} else {
			(a.1, b.1, a.0, b.0)
		};
		if h >= lo && h <= hi {
			let t = if hi == lo { 0.0 } else { (h - lo) / (hi - lo) };
			return out_lo + t * (out_hi - out_lo);
		}
	}
	h
}

/// A screen (HSL) hue angle as an RYB paint-wheel angle.
pub fn rgb_hue_to_ryb(h: f32) -> f32 {
	anchor_map(h, false)
}

/// An RYB paint-wheel angle as a screen (HSL) hue angle.
pub fn ryb_hue_to_rgb(h: f32) -> f32 {
	anchor_map(h, true)
}

/// Rotate a color's hue on the RYB paint wheel by `deg` degrees:
/// the rotation is done in RYB hue space; the color's HSL
/// saturation and lightness are untouched. Achromatic colors have
/// no hue to rotate and come back unchanged.
pub fn rotate_hue_ryb(color: Srgb, deg: f32) -> Srgb {
	let (h, s, l) = color.to_hsl();
	if s == 0.0 {
		return color;
	}
	let ryb = rgb_hue_to_ryb(h);
	let back = ryb_hue_to_rgb(ryb + deg);
	srgb_from_hsl(back, s, l)
}

/// sRGB from HSV (hue degrees, saturation and value in percent)
/// — the inverse of `Srgb::to_hsv`.
pub fn srgb_from_hsv(h: f32, s: f32, v: f32) -> Srgb {
	let h = crate::wrap_hue(h) / 60.0;
	let (s, v) = (s.clamp(0.0, 100.0) / 100.0, v.clamp(0.0, 100.0) / 100.0);
	let c = v * s;
	let x = c * (1.0 - (h % 2.0 - 1.0).abs());
	let m = v - c;
	let (r, g, b) = if h < 1.0 {
		(c, x, 0.0)
	} else if h < 2.0 {
		(x, c, 0.0)
	} else if h < 3.0 {
		(0.0, c, x)
	} else if h < 4.0 {
		(0.0, x, c)
	} else if h < 5.0 {
		(x, 0.0, c)
	} else {
		(c, 0.0, x)
	};
	Srgb::new(
		((r + m) * 255.0).round() as u8,
		((g + m) * 255.0).round() as u8,
		((b + m) * 255.0).round() as u8,
	)
}

/// sRGB from HSL (hue degrees, saturation and lightness in percent)
/// — the inverse of `Srgb::to_hsl`.
pub fn srgb_from_hsl(h: f32, s: f32, l: f32) -> Srgb {
	let h = crate::wrap_hue(h) / 60.0;
	let (s, l) = (s.clamp(0.0, 100.0) / 100.0, l.clamp(0.0, 100.0) / 100.0);
	let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
	let x = c * (1.0 - (h % 2.0 - 1.0).abs());
	let m = l - c / 2.0;
	let (r, g, b) = if h < 1.0 {
		(c, x, 0.0)
	} else if h < 2.0 {
		(x, c, 0.0)
	} else if h < 3.0 {
		(0.0, c, x)
	} else if h < 4.0 {
		(0.0, x, c)
	} else if h < 5.0 {
		(x, 0.0, c)
	} else {
		(c, 0.0, x)
	};
	Srgb::new(
		((r + m) * 255.0).round() as u8,
		((g + m) * 255.0).round() as u8,
		((b + m) * 255.0).round() as u8,
	)
}

// ---- Channel sliders -------------------------------------------------
//
// Elshara: "the hex/lch values should be just a selectable slider
// rather than just editable" — every format's channels as sliders,
// so you hear what you are changing while you change it. A channel
// has an integer slider range; the fine-grained systems scale
// (XYZ moves in tenths, xyY chromaticity in thousandths) so the
// slider unit is always an integer and the fine step is the
// smallest step there is. (XYZ speaks in tenths in its readout,
// but its sliders move in hundredths: at the dark end a tenth of
// a unit is several sRGB steps, too coarse to hold a color.) A color's channel values in a format
// fully determine the color: `color_from_channels` runs the same
// from_* conversions the readouts use, backwards.
//
// Two formats are honest exceptions, stated in the help file too:
// - Decimal is ONE packed number (0 to 16777215). A single slider
//   over sixteen million steps cannot be stepped through, so its
//   channels are its three bytes — the base-256 digits of the
//   number, which is what the number is made of.
// - Binary is a readout, not an editing system: it writes the
//   color out as bits, and bits are not values you slide.

/// How a channel slider's value is spoken.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelUnit {
	/// The number itself: "62".
	Plain,
	/// "62 percent".
	Percent,
	/// "187 degrees".
	Degrees,
	/// Two-digit hex: "5E" — the hex format's own voice.
	HexByte,
	/// Degrees with decimals: one slider unit is 10^-decimals of
	/// a degree. LCH's hue needs the tenths — at high chroma a
	/// whole degree of hue is several sRGB steps.
	DegreesFixed(u8),
	/// A fixed-point number with this many decimals: one slider
	/// unit is 10^-decimals of the real value (XYZ hundredths, xyY
	/// chromaticity ten-thousandths).
	Fixed(u8),
}

/// One editable channel of a color format: its name ("lightness"),
/// the system it belongs to ("LCH" — the label a screen reader
/// hears is the two together, "LCH lightness"), its full slider
/// range from the smallest to the largest value it can take, and
/// its fine step (what the arrow keys should move — the smallest
/// step there is).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChannelDef {
	pub name: &'static str,
	pub system: &'static str,
	pub min: i32,
	pub max: i32,
	pub step: i32,
	pub unit: ChannelUnit,
}

impl ChannelDef {
	/// The spoken name: system and channel together, "LCH hue".
	pub fn label(&self) -> String {
		format!("{} {}", self.system, self.name)
	}

	/// Bring a slider value inside this channel's range.
	pub fn clamp(&self, value: i32) -> i32 {
		value.clamp(self.min, self.max)
	}

	/// The value in words, as a readout shows it and an
	/// announcement speaks it.
	pub fn format_value(&self, value: i32) -> String {
		let value = self.clamp(value);
		match self.unit {
			ChannelUnit::Plain => format!("{value}"),
			ChannelUnit::Percent => format!("{value} percent"),
			ChannelUnit::Degrees => format!("{value} degrees"),
			ChannelUnit::DegreesFixed(d) => {
				format!("{:.*} degrees", d as usize, value as f64 / 10f64.powi(d as i32))
			}
			ChannelUnit::HexByte => format!("{:02X}", value as u8),
			ChannelUnit::Fixed(d) => {
				format!("{:.*}", d as usize, value as f64 / 10f64.powi(d as i32))
			}
		}
	}

	/// Slider units to the real channel value.
	fn real(&self, value: i32) -> f64 {
		let value = self.clamp(value);
		match self.unit {
			ChannelUnit::Fixed(d) | ChannelUnit::DegreesFixed(d) => {
				value as f64 / 10f64.powi(d as i32)
			}
			_ => value as f64,
		}
	}

	/// A real channel value as slider units.
	fn units(&self, real: f64) -> i32 {
		match self.unit {
			ChannelUnit::Fixed(d) | ChannelUnit::DegreesFixed(d) => {
				(real * 10f64.powi(d as i32)).round() as i32
			}
			_ => real.round() as i32,
		}
	}
}

/// A format's channels: the sliders it offers, or the reason it
/// offers none.
#[derive(Debug, Clone, PartialEq)]
pub enum ChannelSet {
	Sliders(Vec<ChannelDef>),
	ReadoutOnly(&'static str),
}

const fn ch(
	name: &'static str,
	system: &'static str,
	min: i32,
	max: i32,
	step: i32,
	unit: ChannelUnit,
) -> ChannelDef {
	ChannelDef { name, system, min, max, step, unit }
}

/// The channels of a color format, in the order the format itself
/// writes them (red, green, blue; lightness, chroma, hue; ...).
/// Ranges cover every color sRGB can hold, with headroom: the XYZ
/// family runs past 100 because the scale is D65-relative, not a
/// percent (white's Z is 108.9); LUV's u reaches 175 in sRGB, so
/// Lab's nominal ±128 would clip real colors there; Hunter Lab's
/// b reaches −200 at blue. LCH chroma tops out near 134 in sRGB.
pub fn channels(format: ColorFormat) -> ChannelSet {
	use ChannelUnit::*;
	let rgb = |system: &'static str, step: i32, unit: ChannelUnit| {
		vec![
			ch("red", system, 0, 255, step, unit),
			ch("green", system, 0, 255, step, unit),
			ch("blue", system, 0, 255, step, unit),
		]
	};
	let hue = |system: &'static str| ch("hue", system, 0, 359, 1, Degrees);
	// The Lab family moves in tenths: whole numbers visibly
	// shift dark saturated colors on the way back (rounding a
	// and b by half a unit is several sRGB steps of red at blue).
	let light = |system: &'static str| ch("lightness", system, 0, 1000, 1, Fixed(1));
	ChannelSet::Sliders(match format {
		ColorFormat::Hex => rgb("Hex", 1, HexByte),
		ColorFormat::Rgb => rgb("RGB", 1, Plain),
		ColorFormat::Hsl => vec![
			hue("HSL"),
			ch("saturation", "HSL", 0, 100, 1, Percent),
			ch("lightness", "HSL", 0, 100, 1, Percent),
		],
		ColorFormat::Hsv => vec![
			hue("HSV"),
			ch("saturation", "HSV", 0, 100, 1, Percent),
			ch("value", "HSV", 0, 100, 1, Percent),
		],
		ColorFormat::Cmyk => vec![
			ch("cyan", "CMYK", 0, 100, 1, Percent),
			ch("magenta", "CMYK", 0, 100, 1, Percent),
			ch("yellow", "CMYK", 0, 100, 1, Percent),
			ch("black", "CMYK", 0, 100, 1, Percent),
		],
		ColorFormat::Xyz => vec![
			ch("x", "XYZ", 0, 11000, 1, Fixed(2)),
			ch("y", "XYZ", 0, 11000, 1, Fixed(2)),
			ch("z", "XYZ", 0, 11000, 1, Fixed(2)),
		],
		ColorFormat::XyY => vec![
			ch("x", "xyY", 0, 10000, 1, Fixed(4)),
			ch("y", "xyY", 0, 10000, 1, Fixed(4)),
			ch("luminance", "xyY", 0, 10000, 1, Fixed(2)),
		],
		ColorFormat::Lab => vec![
			light("Lab"),
			ch("a", "Lab", -1280, 1270, 1, Fixed(1)),
			ch("b", "Lab", -1280, 1270, 1, Fixed(1)),
		],
		ColorFormat::Lch => vec![
			light("LCH"),
			ch("chroma", "LCH", 0, 1500, 1, Fixed(1)),
			ch("hue", "LCH", 0, 3599, 1, DegreesFixed(1)),
		],
		ColorFormat::Luv => vec![
			light("LUV"),
			ch("u", "LUV", -2000, 2000, 1, Fixed(1)),
			ch("v", "LUV", -2000, 2000, 1, Fixed(1)),
		],
		ColorFormat::HunterLab => vec![
			light("Hunter Lab"),
			ch("a", "Hunter Lab", -1400, 1400, 1, Fixed(1)),
			ch("b", "Hunter Lab", -2200, 1100, 1, Fixed(1)),
		],
		ColorFormat::Decimal => vec![
			ch("red byte", "Decimal", 0, 255, 1, Plain),
			ch("green byte", "Decimal", 0, 255, 1, Plain),
			ch("blue byte", "Decimal", 0, 255, 1, Plain),
		],
		ColorFormat::WebSafe => rgb("Web safe", 51, Plain),
		ColorFormat::Binary => {
			return ChannelSet::ReadoutOnly(
				"Binary is a readout, not an editing system: it writes the color out as bits, \
				 and bits are not values you can slide. Choose another color format to edit \
				 its channels.",
			);
		}
	})
}

/// A color's channel values in a format, in slider units, one per
/// channel `channels` defines. Empty for binary (a readout).
pub fn channel_values(format: ColorFormat, color: Srgb) -> Vec<i32> {
	let ChannelSet::Sliders(defs) = channels(format) else { return Vec::new() };
	let raw: Vec<f64> = match format {
		ColorFormat::Hex | ColorFormat::Rgb | ColorFormat::Decimal => {
			vec![color.r as f64, color.g as f64, color.b as f64]
		}
		ColorFormat::WebSafe => {
			let s = color.to_web_safe();
			vec![s.r as f64, s.g as f64, s.b as f64]
		}
		ColorFormat::Hsl => {
			let (h, s, l) = color.to_hsl();
			vec![h as f64, s as f64, l as f64]
		}
		ColorFormat::Hsv => {
			let (h, s, v) = color.to_hsv();
			vec![h as f64, s as f64, v as f64]
		}
		ColorFormat::Cmyk => {
			let (c, m, y, k) = color.to_cmyk();
			vec![c, m, y, k]
		}
		ColorFormat::Xyz => {
			let xyz = color.to_xyz();
			vec![xyz.x, xyz.y, xyz.z]
		}
		ColorFormat::XyY => {
			let (x, y, lum) = color.to_xyy();
			vec![x, y, lum]
		}
		ColorFormat::Lab => {
			let lab = color.to_lab();
			vec![lab.l, lab.a, lab.b]
		}
		ColorFormat::Lch => {
			let (l, c, h) = color.to_lch();
			vec![l, c, h]
		}
		ColorFormat::Luv => {
			let (l, u, v) = color.to_luv();
			vec![l, u, v]
		}
		ColorFormat::HunterLab => {
			let (l, a, b) = color.to_hunter_lab();
			vec![l, a, b]
		}
		ColorFormat::Binary => Vec::new(),
	};
	defs.iter().zip(raw).map(|(d, r)| d.clamp(d.units(r))).collect()
}

/// The color a format's channel values name, in slider units:
/// the inverse of `channel_values`, through the same conversions.
/// Values outside a channel's range clamp; a short value list
/// reads its missing channels as each channel's minimum. Binary
/// has no channels and names black — callers show its readout
/// note instead of ever calling this.
pub fn color_from_channels(format: ColorFormat, values: &[i32]) -> Srgb {
	let ChannelSet::Sliders(defs) = channels(format) else { return Srgb::BLACK };
	let real = |i: usize| defs.get(i).map_or(0.0, |d| d.real(*values.get(i).unwrap_or(&0)));
	let byte = |i: usize| {
		defs.get(i).map_or(0, |d| d.clamp(*values.get(i).unwrap_or(&0))) as u8
	};
	match format {
		ColorFormat::Hex | ColorFormat::Rgb => Srgb::new(byte(0), byte(1), byte(2)),
		ColorFormat::Decimal => Srgb::from_decimal(
			(byte(0) as u32) << 16 | (byte(1) as u32) << 8 | byte(2) as u32,
		),
		ColorFormat::WebSafe => Srgb::new(byte(0), byte(1), byte(2)).to_web_safe(),
		ColorFormat::Hsl => srgb_from_hsl(real(0) as f32, real(1) as f32, real(2) as f32),
		ColorFormat::Hsv => srgb_from_hsv(real(0) as f32, real(1) as f32, real(2) as f32),
		ColorFormat::Cmyk => Srgb::from_cmyk(real(0), real(1), real(2), real(3)),
		ColorFormat::Xyz => Srgb::from_xyz(Xyz { x: real(0), y: real(1), z: real(2) }),
		ColorFormat::XyY => Srgb::from_xyy(real(0), real(1), real(2)),
		ColorFormat::Lab => Srgb::from_lab(Lab { l: real(0), a: real(1), b: real(2) }),
		ColorFormat::Lch => Srgb::from_lch(real(0), real(1), real(2)),
		ColorFormat::Luv => Srgb::from_luv(real(0), real(1), real(2)),
		ColorFormat::HunterLab => Srgb::from_hunter_lab(real(0), real(1), real(2)),
		ColorFormat::Binary => Srgb::BLACK,
	}
}

/// A color with one channel set to an absolute slider value —
/// the channel sliders' direct-set semantics in one call: the
/// other channels keep the values they have in this format.
pub fn set_channel(format: ColorFormat, color: Srgb, index: usize, value: i32) -> Srgb {
	let mut values = channel_values(format, color);
	match values.get_mut(index) {
		Some(slot) => {
			*slot = value;
			color_from_channels(format, &values)
		}
		None => color,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn close(a: f64, b: f64, tol: f64) -> bool {
		(a - b).abs() <= tol
	}

	fn channel_close(a: Srgb, b: Srgb, tol: i32) -> bool {
		(a.r as i32 - b.r as i32).abs() <= tol
			&& (a.g as i32 - b.g as i32).abs() <= tol
			&& (a.b as i32 - b.b as i32).abs() <= tol
	}

	/// A spread of colors across the whole cube, corners included.
	fn grid() -> Vec<Srgb> {
		let vals = [0u8, 51, 102, 153, 204, 255];
		let mut out = Vec::new();
		for &r in &vals {
			for &g in &vals {
				for &b in &vals {
					out.push(Srgb::new(r, g, b));
				}
			}
		}
		out
	}

	#[test]
	fn xyz_anchors_and_roundtrip() {
		// White is the D65 white point itself.
		let w = Srgb::WHITE.to_xyz();
		assert!(close(w.x, 95.047, 0.05) && close(w.y, 100.0, 0.05) && close(w.z, 108.883, 0.05), "{w:?}");
		let b = Srgb::BLACK.to_xyz();
		assert!(close(b.x, 0.0, 1e-9) && close(b.y, 0.0, 1e-9) && close(b.z, 0.0, 1e-9));
		// Pure red's published XYZ (D65): 41.25, 21.27, 1.93.
		let r = Srgb::new(255, 0, 0).to_xyz();
		assert!(close(r.x, 41.25, 0.1) && close(r.y, 21.27, 0.1) && close(r.z, 1.93, 0.1), "{r:?}");
		for c in grid() {
			assert!(channel_close(Srgb::from_xyz(c.to_xyz()), c, 1), "{c}");
		}
	}

	#[test]
	fn xyy_anchors_and_roundtrip() {
		// D65 white's chromaticity is the white point by definition.
		let (x, y, lum) = Srgb::WHITE.to_xyy();
		assert!(close(x, 0.31271, 0.001) && close(y, 0.32902, 0.001) && close(lum, 100.0, 0.05));
		assert_eq!(Srgb::BLACK.to_xyy(), (0.0, 0.0, 0.0));
		// Pure red chromaticity is the sRGB gamut corner (0.64, 0.33).
		let (x, y, _) = Srgb::new(255, 0, 0).to_xyy();
		assert!(close(x, 0.64, 0.002) && close(y, 0.33, 0.002), "{x} {y}");
		for c in grid() {
			let (x, y, l) = c.to_xyy();
			if l > 0.0 {
				assert!(channel_close(Srgb::from_xyy(x, y, l), c, 1), "{c}");
			}
		}
	}

	#[test]
	fn cmyk_anchors_and_roundtrip() {
		assert_eq!(Srgb::BLACK.to_cmyk(), (0.0, 0.0, 0.0, 100.0));
		assert_eq!(Srgb::WHITE.to_cmyk(), (0.0, 0.0, 0.0, 0.0));
		let (c, m, y, k) = Srgb::new(255, 0, 0).to_cmyk();
		assert!(close(c, 0.0, 0.01) && close(m, 100.0, 0.01) && close(y, 100.0, 0.01) && close(k, 0.0, 0.01));
		for col in grid() {
			let (c, m, y, k) = col.to_cmyk();
			assert!(channel_close(Srgb::from_cmyk(c, m, y, k), col, 1), "{col}");
		}
	}

	#[test]
	fn lab_anchors_and_roundtrip() {
		let w = Srgb::WHITE.to_lab();
		assert!(close(w.l, 100.0, 0.05) && close(w.a, 0.0, 0.05) && close(w.b, 0.0, 0.05), "{w:?}");
		let b = Srgb::BLACK.to_lab();
		assert!(close(b.l, 0.0, 0.01) && close(b.a, 0.0, 0.01) && close(b.b, 0.0, 0.01), "{b:?}");
		// Published CIE Lab for sRGB red (D65): about 53.2, 80.1, 67.2.
		let r = Srgb::new(255, 0, 0).to_lab();
		assert!(close(r.l, 53.24, 0.3) && close(r.a, 80.09, 0.3) && close(r.b, 67.20, 0.3), "{r:?}");
		for c in grid() {
			assert!(channel_close(Srgb::from_lab(c.to_lab()), c, 1), "{c}");
		}
	}

	#[test]
	fn lch_roundtrip_and_anchors() {
		let (l, c, h) = Srgb::new(255, 0, 0).to_lch();
		assert!(close(l, 53.24, 0.3) && close(c, 104.55, 0.5), "{l} {c}");
		assert!(close(h, 39.99, 0.5), "red LCH hue {h}");
		for col in grid() {
			let (l, c, h) = col.to_lch();
			assert!(channel_close(Srgb::from_lch(l, c, h), col, 1), "{col}");
		}
	}

	#[test]
	fn luv_roundtrip_and_white() {
		let (l, u, v) = Srgb::WHITE.to_luv();
		assert!(close(l, 100.0, 0.05) && close(u, 0.0, 0.1) && close(v, 0.0, 0.1), "{l} {u} {v}");
		assert_eq!(Srgb::BLACK.to_luv(), (0.0, 0.0, 0.0));
		for c in grid() {
			let (l, u, v) = c.to_luv();
			assert!(channel_close(Srgb::from_luv(l, u, v), c, 2), "{c}");
		}
	}

	#[test]
	fn hunter_lab_white_black_and_roundtrip() {
		let (l, a, b) = Srgb::WHITE.to_hunter_lab();
		assert!(close(l, 100.0, 0.1) && close(a, 0.0, 0.15) && close(b, 0.0, 0.15), "{l} {a} {b}");
		assert_eq!(Srgb::BLACK.to_hunter_lab(), (0.0, 0.0, 0.0));
		// Red is firmly on the +a side with L near 46.
		let (l, a, _) = Srgb::new(255, 0, 0).to_hunter_lab();
		assert!(close(l, 46.1, 0.5) && a > 60.0, "{l} {a}");
		for c in grid() {
			let (l, a, b) = c.to_hunter_lab();
			assert!(channel_close(Srgb::from_hunter_lab(l, a, b), c, 2), "{c}");
		}
	}

	#[test]
	fn decimal_web_safe_and_binary() {
		assert_eq!(Srgb::BLACK.to_decimal(), 0);
		assert_eq!(Srgb::WHITE.to_decimal(), 16_777_215);
		assert_eq!(Srgb::new(15, 94, 104).to_decimal(), 0x0F_5E_68);
		for c in grid() {
			assert_eq!(Srgb::from_decimal(c.to_decimal()), c);
		}
		// Web-safe snaps to the 51 grid and is a fixed point.
		assert_eq!(Srgb::new(15, 94, 104).to_web_safe(), Srgb::new(0, 102, 102));
		for c in grid() {
			let s = c.to_web_safe();
			assert_eq!(s.to_web_safe(), s);
			assert!(s.r % 51 == 0 && s.g % 51 == 0 && s.b % 51 == 0);
		}
		assert_eq!(Srgb::new(15, 94, 104).to_binary(), "00001111 01011110 01101000");
		assert_eq!(Srgb::from_binary("00001111 01011110 01101000"), Some(Srgb::new(15, 94, 104)));
		assert_eq!(Srgb::from_binary("111111111111111111111111"), Some(Srgb::WHITE));
		assert_eq!(Srgb::from_binary("1010"), None);
		assert_eq!(Srgb::from_binary("00001111 01011110 0110100x"), None);
		for c in grid() {
			assert_eq!(Srgb::from_binary(&c.to_binary()), Some(c));
		}
	}

	#[test]
	fn dominant_wavelength_anchors() {
		// Published dominant wavelengths of the sRGB primaries (D65):
		// red about 611 nm, green about 549 nm, blue about 464 nm.
		let r = Srgb::new(255, 0, 0).dominant_wavelength().unwrap();
		assert!(!r.complementary && (605.0..=620.0).contains(&r.nanometres), "{r:?}");
		assert!(r.purity > 90.0, "{r:?}");
		let g = Srgb::new(0, 255, 0).dominant_wavelength().unwrap();
		assert!(!g.complementary && (543.0..=556.0).contains(&g.nanometres), "{g:?}");
		let b = Srgb::new(0, 0, 255).dominant_wavelength().unwrap();
		assert!(!b.complementary && (458.0..=472.0).contains(&b.nanometres), "{b:?}");
		// Yellow is a spectrum-side color too, around 570-575.
		let y = Srgb::new(255, 255, 0).dominant_wavelength().unwrap();
		assert!(!y.complementary && (565.0..=580.0).contains(&y.nanometres), "{y:?}");
		// Magenta sits on the purple line: no dominant wavelength,
		// only a complementary one (in the greens).
		let m = Srgb::new(255, 0, 255).dominant_wavelength().unwrap();
		assert!(m.complementary, "{m:?}");
		assert!((500.0..=570.0).contains(&m.nanometres), "{m:?}");
		assert!(m.spoken().contains("complementary wavelength"), "{}", m.spoken());
		// Grays and white have no wavelength at all.
		assert_eq!(Srgb::new(120, 120, 120).dominant_wavelength(), None);
		assert_eq!(Srgb::WHITE.dominant_wavelength(), None);
		assert_eq!(Srgb::BLACK.dominant_wavelength(), None);
	}

	#[test]
	fn wavelength_to_srgb_anchors() {
		// The fit's own anchor hues: 530 reads green, 470 blue, 620 red-orange.
		let g = wavelength_to_srgb(530.0);
		assert!(g.g == 255 && g.r >= 85 && g.r <= 105 && g.b == 0, "{g}");
		let b = wavelength_to_srgb(470.0);
		assert!(b.b == 255 && b.r == 0 && b.g > 50 && b.g < 200, "{b}");
		let r = wavelength_to_srgb(620.0);
		assert!(r.r == 255 && r.b == 0 && r.g < 130, "{r}");
		// Out-of-band inputs clamp instead of panicking.
		assert_eq!(wavelength_to_srgb(100.0), wavelength_to_srgb(380.0));
		assert_eq!(wavelength_to_srgb(2000.0), wavelength_to_srgb(780.0));
	}

	#[test]
	fn frequency_folds_by_octaves() {
		// A440 lands in the orange-red, by the published folding
		// convention (440 Hz doubled 40 times is about 484 THz).
		let (c, nm) = frequency_to_color(440.0).unwrap();
		assert!((600.0..=640.0).contains(&nm), "{nm}");
		assert!(c.r > 200 && c.b == 0, "{c}");
		// An octave up is the same color: that is the whole convention.
		let (c2, nm2) = frequency_to_color(880.0).unwrap();
		assert_eq!(c, c2);
		assert!((nm - nm2).abs() < 0.001);
		// The audible extremes fold in without panicking.
		assert!(frequency_to_color(20.0).is_some());
		assert!(frequency_to_color(20_000.0).is_some());
		assert_eq!(frequency_to_color(0.0), None);
		assert_eq!(frequency_to_color(-5.0), None);
	}

	#[test]
	fn blend_subtract_gradient() {
		let (a, b) = (Srgb::new(200, 60, 40), Srgb::new(40, 60, 200));
		assert_eq!(blend(a, b, 0), a);
		assert_eq!(blend(a, b, 100), b);
		let mid = blend(a, b, 50);
		assert!(mid != a && mid != b);
		// Black-white midpoint in OKLab: OKLCH lightness 0.5, which
		// renders near 99 — darker than CIE Lab's middle gray (119),
		// because the two lightness scales are different curves.
		let gray = blend(Srgb::BLACK, Srgb::WHITE, 50);
		assert!(gray.r >= 92 && gray.r <= 106, "{gray}");
		assert_eq!(gray.r, gray.g);
		assert_eq!(subtract(a, Srgb::BLACK), a);
		assert_eq!(subtract(a, Srgb::WHITE), Srgb::BLACK);
		assert_eq!(subtract(Srgb::new(100, 50, 200), Srgb::new(30, 80, 10)), Srgb::new(70, 0, 190));
		let ramp = gradient(a, b, 5);
		assert_eq!(ramp.len(), 5);
		assert_eq!(ramp[0], a);
		assert_eq!(ramp[4], b);
		assert_eq!(gradient(a, b, 1).len(), 2, "steps clamp to at least 2");
	}

	#[test]
	fn random_color_is_deterministic() {
		assert_eq!(random_color(42), random_color(42));
		assert_ne!(random_color(42), random_color(43));
		assert_eq!(random_color(0), random_color(0), "zero seed is nudged, not stuck");
		let mut rng = XorShift::new(7);
		let (c1, c2) = (rng.color(), rng.color());
		assert_ne!(c1, c2, "the stream advances");
	}

	#[test]
	fn contrast_verdicts_follow_wcag_lines() {
		assert!(contrast_verdict(21.0).starts_with("excellent"));
		assert!(contrast_verdict(7.0).starts_with("excellent"));
		assert!(contrast_verdict(4.5).starts_with("good"));
		assert!(contrast_verdict(3.0).starts_with("fair"));
		assert!(contrast_verdict(2.9).starts_with("poor"));
	}

	#[test]
	fn text_readability_reports_each_text_element() {
		use crate::design::{Design, Shape};
		let mut d = Design::new(400, 200, Srgb::WHITE);
		d.shapes.push(Shape::text("Hello", "Arial", 14, 0, 0, 100, 20, Srgb::BLACK));
		d.shapes.push(Shape::text("Faint", "Arial", 14, 0, 30, 100, 20, Srgb::new(230, 230, 230)));
		let report = text_readability(&d);
		assert!(report.contains("\"Hello\""), "{report}");
		assert!(report.contains("easy to read"), "{report}");
		assert!(report.contains("\"Faint\""), "{report}");
		assert!(report.contains("unreadable"), "{report}");
		assert!(report.contains("not accounted for"), "{report}");
		let empty = Design::new(10, 10, Srgb::WHITE);
		assert!(text_readability(&empty).contains("no text elements"));
	}

	#[test]
	fn ryb_primaries_map_to_themselves() {
		// The paint primaries sit at the same angles as screen hues
		// for red and blue, and RYB yellow (120) is screen yellow (60).
		assert!(close(ryb_hue_to_rgb(0.0) as f64, 0.0, 0.01));
		assert!(close(ryb_hue_to_rgb(120.0) as f64, 60.0, 0.01));
		assert!(close(ryb_hue_to_rgb(240.0) as f64, 240.0, 0.01));
		assert!(close(rgb_hue_to_ryb(0.0) as f64, 0.0, 0.01));
		assert!(close(rgb_hue_to_ryb(60.0) as f64, 120.0, 0.01));
		assert!(close(rgb_hue_to_ryb(240.0) as f64, 240.0, 0.01));
		// The two maps are inverses across the circle.
		for h in (0..360).step_by(15) {
			let back = ryb_hue_to_rgb(rgb_hue_to_ryb(h as f32));
			assert!(close(back as f64, h as f64, 0.5), "{h} -> {back}");
		}
	}

	#[test]
	fn ryb_complement_of_red_is_green_not_cyan() {
		// On the paint wheel, red's opposite is green — the entire
		// reason the wheel exists.
		let comp = rotate_hue_ryb(Srgb::new(255, 0, 0), 180.0);
		assert_eq!(comp, Srgb::new(0, 255, 0), "{comp}");
		// Yellow's opposite is violet (screen hue near 270).
		let comp = rotate_hue_ryb(Srgb::new(255, 255, 0), 180.0);
		let (h, _, _) = comp.to_hsl();
		assert!((h - 270.0).abs() < 8.0, "yellow's RYB complement hue {h}");
		// Achromatic colors have no hue and come back unchanged.
		assert_eq!(rotate_hue_ryb(Srgb::new(120, 120, 120), 180.0), Srgb::new(120, 120, 120));
		// A full turn is identity (within a rounding step).
		let c = Srgb::new(200, 60, 40);
		assert!(channel_close(rotate_hue_ryb(c, 360.0), c, 1));
	}

	#[test]
	fn every_format_has_channels_or_an_honest_reason() {
		let counts: &[(ColorFormat, usize)] = &[
			(ColorFormat::Hex, 3),
			(ColorFormat::Rgb, 3),
			(ColorFormat::Hsl, 3),
			(ColorFormat::Hsv, 3),
			(ColorFormat::Cmyk, 4),
			(ColorFormat::Xyz, 3),
			(ColorFormat::XyY, 3),
			(ColorFormat::Lab, 3),
			(ColorFormat::Lch, 3),
			(ColorFormat::Luv, 3),
			(ColorFormat::HunterLab, 3),
			(ColorFormat::Decimal, 3),
			(ColorFormat::WebSafe, 3),
		];
		for f in ColorFormat::ALL {
			match channels(*f) {
				ChannelSet::Sliders(defs) => {
					let want = counts.iter().find(|(cf, _)| cf == f).map(|(_, n)| *n);
					assert_eq!(Some(defs.len()), want, "{} channel count", f.name());
					for d in &defs {
						assert!(d.min < d.max, "{} {} has an empty range", f.name(), d.name);
						assert!(d.step >= 1, "{} {} has no fine step", f.name(), d.name);
						assert!(!d.label().is_empty());
					}
				}
				ChannelSet::ReadoutOnly(note) => {
					assert_eq!(*f, ColorFormat::Binary, "only binary is readout-only");
					assert!(note.contains("readout, not an editing system"), "{note}");
				}
			}
		}
		// Web safe steps land on the 51 grid and nothing else.
		let ChannelSet::Sliders(defs) = channels(ColorFormat::WebSafe) else {
			panic!("web safe has sliders")
		};
		assert!(defs.iter().all(|d| d.step == 51 && d.min == 0 && d.max == 255));
	}

	#[test]
	fn channel_values_speak_in_words() {
		let defs = |f: ColorFormat| match channels(f) {
			ChannelSet::Sliders(d) => d,
			_ => panic!("{} has sliders", f.name()),
		};
		// Hex speaks its bytes as two-digit hex.
		assert_eq!(defs(ColorFormat::Hex)[0].format_value(0x5E), "5E");
		assert_eq!(defs(ColorFormat::Hex)[0].format_value(5), "05");
		assert_eq!(defs(ColorFormat::Hex)[0].label(), "Hex red");
		// LCH: lightness in tenths, degrees for hue.
		assert_eq!(defs(ColorFormat::Lch)[0].format_value(620), "62.0");
		assert_eq!(defs(ColorFormat::Lch)[2].format_value(1870), "187.0 degrees");
		assert_eq!(defs(ColorFormat::Lch)[2].label(), "LCH hue");
		// Percents say so.
		assert_eq!(defs(ColorFormat::Hsl)[1].format_value(75), "75 percent");
		assert_eq!(defs(ColorFormat::Cmyk)[3].format_value(10), "10 percent");
		// XYZ speaks hundredths, xyY chromaticity ten-thousandths
		// (thousandths visibly shift saturated colors near the
		// gamut edge, where a small y magnifies rounding).
		assert_eq!(defs(ColorFormat::Xyz)[0].format_value(4125), "41.25");
		assert_eq!(defs(ColorFormat::XyY)[0].format_value(3127), "0.3127");
		assert_eq!(defs(ColorFormat::XyY)[2].format_value(235), "2.35");
		// Decimal names its bytes as bytes.
		assert_eq!(defs(ColorFormat::Decimal)[1].label(), "Decimal green byte");
	}

	#[test]
	fn set_channel_matches_the_direct_conversions() {
		let c = Srgb::new(15, 94, 104);
		// Hex red is the byte itself.
		assert_eq!(set_channel(ColorFormat::Hex, c, 0, 0xAB), Srgb::new(0xAB, 94, 104));
		// LCH hue: setting the hue channel is from_lch at that hue
		// (LCH lightness and chroma ride in tenths).
		let (l, ch, _) = c.to_lch();
		let t1 = |x: f64| (x * 10.0).round() / 10.0;
		assert_eq!(set_channel(ColorFormat::Lch, c, 2, 0), Srgb::from_lch(t1(l), t1(ch), 0.0));
		// Lab a likewise: 200 tenths is a 20.0.
		let lab = c.to_lab();
		assert_eq!(
			set_channel(ColorFormat::Lab, c, 1, 200),
			Srgb::from_lab(Lab { l: t1(lab.l), a: 20.0, b: t1(lab.b) })
		);
		// CMYK key to 50.
		let (cm, mm, ym, _) = c.to_cmyk();
		assert_eq!(
			set_channel(ColorFormat::Cmyk, c, 3, 50),
			Srgb::from_cmyk(cm.round(), mm.round(), ym.round(), 50.0)
		);
		// HSL lightness to 50.
		let (h, s, _) = c.to_hsl();
		assert_eq!(
			set_channel(ColorFormat::Hsl, c, 2, 50),
			srgb_from_hsl(h.round(), s.round(), 50.0)
		);
		// XYZ's Y channel is in hundredths: 5000 is Y 50.0.
		let xyz = c.to_xyz();
		assert_eq!(
			set_channel(ColorFormat::Xyz, c, 1, 5000),
			Srgb::from_xyz(Xyz { x: (xyz.x * 100.0).round() / 100.0, y: 50.0, z: (xyz.z * 100.0).round() / 100.0 })
		);
		// Decimal's green byte is the packed number's middle byte.
		assert_eq!(set_channel(ColorFormat::Decimal, c, 1, 0), Srgb::new(15, 0, 104));
		// Web safe snaps: 51 is a grid point, and setting red to it
		// gives exactly the snapped color with red 51.
		assert_eq!(
			set_channel(ColorFormat::WebSafe, c, 0, 51),
			Srgb::new(51, 102, 102)
		);
		// Binary has no channels to set.
		assert_eq!(set_channel(ColorFormat::Binary, c, 0, 1), c);
		assert!(channel_values(ColorFormat::Binary, c).is_empty());
	}

	#[test]
	fn channel_round_trips_rebuild_the_color() {
		// Exact formats rebuild exactly.
		for f in [ColorFormat::Hex, ColorFormat::Rgb, ColorFormat::Decimal] {
			for c in grid() {
				assert_eq!(color_from_channels(f, &channel_values(f, c)), c, "{} {c}", f.name());
			}
		}
		// Web safe rebuilds the snapped color, exactly.
		for c in grid() {
			assert_eq!(
				color_from_channels(ColorFormat::WebSafe, &channel_values(ColorFormat::WebSafe, c)),
				c.to_web_safe()
			);
		}
		// The converted systems rebuild within two sRGB steps —
		// the same tolerance the conversions' own round-trips use.
		// LCH gets four, at one known spot: a saturated gamut-edge
		// color (pure green is the worst) has a channel at exactly
		// zero that polar rounding can only push up, and L, C and
		// hue each rounded to a tenth lands it a few steps high.
		for (f, tol) in [
			(ColorFormat::Hsl, 2),
			(ColorFormat::Hsv, 2),
			(ColorFormat::Cmyk, 2),
			(ColorFormat::Xyz, 2),
			(ColorFormat::XyY, 2),
			(ColorFormat::Lab, 2),
			(ColorFormat::Lch, 4),
			(ColorFormat::Luv, 2),
			(ColorFormat::HunterLab, 2),
		] {
			for c in grid() {
				let back = color_from_channels(f, &channel_values(f, c));
				assert!(channel_close(back, c, tol), "{} {c} -> {back}", f.name());
			}
		}
	}

	#[test]
	fn channel_values_stay_inside_their_ranges() {
		// Gamut extremes included: saturated blue stresses LUV's u
		// and Hunter's b, white stresses XYZ's z.
		let extremes = [
			Srgb::new(0, 0, 255),
			Srgb::new(0, 255, 0),
			Srgb::new(255, 0, 0),
			Srgb::WHITE,
			Srgb::BLACK,
			Srgb::new(255, 0, 255),
		];
		for f in ColorFormat::ALL {
			let ChannelSet::Sliders(defs) = channels(*f) else { continue };
			for c in grid().into_iter().chain(extremes) {
				let values = channel_values(*f, c);
				assert_eq!(values.len(), defs.len());
				for (d, v) in defs.iter().zip(&values) {
					assert!(
						(d.min..=d.max).contains(v),
						"{} {} value {v} outside {}..={} for {c}",
						f.name(),
						d.name,
						d.min,
						d.max
					);
				}
			}
		}
	}
}

