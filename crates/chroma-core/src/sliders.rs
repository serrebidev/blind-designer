//! sliders: the registry of descriptive sliders.
//!
//! A slider is a named, bounded value with a spoken name and a
//! word formatter: "Brush width, 12 pixels", "Frame duration, 600
//! milliseconds". The registry is the single source of each value's
//! range — dialogs that ask for the same value (a brush width, a
//! frame duration, an extract count) take their ranges from here,
//! so a number can never be valid in one place and invalid in
//! another.
//!
//! Each definition names a `SliderTarget`: what the value edits.
//! The core stays pure — definitions, clamping, formatting, the
//! "what changed" announcement, and the color-adjustment math.
//! The app owns the state and dispatches on the target.
//!
//! Which sliders a project shows is the project's choice: the
//! pinned set is stored in the project file, and a new project
//! pins NONE (Elshara: "none, start from scratch") — sliders are
//! added through Project Options, never imposed.

use patois::t;

use crate::animation::{MAX_FRAME_MS, MIN_FRAME_MS};
use crate::Srgb;

/// What a slider's value edits. The app dispatches on this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliderTarget {
	/// The current brush's width in pixels.
	BrushWidth,
	/// The current color's red channel (the brush color).
	ColorRed,
	/// The current color's green channel.
	ColorGreen,
	/// The current color's blue channel.
	ColorBlue,
	/// The color-channels group: one slider per channel of the
	/// current color format (see `crate::formats::channels`). A
	/// group, not a single slider — this def exists so the group
	/// is pin-able in Project Options like every other slider;
	/// the app renders the group where its row would be.
	ColorChannels,
	/// How opaque new elements are when they are added, percent.
	CurrentAlpha,
	/// The selected element's opacity, percent.
	ElementOpacity,
	/// The selected element's size, percent of the size it had
	/// when it was selected.
	ElementScale,
	/// The selected rectangle's corner roundedness, percent:
	/// 0 sharp, 50 fully rounded.
	ElementRoundedness,
	/// The selected stroke's brush width in pixels.
	StrokeWidth,
	/// The last animation frame's duration in milliseconds.
	FrameDuration,
	/// How many colors Extract Colors pulls from an image.
	ExtractCount,
	/// Columns of the tactile grid the next image is described as.
	TactileColumns,
	/// Rows of the tactile grid.
	TactileRows,
	/// Sound cue volume, percent.
	SoundVolume,
	/// Color adjustment: darkening (lightness toward black).
	AdjustShade,
	/// Color adjustment: lightening (lightness toward white).
	AdjustTint,
	/// Color adjustment: softening (chroma toward gray).
	AdjustTone,
	/// Color adjustment: vividness (chroma up).
	AdjustChroma,
	/// Color adjustment: hue shift, in either direction.
	AdjustUndertone,
	/// Descriptor adjustment: warm versus cool (hue toward the
	/// warm or cool pole).
	AdjustWarmth,
	/// Descriptor adjustment: vibrant versus dull (chroma, both
	/// directions on one axis).
	AdjustVibrancy,
	/// Descriptor adjustment: brightness (lightness, both
	/// directions on one axis).
	AdjustBrightness,
	/// Descriptor adjustment: clarity (lightness contrast around
	/// the middle).
	AdjustClarity,
	/// Descriptor adjustment: fogginess (colors drawn toward a
	/// light haze; negative clears the air).
	AdjustFog,
	/// Descriptor adjustment: hard versus soft (chroma and
	/// lightness contrast together).
	AdjustHardness,
}

/// One slider: its stable id (what project files store), its
/// spoken name, its full range from the smallest to the largest
/// possible value, its fine step (what the arrow keys move — the
/// smallest step there is), and what it edits. The coarse
/// Page Up/Down jump is a percent of the range, set per project
/// (see `page_jump`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SliderDef {
	pub id: &'static str,
	pub name: &'static str,
	pub min: i32,
	pub max: i32,
	pub step: i32,
	pub target: SliderTarget,
}

impl SliderDef {
	/// Bring a value inside this slider's range.
	pub fn clamp(&self, value: i32) -> i32 {
		value.clamp(self.min, self.max)
	}

	/// The name as the app shows and speaks it, translated.
	/// `name` itself stays the English source string: it is the
	/// translation key, and tests and lookups use it as-is.
	pub fn display_name(&self) -> String {
		t(self.name)
	}

	/// Is this one of the color-adjustment sliders (the five
	/// parametric ones plus the six descriptor ones)?
	pub fn is_adjustment(&self) -> bool {
		matches!(
			self.target,
			SliderTarget::AdjustShade
				| SliderTarget::AdjustTint
				| SliderTarget::AdjustTone
				| SliderTarget::AdjustChroma
				| SliderTarget::AdjustUndertone
				| SliderTarget::AdjustWarmth
				| SliderTarget::AdjustVibrancy
				| SliderTarget::AdjustBrightness
				| SliderTarget::AdjustClarity
				| SliderTarget::AdjustFog
				| SliderTarget::AdjustHardness
		)
	}

