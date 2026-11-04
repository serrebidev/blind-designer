// Debug builds keep a console, so the log is visible.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Blind Designer — an accessible multispectral color creator and
//! layered drawing tool.
//!
//! Colors live in OKLCH. Nothing is mixed or blended: schemes, shades,
//! tints and tones are pure parameter transforms (hue angle, lightness
//! step, chroma scale), so every result is known before it is seen.
//! Palettes extract free from any image you feed it. Designs are lists of
//! palette-colored shapes with set dimensions, exported as images, with
//! captions generated only where a shape has none of its own.

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use chroma_core::brushes::{Brush, BrushLibrary, PATTERNS};
use chroma_core::design::{css_for_colors, Shape, ShapeKind};
use chroma_core::dictionary;
use chroma_core::drawing::{DepthRole, Drawing, Event};
use chroma_core::viewability::DepthBand;
use chroma_core::palette::{starter_palette, PaletteColor, Provenance};
use chroma_core::plain::{parse_color, parse_position, parse_size};
use chroma_core::project::{Project, ProjectStats, DEFAULT_PAGE_JUMP_PERCENT};
use chroma_core::sliders::{
	change_announcement, page_jump, slider_by_id, AdjustTarget, ColorAdjust, SliderDef,
	SliderTarget, SLIDERS,
};
use chroma_core::formats::{
	blend, channel_values, channels as format_channels, color_from_channels,
	contrast_verdict, frequency_to_color, gradient, random_color, rotate_hue_ryb, subtract,
	text_readability, ChannelSet,
};
use chroma_core::inspect::{inspect_color, inspect_layer, inspect_shape};
use chroma_core::search::{search_catalog, CatalogHit};
use chroma_core::sound::{cue_at, cue_for_shape, tone_wav_at, Cue};
use chroma_core::symbols;
use chroma_core::tactile::{compare_images, tactile_grid};
use chroma_core::templates;
use chroma_core::web::{import_html, parse_css_colors};
use chroma_core::{describe_in, extract_palette, ColorFormat, Oklch, Srgb};
use quiet_list::{QuietList, Select};
use wxdragon::prelude::*;

const ID_OPEN_IMAGE: Id = 101;
const ID_EXPORT_PNG: Id = 102;
const ID_EXPORT_CAPTIONS: Id = 103;
const ID_EXPORT_CSS: Id = 104;
const ID_EXIT: Id = 105;
const ID_COMPLEMENTARY: Id = 201;
const ID_SPLIT: Id = 202;
const ID_ANALOGOUS: Id = 203;
const ID_TRIADIC: Id = 204;
const ID_TETRADIC: Id = 205;
const ID_SHADES: Id = 206;
const ID_TINTS: Id = 207;
const ID_TONES: Id = 208;
const ID_REMOVE_COLOR: Id = 209;
const ID_LOAD_STARTER: Id = 210;
const ID_CANVAS_SIZE: Id = 301;
const ID_ADD_RECT: Id = 302;
const ID_ADD_ELLIPSE: Id = 303;
const ID_SET_BACKGROUND: Id = 304;
const ID_REMOVE_SHAPE: Id = 305;
const ID_ADD_STROKE: Id = 306;
const ID_MOVE_SHAPE: Id = 307;
const ID_ROTATE_SHAPE: Id = 308;
const ID_RECOLOR_SHAPE: Id = 309;
const ID_LAYERS: Id = 310;
const ID_RESIZE_SHAPE: Id = 311;
const ID_ADD_TEXT: Id = 312;
const ID_ADD_IMAGE: Id = 313;
const ID_ELEMENTS: Id = 314;
const ID_SAVE_BRUSH: Id = 315;
const ID_BRUSHES: Id = 316;
const ID_ANIMATION: Id = 317;
const ID_EXPORT_SUMMARY: Id = 106;
const ID_EXPORT_HTML: Id = 107;
const ID_IMPORT_CSS: Id = 108;
const ID_IMPORT_HTML: Id = 109;
const ID_SAVE_DESIGN: Id = 110;
const ID_OPEN_DESIGN: Id = 111;
const ID_EXTRACT_IMAGE: Id = 112;
const ID_EXPORT_GIF: Id = 113;
const ID_EXPORT_ANIM_HTML: Id = 114;
const ID_EXPORT_ANIM_DESC: Id = 115;
const ID_TEMPLATES: Id = 116;
const ID_EXPORT_TACTILE_SVG: Id = 117;
const ID_EXPORT_TACTILE_SHEET: Id = 118;
const ID_EXPORT_BRAILLE: Id = 119;
const ID_COMPARE_IMAGES: Id = 120;
const ID_TACTILE_GRID: Id = 121;
const ID_SAVE_SCENE: Id = 122;
const ID_INSERT_SCENE: Id = 123;
const ID_PATTERN_TEMPLATES: Id = 124;
const ID_DESCRIBE_AREA: Id = 318;
const ID_ADD_SYMBOL: Id = 319;
const ID_STATS: Id = 320;
const ID_PROJECT_OPTIONS: Id = 321;
const ID_VIEWABILITY: Id = 322;
const ID_INSPECT: Id = 323;
const ID_LOCK_TOGGLE: Id = 324;
const ID_SEARCH_INSERT: Id = 325;
const ID_INSPECT_LAYER: Id = 326;
const ID_INSPECT_COLOR: Id = 327;
const ID_ABOUT: Id = 401;
const ID_DICTIONARY: Id = 402;
const ID_HELP_FILE: Id = 403;
const ID_SOUND_CUES: Id = 404;
const ID_FORMAT_HEX: Id = 501;
const ID_FORMAT_RGB: Id = 502;
const ID_FORMAT_HSL: Id = 503;
const ID_FORMAT_HSV: Id = 504;
const ID_ANNOUNCE_SLIDERS: Id = 505;
const ID_LISTS_SIDE_BY_SIDE: Id = 516;
const ID_FORMAT_CMYK: Id = 506;
const ID_FORMAT_XYZ: Id = 507;
const ID_FORMAT_XYY: Id = 508;
const ID_FORMAT_LAB: Id = 509;
const ID_FORMAT_LCH: Id = 510;
const ID_FORMAT_LUV: Id = 511;
const ID_FORMAT_HUNTER: Id = 512;
const ID_FORMAT_DECIMAL: Id = 513;
const ID_FORMAT_WEBSAFE: Id = 514;
const ID_FORMAT_BINARY: Id = 515;
const ID_ADD_COLOR: Id = 214;
const ID_COLOR_TOOLS: Id = 211;
const ID_TEXT_READABILITY: Id = 212;
const ID_SCHEME_SETTINGS: Id = 213;

/// Every View-menu color format item, in menu order: the checkable
/// list the format handler walks to tick exactly one.
const FORMAT_IDS: &[(Id, ColorFormat)] = &[
	(ID_FORMAT_HEX, ColorFormat::Hex),
	(ID_FORMAT_RGB, ColorFormat::Rgb),
	(ID_FORMAT_HSL, ColorFormat::Hsl),
	(ID_FORMAT_HSV, ColorFormat::Hsv),
	(ID_FORMAT_CMYK, ColorFormat::Cmyk),
	(ID_FORMAT_XYZ, ColorFormat::Xyz),
	(ID_FORMAT_XYY, ColorFormat::XyY),
	(ID_FORMAT_LAB, ColorFormat::Lab),
	(ID_FORMAT_LCH, ColorFormat::Lch),
	(ID_FORMAT_LUV, ColorFormat::Luv),
	(ID_FORMAT_HUNTER, ColorFormat::HunterLab),
	(ID_FORMAT_DECIMAL, ColorFormat::Decimal),
	(ID_FORMAT_WEBSAFE, ColorFormat::WebSafe),
	(ID_FORMAT_BINARY, ColorFormat::Binary),
];

/// Every custom menu ID in the app. wxWidgets window IDs: a custom ID
/// must be positive and at or below wxID_HIGHEST (5999); the only
/// negative ID a window may carry is wxID_ANY (-1). Anything else —
/// wxID_SEPARATOR (-2) included — trips the wincmn.cpp CreateBase()
/// "invalid id value" assert at startup (issue #4). The const block
/// below checks this list at compile time, so an out-of-range or
/// duplicated ID fails the build instead of asserting on a user's
/// machine.
const MENU_IDS: &[Id] = &[
	ID_OPEN_IMAGE,
	ID_EXPORT_PNG,
	ID_EXPORT_CAPTIONS,
	ID_EXPORT_CSS,
	ID_EXPORT_SUMMARY,
	ID_EXPORT_HTML,
	ID_IMPORT_CSS,
	ID_IMPORT_HTML,
	ID_SAVE_DESIGN,
	ID_OPEN_DESIGN,
	ID_EXTRACT_IMAGE,
	ID_EXPORT_GIF,
	ID_EXPORT_ANIM_HTML,
	ID_EXPORT_ANIM_DESC,
	ID_TEMPLATES,
	ID_EXPORT_TACTILE_SVG,
	ID_EXPORT_TACTILE_SHEET,
	ID_EXPORT_BRAILLE,
	ID_COMPARE_IMAGES,
	ID_TACTILE_GRID,
	ID_SAVE_SCENE,
	ID_INSERT_SCENE,
	ID_PATTERN_TEMPLATES,
	ID_EXIT,
	ID_COMPLEMENTARY,
	ID_SPLIT,
	ID_ANALOGOUS,
	ID_TRIADIC,
	ID_TETRADIC,
	ID_SHADES,
	ID_TINTS,
	ID_TONES,
	ID_REMOVE_COLOR,
	ID_LOAD_STARTER,
	ID_CANVAS_SIZE,
	ID_ADD_RECT,
	ID_ADD_ELLIPSE,
	ID_SET_BACKGROUND,
	ID_REMOVE_SHAPE,
	ID_ADD_STROKE,
	ID_MOVE_SHAPE,
	ID_ROTATE_SHAPE,
	ID_RECOLOR_SHAPE,
	ID_LAYERS,
	ID_RESIZE_SHAPE,
	ID_ADD_TEXT,
	ID_ADD_IMAGE,
	ID_ELEMENTS,
	ID_SAVE_BRUSH,
	ID_BRUSHES,
	ID_ANIMATION,
	ID_DESCRIBE_AREA,
	ID_ADD_SYMBOL,
	ID_STATS,
	ID_PROJECT_OPTIONS,
	ID_VIEWABILITY,
	ID_INSPECT,
	ID_LOCK_TOGGLE,
	ID_SEARCH_INSERT,
	ID_INSPECT_LAYER,
	ID_INSPECT_COLOR,
	ID_ANNOUNCE_SLIDERS,
	ID_LISTS_SIDE_BY_SIDE,
	ID_ABOUT,
	ID_DICTIONARY,
	ID_HELP_FILE,
	ID_SOUND_CUES,
	ID_FORMAT_HEX,
	ID_FORMAT_RGB,
	ID_FORMAT_HSL,
	ID_FORMAT_HSV,
	ID_FORMAT_CMYK,
	ID_FORMAT_XYZ,
	ID_FORMAT_XYY,
	ID_FORMAT_LAB,
	ID_FORMAT_LCH,
	ID_FORMAT_LUV,
	ID_FORMAT_HUNTER,
	ID_FORMAT_DECIMAL,
	ID_FORMAT_WEBSAFE,
	ID_FORMAT_BINARY,
	ID_COLOR_TOOLS,
	ID_TEXT_READABILITY,
	ID_SCHEME_SETTINGS,
	ID_ADD_COLOR,
];

const _: () = {
	let mut i = 0;
	while i < MENU_IDS.len() {
		let id = MENU_IDS[i];
		assert!(id > 0 && id <= ID_HIGHEST, "menu ID out of the valid wx range");
		let mut j = i + 1;
		while j < MENU_IDS.len() {
			assert!(MENU_IDS[j] != id, "duplicate menu ID");
			j += 1;
		}
		i += 1;
	}
};

/// The help file, shipped inside the app: worked examples included.
const HELP_TEXT: &str = include_str!("help.md");

struct State {
	/// Palette colors, each stamped with where it came from.
	palette: Vec<PaletteColor>,
	drawing: Drawing,
	/// Index of the layer new shapes land on.
	layer: usize,
	/// Sound cues while drawing (Help menu toggle, on by default).
	sound_cues: bool,
	/// How color values are spoken and displayed (View menu).
	color_format: ColorFormat,
	/// The analogous scheme's spread in degrees (Color, Schemes,
	/// Scheme Settings). Session-only, like the color format.
	scheme_spread: f32,
	/// True when scheme hue rotations run on the RYB paint wheel
	/// instead of the screen wheel (Scheme Settings). Session-only.
	wheel_ryb: bool,
	/// The current stroke brush: width, color and pattern new strokes
	/// are painted with. Applying a saved brush replaces it.
	brush: Brush,
	/// The saved-brush library, persisted beside the executable.
	brushes: BrushLibrary,
	/// Slider ids this project pins to its Sliders tab (chosen in
	/// Project Options). A new project pins NONE — Elshara: start
	/// from scratch, pin only what you want to keep. Saved in the
	/// design file.
	pinned_sliders: Vec<String>,
	/// What the color-adjustment sliders act on. Saved too.
	slider_target: AdjustTarget,
	/// Page Up/Down jump on the Sliders tab, percent of each
	/// slider's range. Saved too.
	page_jump_percent: u8,
	/// Whether slider changes announce themselves on the status
	/// line ("... Everything else unchanged."). View menu toggle,
	/// on by default.
	announce_sliders: bool,
	/// How opaque new shapes are painted, percent. The Transparency
	/// slider drives it: 100 percent transparent = 0 here.
	current_opacity: u8,
	/// How many colors the next extraction takes (the Extract
	/// colors count slider).
	extract_count: i32,
	/// Tactile grid size for the next grid (the tactile sliders).
	tactile_cols: i32,
	tactile_rows: i32,
	/// How loud the sound cues play, percent (the Sound cue volume
	/// slider).
	sound_volume: u8,
	/// The adjustment sliders' current amounts, and the colors
	/// they apply to: an adjustment always recomputes from the
	/// base captured when it began, so it never compounds.
	adjust: ColorAdjust,
	adjust_base: Option<AdjustBase>,
	/// The selected element's size when the scale slider's current
	/// run began: (layer, shape, width, height).
	scale_base: Option<(usize, usize, u32, u32)>,
	/// The channel sliders' current run: while a run lasts its
	/// channel values are the truth (each move rebuilds the color
	/// from the whole vector, so two channels moved in turn
	/// compose, and nothing compounds through conversions). A
	/// different target, element or format starts a fresh run
	/// from the color's own values.
	channel_run: Option<ChannelRun>,
	/// Whether the notebook shows Palette and Elements side by
	/// side on one "Lists" page (View menu toggle). Session-only,
	/// like the other View toggles: nothing else in the app
	/// persists view state either.
	lists_side_by_side: bool,
}

/// The color-channel sliders' run state: the format the values
/// are in, the one color they edit, and the values themselves,
/// in slider units, one per channel of the format.
struct ChannelRun {
	format: ColorFormat,
	target: ChannelTarget,
	values: Vec<i32>,
}

/// The one color the channel sliders edit: the brush color, or
/// one element's fill. (The layer and depth adjustment targets
/// name many colors at once; channels edit one color at a time.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChannelTarget {
	Brush,
	Element(usize, usize),
}

/// What the color-adjustment sliders are acting on, captured when
/// an adjustment run begins: the current color is the brush color
/// (what new strokes paint with); an element contributes its fill;
/// a layer contributes every element's fill, in order; a depth
/// contributes every element on every layer with that role, each
/// named by (layer, shape).
enum AdjustBase {
	Brush(Srgb),
	Element(usize, usize, Srgb),
	Layer(usize, Vec<Srgb>),
	Depth(DepthRole, Vec<(usize, usize, Srgb)>),
}

/// One row of the Sliders tab: the registry id it drives, the
/// native slider control, and the value readout after it.
struct SliderRow {
	id: &'static str,
	slider: Slider,
	readout: StaticText,
}

/// One row of the color-channels group on the Sliders tab: which
/// channel of the current color format it drives, the native
/// slider control, and the value readout after it.
struct ChannelRow {
	index: usize,
	slider: Slider,
	readout: StaticText,
}

struct Win {
	frame: Frame,
	notebook: Notebook,
	/// The palette and elements lists live behind RefCells: the
	/// Lists Side by Side toggle rebuilds their pages (a wx
	/// widget's parent is fixed at birth), and the new list
	/// replaces the old in the cell.
	palette_list: RefCell<QuietList<usize>>,
	elements_list: RefCell<QuietList<usize>>,
	layers_list: QuietList<usize>,
	/// The notebook pages the lists currently live on — one
	/// combined "Lists" page, or the separate Palette and
	/// Elements pages. Rebuilt (and the old ones destroyed) when
	/// the side-by-side toggle flips.
	list_pages: RefCell<Vec<Panel>>,
	stats_text: StaticText,
	readout: StaticText,
	hex_field: TextCtrl,
	l_spin: SpinCtrlDouble,
	c_spin: SpinCtrlDouble,
	h_spin: SpinCtrlDouble,
	/// The Sliders notebook page, rebuilt whenever the pinned set
	/// or the adjustment target changes (the page is removed from
	/// the notebook and a fresh one inserted at the same index, so
	/// rows always match the project's pins). Its rows, in pin
	/// order, and the adjustment-target picker shown when any
	/// adjustment slider is pinned.
	slider_page: RefCell<Option<Panel>>,
	slider_rows: RefCell<Vec<SliderRow>>,
	slider_target_choice: RefCell<Option<Choice>>,
	/// The color-channels group's rows on the Sliders page (empty
	/// unless the group is pinned) and the note under it, which
	/// says why the channels cannot be edited when they cannot
	/// (a multi-color target, no element selected) — or, for the
	/// binary format, why there are no channel sliders at all.
	channel_rows: RefCell<Vec<ChannelRow>>,
	channel_note: RefCell<Option<StaticText>>,
	/// True while code (not the user) is setting slider values, so
	/// the change handler stays out of refreshes.
	slider_refreshing: std::cell::Cell<bool>,
}

type Shared = Rc<RefCell<State>>;

fn palette_rows(state: &State) -> Vec<(usize, String)> {
	state
		.palette
		.iter()
		.enumerate()
		.map(|(i, pc)| (i, pc.spoken(state.color_format)))
		.collect()
}

/// The main window's element rows are `element_rows` (plain language,
/// defined with the Elements dialog): its payload is the flat index
/// over every layer's shapes, which `locate` turns back into
/// (layer, shape) for the edit actions.

/// Flat shape index -> (layer index, shape index within the layer).
fn locate(state: &State, mut flat: usize) -> Option<(usize, usize)> {
	for (li, layer) in state.drawing.layers.iter().enumerate() {
		if flat < layer.shapes.len() {
			return Some((li, flat));
		}
		flat -= layer.shapes.len();
	}
	None
}

/// Speak the newest log entry: every drawing change announces itself
/// with its exact values.
fn announce(win: &Win, state: &State) {
	if let Some(last) = state.drawing.log.last() {
		let step = last.describe(&state.drawing);
		win.frame.set_status_text(&step, 0);
	}
	// And sound it (Q7): the cue's pitch, pan and length say where the
	// changed shape is and how big it is, alongside the spoken words.
	if state.sound_cues {
		if let Some(cue) = cue_for_last(state) {
			play_cue(cue, state.sound_volume);
		}
	}
}

thread_local! {
	/// The cue currently sounding. wxSound plays asynchronously, so the
	/// object is kept alive here; replacing it stops the previous cue.
	static CURRENT_SOUND: RefCell<Option<Sound>> = const { RefCell::new(None) };
}

fn play_cue(cue: Cue, volume: u8) {
	let sound = Sound::from_data(&tone_wav_at(&cue, volume));
	if sound.is_ok() {
		sound.play(SoundFlags::Async);
		CURRENT_SOUND.with(|c| *c.borrow_mut() = Some(sound));
	}
}

/// The cue for the newest log entry: shape events cue from the shape
/// itself; anything else gets a neutral centre cue as a soft tick.
fn cue_for_last(state: &State) -> Option<Cue> {
	let d = &state.drawing;
	let shape_cue = |layer: &str, index: usize| {
		let l = d.layers.iter().find(|l| l.name == layer)?;
		let s = l.shapes.get(index)?;
		Some(cue_for_shape(s, d.width, d.height))
	};
	match d.log.last()? {
		Event::ShapeAdded { shape, .. } => Some(cue_for_shape(shape, d.width, d.height)),
		Event::ShapeShifted { layer, index, .. }
		| Event::ShapeResized { layer, index, .. }
		| Event::ShapeRotated { layer, index, .. }
		| Event::ShapeRecolored { layer, index, .. } => shape_cue(layer, *index),
		_ => Some(cue_at(0.5, 0.5)),
	}
}

fn wcag(ratio: f32) -> String {
	let verdict = |need: f32| if ratio >= need { "pass" } else { "fail" };
	format!(
		"contrast on the design background {:.1} to 1: AA normal text {}, AA large text {}, AAA normal text {}",
		ratio,
		verdict(4.5),
		verdict(3.0),
		verdict(7.0)
	)
}

fn readout_text(state: &State, color: Srgb) -> String {
	let o = color.to_oklch();
	let comp = o.complementary().to_srgb();
	format!(
		"{}. Complementary: {} ({}). {}",
		describe_in(color, state.color_format),
		comp.format(state.color_format),
		chroma_core::hue_family(comp.to_oklch().h),
		wcag(color.contrast(state.drawing.background))
	)
}

fn refresh_lists(win: &Win, state: &State) {
	win.palette_list.borrow().update_labelled(palette_rows(state), Select::Keep);
	win.elements_list.borrow().update_labelled(element_rows(state), Select::Keep);
	win.layers_list.update_labelled(layer_rows(state), Select::Keep);
	win.stats_text
		.set_label(&ProjectStats::gather(&state.drawing, state.palette.len()).one_line());
	refresh_menu_states(win, state);
	refresh_slider_values(win, state);
}

/// Grey the menu actions that cannot do anything right now, so the
/// menus themselves say what the project is missing: the color
/// schemes, shades/tints/tones, Remove Color and Set Background need
/// a palette color selected; Move/Rotate/Resize/Remove Shape need
/// an element selected; Recolor needs both; the animation
/// description export needs at least one frame. Everything else
/// works on any project, empty or not, and stays enabled.
fn refresh_menu_states(win: &Win, state: &State) {
	let Some(bar) = win.frame.get_menu_bar() else { return };
	let has_color = win.palette_list.borrow().selected().is_some();
	let has_element = win.elements_list.borrow().selected().is_some();
	for id in [
		ID_COMPLEMENTARY,
		ID_SPLIT,
		ID_ANALOGOUS,
		ID_TRIADIC,
		ID_TETRADIC,
		ID_SHADES,
		ID_TINTS,
		ID_TONES,
		ID_REMOVE_COLOR,
		ID_SET_BACKGROUND,
	] {
		bar.enable_item(id, has_color);
	}
	for id in [
		ID_MOVE_SHAPE,
		ID_ROTATE_SHAPE,
		ID_RESIZE_SHAPE,
		ID_REMOVE_SHAPE,
		ID_INSPECT,
		ID_LOCK_TOGGLE,
	] {
		bar.enable_item(id, has_element);
	}
	bar.enable_item(ID_INSPECT_LAYER, win.layers_list.selected().is_some());
	bar.enable_item(ID_INSPECT_COLOR, has_color);
	bar.enable_item(ID_RECOLOR_SHAPE, has_color && has_element);
	bar.enable_item(ID_EXPORT_ANIM_DESC, !state.drawing.frames.is_empty());
}

fn select_color(win: &Win, state: &State, color: Srgb) {
	let o = color.to_oklch();
	win.readout.set_label(&readout_text(state, color));
	win.hex_field.set_value(&color.to_hex());
	win.l_spin.set_value(o.l as f64);
	win.c_spin.set_value(o.c as f64);
	win.h_spin.set_value(o.h as f64);
	win.frame.set_status_text(
		&format!("Selected: {}", describe_in(color, state.color_format)),
		0,
	);
}

fn selected_color(win: &Win, state: &State) -> Option<Srgb> {
	win.palette_list.borrow()
		.selected()
		.and_then(|i| state.palette.get(i))
		.map(|pc| pc.color)
}

/// End the current adjustment / scale run: the next slider move
/// captures a fresh base. Called whenever what the sliders would
/// act on changes identity — a different element or layer is
/// selected, the adjustment target switches, or a new design
/// replaces this one.
fn reset_adjustment(state: &mut State) {
	state.adjust = ColorAdjust::default();
	state.adjust_base = None;
	state.scale_base = None;
	state.channel_run = None;
}

/// Add the starter colors to the palette (the ones not already
/// there) and say so. Shared by Color, Load Starter Colors and
/// Project Options' button of the same job.
fn load_starter_colors(win: &Win, state: &mut State) {
	let fresh: Vec<PaletteColor> = starter_palette()
		.into_iter()
		.filter(|pc| !state.palette.iter().any(|p| p.color == pc.color))
		.collect();
	if fresh.is_empty() {
		win.frame.set_status_text("The starter colors are already in the palette.", 0);
		return;
	}
	let names: Vec<String> = fresh.iter().map(|pc| pc.spoken(state.color_format)).collect();
	state.palette.extend(fresh);
	refresh_lists(win, state);
	win.frame.set_status_text(&format!("Loaded starter colors: {}", names.join("; ")), 0);
}

