//! chroma-core: predictable color work.
//!
//! Colors live natively in OKLCH (lightness, chroma, hue angle). Nothing is
//! ever "mixed" or "blended": every derived color is a pure parameter
//! transform of its source — rotate the hue by a known angle, step the
//! lightness by a known amount, scale the chroma by a known factor. If you
//! know the transform, you know the result before you see it. That is the
//! whole point of the tool this core powers.

use std::fmt;

pub mod design;

/// An sRGB color, 8 bits per channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Srgb {
	pub r: u8,
	pub g: u8,
	pub b: u8,
}

impl Srgb {
	pub const BLACK: Srgb = Srgb { r: 0, g: 0, b: 0 };
	pub const WHITE: Srgb = Srgb { r: 255, g: 255, b: 255 };

	pub fn new(r: u8, g: u8, b: u8) -> Srgb {
		Srgb { r, g, b }
	}

	/// Parse "#RRGGBB" or "RRGGBB" (also the 3-digit shorthand "#RGB").
	pub fn from_hex(s: &str) -> Option<Srgb> {
		let s = s.strip_prefix('#').unwrap_or(s);
		let v = u32::from_str_radix(s, 16).ok()?;
		match s.len() {
			6 => Some(Srgb::new(
				((v >> 16) & 0xff) as u8,
				((v >> 8) & 0xff) as u8,
				(v & 0xff) as u8,
			)),
			3 => Some(Srgb::new(
				(((v >> 8) & 0xf) * 17) as u8,
				(((v >> 4) & 0xf) * 17) as u8,
				((v & 0xf) * 17) as u8,
			)),
			_ => None,
		}
	}

	pub fn to_hex(self) -> String {
		format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
	}

	fn to_linear(self) -> [f32; 3] {
		fn f(u: u8) -> f32 {
			let x = u as f32 / 255.0;
			if x <= 0.04045 {
				x / 12.92
			} else {
				((x + 0.055) / 1.055).powf(2.4)
			}
		}
		[f(self.r), f(self.g), f(self.b)]
	}

	fn from_linear(lin: [f32; 3]) -> Srgb {
		fn f(x: f32) -> u8 {
			let y = if x <= 0.0031308 {
				x * 12.92
			} else {
				1.055 * x.powf(1.0 / 2.4) - 0.055
			};
			(y.clamp(0.0, 1.0) * 255.0).round() as u8
		}
		Srgb::new(f(lin[0]), f(lin[1]), f(lin[2]))
	}

	/// WCAG relative luminance (0.0 black .. 1.0 white).
	pub fn luminance(self) -> f32 {
		let [r, g, b] = self.to_linear();
		0.2126 * r + 0.7152 * g + 0.0722 * b
	}

	/// WCAG contrast ratio against another color (1.0 .. 21.0).
	pub fn contrast(self, other: Srgb) -> f32 {
		let (a, b) = (self.luminance(), other.luminance());
		let (hi, lo) = if a >= b { (a, b) } else { (b, a) };
		(hi + 0.05) / (lo + 0.05)
	}

	pub fn to_oklch(self) -> Oklch {
		let [r, g, b] = self.to_linear();
		// Linear sRGB -> LMS (Ottosson).
		let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
		let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
		let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;
		let (l, m, s) = (l.cbrt(), m.cbrt(), s.cbrt());
		// LMS -> OKLab.
		let lab_l = 0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s;
		let lab_a = 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s;
		let lab_b = 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s;
		Oklch::from_oklab(lab_l, lab_a, lab_b)
	}
}

impl fmt::Display for Srgb {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.to_hex())
	}
}

/// A color in OKLCH: lightness 0..1, chroma 0..~0.4, hue in degrees 0..360.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklch {
	pub l: f32,
	pub c: f32,
	pub h: f32,
}