	/// The value in words, as the readout beside the slider shows
	/// it and the announcement speaks it.
	pub fn format(&self, value: i32) -> String {
		let value = self.clamp(value);
		match self.target {
			SliderTarget::BrushWidth | SliderTarget::StrokeWidth => {
				if value == 1 {
					t("1 pixel")
				} else {
					t("%s pixels").replacen("%s", &(value).to_string(), 1)
				}
			}
			SliderTarget::ColorRed | SliderTarget::ColorGreen | SliderTarget::ColorBlue => {
				t("%s of 255").replacen("%s", &(value).to_string(), 1)
			}
			SliderTarget::ColorChannels => t("one slider per channel of the current color format"),
			SliderTarget::CurrentAlpha | SliderTarget::ElementOpacity => opacity_words(value),
			SliderTarget::ElementScale => t("%s percent").replacen("%s", &(value).to_string(), 1),
			SliderTarget::ElementRoundedness => match value {
				0 => t("sharp corners"),
				50 => t("fully rounded"),
				v => t("corners %s percent").replacen("%s", &(v).to_string(), 1),
			},
			SliderTarget::FrameDuration => {
				t("%s milliseconds").replacen("%s", &(value).to_string(), 1)
			}
			SliderTarget::ExtractCount => {
				if value == 1 {
					t("1 color")
				} else {
					t("%s colors").replacen("%s", &(value).to_string(), 1)
				}
			}
			SliderTarget::TactileColumns => t("%s columns").replacen("%s", &(value).to_string(), 1),
			SliderTarget::TactileRows => t("%s rows").replacen("%s", &(value).to_string(), 1),
			SliderTarget::SoundVolume => match value {
				0 => t("silent"),
				100 => t("full volume"),
				v => t("%s percent").replacen("%s", &(v).to_string(), 1),
			},
			SliderTarget::AdjustShade => {
				if value == 0 {
					t("no darkening")
				} else {
					t("%s percent darker").replacen("%s", &(value).to_string(), 1)
				}
			}
			SliderTarget::AdjustTint => {
				if value == 0 {
					t("no lightening")
				} else {
					t("%s percent lighter").replacen("%s", &(value).to_string(), 1)
				}
			}
			SliderTarget::AdjustTone => {
				if value == 0 {
					t("no softening")
				} else {
					t("%s percent softer").replacen("%s", &(value).to_string(), 1)
				}
			}
			SliderTarget::AdjustChroma => {
				if value == 0 {
					t("no change")
				} else {
					t("%s percent more vivid").replacen("%s", &(value).to_string(), 1)
				}
			}
			SliderTarget::AdjustUndertone => {
				if value == 0 {
					t("no shift")
				} else {
					t("%s degrees").replacen("%s", &(undertone_degrees(value)).to_string(), 1)
				}
			}
			SliderTarget::AdjustWarmth => match value {
				0 => t("neutral"),
				v if v > 0 => t("%s percent warmer").replacen("%s", &(v).to_string(), 1),
				v => t("%s percent cooler").replacen("%s", &(-v).to_string(), 1),
			},
			SliderTarget::AdjustVibrancy => match value {
				0 => t("no change"),
				v if v > 0 => t("%s percent more vibrant").replacen("%s", &(v).to_string(), 1),
				v => t("%s percent duller").replacen("%s", &(-v).to_string(), 1),
			},
			SliderTarget::AdjustBrightness => match value {
				0 => t("no change"),
				v if v > 0 => t("%s percent brighter").replacen("%s", &(v).to_string(), 1),
				v => t("%s percent darker").replacen("%s", &(-v).to_string(), 1),
			},
			SliderTarget::AdjustClarity => match value {
				0 => t("no change"),
				v if v > 0 => t("%s percent clearer").replacen("%s", &(v).to_string(), 1),
				v => t("%s percent flatter").replacen("%s", &(-v).to_string(), 1),
			},
			SliderTarget::AdjustFog => match value {
				0 => t("no fog"),
				v if v > 0 => t("%s percent foggier").replacen("%s", &(v).to_string(), 1),
				v => t("%s percent less fog").replacen("%s", &(-v).to_string(), 1),
			},
			SliderTarget::AdjustHardness => match value {
				0 => t("no change"),
				v if v > 0 => t("%s percent harder").replacen("%s", &(v).to_string(), 1),
				v => t("%s percent softer").replacen("%s", &(-v).to_string(), 1),
			},
		}
	}
}

/// Opacity in words: the ends get their plain names, the middle
/// is a percent opaque.
fn opacity_words(percent: i32) -> String {
	match percent {
		100 => t("fully opaque"),
		0 => t("fully transparent"),
		v => t("%s percent opaque").replacen("%s", &(v).to_string(), 1),
	}
}

/// The undertone slider's value (-100..=100) as a hue rotation in
/// degrees: the full sweep of the slider is 30 degrees each way.
pub fn undertone_degrees(value: i32) -> i32 {
	(value.clamp(-100, 100) as f32 * 0.3).round() as i32
}

const fn def(
	id: &'static str,
	name: &'static str,
	min: i32,
	max: i32,
	step: i32,
	target: SliderTarget,
) -> SliderDef {
	SliderDef {
		id,
		name,
		min,
		max,
		step,
		target,
	}
}

