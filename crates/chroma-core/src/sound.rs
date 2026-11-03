//! sound: pitch and position cues while drawing (spec Q7).
//!
//! Speech readouts give exact values, but they cannot run continuously
//! while a shape moves. A cue can: the horizontal position of a shape
//! becomes a stereo pan (left ear to right ear), the vertical position
//! becomes a pitch (top of the canvas high, bottom low), and the size
//! becomes the length of the tone (a bigger shape sounds longer). Every
//! mapping here is pure arithmetic on the shape's numbers, so a cue is
//! as predictable as everything else in this core: hear the tone, know
//! where the shape is.
//!
//! The module also renders a cue to WAV bytes (16-bit stereo PCM), so
//! the app side only has to hand the bytes to whatever plays sound —
//! no audio logic lives outside this tested code.

use crate::design::Shape;

/// The lowest cue pitch: the bottom edge of the canvas (220 Hz, A3).
pub const MIN_FREQ_HZ: f32 = 220.0;
/// The highest cue pitch: the top edge of the canvas (880 Hz, A5).
pub const MAX_FREQ_HZ: f32 = 880.0;
/// A point-sized shape sounds this long.
pub const MIN_DURATION_MS: u32 = 120;
/// A canvas-filling shape sounds this long.
pub const MAX_DURATION_MS: u32 = 360;

/// Sample rate of the rendered WAV tones.
pub const SAMPLE_RATE: u32 = 22_050;

/// One sound cue: a sine tone at a pitch, panned in stereo, for a time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cue {
	/// Pitch in Hz, between MIN_FREQ_HZ and MAX_FREQ_HZ.
	pub freq_hz: f32,
	/// Stereo position: -1.0 hard left, 0.0 centre, 1.0 hard right.
	pub pan: f32,
	/// Length of the tone in milliseconds.
	pub duration_ms: u32,
}

fn clamp01(v: f32) -> f32 {
	if v.is_finite() {
		v.clamp(0.0, 1.0)
	} else {
		0.0
	}
}

/// The cue for a normalized canvas position: `nx` 0.0 at the left edge
/// to 1.0 at the right, `ny` 0.0 at the top edge to 1.0 at the bottom.
/// Pitch falls as `ny` grows (top is high), pan follows `nx`, and the
/// duration is the shortest one — a bare position has no size.
pub fn cue_at(nx: f32, ny: f32) -> Cue {
	let (nx, ny) = (clamp01(nx), clamp01(ny));
	// Log-scale pitch: equal steps down the canvas sound like equal
	// steps in pitch, the way ears actually hear.
	let freq = MIN_FREQ_HZ * (MAX_FREQ_HZ / MIN_FREQ_HZ).powf(1.0 - ny);
	Cue { freq_hz: freq, pan: nx * 2.0 - 1.0, duration_ms: MIN_DURATION_MS }
}

/// The cue for a shape on a canvas: pitch and pan from the shape's
/// centre, duration from its area — the bigger the shape, the longer
/// the tone, up to the canvas-filling maximum.
pub fn cue_for_shape(shape: &Shape, canvas_width: u32, canvas_height: u32) -> Cue {
	let (cx, cy) = shape.center();
	let (w, h) = (canvas_width.max(1) as f32, canvas_height.max(1) as f32);
	let mut cue = cue_at(cx as f32 / w, cy as f32 / h);
	let area = (shape.w as f32 * shape.h as f32) / (w * h);
	let frac = area.sqrt().clamp(0.0, 1.0);
	cue.duration_ms =
		MIN_DURATION_MS + ((MAX_DURATION_MS - MIN_DURATION_MS) as f32 * frac) as u32;
	cue
}