impl Oklch {
	pub fn new(l: f32, c: f32, h: f32) -> Oklch {
		Oklch {
			l: l.clamp(0.0, 1.0),
			c: c.max(0.0),
			h: wrap_hue(h),
		}
	}

	fn from_oklab(l: f32, a: f32, b: f32) -> Oklch {
		Oklch {
			l: l.clamp(0.0, 1.0),
			c: (a * a + b * b).sqrt(),
			h: wrap_hue(b.atan2(a).to_degrees()),
		}
	}

	fn to_oklab(self) -> (f32, f32, f32) {
		let hr = self.h.to_radians();
		(self.l, self.c * hr.cos(), self.c * hr.sin())
	}

	fn to_linear_rgb(self) -> [f32; 3] {
		let (l, a, b) = self.to_oklab();
		// OKLab -> LMS.
		let ll = l + 0.3963377774 * a + 0.2158037573 * b;
		let mm = l - 0.1055613458 * a - 0.0638541728 * b;
		let ss = l - 0.0894841775 * a - 1.2914855480 * b;
		let (ll, mm, ss) = (ll * ll * ll, mm * mm * mm, ss * ss * ss);
		// LMS -> linear sRGB.
		[
			4.0767416621 * ll - 3.3077115913 * mm + 0.2309699292 * ss,
			-1.2684380046 * ll + 2.6097574011 * mm - 0.3413193965 * ss,
			-0.0041960863 * ll - 0.7034186147 * mm + 1.7076147010 * ss,
		]
	}

	fn in_srgb_gamut(self) -> bool {
		self.to_linear_rgb().iter().all(|v| (-0.0001..=1.0001).contains(v))
	}

	/// Convert to sRGB. If the color sits outside the sRGB gamut, the
	/// chroma is reduced (lightness and hue held exact) until it fits —
	/// never a channel clamp, which would silently shift the hue and
	/// break the relationships this library exists to guarantee. A
	/// complementary color therefore stays at exactly hue + 180.
	pub fn to_srgb(self) -> Srgb {
		if self.in_srgb_gamut() {
			return Srgb::from_linear(self.to_linear_rgb());
		}
		// Bisect chroma down to the gamut boundary.
		let (mut lo, mut hi) = (0.0f32, self.c);
		for _ in 0..16 {
			let mid = (lo + hi) / 2.0;
			if (Oklch { c: mid, ..self }).in_srgb_gamut() {
				lo = mid;
			} else {
				hi = mid;
			}
		}
		let mapped = Oklch { c: lo, ..self };
		Srgb::from_linear(mapped.to_linear_rgb())
	}

	// ---- Pure parameter transforms: the predictable core. ----

	/// Rotate the hue by `deg` degrees (wraps at 360).
	pub fn rotate(self, deg: f32) -> Oklch {
		Oklch { h: wrap_hue(self.h + deg), ..self }
	}

	/// Set the lightness exactly (0..1).
	pub fn with_lightness(self, l: f32) -> Oklch {
		Oklch { l: l.clamp(0.0, 1.0), ..self }
	}

	/// Step the lightness by a signed amount.
	pub fn step_lightness(self, dl: f32) -> Oklch {
		self.with_lightness(self.l + dl)
	}

	/// Scale the chroma by a factor (1.0 = unchanged, 0.0 = gray).
	pub fn scale_chroma(self, f: f32) -> Oklch {
		Oklch { c: (self.c * f).max(0.0), ..self }
	}

	// ---- Schemes: known hue angles from the base color. ----

	/// The complementary color: hue + 180 degrees, same L and C.
	pub fn complementary(self) -> Oklch {
		self.rotate(180.0)
	}

	/// Split-complementary: hue +/- 150 degrees.
	pub fn split_complementary(self) -> [Oklch; 2] {
		[self.rotate(150.0), self.rotate(-150.0)]
	}

	/// Analogous: hue +/- `spread` degrees (30 is the classic step).
	pub fn analogous(self, spread: f32) -> [Oklch; 2] {
		[self.rotate(-spread), self.rotate(spread)]
	}