/// Every slider the app knows, in the order Project Options lists
/// them: the current brush and color first, then the selected
/// element, then the project-wide values, then the color
/// adjustments — parametric first, descriptors after.
pub const SLIDERS: &[SliderDef] = &[
	def(
		"brush-width",
		"Brush width",
		1,
		200,
		1,
		SliderTarget::BrushWidth,
	),
	def("color-red", "Color red", 0, 255, 1, SliderTarget::ColorRed),
	def(
		"color-green",
		"Color green",
		0,
		255,
		1,
		SliderTarget::ColorGreen,
	),
	def(
		"color-blue",
		"Color blue",
		0,
		255,
		1,
		SliderTarget::ColorBlue,
	),
	def(
		"color-channels",
		"Color channels (always on the Sliders tab)",
		0,
		100,
		1,
		SliderTarget::ColorChannels,
	),
	def(
		"transparency",
		"Transparency",
		0,
		100,
		1,
		SliderTarget::CurrentAlpha,
	),
	def(
		"element-opacity",
		"Element transparency",
		0,
		100,
		1,
		SliderTarget::ElementOpacity,
	),
	def(
		"element-scale",
		"Element scale",
		10,
		400,
		1,
		SliderTarget::ElementScale,
	),
	def(
		"element-roundedness",
		"Roundedness",
		0,
		50,
		1,
		SliderTarget::ElementRoundedness,
	),
	def(
		"stroke-width",
		"Stroke width",
		1,
		200,
		1,
		SliderTarget::StrokeWidth,
	),
	def(
		"frame-duration",
		"Frame duration",
		MIN_FRAME_MS as i32,
		MAX_FRAME_MS as i32,
		10,
		SliderTarget::FrameDuration,
	),
	def(
		"extract-count",
		"Extract colors count",
		1,
		64,
		1,
		SliderTarget::ExtractCount,
	),
	def(
		"tactile-columns",
		"Tactile grid columns",
		1,
		80,
		1,
		SliderTarget::TactileColumns,
	),
	def(
		"tactile-rows",
		"Tactile grid rows",
		1,
		60,
		1,
		SliderTarget::TactileRows,
	),
	def(
		"sound-volume",
		"Sound cue volume",
		0,
		100,
		1,
		SliderTarget::SoundVolume,
	),
	def(
		"adjust-shade",
		"Shade",
		0,
		100,
		1,
		SliderTarget::AdjustShade,
	),
	def("adjust-tint", "Tint", 0, 100, 1, SliderTarget::AdjustTint),
	def("adjust-tone", "Tone", 0, 100, 1, SliderTarget::AdjustTone),
	def(
		"adjust-chroma",
		"Chroma",
		0,
		100,
		1,
		SliderTarget::AdjustChroma,
	),
	def(
		"adjust-undertone",
		"Undertone",
		-100,
		100,
		1,
		SliderTarget::AdjustUndertone,
	),
	def(
		"adjust-warmth",
		"Temperature",
		-100,
		100,
		1,
		SliderTarget::AdjustWarmth,
	),
	def(
		"adjust-vibrancy",
		"Vibrancy",
		-100,
		100,
		1,
		SliderTarget::AdjustVibrancy,
	),
	def(
		"adjust-brightness",
		"Brightness",
		-100,
		100,
		1,
		SliderTarget::AdjustBrightness,
	),
	def(
		"adjust-clarity",
		"Clarity",
		-100,
		100,
		1,
		SliderTarget::AdjustClarity,
	),
	def(
		"adjust-fog",
		"Fogginess",
		-100,
		100,
		1,
		SliderTarget::AdjustFog,
	),
	def(
		"adjust-hardness",
		"Hardness",
		-100,
		100,
		1,
		SliderTarget::AdjustHardness,
	),
];

/// Extraction anchor: xgettext only collects literal `t("...")`
/// calls, and the SLIDERS table above is data, so every table name
/// is listed here once as a literal. A test keeps this list in
/// step with the table.
#[doc(hidden)]
pub fn slider_name_anchors() -> Vec<String> {
	vec![
		t("Brush width"),
		t("Color red"),
		t("Color green"),
		t("Color blue"),
		t("Color channels (always on the Sliders tab)"),
		t("Transparency"),
		t("Element transparency"),
		t("Element scale"),
		t("Roundedness"),
		t("Stroke width"),
		t("Frame duration"),
		t("Extract colors count"),
		t("Tactile grid columns"),
		t("Tactile grid rows"),
		t("Sound cue volume"),
		t("Shade"),
		t("Tint"),
		t("Tone"),
		t("Chroma"),
		t("Undertone"),
		t("Temperature"),
		t("Vibrancy"),
		t("Brightness"),
		t("Clarity"),
		t("Fogginess"),
		t("Hardness"),
	]
}

/// The Page Up/Down jump for a slider: `percent` of its range
/// (the project's page-jump setting, 10 percent by default),
/// never less than one fine step — a percent of a short range
/// still moves.
pub fn page_jump(def: &SliderDef, percent: u8) -> i32 {
	let span = (def.max - def.min).max(0);
	((span * percent.min(100) as i32) / 100).max(def.step)
}

/// The definition with this id, if the registry has it.
pub fn slider_by_id(id: &str) -> Option<&'static SliderDef> {
	SLIDERS.iter().find(|d| d.id == id)
}

/// The sliders a brand-new project pins: none. Elshara, asked
/// which sliders a project should start with: "none, start from
/// scratch" — the Sliders tab starts empty and the designer pins
/// the ones they want through Project Options.
pub fn default_pins() -> Vec<&'static str> {
	Vec::new()
}

/// The live announcement for a slider change: what changed, in
/// words, and that everything else stayed the same.
pub fn change_announcement(def: &SliderDef, value: i32) -> String {
	t("%s %s. Everything else unchanged.")
		.replacen("%s", &(def.display_name()).to_string(), 1)
		.replacen("%s", &(def.format(value)).to_string(), 1)
}