/// Read one slider's current value — or None when the slider has
/// nothing to act on right now (no element selected, a non-stroke
/// selected for Stroke width, no frames yet). The single place
/// that knows where each slider's value lives.
fn read_slider_value(win: &Win, state: &State, def: &SliderDef) -> Option<i32> {
	match def.target {
		SliderTarget::BrushWidth => Some(state.brush.width as i32),
		SliderTarget::ColorRed => Some(state.brush.color.r as i32),
		SliderTarget::ColorGreen => Some(state.brush.color.g as i32),
		SliderTarget::ColorBlue => Some(state.brush.color.b as i32),
		SliderTarget::CurrentAlpha => Some(100 - state.current_opacity as i32),
		SliderTarget::ElementOpacity => {
			selected_shape(state, win).map(|(_, _, s)| s.opacity as i32)
		}
		SliderTarget::ElementScale => selected_shape(state, win).map(|(li, si, s)| {
			match state.scale_base {
				Some((bl, bs, bw, _)) if bl == li && bs == si && bw > 0 => {
					(s.w as i32 * 100 / bw as i32).clamp(def.min, def.max)
				}
				_ => 100,
			}
		}),
		SliderTarget::StrokeWidth => match selected_shape(state, win) {
			Some((_, _, s)) if s.kind == ShapeKind::Stroke => Some(s.brush as i32),
			_ => None,
		},
		SliderTarget::ElementRoundedness => match selected_shape(state, win) {
			Some((_, _, s)) if s.kind == ShapeKind::Rectangle => Some(s.corner as i32),
			_ => None,
		},
		SliderTarget::FrameDuration => {
			state.drawing.frames.first().map(|f| f.duration_ms as i32)
		}
		SliderTarget::ExtractCount => Some(state.extract_count),
		SliderTarget::TactileColumns => Some(state.tactile_cols),
		SliderTarget::TactileRows => Some(state.tactile_rows),
		SliderTarget::SoundVolume => Some(state.sound_volume as i32),
		t if def.is_adjustment() => Some(state.adjust.get(t)),
		_ => None,
	}
}

/// The readout words for a slider with nothing to act on.
fn slider_placeholder(def: &SliderDef) -> &'static str {
	match def.target {
		SliderTarget::ElementOpacity | SliderTarget::ElementScale => "no element selected",
		SliderTarget::StrokeWidth => "no stroke selected",
		SliderTarget::ElementRoundedness => "no rectangle selected",
		SliderTarget::FrameDuration => "no frames yet",
		_ => "not available",
	}
}

/// The one color the channel sliders edit right now, and its
/// identity: the brush color for the Current color target, the
/// selected element's fill for the Selected element target.
/// Layer and depth targets name many colors at once and have no
/// single channel color — the group disables itself for them.
fn channel_target(state: &State, win: &Win) -> Option<(ChannelTarget, Srgb)> {
	match state.slider_target {
		AdjustTarget::BaseColor => Some((ChannelTarget::Brush, state.brush.color)),
		AdjustTarget::Element => selected_shape(state, win)
			.map(|(li, si, s)| (ChannelTarget::Element(li, si), s.fill)),
		_ => None,
	}
}

/// Why the channel sliders cannot be edited right now, in words
/// for the note under the group (and the status line, if a stray
/// event reaches a disabled slider). Empty when they can.
fn channel_note_text(state: &State, win: &Win) -> &'static str {
	match state.slider_target {
		AdjustTarget::BaseColor => "",
		AdjustTarget::Element => {
			if selected_shape(state, win).is_some() {
				""
			} else {
				"Select an element to edit its color channels."
			}
		}
		_ => "Channel sliders edit one color at a time: set the color adjustments target \
			to Current color or Selected element to use them.",
	}
}

/// The values the channel sliders should show: an active run on
/// this target in this format shows its own values (they are the
/// truth while it lasts); anything else shows the target color's
/// own values in the current format.
fn channel_display_values(win: &Win, state: &State) -> Option<Vec<i32>> {
	let (id, color) = channel_target(state, win)?;
	if let Some(run) = &state.channel_run {
		if run.format == state.color_format && run.target == id {
			return Some(run.values.clone());
		}
	}
	Some(channel_values(state.color_format, color))
}

/// Push the current values into the channel group's controls,
/// readouts, greying and note. Runs inside every slider refresh,
/// so the group always agrees with the project.
fn refresh_channel_values(win: &Win, state: &State) {
	if win.channel_rows.borrow().is_empty() {
		return;
	}
	let ChannelSet::Sliders(defs) = format_channels(state.color_format) else { return };
	let editable = channel_target(state, win).is_some();
	let values = channel_display_values(win, state);
	win.slider_refreshing.set(true);
	for row in win.channel_rows.borrow().iter() {
		let Some(def) = defs.get(row.index) else { continue };
		row.slider.enable(editable);
		match values.as_ref().and_then(|v| v.get(row.index)) {
			Some(v) => {
				row.slider.set_value(*v);
				row.readout.set_label(&def.format_value(*v));
			}
			None => row.readout.set_label("no color to edit"),
		}
	}
	win.slider_refreshing.set(false);
	if let Some(note) = win.channel_note.borrow().as_ref() {
		note.set_label(channel_note_text(state, win));
	}
}

/// The element the main window has selected, as (layer, shape,
/// shape) — the target of the element sliders.
fn selected_shape<'a>(state: &'a State, win: &Win) -> Option<(usize, usize, &'a Shape)> {
	let flat = win.elements_list.borrow().selected()?;
	let (li, si) = locate(state, flat)?;
	state.drawing.layers.get(li)?.shapes.get(si).map(|s| (li, si, s))
}

/// Push the current values into the Sliders tab's controls and
/// readouts. Runs inside every refresh, so the tab always agrees
/// with the project; guarded so programmatic sets never re-enter
/// the change handler.
fn refresh_slider_values(win: &Win, state: &State) {
	if win.slider_rows.borrow().is_empty() && win.channel_rows.borrow().is_empty() {
		return;
	}
	win.slider_refreshing.set(true);
	for row in win.slider_rows.borrow().iter() {
		let Some(def) = slider_by_id(row.id) else { continue };
		match read_slider_value(win, state, def) {
			Some(value) => {
				row.slider.set_value(value);
				row.readout.set_label(&def.format(value));
			}
			None => row.readout.set_label(slider_placeholder(def)),
		}
	}
	win.slider_refreshing.set(false);
	refresh_channel_values(win, state);
}