	/// Triadic: hue +/- 120 degrees.
	pub fn triadic(self) -> [Oklch; 2] {
		[self.rotate(120.0), self.rotate(240.0)]
	}

	/// Tetradic (rectangle): hue + 60, + 180, + 240 degrees.
	pub fn tetradic(self) -> [Oklch; 3] {
		[self.rotate(60.0), self.rotate(180.0), self.rotate(240.0)]
	}

	/// A ladder of `n` shades: same hue and chroma, lightness stepped evenly
	/// from this color down toward black. Excludes the base color itself.
	pub fn shades(self, n: usize) -> Vec<Oklch> {
		(1..=n)
			.map(|i| self.with_lightness(self.l * (1.0 - i as f32 / (n + 1) as f32)))
			.collect()
	}

	/// A ladder of `n` tints: lightness stepped evenly up toward white.
	pub fn tints(self, n: usize) -> Vec<Oklch> {
		(1..=n)
			.map(|i| self.with_lightness(self.l + (1.0 - self.l) * i as f32 / (n + 1) as f32))
			.collect()
	}

	/// A ladder of `n` tones: chroma scaled down toward gray in even steps.
	pub fn tones(self, n: usize) -> Vec<Oklch> {
		(1..=n)
			.map(|i| self.scale_chroma(1.0 - i as f32 / (n + 1) as f32))
			.collect()
	}
}

fn wrap_hue(h: f32) -> f32 {
	let h = h % 360.0;
	if h < 0.0 {
		h + 360.0
	} else {
		h
	}
}

/// The hue family name for an OKLCH hue angle. Boundaries are anchored on
/// the OKLCH hues of the pure sRGB primaries and secondaries (red 29,
/// orange 57, yellow 110, green 142, cyan/teal 195, blue 264, magenta 328),
/// which sit very differently from their HSL angles.
pub fn hue_family(h: f32) -> &'static str {
	let h = wrap_hue(h);
	match h {
		x if x < 35.0 || x >= 340.0 => "red",
		x if x < 70.0 => "orange",
		x if x < 115.0 => "yellow",
		x if x < 130.0 => "yellow-green",
		x if x < 165.0 => "green",
		x if x < 215.0 => "teal",
		x if x < 280.0 => "blue",
		x if x < 310.0 => "purple",
		_ => "magenta",
	}
}

fn lightness_word(l: f32) -> &'static str {
	match l {
		x if x < 0.15 => "near-black",
		x if x < 0.30 => "very dark",
		x if x < 0.45 => "dark",
		x if x < 0.60 => "medium",
		x if x < 0.75 => "light",
		x if x < 0.90 => "very light",
		_ => "near-white",
	}
}

fn chroma_word(c: f32) -> &'static str {
	match c {
		x if x < 0.02 => "gray",
		x if x < 0.06 => "muted",
		x if x < 0.12 => "moderate",
		x if x < 0.20 => "vivid",
		_ => "electric",
	}
}

/// A spoken-style caption for a color: everything a screen reader user
/// needs to *know* the color without seeing it. Numbers included, because
/// the numbers are what make the next color predictable.
///
/// Example: "dark moderate teal (#0F5E68), lightness 44 percent, chroma
/// 0.07, hue 208 degrees".
pub fn describe(color: Srgb) -> String {
	let o = color.to_oklch();
	if o.c < 0.02 {
		// Achromatic: hue is meaningless, don't invent one.
		return format!(
			"{} gray ({}), lightness {:.0} percent",
			lightness_word(o.l),
			color.to_hex(),
			o.l * 100.0
		);
	}
	format!(
		"{} {} {} ({}), lightness {:.0} percent, chroma {:.2}, hue {:.0} degrees",
		lightness_word(o.l),
		chroma_word(o.c),
		hue_family(o.h),
		color.to_hex(),
		o.l * 100.0,
		o.c,
		o.h
	)
}