/// An element's size at `percent` of its base size, never smaller
/// than one pixel on either side.
pub fn scaled_size(w: u32, h: u32, percent: i32) -> (u32, u32) {
	let p = percent.clamp(10, 400) as u64;
	let scale = |v: u32| ((v as u64 * p) / 100).max(1) as u32;
	(scale(w), scale(h))
}

/// What the color-adjustment sliders act on: the current (base)
/// color, the selected element, every element on the current
/// layer, or every element on the layers of one depth (all the
/// background layers, say — the depth-wide adjustments a scene's
/// planes ask for). Chosen in Project Options and on the Sliders
/// tab, and stored in the project file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AdjustTarget {
	#[default]
	BaseColor,
	Element,
	Layer,
	DepthBackground,
	DepthMidground,
	DepthForeground,
}

impl AdjustTarget {
	/// The stable id project files store.
	pub fn id(&self) -> &'static str {
		match self {
			AdjustTarget::BaseColor => "base",
			AdjustTarget::Element => "element",
			AdjustTarget::Layer => "layer",
			AdjustTarget::DepthBackground => "depth-background",
			AdjustTarget::DepthMidground => "depth-midground",
			AdjustTarget::DepthForeground => "depth-foreground",
		}
	}

	pub fn from_id(id: &str) -> Option<AdjustTarget> {
		match id {
			"base" => Some(AdjustTarget::BaseColor),
			"element" => Some(AdjustTarget::Element),
			"layer" => Some(AdjustTarget::Layer),
			"depth-background" => Some(AdjustTarget::DepthBackground),
			"depth-midground" => Some(AdjustTarget::DepthMidground),
			"depth-foreground" => Some(AdjustTarget::DepthForeground),
			_ => None,
		}
	}

	/// The spoken name the target chooser shows.
	pub fn name(&self) -> String {
		match self {
			AdjustTarget::BaseColor => t("Current color"),
			AdjustTarget::Element => t("Selected element"),
			AdjustTarget::Layer => t("Current layer"),
			AdjustTarget::DepthBackground => t("Background layers"),
			AdjustTarget::DepthMidground => t("Midground layers"),
			AdjustTarget::DepthForeground => t("Foreground layers"),
		}
	}

	/// The depth role this target adjusts, for the depth-wide
	/// targets; `None` for the original three.
	pub fn depth_role(&self) -> Option<crate::drawing::DepthRole> {
		match self {
			AdjustTarget::DepthBackground => Some(crate::drawing::DepthRole::Background),
			AdjustTarget::DepthMidground => Some(crate::drawing::DepthRole::Midground),
			AdjustTarget::DepthForeground => Some(crate::drawing::DepthRole::Foreground),
			_ => None,
		}
	}

	/// All six targets, in chooser order.
	pub const ALL: [AdjustTarget; 6] = [
		AdjustTarget::BaseColor,
		AdjustTarget::Element,
		AdjustTarget::Layer,
		AdjustTarget::DepthBackground,
		AdjustTarget::DepthMidground,
		AdjustTarget::DepthForeground,
	];
}

/// The color adjustments at once, as the sliders hold them: the
/// five parametric ones — shade and tint as percents of the way
/// toward black and white, tone and chroma as percents of chroma
/// scaled down and up, undertone as a signed hue rotation — and
/// the six descriptor ones Elshara named (temperature, vibrancy,
/// brightness, clarity, fogginess, hardness), each a signed
/// slider value where negative is the opposite pole of the word.
///
/// The parametric math is the palette's own variation math (see
/// `Oklch::shades`, `tints`, `tones`): a shade of 20 does to the
/// lightness what the second rung of a shades ladder does. An
/// adjustment is always computed from the color the target had
/// when the adjustment began, never from the previous slider
/// position's result, so dragging back to zero restores the
/// original color exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ColorAdjust {
	pub shade: u8,
	pub tint: u8,
	pub tone: u8,
	pub chroma: u8,
	pub undertone: i32,
	pub warmth: i32,
	pub vibrancy: i32,
	pub brightness: i32,
	pub clarity: i32,
	pub fog: i32,
	pub hardness: i32,
}

impl ColorAdjust {
	pub fn is_zero(&self) -> bool {
		*self == ColorAdjust::default()
	}

	/// Set one adjustment from a slider value, by target.
	pub fn set(&mut self, target: SliderTarget, value: i32) {
		let signed = value.clamp(-100, 100);
		match target {
			SliderTarget::AdjustShade => self.shade = value.clamp(0, 100) as u8,
			SliderTarget::AdjustTint => self.tint = value.clamp(0, 100) as u8,
			SliderTarget::AdjustTone => self.tone = value.clamp(0, 100) as u8,
			SliderTarget::AdjustChroma => self.chroma = value.clamp(0, 100) as u8,
			SliderTarget::AdjustUndertone => self.undertone = signed,
			SliderTarget::AdjustWarmth => self.warmth = signed,
			SliderTarget::AdjustVibrancy => self.vibrancy = signed,
			SliderTarget::AdjustBrightness => self.brightness = signed,
			SliderTarget::AdjustClarity => self.clarity = signed,
			SliderTarget::AdjustFog => self.fog = signed,
			SliderTarget::AdjustHardness => self.hardness = signed,
			_ => {}
		}
	}