/// Render a cue as WAV bytes: a 16-bit stereo PCM sine tone with the
/// pan baked into the channel gains and a short fade at both ends so
/// the tone never clicks. Constant-power panning keeps the loudness
/// steady as the cue moves across the stereo field.
pub fn tone_wav(cue: &Cue) -> Vec<u8> {
	let frames = (SAMPLE_RATE as usize * cue.duration_ms as usize) / 1000;
	let fade = (SAMPLE_RATE as usize / 100).max(1); // 10 ms fades
	// Pan law: angle 0 at hard left, pi/2 at hard right, pi/4 centre.
	let angle = (cue.pan.clamp(-1.0, 1.0) + 1.0) * std::f32::consts::FRAC_PI_4;
	let (gl, gr) = (angle.cos(), angle.sin());
	let mut data: Vec<u8> = Vec::with_capacity(44 + frames * 4);
	// RIFF/WAVE header for 16-bit stereo PCM.
	data.extend_from_slice(b"RIFF");
	data.extend_from_slice(&(36 + (frames * 4) as u32).to_le_bytes());
	data.extend_from_slice(b"WAVEfmt ");
	data.extend_from_slice(&16u32.to_le_bytes());
	data.extend_from_slice(&1u16.to_le_bytes()); // PCM
	data.extend_from_slice(&2u16.to_le_bytes()); // stereo
	data.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
	data.extend_from_slice(&(SAMPLE_RATE * 4).to_le_bytes());
	data.extend_from_slice(&4u16.to_le_bytes()); // block align
	data.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
	data.extend_from_slice(b"data");
	data.extend_from_slice(&((frames * 4) as u32).to_le_bytes());
	for i in 0..frames {
		let t = i as f32 / SAMPLE_RATE as f32;
		let env = (i as f32 / fade as f32)
			.min((frames - i) as f32 / fade as f32)
			.min(1.0);
		let s = (2.0 * std::f32::consts::PI * cue.freq_hz * t).sin() * env * 0.5;
		for g in [gl, gr] {
			let v = (s * g * i16::MAX as f32) as i16;
			data.extend_from_slice(&v.to_le_bytes());
		}
	}
	data
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::design::ShapeKind;
	use crate::Srgb;

	#[test]
	fn pitch_falls_top_to_bottom_and_pan_follows_left_to_right() {
		let top = cue_at(0.5, 0.0);
		let bottom = cue_at(0.5, 1.0);
		assert!((top.freq_hz - MAX_FREQ_HZ).abs() < 1.0, "{}", top.freq_hz);
		assert!((bottom.freq_hz - MIN_FREQ_HZ).abs() < 1.0, "{}", bottom.freq_hz);
		let mid = cue_at(0.5, 0.5);
		assert!(bottom.freq_hz < mid.freq_hz && mid.freq_hz < top.freq_hz);
		assert_eq!(cue_at(0.0, 0.5).pan, -1.0);
		assert_eq!(cue_at(1.0, 0.5).pan, 1.0);
		assert_eq!(cue_at(0.5, 0.5).pan, 0.0);
	}

	#[test]
	fn out_of_range_positions_clamp_to_the_edges() {
		assert_eq!(cue_at(-3.0, 0.5), cue_at(0.0, 0.5));
		assert_eq!(cue_at(9.0, 0.5), cue_at(1.0, 0.5));
		assert_eq!(cue_at(0.5, -1.0), cue_at(0.5, 0.0));
		assert_eq!(cue_at(f32::NAN, f32::NAN), cue_at(0.0, 0.0));
	}

	#[test]
	fn bigger_shapes_sound_longer() {
		let small = Shape::new(ShapeKind::Rectangle, 0, 0, 10, 10, Srgb::BLACK);
		let big = Shape::new(ShapeKind::Rectangle, 0, 0, 400, 300, Srgb::BLACK);
		let full = Shape::new(ShapeKind::Rectangle, 0, 0, 800, 600, Srgb::BLACK);
		assert_eq!(cue_for_shape(&small, 800, 600).duration_ms, MIN_DURATION_MS + 3);
		assert!(cue_for_shape(&big, 800, 600).duration_ms > cue_for_shape(&small, 800, 600).duration_ms);
		assert_eq!(cue_for_shape(&full, 800, 600).duration_ms, MAX_DURATION_MS);
		// Centre is centre, whatever the size: middle pitch, middle pan.
		let c = cue_for_shape(&full, 800, 600);
		assert_eq!(c.pan, 0.0);
	}

	#[test]
	fn wav_is_a_well_formed_stereo_tone_of_the_cues_length() {
		let wav = tone_wav(&Cue { freq_hz: 440.0, pan: 0.0, duration_ms: 200 });
		assert_eq!(&wav[0..4], b"RIFF");
		assert_eq!(&wav[8..12], b"WAVE");
		let frames = (SAMPLE_RATE as usize * 200) / 1000;
		assert_eq!(wav.len(), 44 + frames * 4);
		// Both channels carry signal at centre pan, and the tone starts
		// and ends near silence (the fades).
		let sample = |i: usize, ch: usize| -> i16 {
			i16::from_le_bytes([wav[44 + i * 4 + ch * 2], wav[45 + i * 4 + ch * 2]])
		};
		assert_eq!(sample(0, 0), 0);
		let peak = (0..frames).map(|i| sample(i, 0).abs()).max().unwrap();
		assert!(peak > 5000, "peak {peak}");
		assert!((0..frames).all(|i| sample(i, 0) == sample(i, 1)));
	}

	#[test]
	fn pan_moves_the_energy_between_the_ears() {
		let energy = |pan: f32, ch: usize| -> i64 {
			let wav = tone_wav(&Cue { freq_hz: 440.0, pan, duration_ms: 120 });
			let frames = (wav.len() - 44) / 4;
			(0..frames)
				.map(|i| {
					let v = i16::from_le_bytes([wav[44 + i * 4 + ch * 2], wav[45 + i * 4 + ch * 2]]);
					(v as i64).pow(2)
				})
				.sum()
		};
		assert!(energy(-1.0, 0) > energy(-1.0, 1) * 100);
		assert!(energy(1.0, 1) > energy(1.0, 0) * 100);
	}
}