/// One palette entry: a color plus the share of the source image it covers.
#[derive(Debug, Clone, Copy)]
pub struct PaletteEntry {
	pub color: Srgb,
	pub coverage: f32,
}

/// Extract the dominant colors of an image as a palette.
///
/// The image is sampled down, then median-cut quantized in OKLab space so
/// the boxes split along perceptual differences, not raw RGB distance.
/// Returns at most `max_colors` entries, sorted by coverage, largest first.
/// Deterministic: the same image always yields the same palette.
pub fn extract_palette(img: &image::DynamicImage, max_colors: usize) -> Vec<PaletteEntry> {
	// Sample: cap the pixel count so big photos stay fast.
	let small = img.thumbnail(160, 160).to_rgb8();
	let mut pixels: Vec<(f32, f32, f32)> = Vec::new(); // OKLab triples
	for p in small.pixels() {
		let o = Srgb::new(p[0], p[1], p[2]).to_oklch();
		let (_, a, b) = o.to_oklab();
		pixels.push((o.l, a, b));
	}
	if pixels.is_empty() || max_colors == 0 {
		return Vec::new();
	}
	let total = pixels.len() as f32;

	struct Bucket {
		px: Vec<(f32, f32, f32)>,
	}
	// Median-cut: repeatedly split the bucket with the widest channel range.
	let mut buckets = vec![Bucket { px: pixels }];
	while buckets.len() < max_colors {
		// Find the splittable bucket with the largest channel range.
		let mut best: Option<(usize, usize, f32)> = None; // (bucket, channel, range)
		for (bi, b) in buckets.iter().enumerate() {
			if b.px.len() < 2 {
				continue;
			}
			for ch in 0..3 {
				let (mut lo, mut hi) = (f32::MAX, f32::MIN);
				for p in &b.px {
					let v = [p.0, p.1, p.2][ch];
					lo = lo.min(v);
					hi = hi.max(v);
				}
				let range = hi - lo;
				if best.map_or(true, |(_, _, r)| range > r) {
					best = Some((bi, ch, range));
				}
			}
		}
		let Some((bi, ch, _)) = best else { break };
		let mut bucket = buckets.remove(bi);
		bucket.px.sort_by(|a, b| {
			let (x, y) = ([a.0, a.1, a.2][ch], [b.0, b.1, b.2][ch]);
			x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal)
		});
		let mid = bucket.px.len() / 2;
		let right = bucket.px.split_off(mid);
		buckets.push(bucket);
		buckets.push(Bucket { px: right });
	}

	let mut entries: Vec<PaletteEntry> = buckets
		.iter()
		.filter(|b| !b.px.is_empty())
		.map(|b| {
			let n = b.px.len() as f32;
			let (mut l, mut a, mut bb) = (0.0f32, 0.0f32, 0.0f32);
			for p in &b.px {
				l += p.0;
				a += p.1;
				bb += p.2;
			}
			PaletteEntry {
				color: Oklch::from_oklab(l / n, a / n, bb / n).to_srgb(),
				coverage: b.px.len() as f32 / total,
			}
		})
		.collect();
	entries.sort_by(|a, b| b.coverage.partial_cmp(&a.coverage).unwrap_or(std::cmp::Ordering::Equal));
	// Merge entries that are perceptually the same color (solid areas and
	// JPEG noise split into several buckets with near-identical centers).
	let mut merged: Vec<PaletteEntry> = Vec::new();
	for e in entries {
		let o = e.color.to_oklch();
		let (_, a, b) = o.to_oklab();
		let near = merged.iter_mut().find(|m: &&mut PaletteEntry| {
			let mo = m.color.to_oklch();
			let (_, ma, mb) = mo.to_oklab();
			((mo.l - o.l).powi(2) + (ma - a).powi(2) + (mb - b).powi(2)).sqrt() < 0.02
		});
		match near {
			Some(m) => m.coverage += e.coverage,
			None => merged.push(e),
		}
	}
	merged.sort_by(|a, b| b.coverage.partial_cmp(&a.coverage).unwrap_or(std::cmp::Ordering::Equal));
	merged
}