	/// Read one adjustment back as a slider value, by target.
	pub fn get(&self, target: SliderTarget) -> i32 {
		match target {
			SliderTarget::AdjustShade => self.shade as i32,
			SliderTarget::AdjustTint => self.tint as i32,
			SliderTarget::AdjustTone => self.tone as i32,
			SliderTarget::AdjustChroma => self.chroma as i32,
			SliderTarget::AdjustUndertone => self.undertone,
			SliderTarget::AdjustWarmth => self.warmth,
			SliderTarget::AdjustVibrancy => self.vibrancy,
			SliderTarget::AdjustBrightness => self.brightness,
			SliderTarget::AdjustClarity => self.clarity,
			SliderTarget::AdjustFog => self.fog,
			SliderTarget::AdjustHardness => self.hardness,
			_ => 0,
		}
	}

	/// Apply all the adjustments to a color, parametric first,
	/// descriptors after, in a fixed order.
	pub fn apply(&self, color: Srgb) -> Srgb {
		if self.is_zero() {
			return color;
		}
		let mut o = color.to_oklch();
		if self.shade > 0 {
			// The shades ladder's formula: lightness scaled
			// toward black by the fraction.
			o = o.with_lightness(o.l * (1.0 - self.shade as f32 / 100.0));
		}
		if self.tint > 0 {
			// The tints ladder's formula: lightness stepped the
			// fraction of the remaining way toward white.
			o = o.with_lightness(o.l + (1.0 - o.l) * self.tint as f32 / 100.0);
		}
		if self.tone > 0 {
			// The tones ladder's formula: chroma scaled down.
			o = o.scale_chroma(1.0 - self.tone as f32 / 100.0);
		}
		if self.chroma > 0 {
			o = o.scale_chroma(1.0 + self.chroma as f32 / 100.0);
		}
		if self.undertone != 0 {
			o = o.rotate(undertone_degrees(self.undertone) as f32);
		}
		// Brightness: both lightness ladders on one axis — up is
		// the tints step, down is the shades scale.
		if self.brightness > 0 {
			let f = self.brightness as f32 / 100.0;
			o = o.with_lightness(o.l + (1.0 - o.l) * f);
		} else if self.brightness < 0 {
			let f = -self.brightness as f32 / 100.0;
			o = o.with_lightness(o.l * (1.0 - f));
		}
		// Clarity: lightness contrast around the middle —
		// positive pushes light lighter and dark darker,
		// negative draws everything toward the middle.
		if self.clarity != 0 {
			let f = self.clarity as f32 / 100.0;
			o = if f > 0.0 {
				o.with_lightness(0.5 + (o.l - 0.5) * (1.0 + f))
			} else {
				o.with_lightness(o.l + (0.5 - o.l) * -f)
			};
		}
		// Hardness: chroma and a gentler lightness contrast
		// together — hard is vivid and contrasty, soft is muted
		// and flat.
		if self.hardness != 0 {
			let f = self.hardness as f32 / 100.0;
			o = o.scale_chroma(if f > 0.0 { 1.0 + 0.8 * f } else { 1.0 + f });
			o = if f > 0.0 {
				o.with_lightness(0.5 + (o.l - 0.5) * (1.0 + 0.5 * f))
			} else {
				o.with_lightness(o.l + (0.5 - o.l) * -0.5 * f)
			};
		}
		// Vibrancy: chroma on one axis, up vivid, down dull.
		if self.vibrancy != 0 {
			o = o.scale_chroma(1.0 + self.vibrancy as f32 / 100.0);
		}
		// Temperature: the hue leans toward the warm pole
		// (orange) or the cool pole (blue), along the shortest
		// arc, up to sixty percent of the way at the ends.
		if self.warmth != 0 {
			let (pole, f) = if self.warmth > 0 {
				(57.0, self.warmth as f32 / 100.0 * 0.6)
			} else {
				(264.0, -self.warmth as f32 / 100.0 * 0.6)
			};
			let mut d = (pole - o.h).rem_euclid(360.0);
			if d > 180.0 {
				d -= 360.0;
			}
			o = o.rotate(d * f);
		}
		// Fogginess: colors drawn toward a light haze — the
		// lightness pulled to the haze level, the chroma
		// drained. Negative clears the air: chroma back up,
		// lightness pushed away from the haze.
		if self.fog != 0 {
			let f = self.fog as f32 / 100.0;
			if f > 0.0 {
				o = o.with_lightness(o.l + (0.72 - o.l) * f);
				o = o.scale_chroma(1.0 - f);
			} else {
				o = o.scale_chroma(1.0 - 0.5 * f);
				o = o.with_lightness(o.l + (o.l - 0.72) * -0.5 * f);
			}
		}
		o.to_srgb()
	}