/// Build the color-channels group into the Sliders page: a
/// header, then one row per channel of the CURRENT color format
/// — a label, a NATIVE slider carrying the channel's spoken name
/// ("LCH hue"), and a value readout — editing the one color the
/// adjustments target picker names. Accessibility follows the
/// slider rows: real accessible names, no programmatic focus
/// moves (Ctrl+arrow hops are the user's own keys, as on the
/// Sliders tab). For the binary format there are no sliders:
/// the group is the core's own sentence about why — binary is a
/// readout, not an editing system. When the target names many
/// colors (a layer, a depth) or no element is selected, the
/// sliders grey out and the note under the group says why.
fn build_channel_group(
	win: &Rc<Win>,
	shared: &Shared,
	page: &Panel,
	sizer: &BoxSizer,
	rows: &mut Vec<ChannelRow>,
	note_out: &mut Option<StaticText>,
) {
	let (format, values) = {
		let state = shared.borrow();
		let values =
			channel_target(&state, win).map(|(_, color)| channel_values(state.color_format, color));
		(state.color_format, values)
	};
	let header = StaticText::builder(page).with_label("Color channels").build();
	sizer.add(&header, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	match format_channels(format) {
		ChannelSet::ReadoutOnly(reason) => {
			let note = StaticText::builder(page).with_label(reason).build();
			sizer.add(&note, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Bottom, 5);
			*note_out = Some(note);
		}
		ChannelSet::Sliders(defs) => {
			for (index, def) in defs.iter().enumerate() {
				let label_text = def.label();
				let (value, text) = match values.as_ref().and_then(|v| v.get(index)) {
					Some(v) => (*v, def.format_value(*v)),
					None => (def.min, "no color to edit".to_string()),
				};
				let label = StaticText::builder(page).with_label(&label_text).build();
				let slider = Slider::builder(page)
					.with_min_value(def.min)
					.with_max_value(def.max)
					.with_value(value)
					.build();
				slider.set_name(&label_text);
				slider.enable(values.is_some());
				let readout = StaticText::builder(page).with_label(&text).build();
				let row_sizer = BoxSizer::builder(Orientation::Horizontal).build();
				row_sizer.add(&label, 0, SizerFlag::All, 5);
				row_sizer.add(&slider, 1, SizerFlag::Expand | SizerFlag::All, 5);
				row_sizer.add(&readout, 0, SizerFlag::All, 5);
				sizer.add_sizer(&row_sizer, 0, SizerFlag::Expand, 0);
				{
					let (w, s) = (win.clone(), shared.clone());
					slider.on_slider(move |e| {
						if w.slider_refreshing.get() {
							return;
						}
						apply_channel(&w, &s, index, e.get_value());
					});
				}
				{
					let (w, s) = (win.clone(), shared.clone());
					slider.on_key_down(move |data| {
						channel_key(&w, &s, index, &data);
					});
				}
				rows.push(ChannelRow { index, slider, readout });
			}
			let note = StaticText::builder(page)
				.with_label(channel_note_text(&shared.borrow(), win))
				.build();
			sizer.add(&note, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Bottom, 5);
			*note_out = Some(note);
		}
	}
}

/// (Re)build the Sliders notebook page from the project's pinned
/// sliders. Accessibility: each row is a label, a NATIVE slider
/// (arrow keys move it by its fine step — the smallest step there
/// is; Home and End jump to the ends), and a value readout with a
/// real-words value; the slider itself also carries the spoken
/// name. Page Up/Down jump by the project's page-jump percent;
/// Ctrl+arrow moves focus from slider to slider, the semi-
/// invisible interface Elshara asked for. Focus is never moved
/// except by those Ctrl+arrow keys the user presses. A project
/// with no pins gets a page that says so and where to pin some.
/// The notebook index of the Sliders page, found by its text:
/// the page count before it changes with the Lists Side by
/// Side layout, so a fixed index would remove the wrong page.
fn sliders_page_index(win: &Win) -> Option<usize> {
	(0..win.notebook.get_page_count())
		.find(|&i| win.notebook.get_page_text(i).as_deref() == Some("Sliders"))
}

fn build_sliders_tab(win: &Rc<Win>, shared: &Shared) {
	if win.slider_page.borrow().is_some() {
		if let Some(i) = sliders_page_index(win) {
			win.notebook.remove_page(i);
		}
	}
	let (pins, target, any_adjust, channels_pinned) = {
		let state = shared.borrow();
		let any_adjust = state
			.pinned_sliders
			.iter()
			.filter_map(|id| slider_by_id(id))
			.any(|d| d.is_adjustment());
		let channels_pinned = state
			.pinned_sliders
			.iter()
			.filter_map(|id| slider_by_id(id))
			.any(|d| d.target == SliderTarget::ColorChannels);
		(
			state.pinned_sliders.clone(),
			state.slider_target,
			any_adjust,
			channels_pinned,
		)
	};
	let page = Panel::builder(&win.notebook).build();
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let mut rows: Vec<SliderRow> = Vec::new();
	let mut channel_rows: Vec<ChannelRow> = Vec::new();
	let mut channel_note: Option<StaticText> = None;
	let mut target_choice: Option<Choice> = None;
	if pins.is_empty() {
		let note = StaticText::builder(&page)
			.with_label(
				"No sliders are pinned in this project yet. Pin the ones you want to keep in Design, Project Options.",
			)
			.build();
		sizer.add(&note, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	} else {
		if any_adjust || channels_pinned {
			let label =
				StaticText::builder(&page).with_label("Color adjustments act on").build();
			let choice = Choice::builder(&page).build();
			for t in AdjustTarget::ALL {
				choice.append(t.name());
			}
			let at = AdjustTarget::ALL.iter().position(|t| *t == target).unwrap_or(0);
			choice.set_selection(at as u32);
			let row = BoxSizer::builder(Orientation::Horizontal).build();
			row.add(&label, 0, SizerFlag::All, 5);
			row.add(&choice, 0, SizerFlag::All, 5);
			sizer.add_sizer(&row, 0, SizerFlag::Expand, 0);
			{
				let (w, s, choice) = (win.clone(), shared.clone(), choice.clone());
				choice.on_selection_changed(move |_| {
					let Some(sel) = choice.get_selection() else { return };
					let mut state = s.borrow_mut();
					if let Some(t) = AdjustTarget::ALL.get(sel as usize) {
						if *t != state.slider_target {
							state.slider_target = *t;
							reset_adjustment(&mut state);
						}
					}
					drop(state);
					let state = s.borrow();
					refresh_slider_values(&w, &state);
				});
			}
			target_choice = Some(choice);
		}
		let state = shared.borrow();
		for id in &pins {
			let Some(def) = slider_by_id(id) else { continue };
			if def.target == SliderTarget::ColorChannels {
				build_channel_group(
					win,
					shared,
					&page,
					&sizer,
					&mut channel_rows,
					&mut channel_note,
				);
				continue;
			}
			let (value, text) = match read_slider_value(win, &state, def) {
				Some(v) => (v, def.format(v)),
				None => (def.min, slider_placeholder(def).to_string()),
			};
			let label = StaticText::builder(&page).with_label(def.name).build();
			let slider = Slider::builder(&page)
				.with_min_value(def.min)
				.with_max_value(def.max)
				.with_value(value)
				.build();
			slider.set_name(def.name);
			let readout = StaticText::builder(&page).with_label(&text).build();
			let row_sizer = BoxSizer::builder(Orientation::Horizontal).build();
			row_sizer.add(&label, 0, SizerFlag::All, 5);
			row_sizer.add(&slider, 1, SizerFlag::Expand | SizerFlag::All, 5);
			row_sizer.add(&readout, 0, SizerFlag::All, 5);
			sizer.add_sizer(&row_sizer, 0, SizerFlag::Expand, 0);
			let row_index = rows.len();
			{
				let (w, s) = (win.clone(), shared.clone());
				slider.on_slider(move |e| {
					if w.slider_refreshing.get() {
						return;
					}
					apply_slider(&w, &s, def.id, e.get_value());
				});
			}
			{
				let (w, s) = (win.clone(), shared.clone());
				slider.on_key_down(move |data| {
					slider_key(&w, &s, row_index, &data);
				});
			}
			rows.push(SliderRow { id: def.id, slider, readout });
		}
	}
	page.set_sizer(sizer, true);
	win.notebook.add_page(&page, "Sliders", false, None);
	{
		// The page's context menu (right-click, Applications key):
		// the pins live in Project Options, one step from here.
		let w = win.clone();
		page.on_context_menu(move |_| show_sliders_context(&w));
	}
	*win.slider_page.borrow_mut() = Some(page);
	*win.slider_rows.borrow_mut() = rows;
	*win.channel_rows.borrow_mut() = channel_rows;
	*win.channel_note.borrow_mut() = channel_note;
	*win.slider_target_choice.borrow_mut() = target_choice;
}

/// One list's home: a panel holding the list's label first —
/// so screen readers find it — then the list itself, named the
/// same. Used for the notebook pages and for the two halves of
/// the side-by-side splitter alike.
fn list_panel(parent: &dyn WxWidget, name: &str) -> (Panel, QuietList<usize>) {
	let panel = Panel::builder(parent).build();
	let label = StaticText::builder(&panel).with_label(name).build();
	let list = QuietList::new(&panel, name);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	sizer.add(&label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 5);
	panel.set_sizer(sizer, true);
	(panel, list)
}

/// Everything the Palette list does, registered on the CURRENT
/// list: the Lists Side by Side toggle rebuilds the list, and
/// the new one needs the same wiring. Selection speaks the
/// color; Enter inspects it; the context menu is the palette's
/// own actions, empty palette included.
fn wire_palette_list(win: &Rc<Win>, shared: &Shared) {
	{
		let (w, s) = (win.clone(), shared.clone());
		win.palette_list.borrow().on_selection_changed(move || {
			let state = s.borrow();
			if let Some(c) = selected_color(&w, &state) {
				select_color(&w, &state, c);
			}
			refresh_menu_states(&w, &state);
		});
	}
	{
		let (w, s) = (win.clone(), shared.clone());
		win.palette_list.borrow().on_key_down(move |data| {
			let WindowEventData::Keyboard(kb) = data else { return false };
			if kb.get_key_code() != Some(WXK_RETURN) {
				return false;
			}
			if let Some(i) = w.palette_list.borrow().selected() {
				let state = s.borrow();
				if let Some(text) = inspect_color(&state.drawing, &state.palette, i) {
					report_dialog(&w, "Inspector: color", "Color inspector", &text);
				}
			}
			true
		});
	}
	{
		let w = win.clone();
		win.palette_list.borrow().on_context_menu(move || show_palette_context(&w));
	}
}

/// Everything the Elements list does, registered on the CURRENT
/// list (see wire_palette_list). Selection resets the slider
/// runs aimed at the old element; Enter opens the inspector;
/// the context menu carries the element's actions and the Add
/// items, so an empty list can start a design from its own menu.
fn wire_elements_list(win: &Rc<Win>, shared: &Shared) {
	{
		let (w, s) = (win.clone(), shared.clone());
		win.elements_list.borrow().on_selection_changed(move || {
			// A different element is a different adjustment
			// and scale target: the next slider move starts
			// a fresh run from its colors and size.
			if let Ok(mut state) = s.try_borrow_mut() {
				reset_adjustment(&mut state);
			}
			let state = s.borrow();
			refresh_menu_states(&w, &state);
			refresh_slider_values(&w, &state);
		});
	}
	{
		let (w, s) = (win.clone(), shared.clone());
		win.elements_list.borrow().on_key_down(move |data| {
			let WindowEventData::Keyboard(kb) = data else { return false };
			if kb.get_key_code() != Some(WXK_RETURN) {
				return false;
			}
			inspect_selected(&w, &s);
			true
		});
	}
	{
		let w = win.clone();
		win.elements_list.borrow().on_context_menu(move || show_elements_context(&w));
	}
}

/// (Re)build the notebook pages the Palette and Elements lists
/// live on, in the layout the View toggle chooses: two tabs, or
/// one "Lists" page with the two lists side by side in a
/// splitter. A wx widget's parent is fixed at birth, so a flip
/// rebuilds the pages and lists and rewires them; the removed
/// pages go the way the Sliders tab's rebuilds go (removed from
/// the notebook, wrappers dropped). The Layers and Sliders
/// pages are not touched.
fn rebuild_list_pages(win: &Rc<Win>, shared: &Shared) {
	// The selections survive the rebuild by key.
	let palette_key = win.palette_list.borrow().selected();
	let elements_key = win.elements_list.borrow().selected();
	let old = win.list_pages.borrow().len();
	for _ in 0..old {
		win.notebook.remove_page(0);
	}
	let mut pages = Vec::new();
	if shared.borrow().lists_side_by_side {
		let page = Panel::builder(&win.notebook).build();
		let splitter = SplitterWindow::builder(&page).build();
		let (palette_panel, palette_list) = list_panel(&splitter, "Palette");
		let (elements_panel, elements_list) = list_panel(&splitter, "Elements");
		splitter.split_vertically(&palette_panel, &elements_panel, 380);
		let sizer = BoxSizer::builder(Orientation::Vertical).build();
		sizer.add(&splitter, 1, SizerFlag::Expand | SizerFlag::All, 5);
		page.set_sizer(sizer, true);
		win.notebook.insert_page(0, &page, "Lists", true, None);
		pages.push(page);
		*win.palette_list.borrow_mut() = palette_list;
		*win.elements_list.borrow_mut() = elements_list;
	} else {
		let (palette_page, palette_list) = list_panel(&win.notebook, "Palette");
		let (elements_page, elements_list) = list_panel(&win.notebook, "Elements");
		win.notebook.insert_page(0, &elements_page, "Elements", false, None);
		win.notebook.insert_page(0, &palette_page, "Palette", true, None);
		pages.push(palette_page);
		pages.push(elements_page);
		*win.palette_list.borrow_mut() = palette_list;
		*win.elements_list.borrow_mut() = elements_list;
	}
	*win.list_pages.borrow_mut() = pages;
	wire_palette_list(win, shared);
	wire_elements_list(win, shared);
	let state = shared.borrow();
	refresh_lists(win, &state);
	drop(state);
	win.palette_list.borrow().select(palette_key.as_ref());
	win.elements_list.borrow().select(elements_key.as_ref());
	let state = shared.borrow();
	refresh_menu_states(win, &state);
}

/// The Sliders tab's custom keys (Elshara's semi-invisible
/// interface): Ctrl+arrow hops focus from slider to slider;
/// Page Up/Down jump the value by the project's page-jump percent
/// of the range. Everything else — plain arrows (the fine step),
/// Home/End (the ends) — is the native slider's own behavior, so
/// the event is skipped onward untouched.
fn slider_key(win: &Rc<Win>, shared: &Shared, row_index: usize, data: &WindowEventData) {
	let WindowEventData::Keyboard(kb) = data else {
		data.skip(true);
		return;
	};
	let Some(code) = kb.get_key_code() else {
		data.skip(true);
		return;
	};
	let rows = win.slider_rows.borrow();
	let Some(row) = rows.get(row_index) else {
		data.skip(true);
		return;
	};
	let Some(def) = slider_by_id(row.id) else {
		data.skip(true);
		return;
	};
	if kb.control_down() && matches!(code, WXK_LEFT | WXK_UP | WXK_RIGHT | WXK_DOWN) {
		// Handled here: focus hops, the value does not change.
		let back = matches!(code, WXK_LEFT | WXK_UP);
		let next = if back {
			row_index.saturating_sub(1)
		} else {
			(row_index + 1).min(rows.len() - 1)
		};
		if let Some(dest) = rows.get(next) {
			dest.slider.set_focus();
		}
		return;
	}
	if matches!(code, WXK_PAGEUP | WXK_PAGEDOWN) {
		let percent = shared.borrow().page_jump_percent;
		let jump = page_jump(def, percent);
		let delta = if code == WXK_PAGEUP { jump } else { -jump };
		let value = def.clamp(row.slider.get_value() + delta);
		row.slider.set_value(value);
		drop(rows);
		apply_slider(win, shared, def.id, value);
		return;
	}
	data.skip(true);
}

/// Apply one slider's new value to its target, live, and say what
/// changed (the announcement honors the View toggle). Drawing
/// changes go through the drawing, so they land in the log like
/// every other edit; the status line speaks the slider's own
/// announcement — what changed, and that everything else stayed.
fn apply_slider(win: &Win, shared: &Shared, id: &'static str, value: i32) {
	let Some(def) = slider_by_id(id) else { return };
	let value = def.clamp(value);
	let mut note: Option<String> = None;
	{
		let mut state = shared.borrow_mut();
		match def.target {
			SliderTarget::BrushWidth => {
				state.brush.width = value as u32;
				// The width lives on for the next stroke; strokes
				// already drawn are elements (Stroke width slider).
			}
			SliderTarget::ColorRed
			| SliderTarget::ColorGreen
			| SliderTarget::ColorBlue => {
				// A channel edit is absolute: it becomes the color
				// any later adjustment starts from, so the current
				// adjustment run ends here.
				match def.target {
					SliderTarget::ColorRed => state.brush.color.r = value as u8,
					SliderTarget::ColorGreen => state.brush.color.g = value as u8,
					_ => state.brush.color.b = value as u8,
				}
				reset_adjustment(&mut state);
			}
			SliderTarget::CurrentAlpha => {
				state.current_opacity = (100 - value).clamp(0, 100) as u8;
			}
			SliderTarget::ElementOpacity => match selected_shape(&state, win) {
				Some((li, si, _)) => {
					if state.drawing.set_shape_opacity(li, si, value as u8) {
						refresh_lists(win, &state);
					}
				}
				None => note = Some("Select an element first.".to_string()),
			},
			SliderTarget::ElementScale => match selected_shape(&state, win) {
				Some((li, si, s)) => {
					if s.kind == ShapeKind::Stroke {
						note = Some(
							"A stroke's size is its points; it does not scale.".to_string(),
						);
					} else {
						if state.scale_base.map(|(bl, bs, _, _)| (bl, bs)) != Some((li, si)) {
							state.scale_base = Some((li, si, s.w, s.h));
						}
						let (_, _, bw, bh) = state.scale_base.unwrap();
						let (w, h) = chroma_core::sliders::scaled_size(bw, bh, value);
						state.drawing.resize_shape(li, si, w, h);
						refresh_lists(win, &state);
					}
				}
				None => note = Some("Select an element first.".to_string()),
			},
			SliderTarget::StrokeWidth => match selected_shape(&state, win) {
				Some((li, si, s)) => {
					if s.kind == ShapeKind::Stroke {
						if state.drawing.set_shape_brush_width(li, si, value as u32) {
							refresh_lists(win, &state);
						}
					} else {
						note = Some("Stroke width acts on a selected stroke.".to_string());
					}
				}
				None => note = Some("Select a stroke first.".to_string()),
			},
			SliderTarget::ElementRoundedness => match selected_shape(&state, win) {
				Some((li, si, s)) => {
					if s.kind == ShapeKind::Rectangle {
						if state.drawing.set_shape_corner(li, si, value as u8) {
							refresh_lists(win, &state);
						}
					} else {
						note = Some(
							"Roundedness acts on a selected rectangle; ellipses are already round and strokes have no corners."
								.to_string(),
						);
					}
				}
				None => note = Some("Select a rectangle first.".to_string()),
			},
			SliderTarget::FrameDuration => {
				if state.drawing.frames.is_empty() {
					note = Some(
						"No animation frames yet: add one in Design, Animation.".to_string(),
					);
				} else {
					state.drawing.set_all_frame_durations(value as u32);
					refresh_lists(win, &state);
				}
			}
			SliderTarget::ExtractCount => state.extract_count = value,
			SliderTarget::TactileColumns => state.tactile_cols = value,
			SliderTarget::TactileRows => state.tactile_rows = value,
			SliderTarget::SoundVolume => state.sound_volume = value as u8,
			_ if def.is_adjustment() => {
				note = apply_adjustment(win, &mut state, def, value);
			}
			_ => {}
		}
	}
	// The readout always follows the slider, note or not.
	if let Some(row) = win.slider_rows.borrow().iter().find(|r| r.id == id) {
		row.readout.set_label(&def.format(value));
	}
	match note {
		Some(text) => win.frame.set_status_text(&text, 0),
		None => {
			let announce_on = shared.borrow().announce_sliders;
			if announce_on {
				win.frame.set_status_text(&change_announcement(def, value), 0);
			} else {
				win.frame
					.set_status_text(&format!("{} {}.", def.name, def.format(value)), 0);
			}
		}
	}
}

/// The channel sliders' custom keys, the Sliders tab's own: Ctrl
/// plus an arrow hops focus from channel to channel; Page Up and
/// Page Down jump by the project's page-jump percent of the
/// channel's range. Plain arrows, Home and End are the native
/// slider's own behavior, skipped onward untouched.
fn channel_key(win: &Rc<Win>, shared: &Shared, index: usize, data: &WindowEventData) {
	let WindowEventData::Keyboard(kb) = data else {
		data.skip(true);
		return;
	};
	let Some(code) = kb.get_key_code() else {
		data.skip(true);
		return;
	};
	let rows = win.channel_rows.borrow();
	let Some(pos) = rows.iter().position(|r| r.index == index) else {
		data.skip(true);
		return;
	};
	if kb.control_down() && matches!(code, WXK_LEFT | WXK_UP | WXK_RIGHT | WXK_DOWN) {
		let back = matches!(code, WXK_LEFT | WXK_UP);
		let next = if back {
			pos.saturating_sub(1)
		} else {
			(pos + 1).min(rows.len() - 1)
		};
		if let Some(dest) = rows.get(next) {
			dest.slider.set_focus();
		}
		return;
	}
	if matches!(code, WXK_PAGEUP | WXK_PAGEDOWN) {
		let (format, percent) = {
			let state = shared.borrow();
			(state.color_format, state.page_jump_percent)
		};
		let ChannelSet::Sliders(defs) = format_channels(format) else {
			data.skip(true);
			return;
		};
		let Some(def) = defs.get(index) else {
			data.skip(true);
			return;
		};
		let span = (def.max - def.min).max(0);
		let jump = ((span * percent.min(100) as i32) / 100).max(def.step);
		let delta = if code == WXK_PAGEUP { jump } else { -jump };
		let Some(row) = rows.get(pos) else {
			data.skip(true);
			return;
		};
		let value = def.clamp(row.slider.get_value() + delta);
		row.slider.set_value(value);
		drop(rows);
		apply_channel(win, shared, index, value);
		return;
	}
	data.skip(true);
}

/// Apply one channel slider's new value, live: the channel is
/// set absolutely on the run's value vector and the color is
/// rebuilt from the whole vector, so channels compose and
/// nothing compounds through conversions. The edit lands like
/// the other slider edits — the brush color is replaced (the
/// red/green/blue sliders' rule, which also ends any adjustment
/// run), an element's fill goes through the drawing and its log.
/// The announcement names the channel and its new value, and
/// that everything else stayed the same.
fn apply_channel(win: &Win, shared: &Shared, index: usize, value: i32) {
	let mut note: Option<String> = None;
	let announcement: Option<String> = {
		let mut state = shared.borrow_mut();
		let format = state.color_format;
		let ChannelSet::Sliders(defs) = format_channels(format) else { return };
		let Some(def) = defs.get(index).copied() else { return };
		let value = def.clamp(value);
		let Some((target_id, base_color)) = channel_target(&state, win) else {
			// The sliders sit greyed out in this state; a stray
			// event still gets the straight answer.
			win.frame.set_status_text(channel_note_text(&state, win), 0);
			return;
		};
		let continuing = state
			.channel_run
			.as_ref()
			.is_some_and(|run| run.format == format && run.target == target_id);
		if !continuing {
			state.channel_run = Some(ChannelRun {
				format,
				target: target_id,
				values: channel_values(format, base_color),
			});
		}
		let values = {
			let run = state.channel_run.as_mut().expect("the run was just set");
			if let Some(slot) = run.values.get_mut(index) {
				*slot = value;
			}
			run.values.clone()
		};
		let mut color = color_from_channels(format, &values);
		let applied = if format == ColorFormat::WebSafe {
			// The color snapped to the web-safe grid; the run's
			// values follow it, so the sliders show the color
			// that actually landed, not the one asked for.
			let snapped = channel_values(format, color);
			let landed = snapped.get(index).copied().unwrap_or(value);
			color = color_from_channels(format, &snapped);
			if let Some(run) = state.channel_run.as_mut() {
				run.values = snapped;
			}
			landed
		} else {
			value
		};
		match target_id {
			ChannelTarget::Brush => {
				state.brush.color = color;
				state.adjust = ColorAdjust::default();
				state.adjust_base = None;
			}
			ChannelTarget::Element(li, si) => {
				if state.drawing.set_shape_color_slider(li, si, color) {
					state.adjust = ColorAdjust::default();
					state.adjust_base = None;
					refresh_lists(win, &state);
				} else {
					note = Some("That element is gone.".to_string());
				}
			}
		}
		if let Some(row) = win.channel_rows.borrow().iter().find(|r| r.index == index) {
			row.readout.set_label(&def.format_value(applied));
			if applied != value {
				row.slider.set_value(applied);
			}
		}
		let words = format!("{} {}", def.label(), def.format_value(applied));
		Some(if state.announce_sliders {
			format!("{words}. Everything else unchanged.")
		} else {
			format!("{words}.")
		})
	};
	match note {
		Some(text) => win.frame.set_status_text(&text, 0),
		None => {
			if let Some(text) = announcement {
				win.frame.set_status_text(&text, 0);
			}
		}
	}
}

/// Drive the color-adjustment sliders: set the one amount, then
/// re-apply the whole adjustment from the base captured when the
/// run began (a different target, element or layer starts a new
/// run). Returns a status note when there is nothing to act on.
fn apply_adjustment(
	win: &Win,
	state: &mut State,
	def: &SliderDef,
	value: i32,
) -> Option<String> {
	let matches_base = match (&state.adjust_base, state.slider_target) {
		(Some(AdjustBase::Brush(_)), AdjustTarget::BaseColor) => true,
		(Some(AdjustBase::Element(li, si, _)), AdjustTarget::Element) => {
			selected_shape(state, win).map(|(l, s, _)| (l, s)) == Some((*li, *si))
		}
		(Some(AdjustBase::Layer(li, bases)), AdjustTarget::Layer) => {
			*li == state.layer
				&& state
					.drawing
					.layers
					.get(*li)
					.map(|l| l.shapes.len() == bases.len())
					.unwrap_or(false)
		}
		(Some(AdjustBase::Depth(role, bases)), target) => {
			// The run continues while it still names exactly the
			// role's current shapes (the core re-validates this
			// before painting; here it only decides whether the
			// captured bases are still the run's bases).
			target.depth_role() == Some(*role)
				&& bases.len()
					== state
						.drawing
						.layers
						.iter()
						.filter(|l| l.depth == *role)
						.map(|l| l.shapes.len())
						.sum::<usize>()
				&& bases.iter().all(|(li, si, _)| {
					state
						.drawing
						.layers
						.get(*li)
						.map(|l| l.depth == *role && *si < l.shapes.len())
						.unwrap_or(false)
				})
		}
		_ => false,
	};
	if !matches_base {
		state.adjust = ColorAdjust::default();
		state.adjust_base = match state.slider_target {
			AdjustTarget::BaseColor => Some(AdjustBase::Brush(state.brush.color)),
			AdjustTarget::Element => {
				selected_shape(state, win).map(|(li, si, s)| AdjustBase::Element(li, si, s.fill))
			}
			AdjustTarget::Layer => state.drawing.layers.get(state.layer).map(|l| {
				AdjustBase::Layer(state.layer, l.shapes.iter().map(|s| s.fill).collect())
			}),
			target => target.depth_role().map(|role| {
				let bases: Vec<(usize, usize, Srgb)> = state
					.drawing
					.layers
					.iter()
					.enumerate()
					.filter(|(_, l)| l.depth == role)
					.flat_map(|(li, l)| {
						l.shapes.iter().enumerate().map(move |(si, s)| (li, si, s.fill))
					})
					.collect();
				AdjustBase::Depth(role, bases)
			}),
		};
	}
	if state.adjust_base.is_none() {
		return Some(match state.slider_target {
			AdjustTarget::Element => "Select an element to adjust first.".to_string(),
			AdjustTarget::Layer => "This project has no current layer.".to_string(),
			AdjustTarget::BaseColor => "Nothing to adjust.".to_string(),
			_ => "Nothing to adjust.".to_string(),
		});
	}
	state.adjust.set(def.target, value);
	match &state.adjust_base {
		Some(AdjustBase::Brush(base)) => {
			let base = *base;
			let adjust = state.adjust;
			state.brush.color = adjust.apply(base);
			None
		}
		Some(AdjustBase::Element(li, si, base)) => {
			let (li, si, base) = (*li, *si, *base);
			let adjust = state.adjust;
			if state.drawing.adjust_shape_color(li, si, base, adjust) {
				refresh_lists(win, state);
				None
			} else {
				Some("That element is gone.".to_string())
			}
		}
		Some(AdjustBase::Layer(li, bases)) => {
			let (li, bases) = (*li, bases.clone());
			let adjust = state.adjust;
			if bases.is_empty() {
				Some("The current layer has no elements to adjust.".to_string())
			} else {
				let outcome = state.drawing.adjust_layer_colors(li, &bases, adjust);
				if outcome.changed > 0 {
					refresh_lists(win, state);
					Some(format!(
						"{} element{} changed{}.",
						outcome.changed,
						if outcome.changed == 1 { "" } else { "s" },
						locked_suffix(outcome.locked)
					))
				} else if outcome.locked > 0 {
					Some(format!(
						"Every element the adjustment reached is locked; nothing changed{}",
						locked_suffix(outcome.locked)
					))
				} else {
					Some("That layer changed under the adjustment; move the slider to start again.".to_string())
				}
			}
		}
		Some(AdjustBase::Depth(role, bases)) => {
			let (role, bases) = (*role, bases.clone());
			let adjust = state.adjust;
			if bases.is_empty() {
				Some(format!(
					"No layer is tagged {} yet: tag one in Design, Layers.",
					role.name()
				))
			} else {
				let outcome = state.drawing.adjust_depth_colors(role, &bases, adjust);
				if outcome.changed > 0 {
					refresh_lists(win, state);
					Some(format!(
						"{} element{} changed across the {} layers{}.",
						outcome.changed,
						if outcome.changed == 1 { "" } else { "s" },
						role.name(),
						locked_suffix(outcome.locked)
					))
				} else if outcome.locked > 0 {
					Some(format!(
						"Every element the adjustment reached is locked; nothing changed{}",
						locked_suffix(outcome.locked)
					))
				} else {
					Some("Those layers changed under the adjustment; move the slider to start again.".to_string())
				}
			}
		}
		None => Some("Nothing to adjust.".to_string()),
	}
}

/// The tail of a bulk-change announcement: how many locked
/// elements the change passed over, in the counts form Elshara
/// asked for ("N changed, M locked stayed the same"). Empty
/// when nothing was locked.
fn locked_suffix(locked: usize) -> String {
	if locked == 0 {
		String::new()
	} else {
		format!(", {locked} locked stayed the same")
	}
}

/// The Project Options dialog (Design menu): this project's home
/// for what used to be scattered — the pinned-slider checklist
/// (check = the slider lives on the Sliders tab), what the color
/// adjustments act on, the Page Up/Down jump, and the starter
/// colors. OK applies the pins, target and jump at once; the
/// dialog is modal and closing it returns to the main window.
fn project_options_dialog(win: &Rc<Win>, shared: &Shared) {
	let frame = win.frame;
	let dialog = Dialog::builder(&frame, "Project Options").with_size(560, 640).build();
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let pins_label = StaticText::builder(&dialog)
		.with_label("Sliders pinned to this project's Sliders tab")
		.build();
	sizer.add(&pins_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	let checks = CheckListBox::builder(&dialog).build();
	for def in SLIDERS {
		checks.append(def.name);
	}
	{
		let state = shared.borrow();
		for (i, def) in SLIDERS.iter().enumerate() {
			if state.pinned_sliders.iter().any(|p| p == def.id) {
				checks.check(i as u32, true);
			}
		}
	}
	sizer.add(&checks, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let target_label =
		StaticText::builder(&dialog).with_label("Color adjustments act on").build();
	sizer.add(&target_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	let target_choice = Choice::builder(&dialog).build();
	for t in AdjustTarget::ALL {
		target_choice.append(t.name());
	}
	{
		let state = shared.borrow();
		let at = AdjustTarget::ALL.iter().position(|t| *t == state.slider_target).unwrap_or(0);
		target_choice.set_selection(at as u32);
	}
	sizer.add(&target_choice, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let jump_now = shared.borrow().page_jump_percent;
	let jump_label = StaticText::builder(&dialog)
		.with_label("Page Up and Page Down jump, percent of each slider's range")
		.build();
	let jump_spin = SpinCtrl::builder(&dialog).with_range(1, 100).build();
	jump_spin.set_value(jump_now as i32);
	sizer.add(&jump_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	sizer.add(&jump_spin, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let starter_btn = Button::builder(&dialog).with_label("Load Starter Colors").build();
	sizer.add(&starter_btn, 0, SizerFlag::All, 5);
	let ok = Button::builder(&dialog).with_label("OK").build();
	let cancel = Button::builder(&dialog).with_label("Cancel").build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	buttons.add(&ok, 0, SizerFlag::All, 5);
	buttons.add(&cancel, 0, SizerFlag::All, 5);
	sizer.add_sizer(&buttons, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	{
		let (w, s) = (win.clone(), shared.clone());
		starter_btn.on_click(move |_| {
			let mut state = s.borrow_mut();
			load_starter_colors(&w, &mut state);
		});
	}
	{
		let d = dialog.clone();
		ok.on_click(move |_| d.end_modal(ID_OK));
	}
	{
		let d = dialog.clone();
		cancel.on_click(move |_| d.end_modal(ID_CANCEL));
	}
	if dialog.show_modal() == ID_OK {
		let mut state = shared.borrow_mut();
		state.pinned_sliders = SLIDERS
			.iter()
			.enumerate()
			.filter(|(i, _)| checks.is_checked(*i as u32))
			.map(|(_, def)| def.id.to_string())
			.collect();
		if let Some(sel) = target_choice.get_selection() {
			if let Some(t) = AdjustTarget::ALL.get(sel as usize) {
				if *t != state.slider_target {
					state.slider_target = *t;
					reset_adjustment(&mut state);
				}
			}
		}
		state.page_jump_percent = jump_spin.value().clamp(1, 100) as u8;
		let count = state.pinned_sliders.len();
		drop(state);
		build_sliders_tab(win, shared);
		frame.set_status_text(
			&format!(
				"Project options saved: {count} slider{} pinned to the Sliders tab.",
				if count == 1 { "" } else { "s" }
			),
			0,
		);
	}
}

/// Append colors to the palette, stamped with their provenance, and
/// say what was added.
fn add_colors(win: &Win, state: &mut State, colors: &[Srgb], what: &str, provenance: &Provenance) {
	for c in colors {
		state.palette.push(PaletteColor::new(*c, provenance.clone()));
	}
	refresh_lists(win, state);
	let names: Vec<String> =
		colors.iter().map(|c| describe_in(*c, state.color_format)).collect();
	win.frame.set_status_text(
		&format!("Added {} ({}): {}", what, colors.len(), names.join("; ")),
		0,
	);
}

fn numbers_dialog(
	parent: &Frame,
	title: &str,
	fields: &[(&str, i32, i32, i32)],
) -> Option<Vec<i32>> {
	let dialog = Dialog::builder(parent, title).build();
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let mut spins: Vec<SpinCtrl> = Vec::new();
	for (label, min, max, value) in fields {
		let text = StaticText::builder(&dialog).with_label(label).build();
		let spin = SpinCtrl::builder(&dialog).with_range(*min, *max).build();
		spin.set_value(*value);
		sizer.add(&text, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
		sizer.add(&spin, 0, SizerFlag::Expand | SizerFlag::All, 5);
		spins.push(spin);
	}
	let ok = Button::builder(&dialog).with_label("OK").build();
	let cancel = Button::builder(&dialog).with_label("Cancel").build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	buttons.add(&ok, 0, SizerFlag::All, 5);
	buttons.add(&cancel, 0, SizerFlag::All, 5);
	sizer.add_sizer(&buttons, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	let d = dialog.clone();
	ok.on_click(move |_| d.end_modal(ID_OK));
	let d = dialog.clone();
	cancel.on_click(move |_| d.end_modal(ID_CANCEL));
	if dialog.show_modal() == ID_OK {
		Some(spins.iter().map(|s| s.value()).collect())
	} else {
		None
	}
}

fn file_dialog(parent: &Frame, message: &str, wildcard: &str, save: bool) -> Option<String> {
	let style = if save {
		FileDialogStyle::Save
	} else {
		FileDialogStyle::Open
	};
	let dlg = FileDialog::builder(parent)
		.with_message(message)
		.with_wildcard(wildcard)
		.with_style(style)
		.build();
	if dlg.show_modal() == ID_OK {
		dlg.get_path()
	} else {
		None
	}
}

/// One line of text, asked modally. `None` on cancel.
fn text_dialog(parent: &Frame, title: &str, label: &str, value: &str) -> Option<String> {
	let dialog = Dialog::builder(parent, title).build();
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let text = StaticText::builder(&dialog).with_label(label).build();
	let field = TextCtrl::builder(&dialog).build();
	field.set_value(value);
	sizer.add(&text, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	sizer.add(&field, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let ok = Button::builder(&dialog).with_label("OK").build();
	let cancel = Button::builder(&dialog).with_label("Cancel").build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	buttons.add(&ok, 0, SizerFlag::All, 5);
	buttons.add(&cancel, 0, SizerFlag::All, 5);
	sizer.add_sizer(&buttons, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	let d = dialog.clone();
	ok.on_click(move |_| d.end_modal(ID_OK));
	let d = dialog.clone();
	cancel.on_click(move |_| d.end_modal(ID_CANCEL));
	if dialog.show_modal() == ID_OK {
		Some(field.get_value())
	} else {
		None
	}
}

/// Several labelled one-line text fields in one modal dialog, each
/// prefilled. Returns the field values in order, `None` on cancel.
/// The labels are the fields' accessible names: each label says what
/// the field accepts, in numbers or words.
fn fields_dialog(
	parent: &Frame,
	title: &str,
	fields: &[(&str, &str)],
) -> Option<Vec<String>> {
	let dialog = Dialog::builder(parent, title).build();
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let mut inputs: Vec<TextCtrl> = Vec::new();
	for (label, value) in fields {
		let text = StaticText::builder(&dialog).with_label(label).build();
		let field = TextCtrl::builder(&dialog).build();
		field.set_value(value);
		sizer.add(&text, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
		sizer.add(&field, 0, SizerFlag::Expand | SizerFlag::All, 5);
		inputs.push(field);
	}
	let ok = Button::builder(&dialog).with_label("OK").build();
	let cancel = Button::builder(&dialog).with_label("Cancel").build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	buttons.add(&ok, 0, SizerFlag::All, 5);
	buttons.add(&cancel, 0, SizerFlag::All, 5);
	sizer.add_sizer(&buttons, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	let d = dialog.clone();
	ok.on_click(move |_| d.end_modal(ID_OK));
	let d = dialog.clone();
	cancel.on_click(move |_| d.end_modal(ID_CANCEL));
	if dialog.show_modal() == ID_OK {
		Some(inputs.iter().map(|f| f.get_value()).collect())
	} else {
		None
	}
}

/// Ask where an element goes, how big it is, and what color — each
/// field accepting numbers or plain words (core `plain` parsers):
/// position "120, 80", "top left", "center" or a nudge like "a bit
/// right" from the current position; size "200 by 100" or "a bit
/// wider" from the current size; color a name like "sky blue" or a
/// hex code. A field that does not parse is reported on the status
/// line and nothing is placed.
fn place_dialog(
	win: &Win,
	title: &str,
	current_pos: (i32, i32),
	current_size: (u32, u32),
	default_color: Srgb,
	canvas: (u32, u32),
) -> Option<(i32, i32, u32, u32, Srgb)> {
	let frame = win.frame;
	let v = fields_dialog(
		&frame,
		title,
		&[
			(
				"Position: numbers like 120, 80, or words like top left, center, a bit right",
				&format!("{}, {}", current_pos.0, current_pos.1),
			),
			(
				"Size: numbers like 200 by 100, or words like a bit wider, a bit taller",
				&format!("{} by {}", current_size.0, current_size.1),
			),
			("Color: a name like sky blue, or a hex code", &default_color.to_hex()),
		],
	)?;
	let Some((x, y)) = parse_position(&v[0], canvas.0, canvas.1, Some(current_pos)) else {
		frame.set_status_text(
			"I could not understand that position. Use numbers like 120, 80, or words like top left, center, a bit right.",
			0,
		);
		return None;
	};
	let Some((w, h)) = parse_size(&v[1], Some(current_size)) else {
		frame.set_status_text(
			"I could not understand that size. Use numbers like 200 by 100, or words like a bit wider.",
			0,
		);
		return None;
	};
	let Some(color) = parse_color(&v[2]) else {
		frame.set_status_text(
			"I could not understand that color. Use a name like sky blue, or a hex code like 87CEEB.",
			0,
		);
		return None;
	};
	Some((x, y, w, h, color))
}

/// Speak a freshly added shape with its automatic plain-language
/// description (item: auto-describe on creation), and sound its cue.
fn speak_added(win: &Win, state: &State, layer: usize, index: usize) {
	let Some(shape) =
		state.drawing.layers.get(layer).and_then(|l| l.shapes.get(index)).cloned()
	else {
		return;
	};
	win.frame.set_status_text(
		&format!("Added {}.", state.drawing.plain_of(&shape)),
		0,
	);
	if state.sound_cues {
		play_cue(
			cue_for_shape(&shape, state.drawing.width, state.drawing.height),
			state.sound_volume,
		);
	}
}

/// The brush library's file: beside the executable, so it travels
/// with the app and survives design files coming and going.
fn brush_file_path() -> std::path::PathBuf {
	match std::env::current_exe() {
		Ok(exe) => exe.with_file_name("blind-designer-brushes.txt"),
		Err(_) => std::path::PathBuf::from("blind-designer-brushes.txt"),
	}
}

fn load_brushes() -> BrushLibrary {
	std::fs::read_to_string(brush_file_path())
		.ok()
		.and_then(|text| BrushLibrary::from_text(&text).ok())
		.unwrap_or_default()
}

fn persist_brushes(win: &Win, state: &State) {
	if let Err(e) = std::fs::write(brush_file_path(), state.brushes.to_text()) {
		win.frame.set_status_text(&format!("The brush library could not be saved: {e}"), 0);
	}
}

/// Parses "x,y x,y …" point lists (commas, spaces and semicolons all
/// separate). Needs at least one point, whole numbers only.
fn parse_points(text: &str) -> Option<Vec<(i32, i32)>> {
	let nums: Result<Vec<i32>, _> = text
		.split(|c: char| c == ',' || c == ';' || c.is_whitespace())
		.filter(|t| !t.is_empty())
		.map(|t| t.parse::<i32>())
		.collect();
	let nums = nums.ok()?;
	if nums.is_empty() || nums.len() % 2 != 0 {
		return None;
	}
	Some(nums.chunks(2).map(|p| (p[0], p[1])).collect())
}

/// Grey a dialog's selection-dependent buttons whenever its list
/// has no selection, and keep them live-or-greyed as the selection
/// changes: an action that needs a selected row is never offered as
/// a dead end. Buttons that always work (Add, Close) are not passed
/// in. Called once, after the list and buttons exist; the returned
/// handle re-applies the greying, for dialogs whose own edits can
/// empty the list without a selection event.
fn grey_with_selection(list: &Rc<QuietList<usize>>, buttons: &[Button]) -> Rc<dyn Fn()> {
	let buttons: Vec<Button> = buttons.to_vec();
	let apply = {
		let (list, buttons) = (list.clone(), buttons.clone());
		move || {
			let on = list.selected().is_some();
			for b in &buttons {
				b.enable(on);
			}
		}
	};
	apply();
	let apply: Rc<dyn Fn()> = Rc::new(apply);
	{
		let apply = apply.clone();
		list.on_selection_changed(move || apply());
	}
	apply
}

fn layer_rows(state: &State) -> Vec<(usize, String)> {
	state
		.drawing
		.layers
		.iter()
		.enumerate()
		.map(|(i, l)| {
			let mut label = format!("Layer {}", i + 1);
			if l.depth != DepthRole::Unassigned {
				label.push_str(&format!(", {}", l.depth.name()));
			}
			label.push_str(&format!(": {} ({} shapes)", l.name, l.shapes.len()));
			if i == state.layer {
				label.push_str(" — current");
			}
			if !l.visible {
				label.push_str(" — hidden");
			}
			(i, label)
		})
		.collect()
}

/// The Layers dialog: layers are the organizing unit, so they get a
/// list of their own with the everyday operations as buttons. Every
/// change is announced from the drawing log, like everywhere else.
/// The dialog is modal; the caller refreshes the main lists on return.
fn layers_dialog(win: &Win, shared: &Shared) {
	let frame = win.frame;
	let dialog = Dialog::builder(&frame, "Layers").build();
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let list = QuietList::new(&dialog, "Layers");
	sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 5);
	// The selected layer's depth role: the choice follows the
	// list selection, and choosing a role sets it at once (the
	// change lands in the log like every other layer edit).
	let depth_label =
		StaticText::builder(&dialog).with_label("Depth role of the selected layer").build();
	let depth_choice = Choice::builder(&dialog).build();
	for label in ["No depth role", "Background", "Midground", "Foreground", "Sideground"] {
		depth_choice.append(label);
	}
	depth_choice.set_name("Depth role of the selected layer");
	let depth_row = BoxSizer::builder(Orientation::Horizontal).build();
	depth_row.add(&depth_label, 0, SizerFlag::All, 5);
	depth_row.add(&depth_choice, 0, SizerFlag::All, 5);
	sizer.add_sizer(&depth_row, 0, SizerFlag::All, 5);
	let add = Button::builder(&dialog).with_label("Add Layer\u{2026}").build();
	let rename = Button::builder(&dialog).with_label("Rename\u{2026}").build();
	let remove = Button::builder(&dialog).with_label("Remove").build();
	let toggle = Button::builder(&dialog).with_label("Show / Hide").build();
	let current = Button::builder(&dialog).with_label("Set Current").build();
	let close = Button::builder(&dialog).with_label("Close").build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	for b in [&add, &rename, &remove, &toggle, &current, &close] {
		buttons.add(b, 0, SizerFlag::All, 5);
	}
	sizer.add_sizer(&buttons, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	let list = Rc::new(list);
	let depth_choice = Rc::new(depth_choice);
	{
		let state = shared.borrow();
		list.update_labelled(layer_rows(&state), Select::Start);
	}
	// Point the depth choice at the selected layer's role.
	let sync_depth = {
		let (list, shared, depth_choice) =
			(list.clone(), shared.clone(), depth_choice.clone());
		move || {
			let state = shared.borrow();
			if let Some(i) = list.selected() {
				if let Some(layer) = state.drawing.layers.get(i) {
					let at =
						DepthRole::ALL.iter().position(|r| *r == layer.depth).unwrap_or(0);
					depth_choice.set_selection(at as u32);
				}
			}
		}
	};
	let sync_depth = Rc::new(sync_depth);
	sync_depth();
	{
		let sync_depth = sync_depth.clone();
		list.on_selection_changed(move || sync_depth());
	}
	let grey = grey_with_selection(&list, &[rename, remove, toggle, current]);
	// Refresh the dialog list and speak the newest log entry.
	let sync = {
		let (list, shared, grey, sync_depth) =
			(list.clone(), shared.clone(), grey.clone(), sync_depth.clone());
		move || {
			let state = shared.borrow();
			list.update_labelled(layer_rows(&state), Select::Keep);
			if let Some(last) = state.drawing.log.last() {
				frame.set_status_text(&last.describe(&state.drawing), 0);
			}
			drop(state);
			sync_depth();
			grey();
		}
	};
	let sync = Rc::new(sync);
	{
		let (list, shared, sync, depth_choice) =
			(list.clone(), shared.clone(), sync.clone(), depth_choice.clone());
		(*depth_choice).clone().on_selection_changed(move |_| {
			let Some(i) = list.selected() else { return };
			let Some(sel) = depth_choice.get_selection() else { return };
			let Some(role) = DepthRole::ALL.get(sel as usize) else { return };
			let mut state = shared.borrow_mut();
			if state.drawing.layers.get(i).map(|l| l.depth) != Some(*role) {
				state.drawing.set_layer_depth(i, *role);
				drop(state);
				sync();
			}
		});
	}
	{
		let (shared, sync) = (shared.clone(), sync.clone());
		add.on_click(move |_| {
			let next = shared.borrow().drawing.layers.len() + 1;
			if let Some(name) =
				text_dialog(&frame, "Add layer", "Layer name", &format!("Layer {next}"))
			{
				let mut state = shared.borrow_mut();
				state.layer = state.drawing.add_layer(name.trim());
				drop(state);
				sync();
			}
		});
	}
	{
		let (list, shared, sync) = (list.clone(), shared.clone(), sync.clone());
		rename.on_click(move |_| {
			let Some(i) = list.selected() else { return };
			let old = shared.borrow().drawing.layers[i].name.clone();
			if let Some(name) = text_dialog(&frame, "Rename layer", "Layer name", &old) {
				shared.borrow_mut().drawing.rename_layer(i, name.trim());
				sync();
			}
		});
	}
	{
		let (list, shared, sync) = (list.clone(), shared.clone(), sync.clone());
		remove.on_click(move |_| {
			let Some(i) = list.selected() else { return };
			let mut state = shared.borrow_mut();
			if state.drawing.remove_layer(i) {
				if state.layer >= state.drawing.layers.len() {
					state.layer = state.drawing.layers.len() - 1;
				}
				drop(state);
				sync();
			} else {
				frame.set_status_text("A drawing always keeps one layer.", 0);
			}
		});
	}
	{
		let (list, shared, sync) = (list.clone(), shared.clone(), sync.clone());
		toggle.on_click(move |_| {
			let Some(i) = list.selected() else { return };
			let mut state = shared.borrow_mut();
			let visible = !state.drawing.layers[i].visible;
			state.drawing.set_layer_visible(i, visible);
			drop(state);
			sync();
		});
	}
	{
		let (list, shared) = (list.clone(), shared.clone());
		current.on_click(move |_| {
			let Some(i) = list.selected() else { return };
			let mut state = shared.borrow_mut();
			state.layer = i;
			let name = state.drawing.layers[i].name.clone();
			drop(state);
			frame.set_status_text(&format!("New shapes now land on layer \"{name}\"."), 0);
			let state = shared.borrow();
			list.update_labelled(layer_rows(&state), Select::Keep);
		});
	}
	{
		let dialog = dialog.clone();
		close.on_click(move |_| dialog.end_modal(ID_CANCEL));
	}
	dialog.show_modal();
}

/// Animation frame rows, in play order: frame number, how long it
/// shows, and its one-line description — what frame 1 shows, or what
/// changed since the previous frame for every later frame (the
/// descriptions core computes from the frames themselves).
fn frame_rows(state: &State) -> Vec<(usize, String)> {
	state
		.drawing
		.frames
		.iter()
		.enumerate()
		.map(|(i, f)| {
			(
				i,
				format!(
					"Frame {}: {} milliseconds. {}",
					i + 1,
					f.duration_ms,
					state.drawing.frame_change_description(i).unwrap_or_default()
				),
			)
		})
		.collect()
}

/// The Animation dialog (Design menu): the design's frames in play
/// order, each described in plain language. The canvas is the
/// working copy: Add Frame captures it as a new frame, Load brings
/// a stored frame back onto the canvas for editing, and Replace
/// stores the edited canvas back into the frame. Every change goes
/// through the drawing, so it is logged and announced like
/// everywhere else. The list is labelled "Animation frames"; the
/// dialog is modal and closing it returns to the main window.
fn animation_dialog(win: &Win, shared: &Shared) {
	let frame = win.frame;
	let dialog = Dialog::builder(&frame, "Animation").build();
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let list = QuietList::new(&dialog, "Animation frames");
	sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let add = Button::builder(&dialog).with_label("Add Frame (Copy of Canvas)").build();
	let duplicate = Button::builder(&dialog).with_label("Duplicate").build();
	let remove = Button::builder(&dialog).with_label("Delete").build();
	let up = Button::builder(&dialog).with_label("Move Up").build();
	let down = Button::builder(&dialog).with_label("Move Down").build();
	let duration = Button::builder(&dialog).with_label("Set Duration\u{2026}").build();
	let load = Button::builder(&dialog).with_label("Load into Canvas").build();
	let replace = Button::builder(&dialog).with_label("Replace with Canvas").build();
	let close = Button::builder(&dialog).with_label("Close").build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	for b in [&add, &duplicate, &remove, &up, &down, &duration, &load, &replace, &close] {
		buttons.add(b, 0, SizerFlag::All, 5);
	}
	sizer.add_sizer(&buttons, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	let list = Rc::new(list);
	{
		let state = shared.borrow();
		list.update_labelled(frame_rows(&state), Select::Start);
		if state.drawing.frames.is_empty() {
			frame.set_status_text(
				"No animation frames yet. Add Frame captures the canvas as frame 1; change the canvas and capture again to build the film.",
				0,
			);
		}
	}
	let grey =
		grey_with_selection(&list, &[duplicate, remove, up, down, duration, load, replace]);
	// Refresh the dialog list and speak the newest log entry.
	let sync = {
		let (list, shared, grey) = (list.clone(), shared.clone(), grey.clone());
		move || {
			let state = shared.borrow();
			list.update_labelled(frame_rows(&state), Select::Keep);
			if let Some(last) = state.drawing.log.last() {
				frame.set_status_text(&last.describe(&state.drawing), 0);
			}
			drop(state);
			grey();
		}
	};
	let sync = Rc::new(sync);
	{
		let (shared, sync) = (shared.clone(), sync.clone());
		add.on_click(move |_| {
			shared.borrow_mut().drawing.add_frame();
			sync();
		});
	}
	{
		let (list, shared, sync) = (list.clone(), shared.clone(), sync.clone());
		duplicate.on_click(move |_| {
			let Some(i) = list.selected() else { return };
			shared.borrow_mut().drawing.duplicate_frame(i);
			sync();
		});
	}
	{
		let (list, shared, sync) = (list.clone(), shared.clone(), sync.clone());
		remove.on_click(move |_| {
			let Some(i) = list.selected() else { return };
			shared.borrow_mut().drawing.remove_frame(i);
			sync();
		});
	}
	{
		let (list, shared, sync) = (list.clone(), shared.clone(), sync.clone());
		up.on_click(move |_| {
			let Some(i) = list.selected() else { return };
			if shared.borrow_mut().drawing.move_frame(i, true) {
				sync();
			} else {
				frame.set_status_text("That frame is already first.", 0);
			}
		});
	}
	{
		let (list, shared, sync) = (list.clone(), shared.clone(), sync.clone());
		down.on_click(move |_| {
			let Some(i) = list.selected() else { return };
			if shared.borrow_mut().drawing.move_frame(i, false) {
				sync();
			} else {
				frame.set_status_text("That frame is already last.", 0);
			}
		});
	}
	{
		let (list, shared, sync) = (list.clone(), shared.clone(), sync.clone());
		duration.on_click(move |_| {
			let Some(i) = list.selected() else { return };
			let current = {
				let state = shared.borrow();
				let Some(f) = state.drawing.frames.get(i) else { return };
				f.duration_ms
			};
			// The range comes from the frame-duration slider
			// definition — the dialog and the Sliders tab agree
			// because there is only one source for it.
			let dur_def =
				slider_by_id("frame-duration").expect("the frame-duration slider exists");
			if let Some(v) = numbers_dialog(
				&frame,
				"Frame duration",
				&[("Duration in milliseconds", dur_def.min, dur_def.max, current as i32)],
			) {
				shared.borrow_mut().drawing.set_frame_duration(i, v[0] as u32);
				sync();
			}
		});
	}
	{
		let (list, shared, sync) = (list.clone(), shared.clone(), sync.clone());
		load.on_click(move |_| {
			let Some(i) = list.selected() else { return };
			let mut state = shared.borrow_mut();
			if state.drawing.load_frame(i) {
				// The loaded frame may have fewer layers than the
				// canvas had: keep the current-layer index in range.
				if state.layer >= state.drawing.layers.len() {
					state.layer = state.drawing.layers.len() - 1;
				}
				drop(state);
				sync();
			}
		});
	}
	{
		let (list, shared, sync) = (list.clone(), shared.clone(), sync.clone());
		replace.on_click(move |_| {
			let Some(i) = list.selected() else { return };
			shared.borrow_mut().drawing.update_frame(i);
			sync();
		});
	}
	{
		let dialog = dialog.clone();
		close.on_click(move |_| dialog.end_modal(ID_CANCEL));
	}
	dialog.show_modal();
}

/// Element rows across all layers, in paint order: each element in
/// plain language (kind, position in words, size, color), with its
/// written caption when it has one. The payload is the flat index
/// `locate` understands.
fn element_rows(state: &State) -> Vec<(usize, String)> {
	let mut rows = Vec::new();
	let mut flat = 0;
	for layer in &state.drawing.layers {
		for s in &layer.shapes {
			let mut label = format!("{}: {}", layer.name, state.drawing.plain_of(s));
			if let Some(c) = &s.caption {
				label.push_str(&format!(" Caption: {c}"));
			}
			if s.locked {
				label.push_str(". Locked: bulk changes skip it.");
			}
			if !layer.visible {
				label.push_str(" (layer hidden)");
			}
			rows.push((flat, label));
			flat += 1;
		}
	}
	rows
}

/// The Elements dialog (Design menu): every element in the design,
/// named in plain language, with the everyday operations as buttons —
/// rename its caption, hear its full description, move, resize,
/// recolor or delete it — without hunting through the canvas. The
/// list is labelled "Elements"; every change goes through the
/// drawing, so it is logged and announced like everywhere else.
fn elements_dialog(win: &Win, shared: &Shared) {
	let frame = win.frame;
	let dialog = Dialog::builder(&frame, "Elements").build();
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let list = QuietList::new(&dialog, "Elements");
	sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let caption_btn = Button::builder(&dialog).with_label("Rename Caption\u{2026}").build();
	let describe_btn = Button::builder(&dialog).with_label("Describe").build();
	let move_btn = Button::builder(&dialog).with_label("Move\u{2026}").build();
	let resize_btn = Button::builder(&dialog).with_label("Resize\u{2026}").build();
	let recolor_btn = Button::builder(&dialog).with_label("Recolor\u{2026}").build();
	let delete_btn = Button::builder(&dialog).with_label("Delete").build();
	let close = Button::builder(&dialog).with_label("Close").build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	for b in [&caption_btn, &describe_btn, &move_btn, &resize_btn, &recolor_btn, &delete_btn, &close] {
		buttons.add(b, 0, SizerFlag::All, 5);
	}
	sizer.add_sizer(&buttons, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	let list = Rc::new(list);
	{
		let state = shared.borrow();
		list.update_labelled(element_rows(&state), Select::Start);
	}
	let grey = grey_with_selection(
		&list,
		&[caption_btn, describe_btn, move_btn, resize_btn, recolor_btn, delete_btn],
	);
	// Refresh the list and speak the newest log entry.
	let sync = {
		let (list, shared, grey) = (list.clone(), shared.clone(), grey.clone());
		move || {
			let state = shared.borrow();
			list.update_labelled(element_rows(&state), Select::Keep);
			if let Some(last) = state.drawing.log.last() {
				frame.set_status_text(&last.describe(&state.drawing), 0);
			}
			drop(state);
			grey();
		}
	};
	let sync = Rc::new(sync);
	// The selected element as (layer, shape), if any.
	let picked = {
		let (list, shared) = (list.clone(), shared.clone());
		move || -> Option<(usize, usize)> {
			let flat = list.selected()?;
			locate(&shared.borrow(), flat)
		}
	};
	let picked = Rc::new(picked);
	{
		let (shared, sync, picked) = (shared.clone(), sync.clone(), picked.clone());
		caption_btn.on_click(move |_| {
			let Some((li, si)) = picked() else { return };
			let now = shared.borrow().drawing.layers[li].shapes[si]
				.caption
				.clone()
				.unwrap_or_default();
			if let Some(text) =
				text_dialog(&frame, "Rename caption", "Caption for this element", &now)
			{
				if text.trim().is_empty() {
					frame.set_status_text(
						"Caption unchanged: an empty caption keeps the automatic description.",
						0,
					);
					return;
				}
				shared.borrow_mut().drawing.caption_shape(li, si, text.trim().to_string());
				sync();
			}
		});
	}
	{
		let (shared, picked) = (shared.clone(), picked.clone());
		describe_btn.on_click(move |_| {
			let Some((li, si)) = picked() else { return };
			let state = shared.borrow();
			let shape = &state.drawing.layers[li].shapes[si];
			frame.set_status_text(
				&format!(
					"{}. Full description: {}",
					state.drawing.plain_of(shape),
					state.drawing.description_of(shape)
				),
				0,
			);
		});
	}
	{
		let (shared, sync, picked) = (shared.clone(), sync.clone(), picked.clone());
		move_btn.on_click(move |_| {
			let Some((li, si)) = picked() else { return };
			let (cur, canvas) = {
				let state = shared.borrow();
				let s = &state.drawing.layers[li].shapes[si];
				((s.x, s.y), (state.drawing.width, state.drawing.height))
			};
			let Some(v) = fields_dialog(
				&frame,
				"Move element",
				&[(
					"New position: numbers like 120, 80, or words like top left, center, a bit right",
					&format!("{}, {}", cur.0, cur.1),
				)],
			) else {
				return;
			};
			match parse_position(&v[0], canvas.0, canvas.1, Some(cur)) {
				Some((nx, ny)) => {
					shared.borrow_mut().drawing.shift_shape(li, si, nx - cur.0, ny - cur.1);
					sync();
				}
				None => frame.set_status_text(
					"I could not understand that position. Use numbers like 120, 80, or words like top left, center, a bit right.",
					0,
				),
			}
		});
	}
	{
		let (shared, sync, picked) = (shared.clone(), sync.clone(), picked.clone());
		resize_btn.on_click(move |_| {
			let Some((li, si)) = picked() else { return };
			let (cur, corner) = {
				let state = shared.borrow();
				let s = &state.drawing.layers[li].shapes[si];
				if s.kind == ShapeKind::Stroke {
					frame.set_status_text(
						"A stroke's size is its points; move it or add a new one.",
						0,
					);
					return;
				}
				// Rectangles also carry their corner roundedness
				// here; other kinds show the size field alone.
				let corner = if s.kind == ShapeKind::Rectangle { Some(s.corner) } else { None };
				((s.w, s.h), corner)
			};
			let size_field = (
				"New size: numbers like 200 by 100, or words like a bit wider, a bit taller",
				format!("{} by {}", cur.0, cur.1),
			);
			let corner_field = (
				"Corners: 0 is sharp, 50 is fully rounded",
				corner.map(|c| c.to_string()).unwrap_or_default(),
			);
			let mut fields: Vec<(&str, &str)> = vec![(size_field.0, &size_field.1)];
			if corner.is_some() {
				fields.push((corner_field.0, &corner_field.1));
			}
			let Some(v) = fields_dialog(&frame, "Resize element", &fields) else {
				return;
			};
			let Some((w, h)) = parse_size(&v[0], Some(cur)) else {
				frame.set_status_text(
					"I could not understand that size. Use numbers like 200 by 100, or words like a bit wider.",
					0,
				);
				return;
			};
			let new_corner = if corner.is_some() {
				match v[1].trim().parse::<u8>() {
					Ok(c) => Some(c.min(50)),
					Err(_) => {
						frame.set_status_text(
							"Corners take a number from 0 (sharp) to 50 (fully rounded); nothing was changed.",
							0,
						);
						return;
					}
				}
			} else {
				None
			};
			let mut state = shared.borrow_mut();
			state.drawing.resize_shape(li, si, w, h);
			if let Some(c) = new_corner {
				state.drawing.set_shape_corner(li, si, c);
			}
			drop(state);
			sync();
		});
	}
	{
		let (shared, sync, picked) = (shared.clone(), sync.clone(), picked.clone());
		recolor_btn.on_click(move |_| {
			let Some((li, si)) = picked() else { return };
			let now = shared.borrow().drawing.layers[li].shapes[si].fill;
			let Some(v) = fields_dialog(
				&frame,
				"Recolor element",
				&[("New color: a name like sky blue, or a hex code", &now.to_hex())],
			) else {
				return;
			};
			match parse_color(&v[0]) {
				Some(color) => {
					shared.borrow_mut().drawing.recolor_shape(li, si, color);
					sync();
				}
				None => frame.set_status_text(
					"I could not understand that color. Use a name like sky blue, or a hex code like 87CEEB.",
					0,
				),
			}
		});
	}
	{
		let (shared, sync, picked) = (shared.clone(), sync.clone(), picked.clone());
		delete_btn.on_click(move |_| {
			let Some((li, si)) = picked() else { return };
			shared.borrow_mut().drawing.remove_shape(li, si);
			sync();
		});
	}
	{
		let dialog = dialog.clone();
		close.on_click(move |_| dialog.end_modal(ID_CANCEL));
	}
	dialog.show_modal();
}

/// The Brush Library dialog (Design menu): the saved brushes by
/// name, each spoken in full (width, color, pattern). Apply makes a
/// brush the current one — new strokes are painted with it; Delete
/// removes it from the library file.
fn brushes_dialog(win: &Win, shared: &Shared) {
	let frame = win.frame;
	let dialog = Dialog::builder(&frame, "Brush Library").build();
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let list = QuietList::new(&dialog, "Brush library");
	sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let apply = Button::builder(&dialog).with_label("Apply").build();
	let delete = Button::builder(&dialog).with_label("Delete").build();
	let close = Button::builder(&dialog).with_label("Close").build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	for b in [&apply, &delete, &close] {
		buttons.add(b, 0, SizerFlag::All, 5);
	}
	sizer.add_sizer(&buttons, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	let list = Rc::new(list);
	let rows = |state: &State| -> Vec<(usize, String)> {
		state.brushes.brushes.iter().enumerate().map(|(i, b)| (i, b.spoken())).collect()
	};
	{
		let state = shared.borrow();
		list.update_labelled(rows(&state), Select::Start);
	}
	let grey = grey_with_selection(&list, &[apply, delete]);
	{
		let (list, shared) = (list.clone(), shared.clone());
		apply.on_click(move |_| {
			let Some(i) = list.selected() else { return };
			let mut state = shared.borrow_mut();
			let Some(brush) = state.brushes.brushes.get(i).cloned() else { return };
			let spoken = brush.spoken();
			state.brush = brush;
			frame.set_status_text(
				&format!("Brush applied: {spoken}. New strokes will use it."),
				0,
			);
		});
	}
	{
		let (list, shared, grey) = (list.clone(), shared.clone(), grey.clone());
		delete.on_click(move |_| {
			let Some(i) = list.selected() else { return };
			let mut state = shared.borrow_mut();
			let Some(brush) = state.brushes.brushes.get(i).cloned() else { return };
			let name = brush.name.clone();
			state.brushes.remove(&name);
			if let Err(e) = std::fs::write(brush_file_path(), state.brushes.to_text()) {
				frame.set_status_text(&format!("The brush library could not be saved: {e}"), 0);
				return;
			}
			let rows_now = state.brushes.brushes
				.iter()
				.enumerate()
				.map(|(j, b)| (j, b.spoken()))
				.collect::<Vec<(usize, String)>>();
			drop(state);
			list.update_labelled(rows_now, Select::Keep);
			grey();
			frame.set_status_text(&format!("Deleted brush \"{name}\" from the library."), 0);
		});
	}
	{
		let dialog = dialog.clone();
		close.on_click(move |_| dialog.end_modal(ID_CANCEL));
	}
	dialog.show_modal();
}

/// The Dictionary dialog (Help menu): search the built-in design-terms
/// dictionary (chroma-core::dictionary) and read definitions in a
/// reviewable text field. The list is labelled "Dictionary terms", the
/// definition box "Definition"; opening the dialog moves no focus
/// anywhere else, and closing returns to the main window.
fn dictionary_dialog(win: &Win) {
	let frame = win.frame;
	let dialog = Dialog::builder(&frame, "Design Terms Dictionary").with_size(560, 480).build();
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let search_label = StaticText::builder(&dialog).with_label("Search terms").build();
	let search_field = TextCtrl::builder(&dialog).build();
	let search_button = Button::builder(&dialog).with_label("Search").build();
	let search_row = BoxSizer::builder(Orientation::Horizontal).build();
	search_row.add(&search_label, 0, SizerFlag::All, 5);
	search_row.add(&search_field, 1, SizerFlag::All, 5);
	search_row.add(&search_button, 0, SizerFlag::All, 5);
	sizer.add_sizer(&search_row, 0, SizerFlag::Expand, 0);
	let list_label = StaticText::builder(&dialog).with_label("Dictionary terms").build();
	sizer.add(&list_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	let list = QuietList::<usize>::new(&dialog, "Dictionary terms");
	sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let list = Rc::new(list);
	let def_label = StaticText::builder(&dialog).with_label("Definition").build();
	sizer.add(&def_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	let definition = TextCtrl::builder(&dialog)
		.with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::ReadOnly)
		.build();
	sizer.add(&definition, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let close = Button::builder(&dialog).with_label("Close").build();
	sizer.add(&close, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);

	let rows = |query: &str| -> Vec<(usize, String)> {
		dictionary::search(query)
			.iter()
			.enumerate()
			.map(|(i, en)| (i, en.term.to_string()))
			.collect()
	};
	let show_selected = {
		let (list, definition, search_field) =
			(list.clone(), definition.clone(), search_field.clone());
		move || {
			let hits = dictionary::search(&search_field.get_value());
			if let Some(i) = list.selected() {
				if let Some(en) = hits.get(i) {
					definition.set_value(&format!("{}: {}", en.term, en.definition));
				}
			}
		}
	};
	let show_selected = Rc::new(show_selected);
	list.update_labelled(rows(""), Select::Start);
	show_selected();
	{
		let show = show_selected.clone();
		list.on_selection_changed(move || show());
	}
	{
		let (list, definition, search_field) =
			(list.clone(), definition.clone(), search_field.clone());
		search_button.on_click(move |_| {
			let query = search_field.get_value();
			let hits = dictionary::search(&query);
			list.update_labelled(rows(&query), Select::Start);
			match hits.first() {
				Some(en) => {
					definition.set_value(&format!("{}: {}", en.term, en.definition));
					frame.set_status_text(
						&format!("{} dictionary terms match.", hits.len()),
						0,
					);
				}
				None => {
					definition.set_value("No term matches that search.");
					frame.set_status_text("No dictionary term matches that search.", 0);
				}
			}
		});
	}
	{
		let dialog = dialog.clone();
		close.on_click(move |_| dialog.end_modal(ID_CANCEL));
	}
	dialog.show_modal();
}

/// The Help dialog (Help menu): the shipped help file — how the app
/// works plus step-by-step worked examples — in one read-only,
/// reviewable text field labelled "Help and worked examples".
fn help_dialog(win: &Win) {
	let frame = win.frame;
	let dialog = Dialog::builder(&frame, "Help and Worked Examples").with_size(640, 520).build();
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let label = StaticText::builder(&dialog).with_label("Help and worked examples").build();
	sizer.add(&label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	let text = TextCtrl::builder(&dialog)
		.with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::ReadOnly)
		.build();
	text.set_value(HELP_TEXT);
	sizer.add(&text, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let close = Button::builder(&dialog).with_label("Close").build();
	sizer.add(&close, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	{
		let dialog = dialog.clone();
		close.on_click(move |_| dialog.end_modal(ID_CANCEL));
	}
	dialog.show_modal();
}

/// A read-only report in a modal dialog: image comparisons and
/// tactile grids land here, in a labelled multi-line text field a
/// screen reader can read through at leisure. The dialog is modal,
/// focus moves only by the user's own keys, and Close returns to
/// the main window.
fn report_dialog(win: &Win, title: &str, label: &str, text: &str) {
	let frame = win.frame;
	let dialog = Dialog::builder(&frame, title).with_size(680, 540).build();
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let caption = StaticText::builder(&dialog).with_label(label).build();
	sizer.add(&caption, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	let body = TextCtrl::builder(&dialog)
		.with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::ReadOnly)
		.build();
	body.set_value(text);
	sizer.add(&body, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let close = Button::builder(&dialog).with_label("Close").build();
	sizer.add(&close, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	{
		let dialog = dialog.clone();
		close.on_click(move |_| dialog.end_modal(ID_CANCEL));
	}
	dialog.show_modal();
}

/// The (layer, shape) of the element the Elements list has
/// selected, if any.
fn selected_element_locate(win: &Win, shared: &Shared) -> Option<(usize, usize)> {
	let flat = win.elements_list.borrow().selected()?;
	let state = shared.borrow();
	locate(&state, flat)
}

/// The inspector for the selected element (Design, Inspect
/// Selected; Enter on the Elements list): core writes the
/// report from the drawing itself, the app shows it in the
/// standard report dialog.
fn inspect_selected(win: &Rc<Win>, shared: &Shared) {
	let Some(flat) = win.elements_list.borrow().selected() else { return };
	let state = shared.borrow();
	if let Some(text) = inspect_shape(&state.drawing, flat) {
		report_dialog(win, "Inspector: element", "Element inspector", &text);
	}
}

/// One context-menu item, carrying the SAME command ID as its
/// menu-bar twin, so a popup dispatches through the one dispatch
/// function — no second copy of any action.
fn ctx(menu: &Menu, id: Id, text: &str) {
	menu.append(id, text, "", ItemKind::Normal);
}

/// The Palette list's context menu. An empty palette still gets
/// a menu (Elshara: an empty list should show that something is
/// there and let you start): the three ways colors arrive. With
/// a color selected, its own actions lead.
fn show_palette_context(win: &Rc<Win>) {
	let mut menu = Menu::builder().build();
	if win.palette_list.borrow().selected().is_some() {
		ctx(&menu, ID_INSPECT_COLOR, "Inspect Color…");
		ctx(&menu, ID_SET_BACKGROUND, "Set as Background");
		ctx(&menu, ID_REMOVE_COLOR, "Remove Color");
		menu.append_separator();
	}
	ctx(&menu, ID_ADD_COLOR, "Add Color by Hex…");
	ctx(&menu, ID_LOAD_STARTER, "Load Starter Colors");
	ctx(&menu, ID_OPEN_IMAGE, "Extract from Image…");
	win.palette_list.borrow().popup_menu(&mut menu);
}

/// The Elements list's context menu: with an element selected,
/// its actions (inspect and lock first — the two this wave
/// adds); empty or not, the Add items are always there, so an
/// empty list can start the design from its own menu.
fn show_elements_context(win: &Rc<Win>) {
	let mut menu = Menu::builder().build();
	if win.elements_list.borrow().selected().is_some() {
		ctx(&menu, ID_INSPECT, "Inspect Selected…");
		ctx(&menu, ID_LOCK_TOGGLE, "Lock/Unlock Selected");
		ctx(&menu, ID_MOVE_SHAPE, "Move…");
		ctx(&menu, ID_ROTATE_SHAPE, "Rotate…");
		ctx(&menu, ID_RESIZE_SHAPE, "Resize…");
		ctx(&menu, ID_RECOLOR_SHAPE, "Set to Palette Color");
		ctx(&menu, ID_REMOVE_SHAPE, "Remove Shape");
		menu.append_separator();
	}
	ctx(&menu, ID_ADD_RECT, "Add Rectangle…");
	ctx(&menu, ID_ADD_ELLIPSE, "Add Ellipse…");
	ctx(&menu, ID_ADD_STROKE, "Add Stroke…");
	ctx(&menu, ID_ADD_TEXT, "Add Text…");
	ctx(&menu, ID_ADD_IMAGE, "Add Image…");
	ctx(&menu, ID_ADD_SYMBOL, "Add Symbol…");
	win.elements_list.borrow().popup_menu(&mut menu);
}

/// The Layers list's context menu: inspect the selected layer,
/// or open the Layers dialog (where layers are added, renamed
/// and reordered). A design always has at least one layer, so
/// this list is never empty.
fn show_layers_context(win: &Rc<Win>) {
	let mut menu = Menu::builder().build();
	if win.layers_list.selected().is_some() {
		ctx(&menu, ID_INSPECT_LAYER, "Inspect Layer…");
	}
	ctx(&menu, ID_LAYERS, "Layers…");
	win.layers_list.popup_menu(&mut menu);
}

/// The Sliders page's context menu: the pins live in Project
/// Options, so that is the menu — add or remove sliders right
/// from the area they would appear in.
fn show_sliders_context(win: &Rc<Win>) {
	let mut menu = Menu::builder().build();
	ctx(&menu, ID_PROJECT_OPTIONS, "Project Options… (choose pinned sliders)");
	if let Some(page) = win.slider_page.borrow().as_ref() {
		page.popup_menu(&mut menu, None);
	}
}

/// Put one search hit where the cursor is, and say where it
/// landed. The cursor, honestly defined per kind: a color goes
/// to the palette and becomes the brush color (the color the
/// next thing drawn will use); an element or symbol is copied
/// onto the design — right after the selected element when one
/// is selected, at the canvas centre otherwise; a template
/// replaces the design, the way File, New from Template does.
/// A pattern needs its two colors first, so it opens the
/// pattern dialog instead and returns None here.
fn insert_hit(win: &Rc<Win>, shared: &Shared, hit: &CatalogHit) -> Option<String> {
	match hit {
		CatalogHit::PaletteColor { index, .. } => {
			let color = shared.borrow().palette.get(*index)?.color;
			let state = shared.borrow();
			select_color(win, &state, color);
			let format = state.color_format;
			drop(state);
			shared.borrow_mut().brush.color = color;
			Some(format!(
				"{} is now the brush color: the next element drawn uses it.",
				describe_in(color, format)
			))
		}
		CatalogHit::StarterColor { color, .. } => {
			let mut state = shared.borrow_mut();
			let already = state.palette.iter().any(|pc| pc.color == *color);
			if !already {
				state.palette.push(PaletteColor::new(*color, Provenance::Starter));
			}
			state.brush.color = *color;
			let format = state.color_format;
			drop(state);
			let state = shared.borrow();
			refresh_lists(win, &state);
			select_color(win, &state, *color);
			Some(format!(
				"{} {} the palette and is now the brush color.",
				describe_in(*color, format),
				if already { "was already in" } else { "was added to" }
			))
		}
		CatalogHit::Element { layer, index, .. } => {
			let mut state = shared.borrow_mut();
			let source = state.drawing.layers.get(*layer)?.shapes.get(*index)?.clone();
			let words = state.drawing.plain_of(&source);
			// The landing spot: after the selected element (in
			// its layer, offset a step so the copy is findable),
			// or centred on the current layer.
			let selected = win
				.elements_list
				.borrow()
				.selected()
				.and_then(|flat| locate(&state, flat));
			let note = match selected {
				Some((sli, ssi)) => {
					let mut copy = source;
					copy.x = (copy.x + 10)
						.min((state.drawing.width as i32 - copy.w as i32).max(0));
					copy.y = (copy.y + 10)
						.min((state.drawing.height as i32 - copy.h as i32).max(0));
					let layer_name = state.drawing.layers[sli].name.clone();
					state.drawing.insert_shape(sli, ssi + 1, copy);
					format!("on layer \"{layer_name}\", right after the selected element")
				}
				None => {
					let mut copy = source;
					copy.x = ((state.drawing.width as i32 - copy.w as i32) / 2).max(0);
					copy.y = ((state.drawing.height as i32 - copy.h as i32) / 2).max(0);
					let li = state.layer.min(state.drawing.layers.len() - 1);
					let layer_name = state.drawing.layers[li].name.clone();
					state.drawing.add_shape(li, copy);
					format!("at the centre of the canvas on layer \"{layer_name}\"")
				}
			};
			drop(state);
			let state = shared.borrow();
			refresh_lists(win, &state);
			Some(format!("Copied the {words} {note}."))
		}
		CatalogHit::Symbol { name, .. } => {
			let mut state = shared.borrow_mut();
			let color = state.brush.color;
			// Built the way the Symbols dialog builds them: the
			// symbol's paths in a 100 by 100 box at the centre,
			// stroked with the brush color and captioned.
			let cx = ((state.drawing.width as i32 - 100) / 2).max(0);
			let cy = ((state.drawing.height as i32 - 100) / 2).max(0);
			let paths = symbols::symbol_paths(name, cx, cy, 100, 100)?;
			let li = state.layer.min(state.drawing.layers.len() - 1);
			let layer_name = state.drawing.layers[li].name.clone();
			let opacity = state.current_opacity;
			for path in paths {
				let mut shape = Shape::stroke(path, 3, color);
				shape.opacity = opacity;
				shape.caption = Some(format!("{name} symbol"));
				state.drawing.add_shape(li, shape);
			}
			drop(state);
			let state = shared.borrow();
			refresh_lists(win, &state);
			Some(format!(
				"The {name} symbol was stamped at the centre of the canvas on layer \"{layer_name}\", in the brush color."
			))
		}
		CatalogHit::Template { name, .. } => {
			let design = templates::template(name)?;
			let mut state = shared.borrow_mut();
			state.drawing = design;
			state.layer = 0;
			state.pinned_sliders = Vec::new();
			state.slider_target = AdjustTarget::BaseColor;
			state.page_jump_percent = DEFAULT_PAGE_JUMP_PERCENT;
			reset_adjustment(&mut state);
			drop(state);
			build_sliders_tab(win, shared);
			let state = shared.borrow();
			refresh_lists(win, &state);
			Some(format!(
				"The design is now the {name} template. It is a normal design now: edit it, then Save Design to keep it."
			))
		}
		CatalogHit::Pattern { .. } => None,
	}
}

/// The Search and Insert dialog (Design menu, Ctrl+F): one
/// field over everything the design can gain — palette colors,
/// starter colors, the design's own elements, symbols,
/// templates, patterns — and an Insert button (or Enter on a
/// result) that puts the chosen thing at the cursor. Modelled
/// on the Dictionary dialog: type, press Search, choose, insert.
fn search_dialog(win: &Rc<Win>, shared: &Shared) {
	let frame = win.frame;
	let dialog =
		Dialog::builder(&frame, "Search and Insert").with_size(620, 560).build();
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let search_label = StaticText::builder(&dialog).with_label("Search for").build();
	let field = TextCtrl::builder(&dialog).build();
	let search_button = Button::builder(&dialog).with_label("Search").build();
	let search_row = BoxSizer::builder(Orientation::Horizontal).build();
	search_row.add(&search_label, 0, SizerFlag::All, 5);
	search_row.add(&field, 1, SizerFlag::All, 5);
	search_row.add(&search_button, 0, SizerFlag::All, 5);
	sizer.add_sizer(&search_row, 0, SizerFlag::Expand, 0);
	let list_label = StaticText::builder(&dialog).with_label("Results").build();
	sizer.add(&list_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	let list = QuietList::<usize>::new(&dialog, "Results");
	sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let insert = Button::builder(&dialog).with_label("Insert").build();
	let close = Button::builder(&dialog).with_label("Close").build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	buttons.add(&insert, 0, SizerFlag::All, 5);
	buttons.add(&close, 0, SizerFlag::All, 5);
	sizer.add_sizer(&buttons, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	let list = Rc::new(list);
	let hits: Rc<RefCell<Vec<CatalogHit>>> = Rc::new(RefCell::new(Vec::new()));
	let run_search = {
		let (list, field, hits, shared) =
			(list.clone(), field.clone(), hits.clone(), shared.clone());
		move || {
			let project = {
				let state = shared.borrow();
				Project {
					drawing: state.drawing.clone(),
					palette: state.palette.clone(),
					pinned_sliders: state.pinned_sliders.clone(),
					slider_target: state.slider_target,
					page_jump_percent: state.page_jump_percent,
				}
			};
			let found = search_catalog(&project, &field.get_value());
			let rows: Vec<(usize, String)> = found
				.iter()
				.enumerate()
				.map(|(i, h)| (i, h.label().to_string()))
				.collect();
			*hits.borrow_mut() = found;
			list.update_labelled(rows, Select::Start);
		}
	};
	let run_search = Rc::new(run_search);
	run_search();
	{
		let run = run_search.clone();
		search_button.on_click(move |_| run());
	}
	let do_insert = {
		let (list, hits, dialog, win, shared) =
			(list.clone(), hits.clone(), dialog.clone(), win.clone(), shared.clone());
		move || {
			let Some(i) = list.selected() else { return };
			let hit = hits.borrow().get(i).cloned();
			let Some(hit) = hit else { return };
			if let CatalogHit::Pattern { name, .. } = hit {
				// A pattern needs its two colors first.
				dialog.end_modal(ID_CANCEL);
				pattern_dialog(&win, &shared, Some(name));
				build_sliders_tab(&win, &shared);
				let state = shared.borrow();
				refresh_lists(&win, &state);
				return;
			}
			if let Some(note) = insert_hit(&win, &shared, &hit) {
				dialog.end_modal(ID_OK);
				report_dialog(&win, "Inserted", "What was inserted", &note);
			}
		}
	};
	let do_insert = Rc::new(do_insert);
	{
		let go = do_insert.clone();
		insert.on_click(move |_| go());
	}
	{
		let go = do_insert.clone();
		list.on_key_down(move |data| {
			let WindowEventData::Keyboard(kb) = data else { return false };
			if kb.get_key_code() == Some(WXK_RETURN) {
				go();
				true
			} else {
				false
			}
		});
	}
	{
		let dialog = dialog.clone();
		close.on_click(move |_| dialog.end_modal(ID_CANCEL));
	}
	dialog.show_modal();
}

/// The Pattern dialog (File, New from Pattern; also where a
/// pattern search result lands): pick the weave and the two
/// colors it is woven from — the marks and the ground — then
/// start the design. The palette pickers list the palette; a
/// project with no colors yet is told to add some first.
fn pattern_dialog(win: &Rc<Win>, shared: &Shared, preselect: Option<&str>) {
	let frame = win.frame;
	let (names, colors) = {
		let state = shared.borrow();
		if state.palette.is_empty() {
			frame.set_status_text(
				"A pattern is woven from two palette colors, and this project has none yet: add a color or load the starter colors first.",
				0,
			);
			return;
		}
		(
			state.palette.iter().map(|pc| pc.spoken(state.color_format)).collect::<Vec<_>>(),
			state.palette.iter().map(|pc| pc.color).collect::<Vec<_>>(),
		)
	};
	// The ground defaults to the design's background when the
	// palette holds it, else the second color.
	let background = shared.borrow().drawing.background;
	let ground_default = colors
		.iter()
		.position(|c| *c == background)
		.unwrap_or(if colors.len() > 1 { 1 } else { 0 });
	let dialog = Dialog::builder(&frame, "New from Pattern").build();
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let kind_label = StaticText::builder(&dialog).with_label("Pattern").build();
	sizer.add(&kind_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	let kinds = Choice::builder(&dialog).build();
	for name in templates::PATTERN_NAMES {
		kinds.append(name);
	}
	kinds.set_selection(
		preselect
			.and_then(|p| templates::PATTERN_NAMES.iter().position(|n| *n == p))
			.unwrap_or(0) as u32,
	);
	sizer.add(&kinds, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let first_label =
		StaticText::builder(&dialog).with_label("First color (the marks)").build();
	sizer.add(&first_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	let first = Choice::builder(&dialog).build();
	for row in &names {
		first.append(row);
	}
	first.set_selection(0);
	sizer.add(&first, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let second_label =
		StaticText::builder(&dialog).with_label("Second color (the ground)").build();
	sizer.add(&second_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	let second = Choice::builder(&dialog).build();
	for row in &names {
		second.append(row);
	}
	second.set_selection(ground_default as u32);
	sizer.add(&second, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let start = Button::builder(&dialog).with_label("Start Pattern Design").build();
	let close = Button::builder(&dialog).with_label("Close").build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	buttons.add(&start, 0, SizerFlag::All, 5);
	buttons.add(&close, 0, SizerFlag::All, 5);
	sizer.add_sizer(&buttons, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	{
		let (dialog, shared) = (dialog.clone(), shared.clone());
		start.on_click(move |_| {
			let kind = templates::PATTERN_NAMES
				.get(kinds.get_selection().map(|i| i as usize).unwrap_or(0))
				.copied();
			let a = colors.get(first.get_selection().map(|i| i as usize).unwrap_or(0)).copied();
			let b = colors
				.get(second.get_selection().map(|i| i as usize).unwrap_or(0))
				.copied();
			let (Some(kind), Some(a), Some(b)) = (kind, a, b) else { return };
			let Some(design) = templates::pattern_template(kind, a, b) else { return };
			let mut state = shared.borrow_mut();
			state.drawing = design;
			state.layer = 0;
			// A new project starts from scratch, like a template.
			state.pinned_sliders = Vec::new();
			state.slider_target = AdjustTarget::BaseColor;
			state.page_jump_percent = DEFAULT_PAGE_JUMP_PERCENT;
			reset_adjustment(&mut state);
			drop(state);
			frame.set_status_text(
				&format!("Started a new {kind} design from the two colors. It is a normal design now: edit it, then Save Design to keep it."),
				0,
			);
			dialog.end_modal(ID_OK);
		});
	}
	{
		let dialog = dialog.clone();
		close.on_click(move |_| dialog.end_modal(ID_CANCEL));
	}
	dialog.show_modal();
	// The caller (dispatch / the search dialog) refreshes the
	// lists and sliders tab, the way a template load does.
}

/// The Viewability dialog (Design menu): ask where the viewer
/// stands (an angle in degrees — 0 is straight on from the
/// front, and an element turned more than 90 degrees away from
/// it shows its back) and which depth to judge, then show core's
/// report in the standard report dialog. Both controls carry
/// word labels, like every dialog in the app.
fn viewability_dialog(win: &Win, shared: &Shared) {
	let frame = win.frame;
	let dialog = Dialog::builder(&frame, "Viewability").build();
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let angle_label = StaticText::builder(&dialog)
		.with_label("Viewing angle in degrees (0 is straight on from the front)")
		.build();
	let angle_spin = SpinCtrl::builder(&dialog).with_range(0, 359).build();
	angle_spin.set_value(0);
	sizer.add(&angle_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	sizer.add(&angle_spin, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let band_label = StaticText::builder(&dialog).with_label("Depth to judge").build();
	let band_choice = Choice::builder(&dialog).build();
	for band in DepthBand::ALL {
		band_choice.append(band.name());
	}
	band_choice.set_selection(0);
	band_choice.set_name("Depth to judge");
	sizer.add(&band_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	sizer.add(&band_choice, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let ok = Button::builder(&dialog).with_label("OK").build();
	let cancel = Button::builder(&dialog).with_label("Cancel").build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	buttons.add(&ok, 0, SizerFlag::All, 5);
	buttons.add(&cancel, 0, SizerFlag::All, 5);
	sizer.add_sizer(&buttons, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	{
		let d = dialog.clone();
		ok.on_click(move |_| d.end_modal(ID_OK));
	}
	{
		let d = dialog.clone();
		cancel.on_click(move |_| d.end_modal(ID_CANCEL));
	}
	if dialog.show_modal() == ID_OK {
		let band = band_choice
			.get_selection()
			.and_then(|sel| DepthBand::ALL.get(sel as usize).copied())
			.unwrap_or(DepthBand::All);
		let angle = angle_spin.value().rem_euclid(360);
		let state = shared.borrow();
		report_dialog(
			win,
			"Viewability",
			&format!("How the design reads from {angle} degrees"),
			&state.drawing.viewability_report(angle as f64, band),
		);
	}
}

/// The Scheme Settings dialog (Color, Schemes, Scheme Settings):
/// the two scheme parameters — which wheel hue rotations run on
/// (the screen wheel, or the RYB paint wheel where red's opposite
/// is green) and the analogous spread in degrees. Every control is
/// labelled in words; OK applies both to the session.
fn scheme_settings_dialog(win: &Win, shared: &Shared) {
	let frame = win.frame;
	let (wheel_ryb, spread) = {
		let state = shared.borrow();
		(state.wheel_ryb, state.scheme_spread)
	};
	let dialog = Dialog::builder(&frame, "Scheme Settings").build();
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let wheel_label = StaticText::builder(&dialog).with_label("Harmony wheel").build();
	sizer.add(&wheel_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	let wheel = Choice::builder(&dialog).build();
	wheel.append("Screen wheel (RGB hues)");
	wheel.append("Paint wheel (RYB: red is opposite green)");
	wheel.set_selection(if wheel_ryb { 1 } else { 0 });
	sizer.add(&wheel, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let spread_label =
		StaticText::builder(&dialog).with_label("Analogous spread in degrees").build();
	sizer.add(&spread_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	let spread_spin = SpinCtrl::builder(&dialog).with_range(5, 90).build();
	spread_spin.set_value(spread as i32);
	sizer.add(&spread_spin, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let ok = Button::builder(&dialog).with_label("OK").build();
	let cancel = Button::builder(&dialog).with_label("Cancel").build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	buttons.add(&ok, 0, SizerFlag::All, 5);
	buttons.add(&cancel, 0, SizerFlag::All, 5);
	sizer.add_sizer(&buttons, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	let d = dialog.clone();
	ok.on_click(move |_| d.end_modal(ID_OK));
	let d = dialog.clone();
	cancel.on_click(move |_| d.end_modal(ID_CANCEL));
	if dialog.show_modal() == ID_OK {
		let mut state = shared.borrow_mut();
		state.wheel_ryb = wheel.get_selection() == Some(1);
		state.scheme_spread = spread_spin.value() as f32;
		frame.set_status_text(
			&format!(
				"Scheme settings saved: {} wheel, analogous spread {:.0} degrees.",
				if state.wheel_ryb { "paint (RYB)" } else { "screen (RGB)" },
				state.scheme_spread
			),
			0,
		);
	}
}

/// The Color Tools dialog (Color, Color Tools): the colorhexa-style
/// workshop. Two palette colors are picked by their spoken rows; the
/// tools blend them (in OKLab, at the slider's percent), subtract,
/// build a gradient ramp into the palette, check their WCAG
/// contrast, and report the first color's dominant wavelength. Two
/// more tools need no picked colors: a random color, and the
/// experimental sound-frequency-to-color. Every outcome lands in
/// the labelled Result field; colors the tools make join the
/// palette with Generated provenance. The dialog stays open so
/// several tools can run in one visit; focus is never moved.
fn color_tools_dialog(win: &Rc<Win>, shared: &Shared) {
	let frame = win.frame;
	let dialog = Dialog::builder(&frame, "Color Tools").with_size(640, 560).build();
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let first_label = StaticText::builder(&dialog).with_label("First color").build();
	sizer.add(&first_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	let first = Choice::builder(&dialog).build();
	sizer.add(&first, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let second_label = StaticText::builder(&dialog).with_label("Second color").build();
	sizer.add(&second_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	let second = Choice::builder(&dialog).build();
	sizer.add(&second, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let blend_label =
		StaticText::builder(&dialog).with_label("Blend: percent of the second color").build();
	sizer.add(&blend_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	let blend_row = BoxSizer::builder(Orientation::Horizontal).build();
	let blend_slider = Slider::builder(&dialog)
		.with_min_value(0)
		.with_max_value(100)
		.with_value(50)
		.build();
	blend_slider.set_name("Blend percent of second color");
	blend_row.add(&blend_slider, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let blend_readout = StaticText::builder(&dialog).with_label("50 percent").build();
	blend_row.add(&blend_readout, 0, SizerFlag::All, 5);
	sizer.add_sizer(&blend_row, 0, SizerFlag::Expand, 0);
	let blend_btn = Button::builder(&dialog).with_label("Blend").build();
	let subtract_btn = Button::builder(&dialog).with_label("Subtract Second from First").build();
	let gradient_btn = Button::builder(&dialog).with_label("Gradient…").build();
	let row1 = BoxSizer::builder(Orientation::Horizontal).build();
	for b in [&blend_btn, &subtract_btn, &gradient_btn] {
		row1.add(b, 0, SizerFlag::All, 5);
	}
	sizer.add_sizer(&row1, 0, SizerFlag::All, 5);
	let contrast_btn = Button::builder(&dialog).with_label("Contrast Check").build();
	let wavelength_btn = Button::builder(&dialog).with_label("Dominant Wavelength").build();
	let random_btn = Button::builder(&dialog).with_label("Random Color").build();
	let row2 = BoxSizer::builder(Orientation::Horizontal).build();
	for b in [&contrast_btn, &wavelength_btn, &random_btn] {
		row2.add(b, 0, SizerFlag::All, 5);
	}
	sizer.add_sizer(&row2, 0, SizerFlag::All, 5);
	let freq_btn = Button::builder(&dialog)
		.with_label("Frequency to Color (experimental)…")
		.build();
	let close = Button::builder(&dialog).with_label("Close").build();
	let row3 = BoxSizer::builder(Orientation::Horizontal).build();
	row3.add(&freq_btn, 0, SizerFlag::All, 5);
	row3.add(&close, 0, SizerFlag::All, 5);
	sizer.add_sizer(&row3, 0, SizerFlag::All, 5);
	let result_label = StaticText::builder(&dialog).with_label("Result").build();
	sizer.add(&result_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
	let result = TextCtrl::builder(&dialog)
		.with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::ReadOnly)
		.build();
	sizer.add(&result, 1, SizerFlag::Expand | SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);

	{
		let state = shared.borrow();
		for pc in &state.palette {
			let row = pc.spoken(state.color_format);
			first.append(&row);
			second.append(&row);
		}
		if !state.palette.is_empty() {
			let sel = win
				.palette_list
				.borrow()
				.selected()
				.filter(|i| *i < state.palette.len())
				.unwrap_or(0);
			first.set_selection(sel as u32);
			second.set_selection(if state.palette.len() > 1 { 1 } else { 0 });
		} else {
			result.set_value(
				"The palette is empty. Add colors first (type a hex value, open an image, or use Color, Load Starter Colors); the Random Color and Frequency tools below work anyway.",
			);
		}
	}
	let first = Rc::new(first);
	let second = Rc::new(second);
	let blend_slider = Rc::new(blend_slider);
	let result = Rc::new(result);
	{
		let (for_closure, readout) = (blend_slider.clone(), blend_readout.clone());
		blend_slider.on_slider(move |_| {
			readout.set_label(&format!("{} percent", for_closure.get_value()));
		});
	}
	// The two picked palette colors, or a note in the Result field.
	let picked = {
		let (first, second, result) = (first.clone(), second.clone(), result.clone());
		move |state: &State| -> Option<(Srgb, Srgb)> {
			let (Some(i), Some(j)) = (first.get_selection(), second.get_selection()) else {
				result.set_value("Pick a first and a second color from the palette first.");
				return None;
			};
			match (state.palette.get(i as usize), state.palette.get(j as usize)) {
				(Some(a), Some(b)) => Some((a.color, b.color)),
				_ => {
					result.set_value("Pick a first and a second color from the palette first.");
					None
				}
			}
		}
	};
	let picked = Rc::new(picked);
	{
		let (shared, win, result, slider, picked) = (
			shared.clone(),
			win.clone(),
			result.clone(),
			blend_slider.clone(),
			picked.clone(),
		);
		blend_btn.on_click(move |_| {
			let mut state = shared.borrow_mut();
			let Some((a, b)) = picked(&state) else { return };
			let percent = slider.get_value().clamp(0, 100) as u8;
			let c = blend(a, b, percent);
			let provenance = Provenance::Generated {
				scheme: format!("blend of {} and {}", a.to_hex(), b.to_hex()),
			};
			let text = format!(
				"Blend at {percent} percent of the second color: {}. Added to the palette.",
				describe_in(c, state.color_format)
			);
			add_colors(&win, &mut state, &[c], "blend", &provenance);
			result.set_value(&text);
		});
	}
	{
		let (shared, win, result, picked) =
			(shared.clone(), win.clone(), result.clone(), picked.clone());
		subtract_btn.on_click(move |_| {
			let mut state = shared.borrow_mut();
			let Some((a, b)) = picked(&state) else { return };
			let c = subtract(a, b);
			let provenance = Provenance::Generated {
				scheme: format!("{} minus {}", a.to_hex(), b.to_hex()),
			};
			let text = format!(
				"Subtraction, each channel of the second taken from the first: {}. Added to the palette.",
				describe_in(c, state.color_format)
			);
			add_colors(&win, &mut state, &[c], "subtraction", &provenance);
			result.set_value(&text);
		});
	}
	{
		let (shared, win, result, picked) =
			(shared.clone(), win.clone(), result.clone(), picked.clone());
		gradient_btn.on_click(move |_| {
			let Some(v) = numbers_dialog(
				&frame,
				"Gradient steps",
				&[("Steps in the ramp, both end colors included", 2, 32, 7)],
			) else {
				return;
			};
			let mut state = shared.borrow_mut();
			let Some((a, b)) = picked(&state) else { return };
			let ramp = gradient(a, b, v[0] as usize);
			let provenance = Provenance::Generated {
				scheme: format!("gradient from {} to {}", a.to_hex(), b.to_hex()),
			};
			let text = format!(
				"Gradient of {} steps from {} to {} added to the palette.",
				ramp.len(),
				describe_in(a, state.color_format),
				describe_in(b, state.color_format)
			);
			add_colors(&win, &mut state, &ramp, "gradient", &provenance);
			result.set_value(&text);
		});
	}
	{
		let (shared, result, picked) = (shared.clone(), result.clone(), picked.clone());
		contrast_btn.on_click(move |_| {
			let state = shared.borrow();
			let Some((a, b)) = picked(&state) else { return };
			let ratio = a.contrast(b);
			result.set_value(&format!(
				"{} against {}: contrast {:.1} to 1 — {}.",
				describe_in(a, state.color_format),
				describe_in(b, state.color_format),
				ratio,
				contrast_verdict(ratio)
			));
		});
	}
	{
		let (shared, result, first) = (shared.clone(), result.clone(), first.clone());
		wavelength_btn.on_click(move |_| {
			let state = shared.borrow();
			let Some(i) = first.get_selection() else {
				result.set_value("Pick a first color from the palette first.");
				return;
			};
			let Some(pc) = state.palette.get(i as usize) else { return };
			match pc.color.dominant_wavelength() {
				Some(w) => result.set_value(&format!(
					"{}: {}.",
					describe_in(pc.color, state.color_format),
					w.spoken()
				)),
				None => result.set_value(&format!(
					"{} has no dominant wavelength: it is achromatic, a gray with no hue direction.",
					describe_in(pc.color, state.color_format)
				)),
			}
		});
	}
	{
		let (shared, win, result) = (shared.clone(), win.clone(), result.clone());
		random_btn.on_click(move |_| {
			let seed = std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.map(|d| d.as_nanos() as u64)
				.unwrap_or(0x5EED);
			let c = random_color(seed);
			let mut state = shared.borrow_mut();
			let provenance =
				Provenance::Generated { scheme: "random color".to_string() };
			let text = format!(
				"Random color: {}. Added to the palette.",
				describe_in(c, state.color_format)
			);
			add_colors(&win, &mut state, &[c], "random color", &provenance);
			drop(state);
			result.set_value(&text);
		});
	}
	{
		let (shared, win, result) = (shared.clone(), win.clone(), result.clone());
		freq_btn.on_click(move |_| {
			let Some(v) = numbers_dialog(
				&frame,
				"Frequency to color (experimental)",
				&[("Sound frequency in hertz", 1, 1_000_000, 440)],
			) else {
				return;
			};
			let hz = v[0] as f64;
			let Some((c, nm)) = frequency_to_color(hz) else {
				result.set_value("That frequency could not be folded into the visible band.");
				return;
			};
			let mut state = shared.borrow_mut();
			let provenance = Provenance::Generated {
				scheme: format!("frequency {hz:.0} hertz (experimental)"),
			};
			let text = format!(
				"Experimental: {hz:.0} hertz folded up by octaves lands at {nm:.0} nanometres: {}. Added to the palette. Octaves of sound are mapped to octaves of light by convention, not by physics.",
				describe_in(c, state.color_format)
			);
			add_colors(&win, &mut state, &[c], "frequency color", &provenance);
			drop(state);
			result.set_value(&text);
		});
	}
	{
		let dialog = dialog.clone();
		close.on_click(move |_| dialog.end_modal(ID_CANCEL));
	}
	dialog.show_modal();
}

fn template_rows() -> Vec<(usize, String)> {
	templates::TEMPLATE_NAMES
		.iter()
		.enumerate()
		.map(|(i, name)| {
			let count = templates::template(name)
				.map(|d| d.layers.iter().map(|l| l.shapes.len()).sum::<usize>())
				.unwrap_or(0);
			(i, format!("{name}: {count} elements on an 800 by 600 canvas"))
		})
		.collect()
}

/// The Templates dialog (File, New from Template): the built-in
/// starter designs, listed with their element counts. Loading one
/// replaces the current drawing — the template is an ordinary
/// design from then on: it edits, saves and exports like anything
/// drawn by hand. The list is labelled "Templates".
fn templates_dialog(win: &Win, shared: &Shared) {
	let frame = win.frame;
	let dialog = Dialog::builder(&frame, "New from Template").build();
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let list = QuietList::new(&dialog, "Templates");
	sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let load = Button::builder(&dialog).with_label("Load Template").build();
	let close = Button::builder(&dialog).with_label("Close").build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	buttons.add(&load, 0, SizerFlag::All, 5);
	buttons.add(&close, 0, SizerFlag::All, 5);
	sizer.add_sizer(&buttons, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	let list = Rc::new(list);
	list.update_labelled(template_rows(), Select::Start);
	{
		let (list, shared, dialog) = (list.clone(), shared.clone(), dialog.clone());
		load.on_click(move |_| {
			let Some(i) = list.selected() else { return };
			let name = templates::TEMPLATE_NAMES[i];
			if let Some(design) = templates::template(name) {
				let mut state = shared.borrow_mut();
				state.drawing = design;
				state.layer = 0;
				// A new project starts from scratch (Elshara):
				// no pinned sliders, default target and page jump.
				state.pinned_sliders = Vec::new();
				state.slider_target = AdjustTarget::BaseColor;
				state.page_jump_percent = DEFAULT_PAGE_JUMP_PERCENT;
				reset_adjustment(&mut state);
				drop(state);
				frame.set_status_text(
					&format!("Started a new design from the {name} template. It is a normal design now: edit it, then Save Design to keep it."),
					0,
				);
				dialog.end_modal(ID_OK);
			}
		});
	}
	{
		let dialog = dialog.clone();
		close.on_click(move |_| dialog.end_modal(ID_CANCEL));
	}
	dialog.show_modal();
}

/// The Symbols dialog (Design, Add Symbol): the starter symbol
/// stamps. Adding one asks where it goes and how big, in numbers or
/// plain words like every other element, then stamps its strokes
/// onto the current layer under one shared caption. The list is
/// labelled "Symbols"; the dialog stays open so several symbols can
/// be added in one visit.
fn symbols_dialog(win: &Win, shared: &Shared) {
	let frame = win.frame;
	let dialog = Dialog::builder(&frame, "Add Symbol").build();
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let list = QuietList::new(&dialog, "Symbols");
	sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let add = Button::builder(&dialog).with_label("Add Symbol\u{2026}").build();
	let close = Button::builder(&dialog).with_label("Close").build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	buttons.add(&add, 0, SizerFlag::All, 5);
	buttons.add(&close, 0, SizerFlag::All, 5);
	sizer.add_sizer(&buttons, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	let rows: Vec<(usize, String)> = symbols::SYMBOL_NAMES
		.iter()
		.enumerate()
		.map(|(i, name)| (i, format!("{name} symbol")))
		.collect();
	let list = Rc::new(list);
	list.update_labelled(rows, Select::Start);
	{
		let (list, shared) = (list.clone(), shared.clone());
		add.on_click(move |_| {
			let Some(i) = list.selected() else { return };
			let name = symbols::SYMBOL_NAMES[i];
			let (canvas, color) = {
				let state = shared.borrow();
				((state.drawing.width, state.drawing.height), state.brush.color)
			};
			// The same three questions place_dialog asks, in the
			// fields form dialogs can use: numbers or plain words.
			let Some(v) = fields_dialog(
				&frame,
				&format!("Add {name} symbol"),
				&[
					(
						"Position: numbers like 120, 80, or words like top left, center, a bit right",
						"60, 60",
					),
					("Size: numbers like 200 by 100, or words like a bit wider, a bit taller", "100 by 100"),
					("Color: a name like sky blue, or a hex code", &color.to_hex()),
				],
			) else {
				return;
			};
			let (Some((x, y)), Some((w, h)), Some(color)) = (
				parse_position(&v[0], canvas.0, canvas.1, Some((60, 60))),
				parse_size(&v[1], Some((100, 100))),
				parse_color(&v[2]),
			) else {
				frame.set_status_text(
					"One of those fields did not parse. Position and size take numbers or plain words; color takes a name like sky blue or a hex code.",
					0,
				);
				return;
			};
			let Some(paths) = symbols::symbol_paths(name, x, y, w, h) else { return };
			let mut state = shared.borrow_mut();
			let layer = state.layer;
			let mut added = 0;
			for path in paths {
				let mut shape = Shape::stroke(path, 3, color);
				shape.opacity = state.current_opacity;
				shape.caption = Some(format!("{name} symbol"));
				if state.drawing.add_shape(layer, shape).is_some() {
					added += 1;
				}
			}
			drop(state);
			if added > 0 {
				frame.set_status_text(
					&format!(
						"Added {name} symbol ({} stroke{}).",
						added,
						if added == 1 { "" } else { "s" }
					),
					0,
				);
			}
		});
	}
	{
		let dialog = dialog.clone();
		close.on_click(move |_| dialog.end_modal(ID_CANCEL));
	}
	dialog.show_modal();
}

/// Extract a palette from an image the user picks: they choose how
/// many colors (default 8, up to 64). With `set_background` the palette
/// is replaced and the largest color becomes the design background
/// (File, Open Image); without it the colors are simply appended,
/// stamped with the image they came from (File, Extract Colors from
/// Image) — any image file can be mined, not just a background.
fn extract_image_dialog(win: &Win, shared: &Shared, set_background: bool) {
	let frame = win.frame;
	let Some(path) = file_dialog(
		&frame,
		"Choose an image to extract colors from",
		"Image files (*.png;*.jpg;*.jpeg;*.webp)|*.png;*.jpg;*.jpeg;*.webp",
		false,
	) else {
		return;
	};
	// The count's range comes from its slider definition — one
	// source for the dialog, the Sliders tab, and Project Options.
	let count_def = slider_by_id("extract-count").expect("the extract-count slider exists");
	let now = shared.borrow().extract_count;
	let Some(v) = numbers_dialog(
		&frame,
		"How many colors to extract",
		&[("Number of colors", count_def.min, count_def.max, now)],
	) else {
		return;
	};
	shared.borrow_mut().extract_count = v[0];
	match image::open(&path) {
		Ok(img) => {
			let entries = extract_palette(&img, v[0] as usize);
			let name = Path::new(&path)
				.file_name()
				.map(|n| n.to_string_lossy().into_owned())
				.unwrap_or_else(|| path.clone());
			let extracted: Vec<PaletteColor> = entries
				.iter()
				.map(|e| PaletteColor::extracted(e.color, e.coverage, &name))
				.collect();
			let mut state = shared.borrow_mut();
			if set_background {
				if let Some(first) = entries.first() {
					state.drawing.set_background(first.color);
				}
				state.palette = extracted;
			} else {
				state.palette.extend(extracted);
			}
			refresh_lists(win, &state);
			frame.set_status_text(
				&format!(
					"Extracted {} colors from {name}.{}",
					entries.len(),
					if set_background {
						" The largest is now the design background."
					} else {
						" They were added to the palette."
					}
				),
				0,
			);
			if set_background {
				if let Some(first) = entries.first() {
					let color = first.color;
					select_color(win, &state, color);
				}
			}
		}
		Err(e) => {
			frame.set_status_text(&format!("Could not open that image: {e}"), 0);
		}
	}
}

fn dispatch(win: &Rc<Win>, shared: &Shared, id: Id) {
	let frame = win.frame;
	match id {
		ID_EXIT => frame.close(true),
		ID_DICTIONARY => dictionary_dialog(win),
		ID_HELP_FILE => help_dialog(win),
		ID_SOUND_CUES => {
			let mut state = shared.borrow_mut();
			state.sound_cues = !state.sound_cues;
			frame.set_status_text(
				if state.sound_cues {
					"Sound cues on: drawing changes play a pitch and position tone."
				} else {
					"Sound cues off. Spoken readouts are unchanged."
				},
				0,
			);
		}
		ID_ABOUT => {
			MessageDialog::builder(
				&frame,
				"Blind Designer\n\nAn accessible multispectral color creator and \
				 layered drawing tool. Colors live in OKLCH and are never mixed \
				 or blended: every scheme, shade, tint and tone is a pure \
				 parameter change you can predict. Palettes extract free from \
				 any image. Every drawing change is logged, step by step. \
				 Built with wxDragon.",
				"About Blind Designer",
			)
			.build()
			.show_modal();
		}
		ID_OPEN_IMAGE => extract_image_dialog(win, shared, true),
		ID_EXTRACT_IMAGE => extract_image_dialog(win, shared, false),
		ID_PROJECT_OPTIONS => project_options_dialog(win, shared),
		ID_ANNOUNCE_SLIDERS => {
			let mut state = shared.borrow_mut();
			state.announce_sliders = !state.announce_sliders;
			if let Some(bar) = frame.get_menu_bar() {
				bar.check_item(ID_ANNOUNCE_SLIDERS, state.announce_sliders);
			}
			frame.set_status_text(
				if state.announce_sliders {
					"Slider announcements on: moving a slider says what changed and that everything else stayed the same."
				} else {
					"Slider announcements off: moving a slider says only its name and value."
				},
				0,
			);
		}
		ID_SAVE_DESIGN => {
			if let Some(path) = file_dialog(
				&frame,
				"Save design",
				"Blind Designer design (*.bdd)|*.bdd",
				true,
			) {
				let state = shared.borrow();
				let project = Project {
					drawing: state.drawing.clone(),
					palette: state.palette.clone(),
					pinned_sliders: state.pinned_sliders.clone(),
					slider_target: state.slider_target,
					page_jump_percent: state.page_jump_percent,
				};
				match std::fs::write(&path, project.to_text()) {
					Ok(()) => frame.set_status_text(&format!("Saved design to {path}"), 0),
					Err(e) => frame.set_status_text(&format!("Save failed: {e}"), 0),
				}
			}
		}
		ID_OPEN_DESIGN => {
			if let Some(path) = file_dialog(
				&frame,
				"Open design",
				"Blind Designer design (*.bdd)|*.bdd",
				false,
			) {
				match std::fs::read_to_string(&path) {
					Ok(text) => match Project::from_text(&text) {
						Ok(project) => {
							let mut state = shared.borrow_mut();
							state.drawing = project.drawing;
							state.palette = project.palette;
							state.pinned_sliders = project.pinned_sliders;
							state.slider_target = project.slider_target;
							state.page_jump_percent = project.page_jump_percent;
							reset_adjustment(&mut state);
							state.layer = 0;
							let layers = state.drawing.layers.len();
							let shapes: usize =
								state.drawing.layers.iter().map(|l| l.shapes.len()).sum();
							let colors = state.palette.len();
							drop(state);
							build_sliders_tab(win, shared);
							let state = shared.borrow();
							refresh_lists(win, &state);
							frame.set_status_text(
								&format!(
									"Opened design from {path}: {layers} layers, {shapes} shapes, {colors} palette colors. It replaces the previous design."
								),
								0,
							);
						}
						Err(e) => frame.set_status_text(
							&format!("That file could not be opened as a design: {e}"),
							0,
						),
					},
					Err(e) => {
						frame.set_status_text(&format!("Could not read that file: {e}"), 0)
					}
				}
			}
		}
		ID_FORMAT_HEX | ID_FORMAT_RGB | ID_FORMAT_HSL | ID_FORMAT_HSV | ID_FORMAT_CMYK
		| ID_FORMAT_XYZ | ID_FORMAT_XYY | ID_FORMAT_LAB | ID_FORMAT_LCH | ID_FORMAT_LUV
		| ID_FORMAT_HUNTER | ID_FORMAT_DECIMAL | ID_FORMAT_WEBSAFE | ID_FORMAT_BINARY => {
			let Some(&(_, format)) = FORMAT_IDS.iter().find(|(fid, _)| *fid == id) else {
				return;
			};
			let current;
			{
				let mut state = shared.borrow_mut();
				state.color_format = format;
				if let Some(bar) = frame.get_menu_bar() {
					for (fid, f) in FORMAT_IDS {
						bar.check_item(*fid, *f == format);
					}
				}
				refresh_lists(win, &state);
				current = selected_color(win, &state);
			}
			// The channel group's sliders are the new format's
			// channels: rebuild the Sliders tab to match.
			build_sliders_tab(win, shared);
			match current {
				Some(c) => {
					let state = shared.borrow();
					select_color(win, &state, c);
					frame.set_status_text(
						&format!(
							"Color values now shown as {}, for example {}.",
							format.name(),
							c.format(format)
						),
						0,
					);
				}
				None => frame.set_status_text(
					&format!("Color values now shown as {}.", format.name()),
					0,
				),
			}
		}
		ID_EXPORT_PNG => {
			if let Some(path) = file_dialog(&frame, "Export design as PNG", "PNG image (*.png)|*.png", true) {
				let state = shared.borrow();
				match state.drawing.flatten().render().save(&path) {
					Ok(()) => frame.set_status_text(&format!("Exported PNG to {path}"), 0),
					Err(e) => frame.set_status_text(&format!("Export failed: {e}"), 0),
				}
			}
		}
		ID_EXPORT_CAPTIONS => {
			if let Some(path) =
				file_dialog(&frame, "Export caption sheet", "Text file (*.txt)|*.txt", true)
			{
				let state = shared.borrow();
				match std::fs::write(&path, state.drawing.flatten().caption_sheet()) {
					Ok(()) => frame.set_status_text(&format!("Exported captions to {path}"), 0),
					Err(e) => frame.set_status_text(&format!("Export failed: {e}"), 0),
				}
			}
		}
		ID_EXPORT_CSS => {
			if let Some(path) =
				file_dialog(&frame, "Export palette as CSS", "CSS file (*.css)|*.css", true)
			{
				let state = shared.borrow();
				let colors: Vec<Srgb> = if state.palette.is_empty() {
					vec![state.drawing.background]
				} else {
					state.palette.iter().map(|pc| pc.color).collect()
				};
				match std::fs::write(&path, css_for_colors(&colors, "color")) {
					Ok(()) => frame.set_status_text(&format!("Exported CSS to {path}"), 0),
					Err(e) => frame.set_status_text(&format!("Export failed: {e}"), 0),
				}
			}
		}
		ID_EXPORT_SUMMARY => {
			if let Some(path) = file_dialog(
				&frame,
				"Export recreation file (step-by-step summary)",
				"Text file (*.txt)|*.txt",
				true,
			) {
				let state = shared.borrow();
				match std::fs::write(&path, state.drawing.summary()) {
					Ok(()) => frame.set_status_text(&format!("Exported recreation file to {path}"), 0),
					Err(e) => frame.set_status_text(&format!("Export failed: {e}"), 0),
				}
			}
		}
		ID_EXPORT_HTML => {
			if let Some(path) = file_dialog(
				&frame,
				"Export design as an HTML page",
				"HTML page (*.html)|*.html",
				true,
			) {
				let state = shared.borrow();
				match std::fs::write(&path, state.drawing.flatten().to_html("Blind Designer design")) {
					Ok(()) => frame.set_status_text(&format!("Exported HTML page to {path}"), 0),
					Err(e) => frame.set_status_text(&format!("Export failed: {e}"), 0),
				}
			}
		}
		ID_EXPORT_GIF => {
			if let Some(path) = file_dialog(
				&frame,
				"Export animation as a GIF",
				"GIF image (*.gif)|*.gif",
				true,
			) {
				let state = shared.borrow();
				let count =
					if state.drawing.frames.is_empty() { 1 } else { state.drawing.frames.len() };
				match state.drawing.to_gif() {
					Ok(bytes) => match std::fs::write(&path, &bytes) {
						Ok(()) => frame.set_status_text(
							&format!("Exported animated GIF ({count} frames) to {path}"),
							0,
						),
						Err(e) => {
							frame.set_status_text(&format!("Export failed: {e}"), 0)
						}
					},
					Err(e) => frame.set_status_text(&format!("Export failed: {e}"), 0),
				}
			}
		}
		ID_EXPORT_ANIM_HTML => {
			if let Some(path) = file_dialog(
				&frame,
				"Export animation as an HTML page",
				"HTML page (*.html)|*.html",
				true,
			) {
				let state = shared.borrow();
				match std::fs::write(
					&path,
					state.drawing.to_html_animated("Blind Designer animation"),
				) {
					Ok(()) => {
						frame.set_status_text(&format!("Exported animated HTML page to {path}"), 0)
					}
					Err(e) => frame.set_status_text(&format!("Export failed: {e}"), 0),
				}
			}
		}
		ID_EXPORT_ANIM_DESC => {
			if let Some(path) = file_dialog(
				&frame,
				"Export animation description (frame by frame)",
				"Text file (*.txt)|*.txt",
				true,
			) {
				let state = shared.borrow();
				match std::fs::write(&path, state.drawing.animation_description()) {
					Ok(()) => frame.set_status_text(
						&format!("Exported animation description to {path}"),
						0,
					),
					Err(e) => frame.set_status_text(&format!("Export failed: {e}"), 0),
				}
			}
		}
		ID_IMPORT_CSS => {
			if let Some(path) = file_dialog(
				&frame,
				"Import colors from a CSS file",
				"CSS file (*.css)|*.css",
				false,
			) {
				match std::fs::read_to_string(&path) {
					Ok(text) => {
						let found = parse_css_colors(&text);
						if found.is_empty() {
							frame.set_status_text("No colors found in that CSS file.", 0);
						} else {
							let mut state = shared.borrow_mut();
							for c in &found {
								state.palette.push(PaletteColor::imported(c.color, &c.source));
							}
							refresh_lists(win, &state);
							let format = state.color_format;
							let notes: Vec<String> = found
								.iter()
								.take(6)
								.map(|c| format!("{} ({})", c.color.format(format), c.source))
								.collect();
							frame.set_status_text(
								&format!(
									"Imported {} colors from CSS: {}",
									found.len(),
									notes.join("; ")
								),
								0,
							);
						}
					}
					Err(e) => {
						frame.set_status_text(&format!("Could not read that CSS file: {e}"), 0)
					}
				}
			}
		}
		ID_IMPORT_HTML => {
			if let Some(path) = file_dialog(
				&frame,
				"Import an HTML page as a design",
				"HTML file (*.html;*.htm)|*.html;*.htm",
				false,
			) {
				match std::fs::read_to_string(&path) {
					Ok(text) => {
						let imp = import_html(&text);
						let mut drawing = Drawing::new(
							imp.design.width,
							imp.design.height,
							imp.design.background,
						);
						for shape in imp.design.shapes {
							drawing.add_shape(0, shape);
						}
						let shapes = drawing.layers[0].shapes.len();
						let mut state = shared.borrow_mut();
						state.drawing = drawing;
						state.layer = 0;
						state.palette = imp
							.colors
							.iter()
							.map(|c| PaletteColor::imported(c.color, &c.source))
							.collect();
						if state.palette.is_empty() {
							let bg = state.drawing.background;
							state.palette.push(PaletteColor::imported(
								bg,
								"the imported design's background",
							));
						}
						let colors = state.palette.len();
						refresh_lists(win, &state);
						frame.set_status_text(
							&format!(
								"Imported {path}: {shapes} shapes on the canvas, {colors} colors in the palette. The imported design replaces the previous one."
							),
							0,
						);
					}
					Err(e) => {
						frame.set_status_text(&format!("Could not read that HTML file: {e}"), 0)
					}
				}
			}
		}
		ID_ADD_COLOR => {
			// Adding a color by hand happens in the hex field on
			// the main window (the palette context menu's way in):
			// take the user there and say so.
			win.hex_field.set_focus();
			frame.set_status_text(
				"Type a hex value in the Hex value field, then press Apply Hex to add the color.",
				0,
			);
		}
		ID_LOAD_STARTER => {
			let mut state = shared.borrow_mut();
			load_starter_colors(win, &mut state);
		}
		ID_STATS => {
			let report = {
				let state = shared.borrow();
				ProjectStats::gather(&state.drawing, state.palette.len()).report()
			};
			report_dialog(win, "Project statistics", "What is in this project", &report);
		}
		ID_SCHEME_SETTINGS => scheme_settings_dialog(win, shared),
		ID_COLOR_TOOLS => color_tools_dialog(win, shared),
		ID_TEXT_READABILITY => {
			let report = {
				let state = shared.borrow();
				text_readability(&state.drawing.flatten())
			};
			report_dialog(
				win,
				"Text Readability",
				"How readable each text element is",
				&report,
			);
		}
		ID_COMPLEMENTARY | ID_SPLIT | ID_ANALOGOUS | ID_TRIADIC | ID_TETRADIC | ID_SHADES
		| ID_TINTS | ID_TONES => {
			let mut state = shared.borrow_mut();
			let Some(base) = selected_color(win, &state) else {
				frame.set_status_text("Select a palette color first.", 0);
				return;
			};
			let o = base.to_oklch();
			// Hue schemes rotate by their offsets, on the screen
			// wheel (OKLCH hue) or the RYB paint wheel per Scheme
			// Settings; shades/tints/tones are lightness/chroma
			// ladders and never touch the wheel.
			let (colors, what): (Vec<Srgb>, String) = match id {
				ID_SHADES => (o.shades(4).iter().map(|c| c.to_srgb()).collect(), "shades".to_string()),
				ID_TINTS => (o.tints(4).iter().map(|c| c.to_srgb()).collect(), "tints".to_string()),
				ID_TONES => (o.tones(4).iter().map(|c| c.to_srgb()).collect(), "tones".to_string()),
				_ => {
					let spread = state.scheme_spread;
					let (offsets, label): (Vec<f32>, String) = match id {
						ID_COMPLEMENTARY => (vec![180.0], "complementary".to_string()),
						ID_SPLIT => (vec![150.0, -150.0], "split complementary".to_string()),
						ID_ANALOGOUS => (
							vec![-spread, spread],
							format!("analogous ({spread:.0} degrees)"),
						),
						ID_TRIADIC => (vec![120.0, 240.0], "triadic".to_string()),
						_ => (vec![60.0, 180.0, 240.0], "tetradic".to_string()),
					};
					let colors = if state.wheel_ryb {
						offsets.iter().map(|d| rotate_hue_ryb(base, *d)).collect()
					} else {
						offsets.iter().map(|d| o.rotate(*d).to_srgb()).collect()
					};
					let label = if state.wheel_ryb {
						format!("{label} (paint wheel)")
					} else {
						label
					};
					(colors, label)
				}
			};
			let provenance =
				Provenance::Generated { scheme: format!("{what} of {}", base.to_hex()) };
			add_colors(win, &mut state, &colors, &what, &provenance);
		}
		ID_REMOVE_COLOR => {
			let mut state = shared.borrow_mut();
			if let Some(i) = win.palette_list.borrow().selected() {
				if i < state.palette.len() {
					let gone = state.palette.remove(i).color;
					let label = describe_in(gone, state.color_format);
					refresh_lists(win, &state);
					frame.set_status_text(&format!("Removed {label}"), 0);
				}
			} else {
				frame.set_status_text("Select a palette color first.", 0);
			}
		}
		ID_SET_BACKGROUND => {
			let mut state = shared.borrow_mut();
			if let Some(c) = selected_color(win, &state) {
				state.drawing.set_background(c);
				refresh_lists(win, &state);
				announce(win, &state);
				select_color(win, &state, c);
			} else {
				frame.set_status_text("Select a palette color first.", 0);
			}
		}
		ID_CANVAS_SIZE => {
			let (w, h) = {
				let state = shared.borrow();
				(state.drawing.width, state.drawing.height)
			};
			if let Some(v) = numbers_dialog(
				&frame,
				"Canvas size",
				&[
					("Width in pixels", 16, 8000, w as i32),
					("Height in pixels", 16, 8000, h as i32),
				],
			) {
				let mut state = shared.borrow_mut();
				state.drawing.set_canvas(v[0] as u32, v[1] as u32);
				announce(win, &state);
				refresh_lists(win, &state);
			}
		}
		ID_ADD_RECT | ID_ADD_ELLIPSE => {
			let kind = if id == ID_ADD_RECT {
				ShapeKind::Rectangle
			} else {
				ShapeKind::Ellipse
			};
			let setup = {
				let state = shared.borrow();
				selected_color(win, &state)
					.map(|c| (c, state.color_format, (state.drawing.width, state.drawing.height)))
			};
			let Some((fill, format, canvas)) = setup else {
				frame.set_status_text("Select a palette color to fill the shape first.", 0);
				return;
			};
			if let Some((x, y, w, h, color)) = place_dialog(
				win,
				&format!("Add {} filled with {}", kind.name(), describe_in(fill, format)),
				(0, 0),
				(200, 120),
				fill,
				canvas,
			) {
				let mut state = shared.borrow_mut();
				let layer = state.layer;
				let mut shape = Shape::new(kind, x, y, w, h, color);
				shape.opacity = state.current_opacity;
				let index = state.drawing.add_shape(layer, shape);
				refresh_lists(win, &state);
				if let Some(index) = index {
					speak_added(win, &state, layer, index);
				}
			}
		}
		ID_ADD_TEXT => {
			let Some(phrase) = text_dialog(&frame, "Add text", "Text to place in the design", "")
			else {
				return;
			};
			if phrase.trim().is_empty() {
				frame.set_status_text("The text was empty; nothing was added.", 0);
				return;
			}
			let Some(font) = text_dialog(&frame, "Add text", "Font name", "Arial") else {
				return;
			};
			let Some(v) = numbers_dialog(
				&frame,
				"Add text",
				&[("Font size in points", 6, 200, 24)],
			) else {
				return;
			};
			let size = v[0] as u32;
			let setup = {
				let state = shared.borrow();
				(
					selected_color(win, &state).unwrap_or(Srgb::BLACK),
					(state.drawing.width, state.drawing.height),
				)
			};
			// The prefilled box hugs the phrase at the chosen size.
			let est = (
				(phrase.chars().count() as u32 * size * 3 / 5).max(10),
				(size * 7 / 5).max(10),
			);
			if let Some((x, y, w, h, color)) = place_dialog(
				win,
				&format!("Add text \"{}\"", phrase.trim()),
				(0, 0),
				est,
				setup.0,
				setup.1,
			) {
				let mut state = shared.borrow_mut();
				let layer = state.layer;
				let mut shape = Shape::text(phrase.trim(), font.trim(), size, x, y, w, h, color);
				shape.opacity = state.current_opacity;
				let index = state.drawing.add_shape(layer, shape);
				refresh_lists(win, &state);
				if let Some(index) = index {
					speak_added(win, &state, layer, index);
				}
			}
		}
		ID_ADD_IMAGE => {
			let Some(path) = file_dialog(
				&frame,
				"Choose an image to place in the design",
				"Image files (*.png;*.jpg;*.jpeg;*.webp)|*.png;*.jpg;*.jpeg;*.webp",
				false,
			) else {
				return;
			};
			let setup = {
				let state = shared.borrow();
				(
					selected_color(win, &state).unwrap_or(Srgb::new(128, 128, 128)),
					(state.drawing.width, state.drawing.height),
				)
			};
			if let Some((x, y, w, h, color)) = place_dialog(
				win,
				"Add image (the color is its placeholder, shown when the file is missing)",
				(0, 0),
				(200, 150),
				setup.0,
				setup.1,
			) {
				let mut state = shared.borrow_mut();
				let layer = state.layer;
				let mut shape = Shape::image(&path, x, y, w, h, color);
				shape.opacity = state.current_opacity;
				let index = state.drawing.add_shape(layer, shape);
				refresh_lists(win, &state);
				if let Some(index) = index {
					speak_added(win, &state, layer, index);
				}
			}
		}
		ID_ADD_STROKE => {
			// The stroke is painted with the current brush: its width,
			// color and pattern prefill the dialog, and whatever is
			// used becomes the current brush for the next stroke.
			let brush = shared.borrow().brush.clone();
			let Some(text) = text_dialog(
				&frame,
				"Add stroke",
				"Points as x,y pairs, separated by spaces (for example: 10,20 100,20 100,80)",
				"10,10 200,10 200,100",
			) else {
				return;
			};
			let Some(points) = parse_points(&text).filter(|p| p.len() >= 2) else {
				frame.set_status_text(
					"Points need to be x,y pairs, at least two of them, whole numbers.",
					0,
				);
				return;
			};
			let Some(v) = fields_dialog(
				&frame,
				"Stroke brush",
				&[
					("Brush width in pixels", &brush.width.to_string()),
					("Color: a name like sky blue, or a hex code", &brush.color.to_hex()),
					("Pattern: solid, dashed or dotted", &brush.pattern),
				],
			) else {
				return;
			};
			// The brush width's range comes from its slider
			// definition — the dialog, the Sliders tab and
			// Project Options all read the same numbers.
			let width_def = slider_by_id("brush-width").expect("the brush-width slider exists");
			let Some(width) = v[0]
				.trim()
				.parse::<i32>()
				.ok()
				.filter(|w| (width_def.min..=width_def.max).contains(w))
				.map(|w| w as u32)
			else {
				frame.set_status_text(
					&format!(
						"The brush width needs to be a whole number from {} to {}.",
						width_def.min, width_def.max
					),
					0,
				);
				return;
			};
			let Some(color) = parse_color(&v[1]) else {
				frame.set_status_text(
					"I could not understand that color. Use a name like sky blue, or a hex code like 87CEEB.",
					0,
				);
				return;
			};
			let pattern = v[2].trim().to_lowercase();
			if !PATTERNS.contains(&pattern.as_str()) {
				frame.set_status_text("The pattern needs to be solid, dashed or dotted.", 0);
				return;
			}
			let mut state = shared.borrow_mut();
			let brush_name = state.brush.name.clone();
			state.brush = Brush::new(&brush_name, width, color, &pattern);
			let layer = state.layer;
			let mut shape = Shape::stroke(points, width, color);
			shape.pattern = pattern;
			shape.opacity = state.current_opacity;
			let index = state.drawing.add_shape(layer, shape);
			refresh_lists(win, &state);
			if let Some(index) = index {
				speak_added(win, &state, layer, index);
			}
		}
		ID_MOVE_SHAPE | ID_ROTATE_SHAPE => {
			let Some(flat) = win.elements_list.borrow().selected() else {
				frame.set_status_text("Select a shape first.", 0);
				return;
			};
			let at = {
				let state = shared.borrow();
				locate(&state, flat)
			};
			let Some((li, si)) = at else { return };
			if id == ID_MOVE_SHAPE {
				let (cur, canvas) = {
					let state = shared.borrow();
					let s = &state.drawing.layers[li].shapes[si];
					((s.x, s.y), (state.drawing.width, state.drawing.height))
				};
				if let Some(v) = fields_dialog(
					&frame,
					"Move shape",
					&[(
						"New position: numbers like 120, 80, or words like top left, center, a bit right",
						&format!("{}, {}", cur.0, cur.1),
					)],
				) {
					match parse_position(&v[0], canvas.0, canvas.1, Some(cur)) {
						Some((nx, ny)) => {
							let mut state = shared.borrow_mut();
							state.drawing.shift_shape(li, si, nx - cur.0, ny - cur.1);
							refresh_lists(win, &state);
							announce(win, &state);
						}
						None => frame.set_status_text(
							"I could not understand that position. Use numbers like 120, 80, or words like top left, center, a bit right.",
							0,
						),
					}
				}
			} else {
				let now = {
					let state = shared.borrow();
					state.drawing.layers[li].shapes[si].angle
				};
				if let Some(v) = numbers_dialog(
					&frame,
					"Rotate shape",
					&[("Angle in degrees, clockwise", -360, 360, now as i32)],
				) {
					let mut state = shared.borrow_mut();
					state.drawing.rotate_shape(li, si, v[0] as f64);
					refresh_lists(win, &state);
					announce(win, &state);
				}
			}
		}
		ID_RESIZE_SHAPE => {
			let Some(flat) = win.elements_list.borrow().selected() else {
				frame.set_status_text("Select a shape first.", 0);
				return;
			};
			let at = {
				let state = shared.borrow();
				locate(&state, flat)
			};
			let Some((li, si)) = at else { return };
			let (w, h, stroke) = {
				let state = shared.borrow();
				let s = &state.drawing.layers[li].shapes[si];
				(s.w, s.h, s.kind == ShapeKind::Stroke)
			};
			if stroke {
				frame.set_status_text("A stroke's size is its points; move it or add a new one.", 0);
				return;
			}
			if let Some(v) = fields_dialog(
				&frame,
				"Resize shape",
				&[(
					"New size: numbers like 200 by 100, or words like a bit wider, a bit taller",
					&format!("{w} by {h}"),
				)],
			) {
				match parse_size(&v[0], Some((w, h))) {
					Some((nw, nh)) => {
						let mut state = shared.borrow_mut();
						state.drawing.resize_shape(li, si, nw, nh);
						refresh_lists(win, &state);
						announce(win, &state);
					}
					None => frame.set_status_text(
						"I could not understand that size. Use numbers like 200 by 100, or words like a bit wider.",
						0,
					),
				}
			}
		}
		ID_RECOLOR_SHAPE => {
			let mut state = shared.borrow_mut();
			let (Some(flat), Some(c)) = (win.elements_list.borrow().selected(), selected_color(win, &state))
			else {
				frame.set_status_text("Select a shape and a palette color first.", 0);
				return;
			};
			if let Some((li, si)) = locate(&state, flat) {
				state.drawing.recolor_shape(li, si, c);
				refresh_lists(win, &state);
				announce(win, &state);
			}
		}
		ID_LAYERS => {
			layers_dialog(win, shared);
			let state = shared.borrow();
			refresh_lists(win, &state);
		}
		ID_ANIMATION => {
			animation_dialog(win, shared);
			// Loading a frame replaces the canvas shapes: any
			// captured adjustment base is stale now.
			reset_adjustment(&mut shared.borrow_mut());
			let state = shared.borrow();
			refresh_lists(win, &state);
		}
		ID_ELEMENTS => {
			elements_dialog(win, shared);
			// The dialog can recolor shapes: end the adjustment
			// run so its captured base can't revert that.
			reset_adjustment(&mut shared.borrow_mut());
			let state = shared.borrow();
			refresh_lists(win, &state);
		}
		ID_TEMPLATES => {
			templates_dialog(win, shared);
			// A template is a new project: its slider settings were
			// reset as it loaded, so the tab rebuilds to match.
			build_sliders_tab(win, shared);
			let state = shared.borrow();
			refresh_lists(win, &state);
		}
		ID_ADD_SYMBOL => {
			symbols_dialog(win, shared);
			let state = shared.borrow();
			refresh_lists(win, &state);
		}
		ID_PATTERN_TEMPLATES => {
			pattern_dialog(win, shared, None);
			// A pattern is a new project too: rebuild the tab the
			// way a template load does.
			build_sliders_tab(win, shared);
			let state = shared.borrow();
			refresh_lists(win, &state);
		}
		ID_INSPECT => inspect_selected(win, shared),
		ID_INSPECT_LAYER => {
			if let Some(i) = win.layers_list.selected() {
				let state = shared.borrow();
				if let Some(text) = inspect_layer(&state.drawing, i) {
					report_dialog(win, "Inspector: layer", "Layer inspector", &text);
				}
			}
		}
		ID_INSPECT_COLOR => {
			if let Some(i) = win.palette_list.borrow().selected() {
				let state = shared.borrow();
				if let Some(text) =
					inspect_color(&state.drawing, &state.palette, i)
				{
					report_dialog(win, "Inspector: color", "Color inspector", &text);
				}
			}
		}
		ID_LOCK_TOGGLE => {
			if let Some((li, si)) = selected_element_locate(win, shared) {
				let now = {
					let mut state = shared.borrow_mut();
					let next = !state.drawing.layers[li].shapes[si].locked;
					state.drawing.set_shape_locked(li, si, next);
					next
				};
				let state = shared.borrow();
				refresh_lists(win, &state);
				frame.set_status_text(
					if now {
						"Element locked: bulk changes will skip it; direct edits still work."
					} else {
						"Element unlocked: bulk changes reach it again."
					},
					0,
				);
			}
		}
		ID_SEARCH_INSERT => search_dialog(win, shared),
		ID_LISTS_SIDE_BY_SIDE => {
			let on = {
				let mut state = shared.borrow_mut();
				state.lists_side_by_side = !state.lists_side_by_side;
				state.lists_side_by_side
			};
			if let Some(bar) = frame.get_menu_bar() {
				bar.check_item(ID_LISTS_SIDE_BY_SIDE, on);
			}
			rebuild_list_pages(win, shared);
			frame.set_status_text(
				if on {
					"Palette and Elements now sit side by side on the Lists page."
				} else {
					"Palette and Elements are back on their own tabs."
				},
				0,
			);
		}
		ID_VIEWABILITY => viewability_dialog(win, shared),
		ID_SAVE_SCENE => {
			if let Some(path) = file_dialog(
				&frame,
				"Save scene (a design file other designs can insert)",
				"Blind Designer scene (*.bdd)|*.bdd",
				true,
			) {
				let state = shared.borrow();
				// A scene file is a design file: the drawing on
				// its own, with an empty palette — what Insert
				// Scene reads back.
				let scene = Project {
					drawing: state.drawing.clone(),
					palette: Vec::new(),
					pinned_sliders: Vec::new(),
					slider_target: AdjustTarget::BaseColor,
					page_jump_percent: DEFAULT_PAGE_JUMP_PERCENT,
				};
				match std::fs::write(&path, scene.to_text()) {
					Ok(()) => frame.set_status_text(&format!("Saved scene to {path}"), 0),
					Err(e) => frame.set_status_text(&format!("Save failed: {e}"), 0),
				}
			}
		}
		ID_INSERT_SCENE => {
			if let Some(path) = file_dialog(
				&frame,
				"Insert a scene file",
				"Blind Designer scene (*.bdd)|*.bdd",
				false,
			) {
				match std::fs::read_to_string(&path) {
					Ok(text) => match Project::from_text(&text) {
						Ok(scene) => {
							let name = Path::new(&path)
								.file_stem()
								.map(|n| n.to_string_lossy().into_owned())
								.unwrap_or_else(|| "Scene".to_string());
							let mut state = shared.borrow_mut();
							let layer = state.layer;
							let count =
								state.drawing.insert_scene(layer, &scene.drawing, &name);
							let layer_name = state
								.drawing
								.layers
								.get(layer)
								.map(|l| l.name.clone())
								.unwrap_or_default();
							drop(state);
							if count > 0 {
								// The inserted elements are new
								// shapes: an adjustment run's
								// captured bases no longer match.
								reset_adjustment(&mut shared.borrow_mut());
								let state = shared.borrow();
								refresh_lists(win, &state);
								announce(win, &state);
								frame.set_status_text(
									&format!(
										"Inserted scene \"{name}\": {count} elements onto layer \"{layer_name}\", captioned as one group."
									),
									0,
								);
							} else {
								frame.set_status_text(
									"That scene has no painted elements to insert.",
									0,
								);
							}
						}
						Err(e) => frame.set_status_text(
							&format!("That file could not be opened as a scene: {e}"),
							0,
						),
					},
					Err(e) => {
						frame.set_status_text(&format!("Could not read that file: {e}"), 0)
					}
				}
			}
		}
		ID_DESCRIBE_AREA => {
			if let Some(area) = text_dialog(
				&frame,
				"Describe area",
				"Area: a name like top left, center, bottom right — or a box as across, down, width, height",
				"center",
			) {
				let state = shared.borrow();
				match state.drawing.describe_area_text(&area) {
					Some(text) => report_dialog(
						win,
						"Describe area",
						&format!("Elements in the {area} area"),
						&text,
					),
					None => frame.set_status_text(
						"I could not understand that area. Name a part of the canvas, like top left or center, or give a box as four numbers.",
						0,
					),
				}
			}
		}
		ID_EXPORT_TACTILE_SVG => {
			if let Some(path) = file_dialog(
				&frame,
				"Export raised-line SVG (one line style per layer)",
				"SVG drawing (*.svg)|*.svg",
				true,
			) {
				let state = shared.borrow();
				match std::fs::write(&path, state.drawing.to_tactile_svg()) {
					Ok(()) => frame
						.set_status_text(&format!("Exported raised-line SVG to {path}"), 0),
					Err(e) => frame.set_status_text(&format!("Export failed: {e}"), 0),
				}
			}
		}
		ID_EXPORT_TACTILE_SHEET => {
			if let Some(path) = file_dialog(
				&frame,
				"Export tactile sheet (element by element, in reading order)",
				"Text file (*.txt)|*.txt",
				true,
			) {
				let state = shared.borrow();
				match std::fs::write(&path, state.drawing.tactile_sheet()) {
					Ok(()) => frame
						.set_status_text(&format!("Exported tactile sheet to {path}"), 0),
					Err(e) => frame.set_status_text(&format!("Export failed: {e}"), 0),
				}
			}
		}
		ID_EXPORT_BRAILLE => {
			if let Some(path) = file_dialog(
				&frame,
				"Export braille labels (Grade 1)",
				"Braille labels (*.brf)|*.brf",
				true,
			) {
				let state = shared.borrow();
				match std::fs::write(&path, state.drawing.braille_labels()) {
					Ok(()) => frame
						.set_status_text(&format!("Exported braille labels to {path}"), 0),
					Err(e) => frame.set_status_text(&format!("Export failed: {e}"), 0),
				}
			}
		}
		ID_COMPARE_IMAGES => {
			let images = "Image files (*.png;*.jpg;*.jpeg;*.webp)|*.png;*.jpg;*.jpeg;*.webp";
			if let Some(first_path) = file_dialog(&frame, "Choose the first image", images, false) {
				if let Some(second_path) =
					file_dialog(&frame, "Choose the second image", images, false)
				{
					let name = |p: &str| {
						Path::new(p)
							.file_name()
							.map(|n| n.to_string_lossy().into_owned())
							.unwrap_or_else(|| p.to_string())
					};
					match (image::open(&first_path), image::open(&second_path)) {
						(Ok(a), Ok(b)) => report_dialog(
							win,
							"Image comparison",
							"How the two images compare",
							&compare_images(&a, &b).report(&name(&first_path), &name(&second_path)),
						),
						_ => frame.set_status_text(
							"One of those files could not be read as an image.",
							0,
						),
					}
				}
			}
		}
		ID_TACTILE_GRID => {
			let images = "Image files (*.png;*.jpg;*.jpeg;*.webp)|*.png;*.jpg;*.jpeg;*.webp";
			if let Some(path) = file_dialog(
				&frame,
				"Choose an image to feel as a tactile grid",
				images,
				false,
			) {
				// The ranges come from the tactile slider
				// definitions; the values are the project's,
				// shared with the Sliders tab.
				let col_def =
					slider_by_id("tactile-columns").expect("the tactile-columns slider exists");
				let row_def =
					slider_by_id("tactile-rows").expect("the tactile-rows slider exists");
				let (cols_now, rows_now) = {
					let state = shared.borrow();
					(state.tactile_cols, state.tactile_rows)
				};
				if let Some(v) = numbers_dialog(
					&frame,
					"Tactile grid size",
					&[
						("Columns", col_def.min, col_def.max, cols_now),
						("Rows", row_def.min, row_def.max, rows_now),
					],
				) {
					{
						let mut state = shared.borrow_mut();
						state.tactile_cols = v[0];
						state.tactile_rows = v[1];
					}
					match image::open(&path) {
						Ok(img) => {
							let grid = tactile_grid(&img, v[0] as u32, v[1] as u32);
							let name = Path::new(&path)
								.file_name()
								.map(|n| n.to_string_lossy().into_owned())
								.unwrap_or_else(|| path.clone());
							report_dialog(
								win,
								"Tactile grid",
								"The picture as a grid of shades",
								&grid.sheet(&name),
							);
						}
						Err(e) => frame
							.set_status_text(&format!("Could not open that image: {e}"), 0),
					}
				}
			}
		}
		ID_SAVE_BRUSH => {
			let current = shared.borrow().brush.clone();
			let default = if current.name.is_empty() {
				"My brush".to_string()
			} else {
				current.name.clone()
			};
			if let Some(name) = text_dialog(
				&frame,
				"Save current brush",
				&format!(
					"Name for this brush ({} pixel, {}, {} pattern)",
					current.width,
					chroma_core::color_words(current.color),
					current.pattern
				),
				&default,
			) {
				if name.trim().is_empty() {
					frame.set_status_text("The brush was not saved: it needs a name.", 0);
					return;
				}
				let mut state = shared.borrow_mut();
				let brush = Brush::new(name.trim(), current.width, current.color, &current.pattern);
				let spoken = brush.spoken();
				state.brush = brush.clone();
				state.brushes.save(brush);
				persist_brushes(win, &state);
				frame.set_status_text(&format!("Saved brush: {spoken}."), 0);
			}
		}
		ID_BRUSHES => brushes_dialog(win, shared),
		ID_REMOVE_SHAPE => {
			let mut state = shared.borrow_mut();
			if let Some(flat) = win.elements_list.borrow().selected() {
				if let Some((li, si)) = locate(&state, flat) {
					state.drawing.remove_shape(li, si);
					refresh_lists(win, &state);
					announce(win, &state);
				}
			} else {
				frame.set_status_text("Select a shape first.", 0);
			}
		}
		_ => {}
	}
}

fn build_menus(frame: &Frame) {
	// Menus are grouped by the job being done (Elshara's feedback):
	// File carries Import and Export submenus instead of two dozen
	// flat items; the old Palette menu is now Color, with Schemes
	// and Variations submenus; Design groups its Add and Arrange
	// commands. Every item's help string says in plain language what
	// it does — the status line shows it while the item is rested on.
	let import = Menu::builder().build();
	import.append(ID_OPEN_IMAGE, "&Open Image…\tCtrl+O", "Open an image: its colors replace the palette and its largest color becomes the background.", ItemKind::Normal);
	import.append(ID_EXTRACT_IMAGE, "&Extract Colors from Image…", "Add an image's colors to the palette, leaving the design untouched.", ItemKind::Normal);
	import.append(ID_IMPORT_CSS, "&CSS Colors…", "Add every color found in a CSS file to the palette.", ItemKind::Normal);
	import.append(ID_IMPORT_HTML, "&HTML Design…", "Turn a simple HTML page (inline styles) into a design.", ItemKind::Normal);
	import.append(ID_COMPARE_IMAGES, "Compare I&mages…", "Compare two images: their sizes, shared colors, which is brighter, which is blurrier.", ItemKind::Normal);
	import.append(ID_TACTILE_GRID, "Describe Image as &Tactile Grid…", "Describe any image as a grid of shades, in braille patterns and in words.", ItemKind::Normal);

	let export = Menu::builder().build();
	export.append(ID_EXPORT_PNG, "&PNG…\tCtrl+E", "Save the design as a PNG picture.", ItemKind::Normal);
	export.append(ID_EXPORT_CAPTIONS, "&Captions…", "Save a text sheet describing every shape in words.", ItemKind::Normal);
	export.append(ID_EXPORT_CSS, "C&SS…", "Save the palette as CSS variables, ready for a web page.", ItemKind::Normal);
	export.append(ID_EXPORT_SUMMARY, "&Summary (recreation file)…", "Save the step-by-step log that recreates this design exactly.", ItemKind::Normal);
	export.append(ID_EXPORT_HTML, "HT&ML Page…", "Save the design as a standalone web page whose shapes carry their captions as accessible names.", ItemKind::Normal);
	export.append(ID_EXPORT_GIF, "Animated &GIF…", "Save the animation as a looping GIF; with no frames, the design as one still picture.", ItemKind::Normal);
	export.append(ID_EXPORT_ANIM_HTML, "Animated HTML…", "Save the animation as a web page that steps through the frames.", ItemKind::Normal);
	export.append(ID_EXPORT_ANIM_DESC, "Animation &Description…", "Save the film as words: every frame, its duration, and what changed.", ItemKind::Normal);
	export.append(ID_EXPORT_TACTILE_SVG, "&Raised-Line SVG…", "Save the design as outlines for swell paper or an embosser, one line style per layer.", ItemKind::Normal);
	export.append(ID_EXPORT_TACTILE_SHEET, "Tactile &Sheet…", "Save the design as a text walk, element by element in reading order, with braille labels.", ItemKind::Normal);
	export.append(ID_EXPORT_BRAILLE, "&Braille Labels…", "Save the elements' captions as Grade 1 braille labels.", ItemKind::Normal);

	let file = Menu::builder().build();
	file.append(ID_OPEN_DESIGN, "Open &Design…", "Open a saved design file, replacing the current design.", ItemKind::Normal);
	file.append(ID_SAVE_DESIGN, "&Save Design…", "Save the whole design — canvas, layers, palette and log — to a file.", ItemKind::Normal);
	file.append(ID_SAVE_SCENE, "Save Sc&ene…", "Save the current design as a scene file, ready to insert into other designs as a group.", ItemKind::Normal);
	file.append(ID_INSERT_SCENE, "Insert &Scene…", "Insert a saved scene's elements onto the current layer, captioned as one group under the scene's name.", ItemKind::Normal);
	file.append(ID_TEMPLATES, "New from &Template…", "Start a new design from a built-in layout: birthday card, house, face, chart, portrait or landscape.", ItemKind::Normal);
	file.append(ID_PATTERN_TEMPLATES, "New from &Pattern…", "Start a new design as a two-color pattern — checkerboard, stripes or dots — woven from two palette colors you pick.", ItemKind::Normal);
	file.append_separator();
	file.append_submenu(import, "&Import", "Bring images, colors and designs in.");
	file.append_submenu(export, "&Export", "Write the design out: pictures, pages, text and touch.");
	file.append_separator();
	file.append(ID_EXIT, "E&xit", "Close Blind Designer.", ItemKind::Normal);

	let schemes = Menu::builder().build();
	schemes.append(ID_COMPLEMENTARY, "&Complementary (hue + 180)", "Add the color opposite the selected one on the hue circle.", ItemKind::Normal);
	schemes.append(ID_SPLIT, "&Split Complementary (hue ± 150)", "Add the two colors 150 degrees either side of the selected one.", ItemKind::Normal);
	schemes.append(ID_ANALOGOUS, "&Analogous (hue ± spread)", "Add the two neighboring colors, the Scheme Settings spread either side (30 degrees to start).", ItemKind::Normal);
	schemes.append(ID_TRIADIC, "&Triadic (hue ± 120)", "Add the two colors a third of the circle away, either side.", ItemKind::Normal);
	schemes.append(ID_TETRADIC, "T&etradic", "Add the three colors that complete a rectangle on the hue circle.", ItemKind::Normal);
	schemes.append_separator();
	schemes.append(ID_SCHEME_SETTINGS, "Scheme &Settings…", "Choose the harmony wheel (screen or paint) and the analogous spread in degrees.", ItemKind::Normal);

	let variations = Menu::builder().build();
	variations.append(ID_SHADES, "Add 4 S&hades (lightness down)", "Add four darker versions of the selected color.", ItemKind::Normal);
	variations.append(ID_TINTS, "Add 4 &Tints (lightness up)", "Add four lighter versions of the selected color.", ItemKind::Normal);
	variations.append(ID_TONES, "Add 4 Tone&s (chroma down)", "Add four softer, less saturated versions of the selected color.", ItemKind::Normal);

	let color = Menu::builder().build();
	color.append_submenu(schemes, "&Schemes", "New colors computed from the selected color.");
	color.append_submenu(variations, "&Shades, Tints and Tones", "Darker, lighter and softer versions of the selected color.");
	color.append_separator();
	color.append(ID_COLOR_TOOLS, "Color &Tools…", "Blend, subtract and gradient between two palette colors, random colors, contrast checks, dominant wavelength, and sound frequency to color.", ItemKind::Normal);
	color.append(ID_TEXT_READABILITY, "Te&xt Readability…", "How readable every text element in the design is against its background.", ItemKind::Normal);
	color.append_separator();
	color.append(ID_LOAD_STARTER, "&Load Starter Colors", "Add the starter colors to the palette. A new palette starts empty; these join only when you ask.", ItemKind::Normal);
	color.append(ID_REMOVE_COLOR, "&Remove Selected Color", "Take the selected color out of the palette.", ItemKind::Normal);
	color.append_separator();
	color.append(ID_SET_BACKGROUND, "Set &Background to Selected Color", "Make the selected palette color the design's background.", ItemKind::Normal);

	let add = Menu::builder().build();
	add.append(ID_ADD_RECT, "&Rectangle…", "Place a rectangle, by numbers or in plain words.", ItemKind::Normal);
	add.append(ID_ADD_ELLIPSE, "&Ellipse…", "Place an ellipse; equal width and height make a circle.", ItemKind::Normal);
	add.append(ID_ADD_STROKE, "Stro&ke…", "Draw a line through points you list, with the current brush.", ItemKind::Normal);
	add.append(ID_ADD_TEXT, "&Text…", "Place words in the design, in a named font at a chosen size.", ItemKind::Normal);
	add.append(ID_ADD_IMAGE, "&Image…", "Place an image file in the design, scaled into a box you give it.", ItemKind::Normal);
	add.append(ID_ADD_SYMBOL, "S&ymbol…", "Stamp a ready-made symbol: star, heart, arrow, check, cross, sun or moon.", ItemKind::Normal);

	let arrange = Menu::builder().build();
	arrange.append(ID_MOVE_SHAPE, "&Move…", "Move the selected element, by numbers or in plain words.", ItemKind::Normal);
	arrange.append(ID_ROTATE_SHAPE, "R&otate…", "Turn the selected element by an angle in degrees.", ItemKind::Normal);
	arrange.append(ID_RESIZE_SHAPE, "Resi&ze…", "Change the selected element's size, by numbers or in plain words.", ItemKind::Normal);
	arrange.append(ID_RECOLOR_SHAPE, "Set to Palette &Color", "Give the selected element the selected palette color.", ItemKind::Normal);
	arrange.append(ID_REMOVE_SHAPE, "Remove Sha&pe", "Delete the selected element from the design.", ItemKind::Normal);

	let design = Menu::builder().build();
	design.append(ID_CANVAS_SIZE, "Canvas &Size…", "Change the canvas width and height in pixels.", ItemKind::Normal);
	design.append_separator();
	design.append_submenu(add, "&Add", "Add a new element to the design.");
	design.append_submenu(arrange, "A&rrange Selected Shape", "Move, turn, resize, recolor or remove the selected element.");
	design.append_separator();
	design.append(ID_ELEMENTS, "E&lements…", "Every element in plain language, with rename, describe, move, resize, recolor and delete.", ItemKind::Normal);
	design.append(ID_INSPECT, "In&spect Selected…", "The inspector: what the selected element is, where it is used, what can change it and what happens if it is left alone. Enter on the Elements list does the same.", ItemKind::Normal);
	design.append(ID_LOCK_TOGGLE, "&Lock/Unlock Selected", "Lock the selected element so bulk changes skip it, or unlock it so they reach it again. Direct edits work either way.", ItemKind::Normal);
	design.append(ID_SEARCH_INSERT, "&Search and Insert…\tCtrl+F", "Search palette colors, starter colors, elements, symbols and templates by name, and insert what you find at the cursor.", ItemKind::Normal);
	design.append(ID_LAYERS, "&Layers…", "Add, rename, reorder, show or hide layers, and choose where new shapes land.", ItemKind::Normal);
	design.append(ID_ANIMATION, "Anim&ation…", "Build a film from frames of the canvas, each with its own duration.", ItemKind::Normal);
	design.append(ID_DESCRIBE_AREA, "&Describe Area…", "Hear only the elements inside one part of the canvas.", ItemKind::Normal);
	design.append(ID_VIEWABILITY, "&Viewability…", "Report how the design reads from a viewing angle: what is viewable, what is hidden behind something else, what is turned away, per depth.", ItemKind::Normal);
	design.append(ID_STATS, "&Project Statistics…", "How many layers, elements, palette colors and animation frames this project has.", ItemKind::Normal);
	design.append(ID_PROJECT_OPTIONS, "Project &Options…", "Choose which sliders this project pins to its Sliders tab, what the color adjustments act on, how far Page Up and Page Down jump, and load the starter colors.", ItemKind::Normal);
	design.append_separator();
	design.append(ID_SAVE_BRUSH, "Save Current Br&ush…", "Keep the current brush — width, color, pattern — in the brush library under a name.", ItemKind::Normal);
	design.append(ID_BRUSHES, "Brush Li&brary…", "The saved brushes; apply one to make it current for new strokes.", ItemKind::Normal);

	let view = Menu::builder().build();
	view.append(ID_FORMAT_HEX, "Color Values as &Hex", "Show and speak color values as hex codes, like #3A7CA5.", ItemKind::Check);
	view.append(ID_FORMAT_RGB, "Color Values as &RGB", "Show and speak color values as red, green and blue numbers, 0 to 255.", ItemKind::Check);
	view.append(ID_FORMAT_HSL, "Color Values as HS&L", "Show and speak color values as hue, saturation and lightness.", ItemKind::Check);
	view.append(ID_FORMAT_HSV, "Color Values as HS&V", "Show and speak color values as hue, saturation and value.", ItemKind::Check);
	view.append(ID_FORMAT_CMYK, "Color Values as CM&YK", "Show and speak color values as cyan, magenta, yellow and black ink percents.", ItemKind::Check);
	view.append(ID_FORMAT_XYZ, "Color Values as CIE &XYZ", "Show and speak color values as CIE XYZ tristimulus numbers.", ItemKind::Check);
	view.append(ID_FORMAT_XYY, "Color Values as x&yY", "Show and speak color values as chromaticity x and y plus luminance Y.", ItemKind::Check);
	view.append(ID_FORMAT_LAB, "Color Values as CIE L&ab", "Show and speak color values as CIE lightness and the a and b opponent axes.", ItemKind::Check);
	view.append(ID_FORMAT_LCH, "Color Values as CIE LC&H", "Show and speak color values as CIE lightness, chroma and hue.", ItemKind::Check);
	view.append(ID_FORMAT_LUV, "Color Values as CIE LU&V", "Show and speak color values as CIE lightness and the u and v axes.", ItemKind::Check);
	view.append(ID_FORMAT_HUNTER, "Color Values as &Hunter Lab", "Show and speak color values in the 1948 Hunter Lab system (different numbers from CIE Lab).", ItemKind::Check);
	view.append(ID_FORMAT_DECIMAL, "Color Values as &Decimal", "Show and speak color values as one decimal number, 0 to 16777215.", ItemKind::Check);
	view.append(ID_FORMAT_WEBSAFE, "Color Values as &Web Safe", "Show and speak each color snapped to the nearest web-safe color.", ItemKind::Check);
	view.append(ID_FORMAT_BINARY, "Color Values as &Binary", "Show and speak color values as a 24-bit binary string.", ItemKind::Check);
	view.append_separator();
	view.append(ID_ANNOUNCE_SLIDERS, "&Announce Slider Changes", "Moving a slider says what changed and that everything else stayed the same. Off: only the slider's name and new value are said.", ItemKind::Check);
	view.append(ID_LISTS_SIDE_BY_SIDE, "Lists &Side by Side", "Show the Palette and Elements lists side by side on one Lists page instead of on separate tabs. This session only.", ItemKind::Check);

	let help = Menu::builder().build();
	help.append(ID_DICTIONARY, "&Dictionary…", "Plain-language definitions of the color and design terms the app uses.", ItemKind::Normal);
	help.append(ID_HELP_FILE, "&Help and Worked Examples…", "How the app works, what every setting changes, and step-by-step examples.", ItemKind::Normal);
	help.append(ID_SOUND_CUES, "&Sound Cues", "Play a tone for each drawing change: pitch says height, stereo says left or right, length says size.", ItemKind::Check);
	help.append_separator();
	help.append(ID_ABOUT, "&About Blind Designer", "What Blind Designer is.", ItemKind::Normal);

	let bar = MenuBar::builder()
		.append(file, "&File")
		.append(color, "&Color")
		.append(design, "&Design")
		.append(view, "&View")
		.append(help, "&Help")
		.build();
	bar.check_item(ID_SOUND_CUES, true); // sound cues start on
	bar.check_item(ID_FORMAT_HEX, true); // color values start as hex
	bar.check_item(ID_ANNOUNCE_SLIDERS, true); // slider announcements start on
	frame.set_menu_bar(bar);
}

fn main() {
	let _ = wxdragon::main(|_| {
		let frame = Frame::builder()
			.with_title("Blind Designer")
			.with_size(Size::new(800, 700))
			.build();
		let panel = Panel::builder(&frame).build();

		// The project's lists live in a tabbed notebook in the main
		// window (Elshara's feedback): Palette, Elements and Layers
		// are tabs of one control, reachable by Tab and Ctrl+Tab,
		// instead of living only inside dialogs. Each page is a
		// panel holding the list's label first — so screen readers
		// find it — then the list itself, named the same as its page.
		// The lists show only what the project actually contains:
		// a new project starts with an empty palette, no elements
		// and its one real layer.
		let notebook = Notebook::builder(&panel).build();
		let (palette_page, palette_list) = list_panel(&notebook, "Palette");
		let (elements_page, elements_list) = list_panel(&notebook, "Elements");
		let (layers_page, layers_list) = list_panel(&notebook, "Layers");
		notebook.add_page(&palette_page, "Palette", true, None);
		notebook.add_page(&elements_page, "Elements", false, None);
		notebook.add_page(&layers_page, "Layers", false, None);

		let readout_label = StaticText::builder(&panel).with_label("Selected color").build();
		let readout = StaticText::builder(&panel)
			.with_label("No color selected yet. Open an image, or add a color by hex.")
			.build();

		let hex_label = StaticText::builder(&panel).with_label("Hex value").build();
		let hex_field = TextCtrl::builder(&panel).build();
		hex_field.set_value("#3A7CA5");
		let apply_hex = Button::builder(&panel).with_label("Apply Hex").build();

		let l_label = StaticText::builder(&panel).with_label("Lightness (0 to 1)").build();
		let l_spin = SpinCtrlDouble::builder(&panel).with_range(0.0, 1.0).build();
		let c_label = StaticText::builder(&panel).with_label("Chroma (0 to 0.4)").build();
		let c_spin = SpinCtrlDouble::builder(&panel).with_range(0.0, 0.4).build();
		let h_label = StaticText::builder(&panel).with_label("Hue (0 to 360 degrees)").build();
		let h_spin = SpinCtrlDouble::builder(&panel).with_range(0.0, 360.0).build();
		l_spin.set_increment(0.01);
		l_spin.set_digits(2);
		c_spin.set_increment(0.01);
		c_spin.set_digits(2);
		h_spin.set_increment(1.0);
		h_spin.set_digits(0);
		let apply_lch = Button::builder(&panel).with_label("Apply L C H").build();

		let stats_label = StaticText::builder(&panel).with_label("Project statistics").build();
		let stats_text = StaticText::builder(&panel).build();

		let sizer = BoxSizer::builder(Orientation::Vertical).build();
		sizer.add(&notebook, 1, SizerFlag::Expand | SizerFlag::All, 5);
		sizer.add(&readout_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
		sizer.add(&readout, 0, SizerFlag::Expand | SizerFlag::All, 5);
		let editor = BoxSizer::builder(Orientation::Horizontal).build();
		editor.add(&hex_label, 0, SizerFlag::All, 5);
		editor.add(&hex_field, 1, SizerFlag::All, 5);
		editor.add(&apply_hex, 0, SizerFlag::All, 5);
		sizer.add_sizer(&editor, 0, SizerFlag::Expand, 0);
		let spins = BoxSizer::builder(Orientation::Horizontal).build();
		spins.add(&l_label, 0, SizerFlag::All, 5);
		spins.add(&l_spin, 1, SizerFlag::All, 5);
		spins.add(&c_label, 0, SizerFlag::All, 5);
		spins.add(&c_spin, 1, SizerFlag::All, 5);
		spins.add(&h_label, 0, SizerFlag::All, 5);
		spins.add(&h_spin, 1, SizerFlag::All, 5);
		spins.add(&apply_lch, 0, SizerFlag::All, 5);
		sizer.add_sizer(&spins, 0, SizerFlag::Expand, 0);
		sizer.add(&stats_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
		sizer.add(&stats_text, 0, SizerFlag::Expand | SizerFlag::All, 5);
		panel.set_sizer(sizer, true);

		// wxID_ANY (-1): "assign an ID for me". This call used to pass -2
		// under a comment claiming it was wxID_ANY — in wx, -2 is
		// wxID_SEPARATOR, "invalid for normal items", and it trips the
		// wincmn.cpp CreateBase() "invalid id value" assert at startup
		// (issue #4).
		frame.create_status_bar(1, 0, ID_ANY as i32, "Status");
		build_menus(&frame);

		let start = Srgb::new(58, 124, 165);
		let shared: Shared = Rc::new(RefCell::new(State {
			// A fresh project starts with an EMPTY palette: the list
			// shows only colors the project actually gains. The
			// starter colors arrive through Color, Load Starter
			// Colors, when the user asks for them.
			palette: Vec::new(),
			drawing: Drawing::new(800, 600, Srgb::WHITE),
			layer: 0,
			sound_cues: true,
			color_format: ColorFormat::Hex,
			scheme_spread: 30.0,
			wheel_ryb: false,
			brush: Brush::new("", 3, start, "solid"),
			brushes: load_brushes(),
			// A new project starts from scratch (Elshara): no
			// pinned sliders, adjustments on the current color,
			// the default page jump.
			pinned_sliders: Vec::new(),
			slider_target: AdjustTarget::BaseColor,
			page_jump_percent: DEFAULT_PAGE_JUMP_PERCENT,
			announce_sliders: true,
			current_opacity: 100,
			extract_count: 8,
			tactile_cols: 24,
			tactile_rows: 12,
			sound_volume: 100,
			adjust: ColorAdjust::default(),
			adjust_base: None,
			scale_base: None,
			channel_run: None,
			lists_side_by_side: false,
		}));
		let win = Rc::new(Win {
			frame,
			notebook,
			palette_list: RefCell::new(palette_list),
			elements_list: RefCell::new(elements_list),
			layers_list,
			list_pages: RefCell::new(vec![palette_page, elements_page]),
			stats_text,
			readout,
			hex_field,
			l_spin,
			c_spin,
			h_spin,
			slider_page: RefCell::new(None),
			slider_rows: RefCell::new(Vec::new()),
			channel_rows: RefCell::new(Vec::new()),
			channel_note: RefCell::new(None),
			slider_target_choice: RefCell::new(None),
			slider_refreshing: std::cell::Cell::new(false),
		});

		build_sliders_tab(&win, &shared);
		{
			let state = shared.borrow();
			refresh_lists(&win, &state);
		}

		wire_palette_list(&win, &shared);
		wire_elements_list(&win, &shared);
		{
			let (w, s) = (win.clone(), shared.clone());
			win.layers_list.on_key_down(move |data| {
				let WindowEventData::Keyboard(kb) = data else { return false };
				if kb.get_key_code() != Some(WXK_RETURN) {
					return false;
				}
				if let Some(i) = w.layers_list.selected() {
					let state = s.borrow();
					if let Some(text) = inspect_layer(&state.drawing, i) {
						report_dialog(&w, "Inspector: layer", "Layer inspector", &text);
					}
				}
				true
			});
		}
		{
			let w = win.clone();
			win.layers_list.on_context_menu(move || show_layers_context(&w));
		}
		{
			let (w, s) = (win.clone(), shared.clone());
			win.layers_list.on_selection_changed(move || {
				// Selecting a layer in the main window makes it the
				// current layer: where new shapes land. (Renaming,
				// removing and hiding stay in Design, Layers.)
				let Some(i) = w.layers_list.selected() else { return };
				let name = {
					let Ok(mut state) = s.try_borrow_mut() else { return };
					if i >= state.drawing.layers.len() || state.layer == i {
						return;
					}
					state.layer = i;
					// A different current layer is a different
					// adjustment target: fresh run next move.
					reset_adjustment(&mut state);
					state.drawing.layers[i].name.clone()
				};
				let state = s.borrow();
				refresh_lists(&w, &state);
				drop(state);
				w.frame.set_status_text(
					&format!("New shapes now land on layer \"{name}\"."),
					0,
				);
			});
		}
		{
			let (w, s) = (win.clone(), shared.clone());
			win.frame.on_menu_selected(move |e| dispatch(&w, &s, e.get_id()));
		}
		{
			let (w, s) = (win.clone(), shared.clone());
			apply_hex.on_click(move |_| {
				let text = w.hex_field.get_value();
				match Srgb::from_hex(text.trim()) {
					Some(c) => {
						let mut state = s.borrow_mut();
						if let Some(i) = w.palette_list.borrow().selected() {
							if i < state.palette.len() {
								state.palette[i] = PaletteColor::typed(c);
								refresh_lists(&w, &state);
								select_color(&w, &state, c);
								return;
							}
						}
						state.palette.push(PaletteColor::typed(c));
						refresh_lists(&w, &state);
						select_color(&w, &state, c);
					}
					None => {
						w.frame
							.set_status_text("That is not a hex color. Use RRGGBB, like 3A7CA5.", 0);
					}
				}
			});
		}
		{
			let (w, s) = (win.clone(), shared.clone());
			apply_lch.on_click(move |_| {
				let o = Oklch::new(
					w.l_spin.get_value() as f32,
					w.c_spin.get_value() as f32,
					w.h_spin.get_value() as f32,
				);
				let c = o.to_srgb();
				let mut state = s.borrow_mut();
				if let Some(i) = w.palette_list.borrow().selected() {
					if i < state.palette.len() {
						state.palette[i] = PaletteColor::typed(c);
						refresh_lists(&w, &state);
						select_color(&w, &state, c);
						return;
					}
				}
				state.palette.push(PaletteColor::typed(c));
				refresh_lists(&w, &state);
				select_color(&w, &state, c);
			});
		}

		frame.show(true);
		frame.centre();
	});
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Issue #4 regression guard: every custom menu ID is a positive
	/// value inside the valid wx range and appears exactly once. (The
	/// const block beside MENU_IDS enforces the same at compile time;
	/// this test keeps the rule visible in the suite's output.)
	#[test]
	fn menu_ids_are_unique_and_in_wx_range() {
		for (i, id) in MENU_IDS.iter().enumerate() {
			assert!(*id > 0 && *id <= ID_HIGHEST, "menu ID {id} out of the valid wx range");
			assert!(!MENU_IDS[i + 1..].contains(id), "duplicate menu ID {id}");
		}
	}

	/// The status bar's ID is wxID_ANY — the only negative ID wx allows
	/// a window to carry. Passing -2 (wxID_SEPARATOR) here was the
	/// startup assert in issue #4.
	#[test]
	fn id_any_is_the_assign_for_me_value() {
		assert_eq!(ID_ANY, -1);
	}
}