#[cfg(test)]
mod tests {
	use super::*;

	fn roundtrip(c: Srgb) {
		let back = c.to_oklch().to_srgb();
		for (a, b) in [(c.r, back.r), (c.g, back.g), (c.b, back.b)] {
			assert!((a as i32 - b as i32).abs() <= 1, "{c} -> {back}");
		}
	}

	#[test]
	fn hex_and_roundtrip() {
		assert_eq!(Srgb::from_hex("#0F5E68").unwrap().to_hex(), "#0F5E68");
		assert_eq!(Srgb::from_hex("#fff").unwrap(), Srgb::WHITE);
		assert_eq!(Srgb::from_hex("nope"), None);
		for c in [
			Srgb::new(15, 94, 104),
			Srgb::new(255, 128, 0),
			Srgb::new(34, 34, 34),
			Srgb::WHITE,
			Srgb::BLACK,
		] {
			roundtrip(c);
		}
	}

	#[test]
	fn complementary_is_180() {
		let base = Srgb::new(200, 60, 40).to_oklch();
		let comp = base.complementary();
		let d = (comp.h - base.h).abs();
		assert!((d - 180.0).abs() < 0.01, "hue diff {d}");
		assert_eq!(comp.l, base.l);
		assert_eq!(comp.c, base.c);
	}

	#[test]
	fn ladders_are_monotonic() {
		let base = Oklch::new(0.6, 0.15, 250.0);
		let shades = base.shades(4);
		assert!(shades.windows(2).all(|w| w[0].l > w[1].l));
		assert!(shades.iter().all(|s| s.l < base.l));
		let tints = base.tints(4);
		assert!(tints.windows(2).all(|w| w[0].l < w[1].l));
		let tones = base.tones(4);
		assert!(tones.windows(2).all(|w| w[0].c > w[1].c));
	}

	#[test]
	fn known_contrast() {
		// Black on white is the maximum ratio, 21:1.
		let r = Srgb::BLACK.contrast(Srgb::WHITE);
		assert!((r - 21.0).abs() < 0.1, "ratio {r}");
		assert_eq!(Srgb::WHITE.contrast(Srgb::WHITE), 1.0);
	}

	#[test]
	fn describe_known_colors() {
		let teal = describe(Srgb::new(15, 94, 104));
		assert!(teal.contains("teal"), "{teal}");
		assert!(teal.contains("#0F5E68"), "{teal}");
		let gray = describe(Srgb::new(120, 120, 120));
		assert!(gray.contains("gray"), "{gray}");
		assert!(!gray.contains("hue"), "{gray}");
	}

	#[test]
	fn palette_of_solid_image() {
		let img = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
			64,
			64,
			image::Rgb([200, 60, 40]),
		));
		let pal = extract_palette(&img, 6);
		assert_eq!(pal.len(), 1);
		assert!((pal[0].coverage - 1.0).abs() < 0.001);
		assert_eq!(pal[0].color, Srgb::new(200, 60, 40));
	}

	#[test]
	fn palette_of_two_halves() {
		let mut raw = image::RgbImage::new(64, 64);
		for (x, _y, p) in raw.enumerate_pixels_mut() {
			*p = if x < 32 {
				image::Rgb([10, 10, 200])
			} else {
				image::Rgb([230, 230, 230])
			};
		}
		let pal = extract_palette(&image::DynamicImage::ImageRgb8(raw), 4);
		assert!(pal.len() >= 2, "got {}", pal.len());
		assert!((pal[0].coverage - 0.5).abs() < 0.05);
	}
}