	/// The adjustments in words, for the log: only the ones that
	/// are actually doing something, in slider order.
	pub fn describe(&self) -> String {
		if self.is_zero() {
			return t("no adjustment");
		}
		let mut parts: Vec<String> = Vec::new();
		if self.shade > 0 {
			parts.push(t("shaded %s percent").replacen("%s", &(self.shade).to_string(), 1));
		}
		if self.tint > 0 {
			parts.push(t("tinted %s percent").replacen("%s", &(self.tint).to_string(), 1));
		}
		if self.tone > 0 {
			parts.push(t("toned down %s percent").replacen("%s", &(self.tone).to_string(), 1));
		}
		if self.chroma > 0 {
			parts.push(t("chroma raised %s percent").replacen("%s", &(self.chroma).to_string(), 1));
		}
		if self.undertone != 0 {
			parts.push(t("undertone shifted %s degrees").replacen(
				"%s",
				&(undertone_degrees(self.undertone)).to_string(),
				1,
			));
		}
		if self.warmth > 0 {
			parts.push(t("warmed %s percent").replacen("%s", &(self.warmth).to_string(), 1));
		} else if self.warmth < 0 {
			parts.push(t("cooled %s percent").replacen("%s", &(-self.warmth).to_string(), 1));
		}
		if self.vibrancy > 0 {
			parts.push(t("made %s percent more vibrant").replacen(
				"%s",
				&(self.vibrancy).to_string(),
				1,
			));
		} else if self.vibrancy < 0 {
			parts.push(t("dulled %s percent").replacen("%s", &(-self.vibrancy).to_string(), 1));
		}
		if self.brightness > 0 {
			parts.push(t("brightened %s percent").replacen(
				"%s",
				&(self.brightness).to_string(),
				1,
			));
		} else if self.brightness < 0 {
			parts.push(t("darkened %s percent").replacen("%s", &(-self.brightness).to_string(), 1));
		}
		if self.clarity > 0 {
			parts.push(t("clarity raised %s percent").replacen(
				"%s",
				&(self.clarity).to_string(),
				1,
			));
		} else if self.clarity < 0 {
			parts.push(t("contrast flattened %s percent").replacen(
				"%s",
				&(-self.clarity).to_string(),
				1,
			));
		}
		if self.fog > 0 {
			parts.push(t("fogged %s percent").replacen("%s", &(self.fog).to_string(), 1));
		} else if self.fog < 0 {
			parts.push(t("defogged %s percent").replacen("%s", &(-self.fog).to_string(), 1));
		}
		if self.hardness > 0 {
			parts.push(t("hardened %s percent").replacen("%s", &(self.hardness).to_string(), 1));
		} else if self.hardness < 0 {
			parts.push(t("softened %s percent").replacen("%s", &(-self.hardness).to_string(), 1));
		}
		parts.join(", ")
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn registry_is_sane_and_ids_are_unique() {
		for (i, d) in SLIDERS.iter().enumerate() {
			assert!(d.min < d.max, "{} has an empty range", d.id);
			assert!(d.step >= 1, "{} has no fine step", d.id);
			assert!(
				!SLIDERS[i + 1..].iter().any(|o| o.id == d.id),
				"duplicate id {}",
				d.id
			);
			assert_eq!(slider_by_id(d.id).map(|s| s.target), Some(d.target));
		}
		assert!(slider_by_id("no-such-slider").is_none());
		// The ranges the dialogs have always used come from here.
		let frame = slider_by_id("frame-duration").unwrap();
		assert_eq!(
			(frame.min, frame.max),
			(MIN_FRAME_MS as i32, MAX_FRAME_MS as i32)
		);
		assert_eq!(
			slider_by_id("extract-count").map(|d| (d.min, d.max)),
			Some((1, 64))
		);
		assert_eq!(
			slider_by_id("brush-width").map(|d| (d.min, d.max)),
			Some((1, 200))
		);
	}

	#[test]
	fn a_new_project_pins_no_sliders() {
		// Elshara: "none, start from scratch."
		assert!(default_pins().is_empty());
	}

	#[test]
	fn clamping_holds_values_in_range() {
		let brush = slider_by_id("brush-width").unwrap();
		assert_eq!(brush.clamp(0), 1);
		assert_eq!(brush.clamp(999), 200);
		assert_eq!(brush.clamp(12), 12);
		let under = slider_by_id("adjust-undertone").unwrap();
		assert_eq!(under.clamp(-400), -100);
	}

	#[test]
	fn values_format_in_words() {
		let f = |id: &str, v: i32| slider_by_id(id).unwrap().format(v);
		assert_eq!(f("brush-width", 1), "1 pixel");
		assert_eq!(f("brush-width", 12), "12 pixels");
		assert_eq!(f("color-red", 200), "200 of 255");
		assert_eq!(f("transparency", 100), "fully opaque");
		assert_eq!(f("transparency", 0), "fully transparent");
		assert_eq!(f("transparency", 75), "75 percent opaque");
		assert_eq!(f("element-scale", 150), "150 percent");
		assert_eq!(f("frame-duration", 600), "600 milliseconds");
		assert_eq!(f("extract-count", 1), "1 color");
		assert_eq!(f("extract-count", 8), "8 colors");
		assert_eq!(f("sound-volume", 0), "silent");
		assert_eq!(f("sound-volume", 100), "full volume");
		assert_eq!(f("sound-volume", 40), "40 percent");
		assert_eq!(f("adjust-shade", 0), "no darkening");
		assert_eq!(f("adjust-shade", 20), "20 percent darker");
		assert_eq!(f("adjust-undertone", 0), "no shift");
		assert_eq!(f("adjust-undertone", 100), "30 degrees");
		assert_eq!(f("adjust-undertone", -50), "-15 degrees");
	}

	#[test]
	fn the_announcement_names_the_change_and_the_rest_unchanged() {
		let brush = slider_by_id("brush-width").unwrap();
		assert_eq!(
			change_announcement(brush, 12),
			"Brush width 12 pixels. Everything else unchanged."
		);
		let shade = slider_by_id("adjust-shade").unwrap();
		assert_eq!(
			change_announcement(shade, 20),
			"Shade 20 percent darker. Everything else unchanged."
		);
	}

	#[test]
	fn scaled_size_scales_and_never_vanishes() {
		assert_eq!(scaled_size(200, 100, 100), (200, 100));
		assert_eq!(scaled_size(200, 100, 50), (100, 50));
		assert_eq!(scaled_size(200, 100, 200), (400, 200));
		assert_eq!(scaled_size(3, 3, 10), (1, 1));
	}

	fn sky() -> Srgb {
		Srgb::new(58, 124, 165)
	}

	#[test]
	fn zero_adjustment_is_the_identity() {
		assert_eq!(ColorAdjust::default().apply(sky()), sky());
		let mut adj = ColorAdjust::default();
		adj.set(SliderTarget::AdjustShade, 40);
		adj.set(SliderTarget::AdjustShade, 0);
		assert!(adj.is_zero());
		assert_eq!(adj.apply(sky()), sky());
	}

	#[test]
	fn shade_darkens_and_tint_lightens() {
		let base = sky().to_oklch();
		let shaded = ColorAdjust {
			shade: 50,
			..Default::default()
		}
		.apply(sky())
		.to_oklch();
		assert!(
			shaded.l < base.l - 0.1,
			"shade 50 should halve lightness: {} vs {}",
			shaded.l,
			base.l
		);
		let tinted = ColorAdjust {
			tint: 50,
			..Default::default()
		}
		.apply(sky())
		.to_oklch();
		assert!(
			tinted.l > base.l + 0.1,
			"tint 50 should step halfway to white"
		);
		// Full shade is lightness zero and full tint is white,
		// like the ladders (lightness zero quantizes to a hair
		// above pure black in sRGB, exactly as the ladder does).
		let black = ColorAdjust {
			shade: 100,
			..Default::default()
		}
		.apply(sky());
		assert!(black.r <= 2 && black.g <= 2 && black.b <= 2, "{black:?}");
		assert_eq!(
			ColorAdjust {
				tint: 100,
				..Default::default()
			}
			.apply(sky()),
			Srgb::WHITE
		);
	}

	#[test]
	fn tone_softens_and_chroma_vivifies() {
		let base = sky().to_oklch();
		let toned = ColorAdjust {
			tone: 50,
			..Default::default()
		}
		.apply(sky())
		.to_oklch();
		assert!(
			toned.c < base.c * 0.6,
			"tone 50 should halve chroma: {} vs {}",
			toned.c,
			base.c
		);
		assert_eq!(
			ColorAdjust {
				tone: 100,
				..Default::default()
			}
			.apply(sky())
			.to_oklch()
			.c,
			0.0,
			"full tone is gray"
		);
		let vivid = ColorAdjust {
			chroma: 50,
			..Default::default()
		}
		.apply(sky())
		.to_oklch();
		assert!(
			vivid.c > base.c * 1.3,
			"chroma 50 should raise chroma: {} vs {}",
			vivid.c,
			base.c
		);
	}

	#[test]
	fn undertone_rotates_the_hue() {
		let base = sky().to_oklch();
		let shifted = ColorAdjust {
			undertone: 100,
			..Default::default()
		}
		.apply(sky())
		.to_oklch();
		let diff = (shifted.h - base.h).rem_euclid(360.0);
		assert!(
			(diff - 30.0).abs() < 3.0,
			"undertone 100 should rotate +30 degrees, got {diff}"
		);
		let back = ColorAdjust {
			undertone: -100,
			..Default::default()
		}
		.apply(sky())
		.to_oklch();
		let diff = (back.h - base.h).rem_euclid(360.0);
		assert!(
			(diff - 330.0).abs() < 3.0,
			"undertone -100 should rotate -30 degrees, got {diff}"
		);
	}

	#[test]
	fn adjustments_describe_in_words() {
		assert_eq!(ColorAdjust::default().describe(), "no adjustment");
		let adj = ColorAdjust {
			shade: 20,
			undertone: 50,
			..Default::default()
		};
		assert_eq!(
			adj.describe(),
			"shaded 20 percent, undertone shifted 15 degrees"
		);
	}

	#[test]
	fn page_jump_is_a_percent_of_the_range() {
		let brush = slider_by_id("brush-width").unwrap(); // range 199
		assert_eq!(page_jump(brush, 10), 19);
		assert_eq!(page_jump(brush, 100), 199);
		// A percent of a short range still moves at least one step.
		let extract = slider_by_id("extract-count").unwrap(); // range 63
		assert_eq!(page_jump(extract, 1), 1);
		// Signed ranges measure the whole span.
		let warmth = slider_by_id("adjust-warmth").unwrap(); // range 200
		assert_eq!(page_jump(warmth, 10), 20);
	}

	#[test]
	fn descriptor_values_format_in_words() {
		let f = |id: &str, v: i32| slider_by_id(id).unwrap().format(v);
		assert_eq!(f("adjust-warmth", 0), "neutral");
		assert_eq!(f("adjust-warmth", 20), "20 percent warmer");
		assert_eq!(f("adjust-warmth", -20), "20 percent cooler");
		assert_eq!(f("adjust-vibrancy", 30), "30 percent more vibrant");
		assert_eq!(f("adjust-vibrancy", -30), "30 percent duller");
		assert_eq!(f("adjust-brightness", -10), "10 percent darker");
		assert_eq!(f("adjust-clarity", 10), "10 percent clearer");
		assert_eq!(f("adjust-fog", 0), "no fog");
		assert_eq!(f("adjust-fog", 45), "45 percent foggier");
		assert_eq!(f("adjust-hardness", -25), "25 percent softer");
	}

	#[test]
	fn warmth_leans_the_hue_toward_the_poles() {
		// Sky blue's hue sits near the cool pole: warming moves it
		// toward orange, cooling barely moves it (already cool).
		let base = sky().to_oklch();
		let dist = |h: f32, pole: f32| {
			let d = (h - pole).rem_euclid(360.0);
			d.min(360.0 - d)
		};
		let warmed = ColorAdjust {
			warmth: 100,
			..Default::default()
		}
		.apply(sky())
		.to_oklch();
		assert!(
			dist(warmed.h, 57.0) < dist(base.h, 57.0) - 20.0,
			"warm should close on orange"
		);
		let cooled = ColorAdjust {
			warmth: -100,
			..Default::default()
		}
		.apply(sky())
		.to_oklch();
		assert!(
			dist(cooled.h, 264.0) <= dist(base.h, 264.0) + 1.0,
			"cool stays near blue"
		);
	}

	#[test]
	fn vibrancy_and_brightness_move_both_ways() {
		let base = sky().to_oklch();
		let vivid = ColorAdjust {
			vibrancy: 50,
			..Default::default()
		}
		.apply(sky())
		.to_oklch();
		assert!(vivid.c > base.c * 1.3, "vibrancy up raises chroma");
		let dull = ColorAdjust {
			vibrancy: -50,
			..Default::default()
		}
		.apply(sky())
		.to_oklch();
		assert!(dull.c < base.c * 0.6, "vibrancy down dulls");
		let bright = ColorAdjust {
			brightness: 50,
			..Default::default()
		}
		.apply(sky())
		.to_oklch();
		assert!(bright.l > base.l + 0.1, "brightness up lightens");
		let dark = ColorAdjust {
			brightness: -50,
			..Default::default()
		}
		.apply(sky())
		.to_oklch();
		assert!(dark.l < base.l - 0.1, "brightness down darkens");
	}

	#[test]
	fn clarity_pushes_lightness_away_from_the_middle() {
		// A dark color gets darker, a light color lighter.
		let dark = Srgb::new(30, 30, 40).to_oklch();
		let pushed = ColorAdjust {
			clarity: 60,
			..Default::default()
		}
		.apply(Srgb::new(30, 30, 40))
		.to_oklch();
		assert!(pushed.l < dark.l, "clarity darkens a dark color further");
		let flat = ColorAdjust {
			clarity: -60,
			..Default::default()
		}
		.apply(Srgb::new(30, 30, 40))
		.to_oklch();
		assert!(flat.l > dark.l, "negative clarity draws toward the middle");
	}

	#[test]
	fn fog_drains_chroma_toward_the_haze() {
		let base = sky().to_oklch();
		let fogged = ColorAdjust {
			fog: 60,
			..Default::default()
		}
		.apply(sky())
		.to_oklch();
		assert!(fogged.c < base.c * 0.5, "fog drains chroma");
		assert!(
			(fogged.l - 0.72).abs() < (base.l - 0.72).abs(),
			"fog pulls lightness to the haze"
		);
		let cleared = ColorAdjust {
			fog: -60,
			..Default::default()
		}
		.apply(sky())
		.to_oklch();
		assert!(cleared.c > base.c, "negative fog restores chroma");
	}

	#[test]
	fn hardness_raises_chroma_and_contrast_softness_lowers_them() {
		let base = sky().to_oklch();
		let hard = ColorAdjust {
			hardness: 50,
			..Default::default()
		}
		.apply(sky())
		.to_oklch();
		assert!(hard.c > base.c * 1.2, "hard raises chroma");
		let soft = ColorAdjust {
			hardness: -50,
			..Default::default()
		}
		.apply(sky())
		.to_oklch();
		assert!(soft.c < base.c * 0.6, "soft lowers chroma");
	}

	#[test]
	fn roundedness_formats_its_ends_and_middle() {
		let f = |v: i32| slider_by_id("element-roundedness").unwrap().format(v);
		assert_eq!(f(0), "sharp corners");
		assert_eq!(f(30), "corners 30 percent");
		assert_eq!(f(50), "fully rounded");
		let def = slider_by_id("element-roundedness").unwrap();
		assert_eq!((def.min, def.max), (0, 50));
	}

	#[test]
	fn depth_targets_name_the_layers_of_one_depth() {
		assert_eq!(AdjustTarget::DepthBackground.name(), "Background layers");
		assert_eq!(AdjustTarget::DepthMidground.name(), "Midground layers");
		assert_eq!(AdjustTarget::DepthForeground.name(), "Foreground layers");
		assert_eq!(
			AdjustTarget::DepthBackground.depth_role(),
			Some(crate::drawing::DepthRole::Background)
		);
		assert_eq!(AdjustTarget::Layer.depth_role(), None);
		assert_eq!(
			AdjustTarget::from_id("depth-foreground"),
			Some(AdjustTarget::DepthForeground)
		);
	}

	#[test]
	fn adjust_target_ids_round_trip() {
		for t in AdjustTarget::ALL {
			assert_eq!(AdjustTarget::from_id(t.id()), Some(t));
		}
		assert_eq!(AdjustTarget::from_id("bogus"), None);
	}
}

#[cfg(test)]
mod anchor_tests {
	use super::*;

	#[test]
	fn slider_name_anchors_match_the_table() {
		let mut table: Vec<&str> = SLIDERS.iter().map(|d| d.name).collect();
		let mut anchors: Vec<String> = slider_name_anchors();
		assert_eq!(
			anchors.len(),
			table.len(),
			"anchor count must match the slider table"
		);
		table.sort_unstable();
		anchors.sort();
		for (want, got) in table.iter().zip(anchors.iter()) {
			assert_eq!(*want, got.as_str());
		}
	}
}
