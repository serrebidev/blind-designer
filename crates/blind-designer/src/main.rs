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
use std::path::{Path, PathBuf};
use std::rc::Rc;

use chroma_core::brushes::{Brush, BrushLibrary, PATTERNS};
use chroma_core::design::{css_for_colors, Heading, Shape, ShapeKind, TextAlign, TextStyle};
use chroma_core::dictionary;
use chroma_core::drawing::{DepthRole, Drawing, Event, TreatmentScope};
use chroma_core::formats::{
	blend, channel_values, channels as format_channels, color_from_channels, contrast_verdict,
	frequency_to_color, gradient, random_color, rotate_hue_ryb, subtract, text_readability,
	ChannelSet,
};
use chroma_core::goals::{
	default_slider_ids, goal_report, recognizability_report, seed_drawing, GoalKind, ProjectGoal,
};
use chroma_core::inspect::{inspect_color, inspect_layer, inspect_shape};
use chroma_core::palette::{gray_ramp, starter_palette, PaletteColor, Provenance};
use chroma_core::plain::{parse_color, parse_position, parse_size};
use chroma_core::project::{InteractionMode, Project, ProjectStats, DEFAULT_PAGE_JUMP_PERCENT};
use chroma_core::recognize::{recognize, Recognized};
use chroma_core::search::{search_catalog, CatalogHit};
use chroma_core::sliders::{
	change_announcement, page_jump, slider_by_id, AdjustTarget, ColorAdjust, SliderDef,
	SliderTarget, SLIDERS,
};
use chroma_core::sound::{cue_at, cue_for_shape, tone_wav_at, Cue};
use chroma_core::symbols;
use chroma_core::tactile::{compare_images, tactile_grid};
use chroma_core::templates;
use chroma_core::textstudio::parse_feed_titles;
use chroma_core::variables;
use chroma_core::viewability::DepthBand;
use chroma_core::web::{import_html, parse_css_colors};
use chroma_core::{describe_in, extract_palette, ColorFormat, Oklch, Srgb};
use patois::t;
use quiet_list::{QuietList, Select};
use wxdragon::prelude::*;

// The compiled gettext catalogs from po/, baked in by patois-build,
// plus wxWidgets' own stock-dialog translations on the same language.
patois::embed_domain!();
patois::embed_wx_translations!();

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
const ID_NEW_WIZARD: Id = 125;
const ID_EXPORT_WIZARD: Id = 126;
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
const ID_SETUP_WIZARD: Id = 328;
const ID_ELEMENT_WIZARD: Id = 329;
const ID_RECOGNIZABILITY: Id = 330;
const ID_COLOR_TREATMENT: Id = 331;
const ID_REPLACEMENT_VARIABLES: Id = 332;
const ID_DRAW_POINTER: Id = 333;
const ID_EXPORT_PRACTICE: Id = 127;
const ID_EXPORT_PDF: Id = 128;
const ID_EXPORT_PRESENTATION: Id = 129;
const ID_EXPORT_DOCUMENT: Id = 130;
const ID_PRINT_PDF: Id = 131;
const ID_NEW_WINDOW: Id = 132;
const ID_TEXT_STUDIO: Id = 334;
const ID_EMBED_FEED: Id = 335;
const ID_EMBED_PASTE: Id = 336;
const ID_RECOGNIZE_SHAPE: Id = 337;
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
const ID_FOCUS_MODE: Id = 517;
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
	ID_NEW_WIZARD,
	ID_EXPORT_WIZARD,
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
	ID_SETUP_WIZARD,
	ID_ELEMENT_WIZARD,
	ID_RECOGNIZABILITY,
	ID_COLOR_TREATMENT,
	ID_REPLACEMENT_VARIABLES,
	ID_DRAW_POINTER,
	ID_EXPORT_PRACTICE,
	ID_EXPORT_PDF,
	ID_EXPORT_PRESENTATION,
	ID_EXPORT_DOCUMENT,
	ID_PRINT_PDF,
	ID_NEW_WINDOW,
	ID_TEXT_STUDIO,
	ID_EMBED_FEED,
	ID_EMBED_PASTE,
	ID_RECOGNIZE_SHAPE,
	ID_ANNOUNCE_SLIDERS,
	ID_LISTS_SIDE_BY_SIDE,
	ID_FOCUS_MODE,
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
		assert!(
			id > 0 && id <= ID_HIGHEST,
			"menu ID out of the valid wx range"
		);
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
	/// The color-channels group's own format, chosen by its
	/// picker on the Sliders tab: it starts at the View format
	/// and then goes its own way — changing one never rewrites
	/// the other. Session-only, like the View format.
	channel_format: ColorFormat,
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
	/// Whether the Focus window is open (View menu toggle).
	/// Session-only, like the other View toggles.
	focus_mode: bool,
	/// The project's stated goal, when the setup wizard has
	/// given it one. Saved in the design file.
	goal: Option<ProjectGoal>,
	/// Grayscale start: this project is being built in grayscale
	/// first, colour treatment to come per part, layer or depth.
	/// A new project starts this way; a loaded project carries
	/// whatever its file says. Saved in the design file.
	grayscale_start: bool,
	/// The project's interaction mode (Elshara's principle:
	/// one interaction surface at a time — Draw, Slide or
	/// Type — switched in Project Options and applied by
	/// `apply_interaction_mode`). A new project starts in
	/// Slide; a loaded project carries its file's mode, and a
	/// file that does not say means Type. Saved in the file.
	interaction_mode: InteractionMode,
	/// The file this project was last saved to or opened from,
	/// when there is one: the unsaved-changes guard saves back
	/// to it without asking for a name again. Session-only.
	design_path: Option<PathBuf>,
	/// The project's fingerprint as it stood when it was last
	/// saved, opened, or started fresh — the guard's baseline.
	/// Session-only; see `project_fingerprint`.
	saved_fingerprint: (usize, usize, usize, bool, usize),
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
/// channel of the group's chosen color format it drives, the
/// native slider control, and the value readout after it.
struct ChannelRow {
	index: usize,
	slider: Slider,
	readout: StaticText,
}

/// One part of a shape the shape-parts group on the Sliders tab
/// puts on a slider: where it sits (Across, Down), how big it is
/// (Width, Height), how it is turned (Rotation), and how round
/// its corners are (Roundedness — rectangles only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShapePart {
	Across,
	Down,
	Width,
	Height,
	Rotation,
	Roundedness,
}

impl ShapePart {
	const ALL: [ShapePart; 6] = [
		ShapePart::Across,
		ShapePart::Down,
		ShapePart::Width,
		ShapePart::Height,
		ShapePart::Rotation,
		ShapePart::Roundedness,
	];

	/// The spoken name, the row's label and the slider's name.
	fn name(self) -> String {
		match self {
			ShapePart::Across => t("Across"),
			ShapePart::Down => t("Down"),
			ShapePart::Width => t("Width"),
			ShapePart::Height => t("Height"),
			ShapePart::Rotation => t("Rotation"),
			ShapePart::Roundedness => t("Roundedness"),
		}
	}

	/// A value in words: "240 pixels", "90 degrees",
	/// "25 percent" — the readout's text and the announcement's.
	fn words(self, value: i32) -> String {
		match self {
			ShapePart::Rotation => t("%s degrees").replacen("%s", &(value).to_string(), 1),
			ShapePart::Roundedness => t("%s percent").replacen("%s", &(value).to_string(), 1),
			_ => t("%s pixels").replacen("%s", &(value).to_string(), 1),
		}
	}
}

/// One row of the shape-parts group: which part it drives, its
/// range (from the canvas, fixed when the page is built), the
/// native slider control, and the value readout after it.
struct ShapeRow {
	part: ShapePart,
	min: i32,
	max: i32,
	slider: Slider,
	readout: StaticText,
}

struct Win {
	frame: Frame,
	/// The main window's content panel: the mode surfaces are
	/// its children, and a mode switch re-runs its layout.
	panel: Panel,
	notebook: Notebook,
	/// The typed-fields surface (the hex row and the L C H
	/// spins row in one panel, so a mode switch shows or hides
	/// them as a unit): visible only in Type mode.
	typed_panel: Panel,
	/// The Draw surface: a panel holding the line that drawing
	/// tools lead in Draw mode and the button that opens the
	/// pointer drawing dialog. Visible only in Draw mode.
	draw_panel: Panel,
	draw_button: Button,
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
	/// The color-channels group's rows on the Sliders page (the
	/// group is a permanent resident of the page) and the note
	/// under it, which says why the channels cannot be edited
	/// when they cannot (a multi-color target, no element
	/// selected) — or, for the binary format, why there are no
	/// channel sliders at all.
	channel_rows: RefCell<Vec<ChannelRow>>,
	channel_note: RefCell<Option<StaticText>>,
	/// The shape-parts group's rows on the Sliders page: the
	/// selected element's position, size, rotation and
	/// roundedness as sliders, refreshed on every selection
	/// change like the other rows.
	shape_rows: RefCell<Vec<ShapeRow>>,
	/// The one-shot timer that rebuilds the Sliders tab after
	/// the channel-format picker changes: the rebuild destroys
	/// the picker itself, so it cannot run inside the picker's
	/// own event — the tick runs it once the event is done.
	channel_rebuild_timer: Timer<Frame>,
	/// True while code (not the user) is setting slider values, so
	/// the change handler stays out of refreshes.
	slider_refreshing: std::cell::Cell<bool>,
	/// The Focus window (View, Focus Mode) and its two texts,
	/// while it is open. The window is non-modal and shows only
	/// the focus line and the built / not-yet body, refreshed by
	/// `refresh_lists` like everything else on the main window.
	focus_frame: RefCell<Option<Frame>>,
	focus_line: RefCell<Option<StaticText>>,
	focus_body: RefCell<Option<TextCtrl>>,
}

type Shared = Rc<RefCell<State>>;

/// Escape cancels a dialog, the way a standard Cancel
/// button would. These dialogs' buttons are closures, not
/// the standard ids wx looks for, so without this binding
/// Escape does nothing at all (Brandon's cross-app report,
/// 2026-11-06; the same fix as Accessible-IPTV-Client-Rust
/// v0.4.21). `CHAR_HOOK` on the dialog itself fires
/// whatever control holds focus.
fn bind_escape_cancel(dialog: Dialog) {
	dialog.bind_internal(EventType::CHAR_HOOK, move |event| {
		if event.get_key_code() == Some(27) {
			dialog.end_modal(ID_CANCEL);
			event.skip(false);
		} else {
			event.skip(true);
		}
	});
}

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
	t("contrast on the design background %s to 1: AA normal text %s, AA large text %s, AAA normal text %s").replacen("%s", &format!("{:.1}", ratio), 1).replacen("%s", verdict(4.5), 1).replacen("%s", verdict(3.0), 1).replacen("%s", verdict(7.0), 1)
}

fn readout_text(state: &State, color: Srgb) -> String {
	let o = color.to_oklch();
	let comp = o.complementary().to_srgb();
	t("%s. Complementary: %s (%s). %s")
		.replacen(
			"%s",
			&(describe_in(color, state.color_format)).to_string(),
			1,
		)
		.replacen("%s", &(comp.format(state.color_format)).to_string(), 1)
		.replacen(
			"%s",
			&(chroma_core::hue_family(comp.to_oklch().h)).to_string(),
			1,
		)
		.replacen(
			"%s",
			&(wcag(color.contrast(state.drawing.background))).to_string(),
			1,
		)
}

/// The whole project as the core sees it: the state's pieces
/// assembled into the one struct the file format, the goal
/// report and the search catalog all consume.
fn current_project(state: &State) -> Project {
	Project {
		drawing: state.drawing.clone(),
		palette: state.palette.clone(),
		pinned_sliders: state.pinned_sliders.clone(),
		slider_target: state.slider_target,
		page_jump_percent: state.page_jump_percent,
		goal: state.goal.clone(),
		grayscale_start: state.grayscale_start,
		interaction_mode: state.interaction_mode,
	}
}

/// A cheap fingerprint of the project's content, for the
/// unsaved-changes guard: the drawing's event-log length (every
/// logged edit grows it), the palette's size, the total element
/// count, whether a goal is stated, and how many sliders are
/// pinned. It can miss an edit that changes none of these, and
/// can stir on a change that is later undone step for step —
/// cheap and conservative is the point: the guard asks, it
/// never silently blocks.
fn project_fingerprint(state: &State) -> (usize, usize, usize, bool, usize) {
	(
		state.drawing.log.len(),
		state.palette.len(),
		state.drawing.layers.iter().map(|l| l.shapes.len()).sum(),
		state.goal.is_some(),
		state.pinned_sliders.len(),
	)
}

/// Save the design: to `path` when one is given (the file the
/// project already lives in), else by asking for a name with
/// the Save Design dialog. On success the path is remembered
/// and the saved fingerprint refreshed, the status line says
/// where it went, and the result is true. A cancelled dialog
/// or a failed write says so and returns false.
fn save_design(win: &Win, shared: &Shared, path: Option<String>) -> bool {
	let frame = win.frame;
	let path = match path {
		Some(p) => Some(p),
		None => file_dialog(
			&frame,
			&t("Save design"),
			&format!("{}|*.bdd", t("Blind Designer design (*.bdd)")),
			true,
		),
	};
	let Some(path) = path else { return false };
	let mut state = shared.borrow_mut();
	let project = current_project(&state);
	match std::fs::write(&path, project.to_text()) {
		Ok(()) => {
			state.design_path = Some(PathBuf::from(&path));
			let fp = project_fingerprint(&state);
			state.saved_fingerprint = fp;
			drop(state);
			frame.set_status_text(
				&t("Saved design to %s").replacen("%s", &(path).to_string(), 1),
				0,
			);
			true
		}
		Err(e) => {
			frame.set_status_text(&t("Save failed: %s").replacen("%s", &(e).to_string(), 1), 0);
			false
		}
	}
}

/// The unsaved-changes guard, asked before anything that would
/// replace or abandon the current project (opening a design,
/// starting from a template, a pattern or the wizard, exiting):
/// a project whose fingerprint still matches its saved
/// baseline passes silently. Otherwise a dialog asks in plain
/// words — Save (to the project's file, or a name it asks for
/// first), Don't Save, or Cancel — and only Save's success or
/// Don't Save lets the action proceed.
fn confirm_discard(win: &Rc<Win>, shared: &Shared, action: &str) -> bool {
	{
		let state = shared.borrow();
		if project_fingerprint(&state) == state.saved_fingerprint {
			return true;
		}
	}
	let dialog = MessageDialog::builder(
		&win.frame,
		&t("This project has unsaved changes. Save it before you %s? \
			 Choose Save to save it first, Don't Save to %s and lose \
			 the changes, or Cancel to stay with this project.")
		.replacen("%s", action, 1)
		.replacen("%s", action, 1),
		&t("Unsaved changes"),
	)
	.with_style(
		MessageDialogStyle::YesNo | MessageDialogStyle::Cancel | MessageDialogStyle::IconQuestion,
	)
	.build();
	dialog.set_yes_no_labels(&t("Save"), &t("Don't Save"));
	match dialog.show_modal() {
		ID_YES => {
			let path = shared
				.borrow()
				.design_path
				.as_ref()
				.map(|p| p.to_string_lossy().into_owned());
			save_design(win, shared, path)
		}
		ID_NO => true,
		_ => false,
	}
}

/// The main window's statistics line: the counts, and — when
/// the project has a stated goal — the goal's two percentages
/// after them.
fn stats_line(state: &State) -> String {
	let mut line = ProjectStats::gather(&state.drawing, state.palette.len()).one_line();
	if state.goal.is_some() {
		if let Some(report) = goal_report(&current_project(state)) {
			line.push_str(&format!(" {}", report.one_line()));
		}
	}
	line
}

fn refresh_lists(win: &Win, state: &State) {
	win.palette_list
		.borrow()
		.update_labelled(palette_rows(state), Select::Keep);
	win.elements_list
		.borrow()
		.update_labelled(element_rows(state), Select::Keep);
	win.layers_list
		.update_labelled(layer_rows(state), Select::Keep);
	win.stats_text.set_label(&stats_line(state));
	refresh_menu_states(win, state);
	refresh_slider_values(win, state);
	refresh_focus_window(win, state);
}

/// Grey the menu actions that cannot do anything right now, so the
/// menus themselves say what the project is missing: the color
/// schemes, shades/tints/tones, Remove Color and Set Background need
/// a palette color selected; Move/Rotate/Resize/Remove Shape need
/// an element selected; Recolor needs both; the animation
/// description export needs at least one frame. Everything else
/// works on any project, empty or not, and stays enabled.
fn refresh_menu_states(win: &Win, state: &State) {
	let Some(bar) = win.frame.get_menu_bar() else {
		return;
	};
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
		ID_ELEMENT_WIZARD,
	] {
		bar.enable_item(id, has_element);
	}
	bar.enable_item(ID_INSPECT_LAYER, win.layers_list.selected().is_some());
	bar.enable_item(ID_INSPECT_COLOR, has_color);
	// Recognize Shape acts on one selected stroke, nothing else.
	bar.enable_item(
		ID_RECOGNIZE_SHAPE,
		selected_shape(state, win).is_some_and(|(_, _, s)| s.kind == ShapeKind::Stroke),
	);
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
		&t("Selected: %s").replacen(
			"%s",
			&(describe_in(color, state.color_format)).to_string(),
			1,
		),
		0,
	);
}

fn selected_color(win: &Win, state: &State) -> Option<Srgb> {
	win.palette_list
		.borrow()
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
		win.frame
			.set_status_text(&t("The starter colors are already in the palette."), 0);
		return;
	}
	let names: Vec<String> = fresh
		.iter()
		.map(|pc| pc.spoken(state.color_format))
		.collect();
	state.palette.extend(fresh);
	refresh_lists(win, state);
	win.frame.set_status_text(
		&t("Loaded starter colors: %s").replacen("%s", &(names.join("; ")).to_string(), 1),
		0,
	);
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
		SliderTarget::ElementScale => {
			selected_shape(state, win).map(|(li, si, s)| match state.scale_base {
				Some((bl, bs, bw, _)) if bl == li && bs == si && bw > 0 => {
					(s.w as i32 * 100 / bw as i32).clamp(def.min, def.max)
				}
				_ => 100,
			})
		}
		SliderTarget::StrokeWidth => match selected_shape(state, win) {
			Some((_, _, s)) if s.kind == ShapeKind::Stroke => Some(s.brush as i32),
			_ => None,
		},
		SliderTarget::ElementRoundedness => match selected_shape(state, win) {
			Some((_, _, s)) if s.kind == ShapeKind::Rectangle => Some(s.corner as i32),
			_ => None,
		},
		SliderTarget::FrameDuration => state.drawing.frames.first().map(|f| f.duration_ms as i32),
		SliderTarget::ExtractCount => Some(state.extract_count),
		SliderTarget::TactileColumns => Some(state.tactile_cols),
		SliderTarget::TactileRows => Some(state.tactile_rows),
		SliderTarget::SoundVolume => Some(state.sound_volume as i32),
		t if def.is_adjustment() => Some(state.adjust.get(t)),
		_ => None,
	}
}

/// The readout words for a slider with nothing to act on.
fn slider_placeholder(def: &SliderDef) -> String {
	match def.target {
		SliderTarget::ElementOpacity | SliderTarget::ElementScale => t("no element selected"),
		SliderTarget::StrokeWidth => t("no stroke selected"),
		SliderTarget::ElementRoundedness => t("no rectangle selected"),
		SliderTarget::FrameDuration => t("no frames yet"),
		_ => t("not available"),
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
		AdjustTarget::Element => {
			selected_shape(state, win).map(|(li, si, s)| (ChannelTarget::Element(li, si), s.fill))
		}
		_ => None,
	}
}

/// Why the channel sliders cannot be edited right now, in words
/// for the note under the group (and the status line, if a stray
/// event reaches a disabled slider). Empty when they can.
fn channel_note_text(state: &State, win: &Win) -> String {
	match state.slider_target {
		AdjustTarget::BaseColor => String::new(),
		AdjustTarget::Element => {
			if selected_shape(state, win).is_some() {
				String::new()
			} else {
				t("Select an element to edit its color channels.")
			}
		}
		_ => t("Channel sliders edit one color at a time: set the color adjustments target to Current color or Selected element to use them."),
	}
}

/// The values the channel sliders should show: an active run on
/// this target in this format shows its own values (they are the
/// truth while it lasts); anything else shows the target color's
/// own values in the current format.
fn channel_display_values(win: &Win, state: &State) -> Option<Vec<i32>> {
	let (id, color) = channel_target(state, win)?;
	if let Some(run) = &state.channel_run {
		if run.format == state.channel_format && run.target == id {
			return Some(run.values.clone());
		}
	}
	Some(channel_values(state.channel_format, color))
}

/// Push the current values into the channel group's controls,
/// readouts, greying and note. Runs inside every slider refresh,
/// so the group always agrees with the project.
fn refresh_channel_values(win: &Win, state: &State) {
	if win.channel_rows.borrow().is_empty() {
		return;
	}
	let ChannelSet::Sliders(defs) = format_channels(state.channel_format) else {
		return;
	};
	let editable = channel_target(state, win).is_some();
	let values = channel_display_values(win, state);
	win.slider_refreshing.set(true);
	for row in win.channel_rows.borrow().iter() {
		let Some(def) = defs.get(row.index) else {
			continue;
		};
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
		note.set_label(&channel_note_text(state, win));
	}
}

/// The element the main window has selected, as (layer, shape,
/// shape) — the target of the element sliders.
fn selected_shape<'a>(state: &'a State, win: &Win) -> Option<(usize, usize, &'a Shape)> {
	let flat = win.elements_list.borrow().selected()?;
	let (li, si) = locate(state, flat)?;
	state
		.drawing
		.layers
		.get(li)?
		.shapes
		.get(si)
		.map(|s| (li, si, s))
}

/// Push the current values into the Sliders tab's controls and
/// readouts. Runs inside every refresh, so the tab always agrees
/// with the project; guarded so programmatic sets never re-enter
/// the change handler.
fn refresh_slider_values(win: &Win, state: &State) {
	if win.slider_rows.borrow().is_empty()
		&& win.channel_rows.borrow().is_empty()
		&& win.shape_rows.borrow().is_empty()
	{
		return;
	}
	win.slider_refreshing.set(true);
	for row in win.slider_rows.borrow().iter() {
		let Some(def) = slider_by_id(row.id) else {
			continue;
		};
		match read_slider_value(win, state, def) {
			Some(value) => {
				row.slider.set_value(value);
				row.readout.set_label(&def.format(value));
			}
			None => row.readout.set_label(&slider_placeholder(def)),
		}
	}
	win.slider_refreshing.set(false);
	refresh_channel_values(win, state);
	refresh_shape_values(win, state);
}

/// Build the color-channels group into the Sliders page: a
/// header, the group's own format picker, then one row per
/// channel of the CHOSEN format — every part of the value on
/// its own slider (hex spells itself out as six digit sliders)
/// — a label, a NATIVE slider carrying the channel's spoken
/// name ("LCH hue"), and a value readout — editing the one
/// color the adjustments target picker names. The picker is
/// the group's own: it starts at the View format and then goes
/// its own way, so every format's breakdown is here whether or
/// not the View menu agrees. Accessibility follows the slider
/// rows: real accessible names, no programmatic focus moves
/// (Ctrl+arrow hops are the user's own keys, as on the Sliders
/// tab). For the binary format there are no sliders: the group
/// is the core's own sentence about why — binary is a readout,
/// not an editing system. When the target names many colors (a
/// layer, a depth) or no element is selected, the sliders grey
/// out and the note under the group says why.
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
		let values = channel_target(&state, win)
			.map(|(_, color)| channel_values(state.channel_format, color));
		(state.channel_format, values)
	};
	let header = StaticText::builder(page)
		.with_label(&t("Color channels"))
		.build();
	sizer.add(
		&header,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	{
		let label = StaticText::builder(page)
			.with_label(&t("Channel format"))
			.build();
		let choice = Choice::builder(page).build();
		for f in ColorFormat::ALL {
			choice.append(&f.name());
		}
		let at = ColorFormat::ALL
			.iter()
			.position(|f| *f == format)
			.unwrap_or(0);
		choice.set_selection(at as u32);
		choice.set_name(&t("Channel format"));
		let row = BoxSizer::builder(Orientation::Horizontal).build();
		row.add(&label, 0, SizerFlag::All, 5);
		row.add(&choice, 0, SizerFlag::All, 5);
		sizer.add_sizer(&row, 0, SizerFlag::Expand, 0);
		{
			let (w, s, choice) = (win.clone(), shared.clone(), choice.clone());
			choice.on_selection_changed(move |_| {
				let Some(sel) = choice.get_selection() else {
					return;
				};
				let Some(&format) = ColorFormat::ALL.get(sel as usize) else {
					return;
				};
				{
					let mut state = s.borrow_mut();
					if state.channel_format == format {
						return;
					}
					state.channel_format = format;
					state.channel_run = None;
				}
				// The rebuild destroys this very Choice, so it
				// waits for the rebuild timer's tick — the pick
				// event has finished by then.
				w.channel_rebuild_timer.start(1, true);
			});
		}
	}
	match format_channels(format) {
		ChannelSet::ReadoutOnly(reason) => {
			let note = StaticText::builder(page).with_label(&t(reason)).build();
			sizer.add(
				&note,
				0,
				SizerFlag::Left | SizerFlag::Right | SizerFlag::Bottom,
				5,
			);
			*note_out = Some(note);
		}
		ChannelSet::Sliders(defs) => {
			for (index, def) in defs.iter().enumerate() {
				let label_text = def.display_label();
				let (value, text) = match values.as_ref().and_then(|v| v.get(index)) {
					Some(v) => (*v, def.format_value(*v)),
					None => (def.min, t("no color to edit")),
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
				rows.push(ChannelRow {
					index,
					slider,
					readout,
				});
			}
			let note = StaticText::builder(page)
				.with_label(&channel_note_text(&shared.borrow(), win))
				.build();
			sizer.add(
				&note,
				0,
				SizerFlag::Left | SizerFlag::Right | SizerFlag::Bottom,
				5,
			);
			*note_out = Some(note);
		}
	}
}

/// What one shape part's slider should show right now: its
/// value (before clamping into the row's range), the readout's
/// words, and whether the slider can be moved. Parts a shape
/// kind does not have say so instead of a number — a stroke's
/// size is its points, and roundedness only affects rectangles
/// — the same answers the dialogs give.
fn shape_part_state(win: &Win, state: &State, part: ShapePart) -> (Option<i32>, String, bool) {
	let Some((_, _, s)) = selected_shape(state, win) else {
		return (None, "no element selected".to_string(), false);
	};
	let (cw, ch) = (state.drawing.width as i32, state.drawing.height as i32);
	match part {
		ShapePart::Across => {
			let v = s.x.clamp(0, cw.max(0));
			(Some(v), part.words(v), true)
		}
		ShapePart::Down => {
			let v = s.y.clamp(0, ch.max(0));
			(Some(v), part.words(v), true)
		}
		ShapePart::Width => {
			if s.kind == ShapeKind::Stroke {
				(
					Some(s.w as i32),
					"a stroke's size is its points".to_string(),
					false,
				)
			} else {
				let v = (s.w as i32).clamp(1, cw.max(1));
				(Some(v), part.words(v), true)
			}
		}
		ShapePart::Height => {
			if s.kind == ShapeKind::Stroke {
				(
					Some(s.h as i32),
					"a stroke's size is its points".to_string(),
					false,
				)
			} else {
				let v = (s.h as i32).clamp(1, ch.max(1));
				(Some(v), part.words(v), true)
			}
		}
		ShapePart::Rotation => {
			let v = ((s.angle.round() as i32) % 360 + 360) % 360;
			(Some(v), part.words(v), true)
		}
		ShapePart::Roundedness => {
			if s.kind == ShapeKind::Rectangle {
				let v = s.corner as i32;
				(Some(v), part.words(v), true)
			} else {
				(
					Some(s.corner as i32),
					"roundedness only affects rectangles".to_string(),
					false,
				)
			}
		}
	}
}

/// Build the shape-parts group into the Sliders page: a header
/// and its note, then the six part rows — label, NATIVE slider
/// carrying the part's name, value readout — the selected
/// element's geometry broken into its parts the way the color
/// channels break a color into its parts (Elshara's principle:
/// every part of a value its own slider). The rows are always
/// here; with nothing selected they sit greyed out over the
/// words "no element selected", like the registry rows do.
/// Ranges come from the canvas at build time: across and down
/// run the canvas, width and height run from one pixel to the
/// canvas, rotation is the full turn, roundedness its percent.
fn build_shape_group(
	win: &Rc<Win>,
	shared: &Shared,
	page: &Panel,
	sizer: &BoxSizer,
	rows: &mut Vec<ShapeRow>,
) {
	let header = StaticText::builder(page)
		.with_label(&t("Shape parts of the selected element"))
		.build();
	sizer.add(
		&header,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let note = StaticText::builder(page)
		.with_label(
			"Roundedness only affects rectangles; a stroke's size is its points, so a stroke's width and height sliders stay off.",
		)
		.build();
	sizer.add(&note, 0, SizerFlag::Left | SizerFlag::Right, 5);
	let (cw, ch) = {
		let state = shared.borrow();
		(state.drawing.width as i32, state.drawing.height as i32)
	};
	for part in ShapePart::ALL {
		let (min, max) = match part {
			ShapePart::Across => (0, cw.max(0)),
			ShapePart::Down => (0, ch.max(0)),
			ShapePart::Width => (1, cw.max(1)),
			ShapePart::Height => (1, ch.max(1)),
			ShapePart::Rotation => (0, 359),
			ShapePart::Roundedness => (0, 50),
		};
		let (value, text, enabled) = {
			let state = shared.borrow();
			shape_part_state(win, &state, part)
		};
		let label = StaticText::builder(page).with_label(&part.name()).build();
		let slider = Slider::builder(page)
			.with_min_value(min)
			.with_max_value(max)
			.with_value(value.unwrap_or(min).clamp(min, max))
			.build();
		slider.set_name(&part.name());
		slider.enable(enabled);
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
				apply_shape_part(&w, &s, part, e.get_value());
			});
		}
		{
			let (w, s) = (win.clone(), shared.clone());
			slider.on_key_down(move |data| {
				shape_key(&w, &s, part, &data);
			});
		}
		rows.push(ShapeRow {
			part,
			min,
			max,
			slider,
			readout,
		});
	}
}

/// Push the selected element's parts into the shape group's
/// controls and readouts. Runs inside every slider refresh, so
/// the group always agrees with the selection; guarded like
/// the other refreshes so programmatic sets never re-enter the
/// change handler.
fn refresh_shape_values(win: &Win, state: &State) {
	if win.shape_rows.borrow().is_empty() {
		return;
	}
	win.slider_refreshing.set(true);
	for row in win.shape_rows.borrow().iter() {
		let (value, text, enabled) = shape_part_state(win, state, row.part);
		row.slider.enable(enabled);
		if let Some(v) = value {
			row.slider.set_value(v.clamp(row.min, row.max));
		}
		row.readout.set_label(&text);
	}
	win.slider_refreshing.set(false);
}

/// Apply one shape part's new value to the selected element,
/// through the same drawing calls the Move, Resize and Rotate
/// dialogs use, so it lands in the log like every other edit —
/// a position run is one merged shift step, a rotation keeps
/// its final angle. The announcement names the part and its
/// new value, and that everything else stayed the same.
fn apply_shape_part(win: &Win, shared: &Shared, part: ShapePart, value: i32) {
	let mut note: Option<String> = None;
	{
		let mut state = shared.borrow_mut();
		match selected_shape(&state, win).map(|(li, si, s)| (li, si, s.clone())) {
			Some((li, si, s)) => {
				let landed = match part {
					ShapePart::Across => state.drawing.set_shape_position(li, si, value, s.y),
					ShapePart::Down => state.drawing.set_shape_position(li, si, s.x, value),
					ShapePart::Width => {
						if s.kind == ShapeKind::Stroke {
							note = Some(t(
								"A stroke's size is its points; move it or add a new one.",
							));
							false
						} else {
							state.drawing.resize_shape(li, si, value.max(1) as u32, s.h)
						}
					}
					ShapePart::Height => {
						if s.kind == ShapeKind::Stroke {
							note = Some(t(
								"A stroke's size is its points; move it or add a new one.",
							));
							false
						} else {
							state.drawing.resize_shape(li, si, s.w, value.max(1) as u32)
						}
					}
					ShapePart::Rotation => state.drawing.rotate_shape(li, si, value as f64),
					ShapePart::Roundedness => {
						if s.kind == ShapeKind::Rectangle {
							state.drawing.set_shape_corner(li, si, value as u8)
						} else {
							note = Some(
								"Roundedness acts on a selected rectangle; ellipses are already round and strokes have no corners."
									.to_string(),
							);
							false
						}
					}
				};
				if landed {
					refresh_lists(win, &state);
				}
			}
			None => note = Some(t("Select an element first.")),
		}
	}
	match note {
		Some(text) => win.frame.set_status_text(&text, 0),
		None => {
			let announce_on = shared.borrow().announce_sliders;
			let words = format!("{} {}", part.name(), part.words(value));
			win.frame.set_status_text(
				&if announce_on {
					t("%s. Everything else unchanged.").replacen("%s", &(words).to_string(), 1)
				} else {
					format!("{words}.")
				},
				0,
			);
		}
	}
}

/// The shape rows' custom keys, the Sliders tab's own: Ctrl
/// plus an arrow hops focus from part to part; Page Up and Page
/// Down jump by the project's page-jump percent of the part's
/// range. Plain arrows, Home and End are the native slider's
/// own behavior, skipped onward untouched.
fn shape_key(win: &Rc<Win>, shared: &Shared, part: ShapePart, data: &WindowEventData) {
	let WindowEventData::Keyboard(kb) = data else {
		data.skip(true);
		return;
	};
	let Some(code) = kb.get_key_code() else {
		data.skip(true);
		return;
	};
	let rows = win.shape_rows.borrow();
	let Some(pos) = rows.iter().position(|r| r.part == part) else {
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
		let percent = shared.borrow().page_jump_percent;
		let Some(row) = rows.get(pos) else {
			data.skip(true);
			return;
		};
		let span = (row.max - row.min).max(0);
		let jump = ((span * percent.min(100) as i32) / 100).max(1);
		let delta = if code == WXK_PAGEUP { jump } else { -jump };
		let value = (row.slider.get_value() + delta).clamp(row.min, row.max);
		row.slider.set_value(value);
		drop(rows);
		apply_shape_part(win, shared, part, value);
		return;
	}
	data.skip(true);
}

/// Design, Recognize Shape from Stroke: read the selected
/// stroke's sampled points and, when they trace a known shape,
/// replace the stroke with the clean shape — same place in the
/// layer, same color, brush width, pattern and opacity, with a
/// caption that says what it was recognized as. The swap goes
/// through the drawing (remove, then insert at the same index),
/// so it lands in the log like every other edit, and `announce`
/// speaks the new element. A stroke that matches nothing known
/// is left exactly as drawn, and the status line says so
/// plainly instead of guessing.
fn recognize_selected(win: &Rc<Win>, shared: &Shared) {
	let mut state = shared.borrow_mut();
	let Some((li, si, old)) = selected_shape(&state, win).map(|(li, si, s)| (li, si, s.clone()))
	else {
		win.frame.set_status_text(&t("Select a stroke first."), 0);
		return;
	};
	if old.kind != ShapeKind::Stroke {
		win.frame.set_status_text(&t("Recognize Shape works on a selected stroke: draw one with the pointer, or add a stroke by its points."),
			0,
		);
		return;
	}
	let mut clean = match recognize(&old.points) {
		Recognized::Line { from, to } => {
			let mut s = Shape::stroke(vec![from, to], old.brush, old.fill);
			s.caption = Some(t("Recognized line"));
			s
		}
		Recognized::Rectangle { x, y, w, h } => {
			let mut s = Shape::new(ShapeKind::Rectangle, x, y, w, h, old.fill);
			s.caption = Some(t("Recognized rectangle"));
			s
		}
		Recognized::Ellipse { x, y, w, h } => {
			let mut s = Shape::new(ShapeKind::Ellipse, x, y, w, h, old.fill);
			s.caption = Some(t("Recognized ellipse"));
			s
		}
		Recognized::Triangle { corners } => {
			let mut pts = corners.to_vec();
			pts.push(corners[0]);
			let mut s = Shape::stroke(pts, old.brush, old.fill);
			s.caption = Some(t("Recognized triangle"));
			s
		}
		Recognized::Unknown => {
			win.frame.set_status_text(
				&t("That stroke was not recognized as a known shape; it stays exactly as drawn."),
				0,
			);
			return;
		}
	};
	// The stroke's own paint carries over: its pattern and
	// opacity (its color and brush width are already placed).
	clean.pattern = old.pattern.clone();
	clean.opacity = old.opacity;
	state.drawing.remove_shape(li, si);
	state.drawing.insert_shape(li, si, clean);
	refresh_lists(win, &state);
	announce(win, &state);
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
/// with no pins gets EVERY slider — pinning trims the tab, it
/// never gates it — under a note saying so.
/// The notebook index of the Sliders page, found by its text:
/// the page count before it changes with the Lists Side by
/// Side layout, so a fixed index would remove the wrong page.
fn sliders_page_index(win: &Win) -> Option<usize> {
	(0..win.notebook.get_page_count())
		.find(|&i| win.notebook.get_page_text(i) == Some(t("Sliders")))
}

/// Take the Sliders page out of the notebook and forget its
/// rows: the page the interaction mode hides. The rows go with
/// it — a refresh against controls that no longer exist would
/// be a lie, and an empty row set refreshes as a no-op.
fn remove_sliders_tab(win: &Win) {
	if win.slider_page.borrow().is_some() {
		if let Some(i) = sliders_page_index(win) {
			win.notebook.remove_page(i);
		}
	}
	*win.slider_page.borrow_mut() = None;
	win.slider_rows.borrow_mut().clear();
	win.channel_rows.borrow_mut().clear();
	win.shape_rows.borrow_mut().clear();
	*win.channel_note.borrow_mut() = None;
	*win.slider_target_choice.borrow_mut() = None;
}

/// Apply the state's interaction mode to the main window
/// (Elshara's principle: only one interaction surface shows
/// at a time, per project): the typed-fields panel shows only
/// in Type mode, the Draw panel only in Draw mode, and the
/// Sliders page exists only in Slide mode — every other
/// build_sliders_tab caller is gated by the same rule, so the
/// page cannot creep back while another mode is showing.
/// Hidden surfaces leave the tab order with their visibility:
/// a hidden panel's controls are not reachable, a removed page
/// is not there at all. With `move_focus`, focus lands on a
/// real control of the surface just shown — the hex field, the
/// Draw button, or the Sliders page's first slider (the
/// palette list when the page somehow has none).
fn apply_interaction_mode(win: &Rc<Win>, shared: &Shared, move_focus: bool) {
	let mode = shared.borrow().interaction_mode;
	win.typed_panel.show(mode == InteractionMode::Type);
	win.draw_panel.show(mode == InteractionMode::Draw);
	if mode == InteractionMode::Slide {
		build_sliders_tab(win, shared);
	} else {
		remove_sliders_tab(win);
	}
	win.panel.layout();
	if !move_focus {
		return;
	}
	match mode {
		InteractionMode::Type => win.hex_field.set_focus(),
		InteractionMode::Draw => win.draw_button.set_focus(),
		InteractionMode::Slide => {
			if let Some(i) = sliders_page_index(win) {
				win.notebook.set_selection(i);
			}
			let landed = {
				let rows = win.slider_rows.borrow();
				if let Some(row) = rows.first() {
					row.slider.set_focus();
					true
				} else {
					false
				}
			};
			if !landed {
				let landed = {
					let rows = win.channel_rows.borrow();
					if let Some(row) = rows.first() {
						row.slider.set_focus();
						true
					} else {
						false
					}
				};
				if !landed {
					win.palette_list.borrow().set_focus();
				}
			}
		}
	}
}

/// Bring the Sliders page forward and put focus on one
/// shape-part slider: the Slide-mode landing for the Arrange
/// commands (Move rests on Across, Resize on Width, Rotate on
/// Rotation). The row may be disabled for the selected element
/// (a stroke's Width) — its callers keep those cases out, the
/// way the Resize dialog refuses a stroke before opening.
fn focus_shape_part(win: &Win, part: ShapePart) {
	if let Some(i) = sliders_page_index(win) {
		win.notebook.set_selection(i);
	}
	if let Some(row) = win.shape_rows.borrow().iter().find(|r| r.part == part) {
		row.slider.set_focus();
	}
}

fn build_sliders_tab(win: &Rc<Win>, shared: &Shared) {
	// The Sliders page is the Slide mode's surface: in any
	// other mode there is no page to build, whoever asked.
	if shared.borrow().interaction_mode != InteractionMode::Slide {
		remove_sliders_tab(win);
		return;
	}
	if win.slider_page.borrow().is_some() {
		if let Some(i) = sliders_page_index(win) {
			win.notebook.remove_page(i);
		}
	}
	let (pins, target, showing_all) = {
		let state = shared.borrow();
		// No pins means EVERY slider, in registry order: the
		// tab must never be a dead end of static text with
		// nothing a keyboard can land on (Elshara's hands-on
		// report). Pinning is how a project trims the tab to
		// its own set — empty pins, the whole registry.
		let showing_all = state.pinned_sliders.is_empty();
		let effective: Vec<String> = if showing_all {
			SLIDERS.iter().map(|d| d.id.to_string()).collect()
		} else {
			state.pinned_sliders.clone()
		};
		(effective, state.slider_target, showing_all)
	};
	let page = Panel::builder(&win.notebook).build();
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let mut rows: Vec<SliderRow> = Vec::new();
	let mut channel_rows: Vec<ChannelRow> = Vec::new();
	let mut channel_note: Option<StaticText> = None;
	let mut shape_rows: Vec<ShapeRow> = Vec::new();
	let target_choice: Option<Choice>;
	if showing_all {
		let note = StaticText::builder(&page)
			.with_label(
				"Showing every slider: this project pins none of its own. Pin a set in Design, Project Options and only those stay; clear the pins and every slider shows again.",
			)
			.build();
		sizer.add(
			&note,
			0,
			SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
			5,
		);
	}
	{
		// The adjustments target picker always shows now: the
		// color-channels group below is a permanent resident of
		// the page, and the picker names the one color the
		// channels (and any pinned adjustments) act on.
		{
			let label = StaticText::builder(&page)
				.with_label(&t("Color adjustments act on"))
				.build();
			let choice = Choice::builder(&page).build();
			for t in AdjustTarget::ALL {
				choice.append(&t.name());
			}
			let at = AdjustTarget::ALL
				.iter()
				.position(|t| *t == target)
				.unwrap_or(0);
			choice.set_selection(at as u32);
			let row = BoxSizer::builder(Orientation::Horizontal).build();
			row.add(&label, 0, SizerFlag::All, 5);
			row.add(&choice, 0, SizerFlag::All, 5);
			sizer.add_sizer(&row, 0, SizerFlag::Expand, 0);
			{
				let (w, s, choice) = (win.clone(), shared.clone(), choice.clone());
				choice.on_selection_changed(move |_| {
					let Some(sel) = choice.get_selection() else {
						return;
					};
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
			let Some(def) = slider_by_id(id) else {
				continue;
			};
			if def.target == SliderTarget::ColorChannels {
				// The channels group is permanent — it is built
				// after this loop, pins or no pins.
				continue;
			}
			let (value, text) = match read_slider_value(win, &state, def) {
				Some(v) => (v, def.format(v)),
				None => (def.min, slider_placeholder(def).to_string()),
			};
			let label = StaticText::builder(&page)
				.with_label(&def.display_name())
				.build();
			let slider = Slider::builder(&page)
				.with_min_value(def.min)
				.with_max_value(def.max)
				.with_value(value)
				.build();
			slider.set_name(&def.display_name());
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
			rows.push(SliderRow {
				id: def.id,
				slider,
				readout,
			});
		}
	}
	// The two per-part groups close the page, always: the
	// color's channels and the selected element's shape parts.
	build_channel_group(
		win,
		shared,
		&page,
		&sizer,
		&mut channel_rows,
		&mut channel_note,
	);
	build_shape_group(win, shared, &page, &sizer, &mut shape_rows);
	page.set_sizer(sizer, true);
	win.notebook.add_page(&page, &t("Sliders"), false, None);
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
	*win.shape_rows.borrow_mut() = shape_rows;
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
	sizer.add(
		&label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
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
			let WindowEventData::Keyboard(kb) = data else {
				return false;
			};
			if kb.get_key_code() != Some(WXK_RETURN) {
				return false;
			}
			if let Some(i) = w.palette_list.borrow().selected() {
				let state = s.borrow();
				if let Some(text) = inspect_color(&state.drawing, &state.palette, i) {
					report_dialog(&w, &t("Inspector: color"), &t("Color inspector"), &text);
				}
			}
			true
		});
	}
	{
		let w = win.clone();
		win.palette_list
			.borrow()
			.on_context_menu(move || show_palette_context(&w));
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
			let WindowEventData::Keyboard(kb) = data else {
				return false;
			};
			if kb.get_key_code() != Some(WXK_RETURN) {
				return false;
			}
			inspect_selected(&w, &s);
			true
		});
	}
	{
		let w = win.clone();
		win.elements_list
			.borrow()
			.on_context_menu(move || show_elements_context(&w));
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
		let (palette_panel, palette_list) = list_panel(&splitter, &t("Palette"));
		let (elements_panel, elements_list) = list_panel(&splitter, &t("Elements"));
		splitter.split_vertically(&palette_panel, &elements_panel, 380);
		let sizer = BoxSizer::builder(Orientation::Vertical).build();
		sizer.add(&splitter, 1, SizerFlag::Expand | SizerFlag::All, 5);
		page.set_sizer(sizer, true);
		win.notebook.insert_page(0, &page, &t("Lists"), true, None);
		pages.push(page);
		*win.palette_list.borrow_mut() = palette_list;
		*win.elements_list.borrow_mut() = elements_list;
	} else {
		let (palette_page, palette_list) = list_panel(&win.notebook, &t("Palette"));
		let (elements_page, elements_list) = list_panel(&win.notebook, &t("Elements"));
		win.notebook
			.insert_page(0, &elements_page, &t("Elements"), false, None);
		win.notebook
			.insert_page(0, &palette_page, &t("Palette"), true, None);
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
	drop(state);
	// Land on the palette list (Elshara's report: after the
	// rebuild the keyboard was nowhere a screen reader could
	// use). This runs only on the user's own toggle, and the
	// palette list exists in both layouts.
	win.palette_list.borrow().set_focus();
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
			SliderTarget::ColorRed | SliderTarget::ColorGreen | SliderTarget::ColorBlue => {
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
				None => note = Some(t("Select an element first.")),
			},
			SliderTarget::ElementScale => match selected_shape(&state, win) {
				Some((li, si, s)) => {
					if s.kind == ShapeKind::Stroke {
						note = Some(t("A stroke's size is its points; it does not scale."));
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
				None => note = Some(t("Select an element first.")),
			},
			SliderTarget::StrokeWidth => match selected_shape(&state, win) {
				Some((li, si, s)) => {
					if s.kind == ShapeKind::Stroke {
						if state.drawing.set_shape_brush_width(li, si, value as u32) {
							refresh_lists(win, &state);
						}
					} else {
						note = Some(t("Stroke width acts on a selected stroke."));
					}
				}
				None => note = Some(t("Select a stroke first.")),
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
				None => note = Some(t("Select a rectangle first.")),
			},
			SliderTarget::FrameDuration => {
				if state.drawing.frames.is_empty() {
					note = Some(t("No animation frames yet: add one in Design, Animation."));
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
				win.frame
					.set_status_text(&change_announcement(def, value), 0);
			} else {
				win.frame.set_status_text(
					&t("%s %s.").replacen("%s", &def.display_name(), 1).replacen(
						"%s",
						&def.format(value),
						1,
					),
					0,
				);
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
			(state.channel_format, state.page_jump_percent)
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
		let format = state.channel_format;
		let ChannelSet::Sliders(defs) = format_channels(format) else {
			return;
		};
		let Some(def) = defs.get(index).copied() else {
			return;
		};
		let value = def.clamp(value);
		let Some((target_id, base_color)) = channel_target(&state, win) else {
			// The sliders sit greyed out in this state; a stray
			// event still gets the straight answer.
			win.frame
				.set_status_text(&channel_note_text(&state, win), 0);
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
					note = Some(t("That element is gone."));
				}
			}
		}
		if let Some(row) = win.channel_rows.borrow().iter().find(|r| r.index == index) {
			row.readout.set_label(&def.format_value(applied));
			if applied != value {
				row.slider.set_value(applied);
			}
		}
		let words = format!("{} {}", def.display_label(), def.format_value(applied));
		Some(if state.announce_sliders {
			t("%s. Everything else unchanged.").replacen("%s", &(words).to_string(), 1)
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
fn apply_adjustment(win: &Win, state: &mut State, def: &SliderDef, value: i32) -> Option<String> {
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
		state.adjust_base =
			match state.slider_target {
				AdjustTarget::BaseColor => Some(AdjustBase::Brush(state.brush.color)),
				AdjustTarget::Element => selected_shape(state, win)
					.map(|(li, si, s)| AdjustBase::Element(li, si, s.fill)),
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
							l.shapes
								.iter()
								.enumerate()
								.map(move |(si, s)| (li, si, s.fill))
						})
						.collect();
					AdjustBase::Depth(role, bases)
				}),
			};
	}
	if state.adjust_base.is_none() {
		return Some(match state.slider_target {
			AdjustTarget::Element => t("Select an element to adjust first."),
			AdjustTarget::Layer => t("This project has no current layer."),
			AdjustTarget::BaseColor => t("Nothing to adjust."),
			_ => t("Nothing to adjust."),
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
				Some(t("That element is gone."))
			}
		}
		Some(AdjustBase::Layer(li, bases)) => {
			let (li, bases) = (*li, bases.clone());
			let adjust = state.adjust;
			if bases.is_empty() {
				Some(t("The current layer has no elements to adjust."))
			} else {
				let outcome = state.drawing.adjust_layer_colors(li, &bases, adjust);
				if outcome.changed > 0 {
					refresh_lists(win, state);
					Some(
						t("%s element%s changed%s.")
							.replacen("%s", &(outcome.changed).to_string(), 1)
							.replacen(
								"%s",
								if outcome.changed == 1 { "" } else { "s" },
								1,
							)
							.replacen("%s", &(locked_suffix(outcome.locked)).to_string(), 1),
					)
				} else if outcome.locked > 0 {
					Some(
						t("Every element the adjustment reached is locked; nothing changed%s")
							.replacen("%s", &(locked_suffix(outcome.locked)).to_string(), 1),
					)
				} else {
					Some(t(
						"That layer changed under the adjustment; move the slider to start again.",
					))
				}
			}
		}
		Some(AdjustBase::Depth(role, bases)) => {
			let (role, bases) = (*role, bases.clone());
			let adjust = state.adjust;
			if bases.is_empty() {
				Some(
					t("No layer is tagged %s yet: tag one in Design, Layers.").replacen(
						"%s",
						&(role.name()).to_string(),
						1,
					),
				)
			} else {
				let outcome = state.drawing.adjust_depth_colors(role, &bases, adjust);
				if outcome.changed > 0 {
					refresh_lists(win, state);
					Some(
						t("%s element%s changed across the %s layers%s.")
							.replacen("%s", &(outcome.changed).to_string(), 1)
							.replacen(
								"%s",
								if outcome.changed == 1 { "" } else { "s" },
								1,
							)
							.replacen("%s", &(role.name()).to_string(), 1)
							.replacen("%s", &(locked_suffix(outcome.locked)).to_string(), 1),
					)
				} else if outcome.locked > 0 {
					Some(
						t("Every element the adjustment reached is locked; nothing changed%s")
							.replacen("%s", &(locked_suffix(outcome.locked)).to_string(), 1),
					)
				} else {
					Some(t("Those layers changed under the adjustment; move the slider to start again."))
				}
			}
		}
		None => Some(t("Nothing to adjust.")),
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
		t(", %s locked stayed the same").replacen("%s", &(locked).to_string(), 1)
	}
}

/// The Project Options dialog (Design menu): this project's home
/// for what used to be scattered — the pinned-slider checklist
/// (checked sliders, and only they, live on the Sliders tab;
/// check none and the tab shows every slider), what the color
/// adjustments act on, the Page Up/Down jump, and the starter
/// colors. OK applies the pins, target and jump at once; the
/// dialog is modal and closing it returns to the main window.
fn project_options_dialog(win: &Rc<Win>, shared: &Shared) {
	let frame = win.frame;
	let dialog = Dialog::builder(&frame, &t("Project Options"))
		.with_size(560, 640)
		.build();
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let pins_label = StaticText::builder(&dialog)
		.with_label(&t("Sliders pinned to this project's Sliders tab (check the ones to keep; with none checked, the tab shows every slider)"))
		.build();
	sizer.add(
		&pins_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let checks = CheckListBox::builder(&dialog).build();
	for def in SLIDERS {
		checks.append(&def.display_name());
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
	let target_label = StaticText::builder(&dialog)
		.with_label(&t("Color adjustments act on"))
		.build();
	sizer.add(
		&target_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let target_choice = Choice::builder(&dialog).build();
	for t in AdjustTarget::ALL {
		target_choice.append(&t.name());
	}
	{
		let state = shared.borrow();
		let at = AdjustTarget::ALL
			.iter()
			.position(|t| *t == state.slider_target)
			.unwrap_or(0);
		target_choice.set_selection(at as u32);
	}
	sizer.add(&target_choice, 0, SizerFlag::Expand | SizerFlag::All, 5);
	// The interaction mode (Elshara's principle): the one
	// place the project's surface is switched — Draw, Slide
	// or Type, only one of them on screen at a time.
	let mode_label = StaticText::builder(&dialog)
		.with_label(&t(
			"Interaction mode (one surface at a time: Draw, Slide or Type)",
		))
		.build();
	sizer.add(
		&mode_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let mode_choice = Choice::builder(&dialog).build();
	for m in InteractionMode::ALL {
		mode_choice.append(&m.name());
	}
	{
		let state = shared.borrow();
		let at = InteractionMode::ALL
			.iter()
			.position(|m| *m == state.interaction_mode)
			.unwrap_or(2);
		mode_choice.set_selection(at as u32);
	}
	sizer.add(&mode_choice, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let jump_now = shared.borrow().page_jump_percent;
	let jump_label = StaticText::builder(&dialog)
		.with_label(&t(
			"Page Up and Page Down jump, percent of each slider's range",
		))
		.build();
	let jump_spin = SpinCtrl::builder(&dialog).with_range(1, 100).build();
	jump_spin.set_value(jump_now as i32);
	sizer.add(
		&jump_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	sizer.add(&jump_spin, 0, SizerFlag::Expand | SizerFlag::All, 5);
	// Grayscale start: build the design in grayscale first —
	// form and legibility before colour — then give parts,
	// layers or depths their colour treatment when they are
	// ready (Design, Apply Colour Treatment). The gray ramp is
	// the palette that mode works from.
	let gray_check = CheckBox::builder(&dialog)
		.with_label(&t(
			"Grayscale start (build in grayscale; colours come later, per part, layer or depth)",
		))
		.build();
	gray_check.set_value(shared.borrow().grayscale_start);
	sizer.add(&gray_check, 0, SizerFlag::All, 5);
	let gray_btn = Button::builder(&dialog)
		.with_label(&t("Load Gray Ramp"))
		.build();
	sizer.add(&gray_btn, 0, SizerFlag::All, 5);
	let starter_btn = Button::builder(&dialog)
		.with_label(&t("Load Starter Colors"))
		.build();
	sizer.add(&starter_btn, 0, SizerFlag::All, 5);
	let ok = Button::builder(&dialog).with_label(&t("OK")).build();
	let cancel = Button::builder(&dialog).with_label(&t("Cancel")).build();
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
		let (w, s) = (win.clone(), shared.clone());
		gray_btn.on_click(move |_| {
			let mut state = s.borrow_mut();
			let fresh: Vec<PaletteColor> = gray_ramp()
				.into_iter()
				.filter(|pc| !state.palette.iter().any(|p| p.color == pc.color))
				.collect();
			if fresh.is_empty() {
				w.frame
					.set_status_text(&t("The gray ramp is already in the palette."), 0);
				return;
			}
			let count = fresh.len();
			state.palette.extend(fresh);
			refresh_lists(&w, &state);
			w.frame.set_status_text(
				&t("Loaded the gray ramp: %s grays, near-black to near-white, for grayscale work.")
					.replacen("%s", &(count).to_string(), 1),
				0,
			);
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
		state.grayscale_start = gray_check.is_checked();
		let mode_before = state.interaction_mode;
		let mut mode_after = mode_before;
		if let Some(sel) = mode_choice.get_selection() {
			if let Some(m) = InteractionMode::ALL.get(sel as usize) {
				mode_after = *m;
				state.interaction_mode = *m;
			}
		}
		let mode_note = if mode_after != mode_before {
			match mode_after {
				InteractionMode::Slide => " Interaction mode is now Slide: the Sliders tab is the surface, and the typed hex and L C H fields are hidden.",
				InteractionMode::Type => " Interaction mode is now Type: the typed hex and L C H fields show, and the Sliders tab is hidden.",
				InteractionMode::Draw => " Interaction mode is now Draw: the pointer drawing tools lead, and the sliders and typed fields are hidden.",
			}
		} else {
			""
		};
		let count = state.pinned_sliders.len();
		let grayscale = state.grayscale_start;
		drop(state);
		apply_interaction_mode(win, shared, true);
		frame.set_status_text(
			&t("Project options saved: %s slider%s pinned to the Sliders tab.%s%s%s")
				.replacen("%s", &(count).to_string(), 1)
				.replacen("%s", if count == 1 { "" } else { "s" }, 1)
				.replacen(
					"%s",
					if count == 0 {
						" With none pinned, the tab shows every slider."
					} else {
						""
					},
					1,
				)
				.replacen(
					"%s",
					if grayscale {
						" Grayscale start is on: colors come later, per part, layer, or depth."
					} else {
						""
					},
					1,
				)
				.replacen("%s", mode_note, 1),
			0,
		);
	}
}

/// Append colors to the palette, stamped with their provenance, and
/// say what was added.
fn add_colors(win: &Win, state: &mut State, colors: &[Srgb], what: &str, provenance: &Provenance) {
	for c in colors {
		state
			.palette
			.push(PaletteColor::new(*c, provenance.clone()));
	}
	refresh_lists(win, state);
	let names: Vec<String> = colors
		.iter()
		.map(|c| describe_in(*c, state.color_format))
		.collect();
	win.frame.set_status_text(
		&t("Added %s (%s): %s")
			.replacen("%s", what, 1)
			.replacen("%s", &(colors.len()).to_string(), 1)
			.replacen("%s", &(names.join("; ")).to_string(), 1),
		0,
	);
}

fn numbers_dialog(
	parent: &Frame,
	title: &str,
	fields: &[(String, i32, i32, i32)],
) -> Option<Vec<i32>> {
	let dialog = Dialog::builder(parent, title).build();
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let mut spins: Vec<SpinCtrl> = Vec::new();
	for (label, min, max, value) in fields {
		let text = StaticText::builder(&dialog).with_label(label).build();
		let spin = SpinCtrl::builder(&dialog).with_range(*min, *max).build();
		spin.set_value(*value);
		sizer.add(
			&text,
			0,
			SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
			5,
		);
		sizer.add(&spin, 0, SizerFlag::Expand | SizerFlag::All, 5);
		spins.push(spin);
	}
	let ok = Button::builder(&dialog).with_label(&t("OK")).build();
	let cancel = Button::builder(&dialog).with_label(&t("Cancel")).build();
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
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let text = StaticText::builder(&dialog).with_label(label).build();
	let field = TextCtrl::builder(&dialog).build();
	field.set_value(value);
	sizer.add(
		&text,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	sizer.add(&field, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let ok = Button::builder(&dialog).with_label(&t("OK")).build();
	let cancel = Button::builder(&dialog).with_label(&t("Cancel")).build();
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
fn fields_dialog(parent: &Frame, title: &str, fields: &[(String, &str)]) -> Option<Vec<String>> {
	let dialog = Dialog::builder(parent, title).build();
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let mut inputs: Vec<TextCtrl> = Vec::new();
	for (label, value) in fields {
		let text = StaticText::builder(&dialog).with_label(label).build();
		let field = TextCtrl::builder(&dialog).build();
		field.set_value(value);
		sizer.add(
			&text,
			0,
			SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
			5,
		);
		sizer.add(&field, 0, SizerFlag::Expand | SizerFlag::All, 5);
		inputs.push(field);
	}
	let ok = Button::builder(&dialog).with_label(&t("OK")).build();
	let cancel = Button::builder(&dialog).with_label(&t("Cancel")).build();
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
				t("Position: numbers like 120, 80, or words like top left, center, a bit right"),
				&format!("{}, {}", current_pos.0, current_pos.1),
			),
			(
				t("Size: numbers like 200 by 100, or words like a bit wider, a bit taller"),
				&t("%s by %s")
					.replacen("%s", &(current_size.0).to_string(), 1)
					.replacen("%s", &(current_size.1).to_string(), 1),
			),
			(
				t("Color: a name like sky blue, or a hex code"),
				&default_color.to_hex(),
			),
		],
	)?;
	let Some((x, y)) = parse_position(&v[0], canvas.0, canvas.1, Some(current_pos)) else {
		frame.set_status_text(&t("I could not understand that position. Use numbers like 120, 80, or words like top left, center, a bit right."),
			0,
		);
		return None;
	};
	let Some((w, h)) = parse_size(&v[1], Some(current_size)) else {
		frame.set_status_text(&t("I could not understand that size. Use numbers like 200 by 100, or words like a bit wider."),
			0,
		);
		return None;
	};
	let Some(color) = parse_color(&v[2]) else {
		frame.set_status_text(&t("I could not understand that color. Use a name like sky blue, or a hex code like 87CEEB."),
			0,
		);
		return None;
	};
	Some((x, y, w, h, color))
}

/// Speak a freshly added shape with its automatic plain-language
/// description (item: auto-describe on creation), and sound its cue.
fn speak_added(win: &Win, state: &State, layer: usize, index: usize) {
	let Some(shape) = state
		.drawing
		.layers
		.get(layer)
		.and_then(|l| l.shapes.get(index))
		.cloned()
	else {
		return;
	};
	win.frame.set_status_text(
		&t("Added %s.").replacen("%s", &(state.drawing.plain_of(&shape)).to_string(), 1),
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
		win.frame.set_status_text(
			&t("The brush library could not be saved: %s").replacen("%s", &(e).to_string(), 1),
			0,
		);
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
			let mut label = t("Layer %s").replacen("%s", &(i + 1).to_string(), 1);
			if l.depth != DepthRole::Unassigned {
				label.push_str(&format!(", {}", l.depth.name()));
			}
			label.push_str(
				&t(": %s (%s shapes)")
					.replacen("%s", &(l.name).to_string(), 1)
					.replacen("%s", &(l.shapes.len()).to_string(), 1),
			);
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
	let dialog = Dialog::builder(&frame, &t("Layers")).build();
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let list = QuietList::new(&dialog, &t("Layers"));
	sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 5);
	// The selected layer's depth role: the choice follows the
	// list selection, and choosing a role sets it at once (the
	// change lands in the log like every other layer edit).
	let depth_label = StaticText::builder(&dialog)
		.with_label(&t("Depth role of the selected layer"))
		.build();
	let depth_choice = Choice::builder(&dialog).build();
	for label in [
		t("No depth role"),
		t("Background"),
		t("Midground"),
		t("Foreground"),
		t("Sideground"),
	] {
		depth_choice.append(&label);
	}
	depth_choice.set_name(&t("Depth role of the selected layer"));
	let depth_row = BoxSizer::builder(Orientation::Horizontal).build();
	depth_row.add(&depth_label, 0, SizerFlag::All, 5);
	depth_row.add(&depth_choice, 0, SizerFlag::All, 5);
	sizer.add_sizer(&depth_row, 0, SizerFlag::All, 5);
	let add = Button::builder(&dialog)
		.with_label(&t("Add Layer\u{2026}"))
		.build();
	let rename = Button::builder(&dialog)
		.with_label(&t("Rename\u{2026}"))
		.build();
	let remove = Button::builder(&dialog).with_label(&t("Remove")).build();
	let toggle = Button::builder(&dialog)
		.with_label(&t("Show / Hide"))
		.build();
	let current = Button::builder(&dialog)
		.with_label(&t("Set Current"))
		.build();
	let close = Button::builder(&dialog).with_label(&t("Close")).build();
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
		let (list, shared, depth_choice) = (list.clone(), shared.clone(), depth_choice.clone());
		move || {
			let state = shared.borrow();
			if let Some(i) = list.selected() {
				if let Some(layer) = state.drawing.layers.get(i) {
					let at = DepthRole::ALL
						.iter()
						.position(|r| *r == layer.depth)
						.unwrap_or(0);
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
		let (list, shared, grey, sync_depth) = (
			list.clone(),
			shared.clone(),
			grey.clone(),
			sync_depth.clone(),
		);
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
		let (list, shared, sync, depth_choice) = (
			list.clone(),
			shared.clone(),
			sync.clone(),
			depth_choice.clone(),
		);
		(*depth_choice).clone().on_selection_changed(move |_| {
			let Some(i) = list.selected() else { return };
			let Some(sel) = depth_choice.get_selection() else {
				return;
			};
			let Some(role) = DepthRole::ALL.get(sel as usize) else {
				return;
			};
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
			if let Some(name) = text_dialog(
				&frame,
				&t("Add layer"),
				&t("Layer name"),
				&t("Layer %s").replacen("%s", &(next).to_string(), 1),
			) {
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
			if let Some(name) = text_dialog(&frame, &t("Rename layer"), &t("Layer name"), &old) {
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
				frame.set_status_text(&t("A drawing always keeps one layer."), 0);
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
			frame.set_status_text(
				&t("New shapes now land on layer \"%s\".").replacen("%s", &(name).to_string(), 1),
				0,
			);
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
				t("Frame %s: %s milliseconds. %s")
					.replacen("%s", &(i + 1).to_string(), 1)
					.replacen("%s", &(f.duration_ms).to_string(), 1)
					.replacen(
						"%s",
						&(state
							.drawing
							.frame_change_description(i)
							.unwrap_or_default())
						.to_string(),
						1,
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
	let dialog = Dialog::builder(&frame, &t("Animation")).build();
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let list = QuietList::new(&dialog, &t("Animation frames"));
	sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let add = Button::builder(&dialog)
		.with_label(&t("Add Frame (Copy of Canvas)"))
		.build();
	let duplicate = Button::builder(&dialog).with_label(&t("Duplicate")).build();
	let remove = Button::builder(&dialog).with_label(&t("Delete")).build();
	let up = Button::builder(&dialog).with_label(&t("Move Up")).build();
	let down = Button::builder(&dialog).with_label(&t("Move Down")).build();
	let duration = Button::builder(&dialog)
		.with_label(&t("Set Duration\u{2026}"))
		.build();
	let load = Button::builder(&dialog)
		.with_label(&t("Load into Canvas"))
		.build();
	let replace = Button::builder(&dialog)
		.with_label(&t("Replace with Canvas"))
		.build();
	let close = Button::builder(&dialog).with_label(&t("Close")).build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	for b in [
		&add, &duplicate, &remove, &up, &down, &duration, &load, &replace, &close,
	] {
		buttons.add(b, 0, SizerFlag::All, 5);
	}
	sizer.add_sizer(&buttons, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	let list = Rc::new(list);
	{
		let state = shared.borrow();
		list.update_labelled(frame_rows(&state), Select::Start);
		if state.drawing.frames.is_empty() {
			frame.set_status_text(&t("No animation frames yet. Add Frame captures the canvas as frame 1; change the canvas and capture again to build the film."),
				0,
			);
		}
	}
	let grey = grey_with_selection(
		&list,
		&[duplicate, remove, up, down, duration, load, replace],
	);
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
				frame.set_status_text(&t("That frame is already first."), 0);
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
				frame.set_status_text(&t("That frame is already last."), 0);
			}
		});
	}
	{
		let (list, shared, sync) = (list.clone(), shared.clone(), sync.clone());
		duration.on_click(move |_| {
			let Some(i) = list.selected() else { return };
			let current = {
				let state = shared.borrow();
				let Some(f) = state.drawing.frames.get(i) else {
					return;
				};
				f.duration_ms
			};
			// The range comes from the frame-duration slider
			// definition — the dialog and the Sliders tab agree
			// because there is only one source for it.
			let dur_def = slider_by_id("frame-duration").expect("the frame-duration slider exists");
			if let Some(v) = numbers_dialog(
				&frame,
				&t("Frame duration"),
				&[(
					t("Duration in milliseconds"),
					dur_def.min,
					dur_def.max,
					current as i32,
				)],
			) {
				shared
					.borrow_mut()
					.drawing
					.set_frame_duration(i, v[0] as u32);
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
				label.push_str(&t(" Caption: %s").replacen("%s", &(c).to_string(), 1));
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
	let dialog = Dialog::builder(&frame, &t("Elements")).build();
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let list = QuietList::new(&dialog, &t("Elements"));
	sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let caption_btn = Button::builder(&dialog)
		.with_label(&t("Rename Caption\u{2026}"))
		.build();
	let describe_btn = Button::builder(&dialog).with_label(&t("Describe")).build();
	let move_btn = Button::builder(&dialog)
		.with_label(&t("Move\u{2026}"))
		.build();
	let resize_btn = Button::builder(&dialog)
		.with_label(&t("Resize\u{2026}"))
		.build();
	let recolor_btn = Button::builder(&dialog)
		.with_label(&t("Recolor\u{2026}"))
		.build();
	let delete_btn = Button::builder(&dialog).with_label(&t("Delete")).build();
	let close = Button::builder(&dialog).with_label(&t("Close")).build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	for b in [
		&caption_btn,
		&describe_btn,
		&move_btn,
		&resize_btn,
		&recolor_btn,
		&delete_btn,
		&close,
	] {
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
		&[
			caption_btn,
			describe_btn,
			move_btn,
			resize_btn,
			recolor_btn,
			delete_btn,
		],
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
			if let Some(text) = text_dialog(
				&frame,
				&t("Rename caption"),
				&t("Caption for this element"),
				&now,
			) {
				if text.trim().is_empty() {
					frame.set_status_text(
						&t("Caption unchanged: an empty caption keeps the automatic description."),
						0,
					);
					return;
				}
				shared
					.borrow_mut()
					.drawing
					.caption_shape(li, si, text.trim().to_string());
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
				&t("%s. Full description: %s")
					.replacen("%s", &(state.drawing.plain_of(shape)).to_string(), 1)
					.replacen("%s", &(state.drawing.description_of(shape)).to_string(), 1),
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
				&t("Move element"),
				&[(t("New position: numbers like 120, 80, or words like top left, center, a bit right"),
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
				None => frame.set_status_text(&t("I could not understand that position. Use numbers like 120, 80, or words like top left, center, a bit right."),
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
					frame.set_status_text(&t("A stroke's size is its points; move it or add a new one."),
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
				t("New size: numbers like 200 by 100, or words like a bit wider, a bit taller"),
				t("%s by %s").replacen("%s", &(cur.0).to_string(), 1).replacen("%s", &(cur.1).to_string(), 1),
			);
			let corner_field = (
				t("Corners: 0 is sharp, 50 is fully rounded"),
				corner.map(|c| c.to_string()).unwrap_or_default(),
			);
			let mut fields: Vec<(String, &str)> = vec![(size_field.0, &size_field.1)];
			if corner.is_some() {
				fields.push((corner_field.0, &corner_field.1));
			}
			let Some(v) = fields_dialog(&frame, &t("Resize element"), &fields) else {
				return;
			};
			let Some((w, h)) = parse_size(&v[0], Some(cur)) else {
				frame.set_status_text(&t("I could not understand that size. Use numbers like 200 by 100, or words like a bit wider."),
					0,
				);
				return;
			};
			let new_corner = if corner.is_some() {
				match v[1].trim().parse::<u8>() {
					Ok(c) => Some(c.min(50)),
					Err(_) => {
						frame.set_status_text(&t("Corners take a number from 0 (sharp) to 50 (fully rounded); nothing was changed."),
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
				&t("Recolor element"),
				&[(t("New color: a name like sky blue, or a hex code"), &now.to_hex())],
			) else {
				return;
			};
			match parse_color(&v[0]) {
				Some(color) => {
					shared.borrow_mut().drawing.recolor_shape(li, si, color);
					sync();
				}
				None => frame.set_status_text(&t("I could not understand that color. Use a name like sky blue, or a hex code like 87CEEB."),
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
	let dialog = Dialog::builder(&frame, &t("Brush Library")).build();
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let list = QuietList::new(&dialog, &t("Brush library"));
	sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let apply = Button::builder(&dialog).with_label(&t("Apply")).build();
	let delete = Button::builder(&dialog).with_label(&t("Delete")).build();
	let close = Button::builder(&dialog).with_label(&t("Close")).build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	for b in [&apply, &delete, &close] {
		buttons.add(b, 0, SizerFlag::All, 5);
	}
	sizer.add_sizer(&buttons, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	let list = Rc::new(list);
	let rows = |state: &State| -> Vec<(usize, String)> {
		state
			.brushes
			.brushes
			.iter()
			.enumerate()
			.map(|(i, b)| (i, b.spoken()))
			.collect()
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
			let Some(brush) = state.brushes.brushes.get(i).cloned() else {
				return;
			};
			let spoken = brush.spoken();
			state.brush = brush;
			frame.set_status_text(
				&t("Brush applied: %s. New strokes will use it.").replacen(
					"%s",
					&(spoken).to_string(),
					1,
				),
				0,
			);
		});
	}
	{
		let (list, shared, grey) = (list.clone(), shared.clone(), grey.clone());
		delete.on_click(move |_| {
			let Some(i) = list.selected() else { return };
			let mut state = shared.borrow_mut();
			let Some(brush) = state.brushes.brushes.get(i).cloned() else {
				return;
			};
			let name = brush.name.clone();
			state.brushes.remove(&name);
			if let Err(e) = std::fs::write(brush_file_path(), state.brushes.to_text()) {
				frame.set_status_text(
					&t("The brush library could not be saved: %s").replacen(
						"%s",
						&(e).to_string(),
						1,
					),
					0,
				);
				return;
			}
			let rows_now = state
				.brushes
				.brushes
				.iter()
				.enumerate()
				.map(|(j, b)| (j, b.spoken()))
				.collect::<Vec<(usize, String)>>();
			drop(state);
			list.update_labelled(rows_now, Select::Keep);
			grey();
			frame.set_status_text(
				&t("Deleted brush \"%s\" from the library.").replacen("%s", &(name).to_string(), 1),
				0,
			);
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
	let dialog = Dialog::builder(&frame, &t("Design Terms Dictionary"))
		.with_size(560, 480)
		.build();
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let search_label = StaticText::builder(&dialog)
		.with_label(&t("Search terms"))
		.build();
	let search_field = TextCtrl::builder(&dialog).build();
	let search_button = Button::builder(&dialog).with_label(&t("Search")).build();
	let search_row = BoxSizer::builder(Orientation::Horizontal).build();
	search_row.add(&search_label, 0, SizerFlag::All, 5);
	search_row.add(&search_field, 1, SizerFlag::All, 5);
	search_row.add(&search_button, 0, SizerFlag::All, 5);
	sizer.add_sizer(&search_row, 0, SizerFlag::Expand, 0);
	let list_label = StaticText::builder(&dialog)
		.with_label(&t("Dictionary terms"))
		.build();
	sizer.add(
		&list_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let list = QuietList::<usize>::new(&dialog, &t("Dictionary terms"));
	sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let list = Rc::new(list);
	let def_label = StaticText::builder(&dialog)
		.with_label(&t("Definition"))
		.build();
	sizer.add(
		&def_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let definition = TextCtrl::builder(&dialog)
		.with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::ReadOnly)
		.build();
	sizer.add(&definition, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let close = Button::builder(&dialog).with_label(&t("Close")).build();
	sizer.add(&close, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);

	let rows = |query: &str| -> Vec<(usize, String)> {
		dictionary::search(query)
			.iter()
			.enumerate()
			.map(|(i, en)| (i, en.display_term()))
			.collect()
	};
	let show_selected = {
		let (list, definition, search_field) =
			(list.clone(), definition.clone(), search_field.clone());
		move || {
			let hits = dictionary::search(&search_field.get_value());
			if let Some(i) = list.selected() {
				if let Some(en) = hits.get(i) {
					definition.set_value(&format!(
						"{}: {}",
						en.display_term(),
						en.display_definition()
					));
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
					definition.set_value(&format!(
						"{}: {}",
						en.display_term(),
						en.display_definition()
					));
					frame.set_status_text(
						&t("%s dictionary terms match.").replacen(
							"%s",
							&(hits.len()).to_string(),
							1,
						),
						0,
					);
				}
				None => {
					definition.set_value(&t("No term matches that search."));
					frame.set_status_text(&t("No dictionary term matches that search."), 0);
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
	let dialog = Dialog::builder(&frame, &t("Help and Worked Examples"))
		.with_size(640, 520)
		.build();
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let label = StaticText::builder(&dialog)
		.with_label(&t("Help and worked examples"))
		.build();
	sizer.add(
		&label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let text = TextCtrl::builder(&dialog)
		.with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::ReadOnly)
		.build();
	text.set_value(HELP_TEXT);
	sizer.add(&text, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let close = Button::builder(&dialog).with_label(&t("Close")).build();
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
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let caption = StaticText::builder(&dialog).with_label(label).build();
	sizer.add(
		&caption,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let body = TextCtrl::builder(&dialog)
		.with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::ReadOnly)
		.build();
	body.set_value(text);
	sizer.add(&body, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let close = Button::builder(&dialog).with_label(&t("Close")).build();
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
	let Some(flat) = win.elements_list.borrow().selected() else {
		return;
	};
	let state = shared.borrow();
	if let Some(text) = inspect_shape(&state.drawing, flat) {
		report_dialog(
			win,
			&t("Inspector: element"),
			&t("Element inspector"),
			&text,
		);
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
		ctx(&menu, ID_INSPECT_COLOR, &t("Inspect Color…"));
		ctx(&menu, ID_SET_BACKGROUND, &t("Set as Background"));
		ctx(&menu, ID_REMOVE_COLOR, &t("Remove Color"));
		menu.append_separator();
	}
	ctx(&menu, ID_ADD_COLOR, &t("Add Color by Hex…"));
	ctx(&menu, ID_LOAD_STARTER, &t("Load Starter Colors"));
	ctx(&menu, ID_OPEN_IMAGE, &t("Extract from Image…"));
	win.palette_list.borrow().popup_menu(&mut menu);
}

/// The Elements list's context menu: with an element selected,
/// its actions (inspect and lock first — the two this wave
/// adds); empty or not, the Add items are always there, so an
/// empty list can start the design from its own menu.
fn show_elements_context(win: &Rc<Win>) {
	let mut menu = Menu::builder().build();
	if win.elements_list.borrow().selected().is_some() {
		ctx(&menu, ID_INSPECT, &t("Inspect Selected…"));
		ctx(&menu, ID_ELEMENT_WIZARD, &t("Element Wizard…"));
		ctx(&menu, ID_LOCK_TOGGLE, &t("Lock/Unlock Selected"));
		ctx(&menu, ID_MOVE_SHAPE, &t("Move…"));
		ctx(&menu, ID_ROTATE_SHAPE, &t("Rotate…"));
		ctx(&menu, ID_RESIZE_SHAPE, &t("Resize…"));
		ctx(&menu, ID_RECOLOR_SHAPE, &t("Set to Palette Color"));
		ctx(&menu, ID_REMOVE_SHAPE, &t("Remove Shape"));
		menu.append_separator();
	}
	ctx(&menu, ID_ADD_RECT, &t("Add Rectangle…"));
	ctx(&menu, ID_ADD_ELLIPSE, &t("Add Ellipse…"));
	ctx(&menu, ID_ADD_STROKE, &t("Add Stroke…"));
	ctx(&menu, ID_ADD_TEXT, &t("Add Text…"));
	ctx(&menu, ID_ADD_IMAGE, &t("Add Image…"));
	ctx(&menu, ID_ADD_SYMBOL, &t("Add Symbol…"));
	win.elements_list.borrow().popup_menu(&mut menu);
}

/// The Layers list's context menu: inspect the selected layer,
/// or open the Layers dialog (where layers are added, renamed
/// and reordered). A design always has at least one layer, so
/// this list is never empty.
fn show_layers_context(win: &Rc<Win>) {
	let mut menu = Menu::builder().build();
	if win.layers_list.selected().is_some() {
		ctx(&menu, ID_INSPECT_LAYER, &t("Inspect Layer…"));
	}
	ctx(&menu, ID_LAYERS, &t("Layers…"));
	win.layers_list.popup_menu(&mut menu);
}

/// The Sliders page's context menu: the pins live in Project
/// Options, so that is the menu — add or remove sliders right
/// from the area they would appear in.
fn show_sliders_context(win: &Rc<Win>) {
	let mut menu = Menu::builder().build();
	ctx(
		&menu,
		ID_PROJECT_OPTIONS,
		&t("Project Options… (choose pinned sliders)"),
	);
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
			Some(
				t("%s is now the brush color: the next element drawn uses it.").replacen(
					"%s",
					&(describe_in(color, format)).to_string(),
					1,
				),
			)
		}
		CatalogHit::StarterColor { color, .. } => {
			let mut state = shared.borrow_mut();
			let already = state.palette.iter().any(|pc| pc.color == *color);
			if !already {
				state
					.palette
					.push(PaletteColor::new(*color, Provenance::Starter));
			}
			state.brush.color = *color;
			let format = state.color_format;
			drop(state);
			let state = shared.borrow();
			refresh_lists(win, &state);
			select_color(win, &state, *color);
			Some(
				t("%s %s the palette and is now the brush color.")
					.replacen("%s", &(describe_in(*color, format)).to_string(), 1)
					.replacen(
						"%s",
						if already {
							"was already in"
						} else {
							"was added to"
						},
						1,
					),
			)
		}
		CatalogHit::Element { layer, index, .. } => {
			let mut state = shared.borrow_mut();
			let source = state
				.drawing
				.layers
				.get(*layer)?
				.shapes
				.get(*index)?
				.clone();
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
					copy.x = (copy.x + 10).min((state.drawing.width as i32 - copy.w as i32).max(0));
					copy.y =
						(copy.y + 10).min((state.drawing.height as i32 - copy.h as i32).max(0));
					let layer_name = state.drawing.layers[sli].name.clone();
					state.drawing.insert_shape(sli, ssi + 1, copy);
					t("on layer \"%s\", right after the selected element").replacen(
						"%s",
						&(layer_name).to_string(),
						1,
					)
				}
				None => {
					let mut copy = source;
					copy.x = ((state.drawing.width as i32 - copy.w as i32) / 2).max(0);
					copy.y = ((state.drawing.height as i32 - copy.h as i32) / 2).max(0);
					let li = state.layer.min(state.drawing.layers.len() - 1);
					let layer_name = state.drawing.layers[li].name.clone();
					state.drawing.add_shape(li, copy);
					t("at the centre of the canvas on layer \"%s\"").replacen(
						"%s",
						&(layer_name).to_string(),
						1,
					)
				}
			};
			drop(state);
			let state = shared.borrow();
			refresh_lists(win, &state);
			Some(
				t("Copied the %s %s.")
					.replacen("%s", &(words).to_string(), 1)
					.replacen("%s", &(note).to_string(), 1),
			)
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
				shape.caption = Some(t("%s symbol").replacen("%s", (name).as_ref(), 1));
				state.drawing.add_shape(li, shape);
			}
			drop(state);
			let state = shared.borrow();
			refresh_lists(win, &state);
			Some(t("The %s symbol was stamped at the centre of the canvas on layer \"%s\", in the brush color.").replacen("%s", (name).as_ref(), 1).replacen("%s", &(layer_name).to_string(), 1))
		}
		CatalogHit::Template { name, .. } => {
			let design = templates::template(name)?;
			let mut state = shared.borrow_mut();
			state.drawing = design;
			state.layer = 0;
			state.pinned_sliders = Vec::new();
			state.slider_target = AdjustTarget::BaseColor;
			state.page_jump_percent = DEFAULT_PAGE_JUMP_PERCENT;
			state.interaction_mode = InteractionMode::Slide;
			reset_adjustment(&mut state);
			drop(state);
			apply_interaction_mode(win, shared, true);
			let state = shared.borrow();
			refresh_lists(win, &state);
			Some(t("The design is now the %s template. It is a normal design now: edit it, then Save Design to keep it.").replacen("%s", (name).as_ref(), 1))
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
	let dialog = Dialog::builder(&frame, &t("Search and Insert"))
		.with_size(620, 560)
		.build();
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let search_label = StaticText::builder(&dialog)
		.with_label(&t("Search for"))
		.build();
	let field = TextCtrl::builder(&dialog).build();
	let search_button = Button::builder(&dialog).with_label(&t("Search")).build();
	let search_row = BoxSizer::builder(Orientation::Horizontal).build();
	search_row.add(&search_label, 0, SizerFlag::All, 5);
	search_row.add(&field, 1, SizerFlag::All, 5);
	search_row.add(&search_button, 0, SizerFlag::All, 5);
	sizer.add_sizer(&search_row, 0, SizerFlag::Expand, 0);
	let list_label = StaticText::builder(&dialog)
		.with_label(&t("Results"))
		.build();
	sizer.add(
		&list_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let list = QuietList::<usize>::new(&dialog, &t("Results"));
	sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let insert = Button::builder(&dialog).with_label(&t("Insert")).build();
	let close = Button::builder(&dialog).with_label(&t("Close")).build();
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
				current_project(&state)
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
		let (list, hits, dialog, win, shared) = (
			list.clone(),
			hits.clone(),
			dialog.clone(),
			win.clone(),
			shared.clone(),
		);
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
				report_dialog(&win, &t("Inserted"), &t("What was inserted"), &note);
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
			let WindowEventData::Keyboard(kb) = data else {
				return false;
			};
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
			frame.set_status_text(&t("A pattern is woven from two palette colors, and this project has none yet: add a color or load the starter colors first."),
				0,
			);
			return;
		}
		(
			state
				.palette
				.iter()
				.map(|pc| pc.spoken(state.color_format))
				.collect::<Vec<_>>(),
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
	let dialog = Dialog::builder(&frame, &t("New from Pattern")).build();
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let kind_label = StaticText::builder(&dialog)
		.with_label(&t("Pattern"))
		.build();
	sizer.add(
		&kind_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let kinds = Choice::builder(&dialog).build();
	for name in templates::PATTERN_NAMES {
		kinds.append(&t(name));
	}
	kinds.set_selection(
		preselect
			.and_then(|p| templates::PATTERN_NAMES.iter().position(|n| *n == p))
			.unwrap_or(0) as u32,
	);
	sizer.add(&kinds, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let first_label = StaticText::builder(&dialog)
		.with_label(&t("First color (the marks)"))
		.build();
	sizer.add(
		&first_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let first = Choice::builder(&dialog).build();
	for row in &names {
		first.append(row);
	}
	first.set_selection(0);
	sizer.add(&first, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let second_label = StaticText::builder(&dialog)
		.with_label(&t("Second color (the ground)"))
		.build();
	sizer.add(
		&second_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let second = Choice::builder(&dialog).build();
	for row in &names {
		second.append(row);
	}
	second.set_selection(ground_default as u32);
	sizer.add(&second, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let start = Button::builder(&dialog)
		.with_label(&t("Start Pattern Design"))
		.build();
	let close = Button::builder(&dialog).with_label(&t("Close")).build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	buttons.add(&start, 0, SizerFlag::All, 5);
	buttons.add(&close, 0, SizerFlag::All, 5);
	sizer.add_sizer(&buttons, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	{
		let (dialog, shared, w) = (dialog.clone(), shared.clone(), win.clone());
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
			state.goal = None;
			state.grayscale_start = true;
			state.interaction_mode = InteractionMode::Slide;
			state.design_path = None;
			reset_adjustment(&mut state);
			// The fresh pattern design is the saved baseline.
			let fp = project_fingerprint(&state);
			state.saved_fingerprint = fp;
			drop(state);
			apply_interaction_mode(&w, &shared, true);
			frame.set_status_text(
				&t("Started a new %s design from the two colors. It is a normal design now: edit it, then Save Design to keep it.").replacen("%s", kind, 1),
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
	let dialog = Dialog::builder(&frame, &t("Viewability")).build();
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let angle_label = StaticText::builder(&dialog)
		.with_label(&t(
			"Viewing angle in degrees (0 is straight on from the front)",
		))
		.build();
	let angle_spin = SpinCtrl::builder(&dialog).with_range(0, 359).build();
	angle_spin.set_value(0);
	sizer.add(
		&angle_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	sizer.add(&angle_spin, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let band_label = StaticText::builder(&dialog)
		.with_label(&t("Depth to judge"))
		.build();
	let band_choice = Choice::builder(&dialog).build();
	for band in DepthBand::ALL {
		band_choice.append(&band.name());
	}
	band_choice.set_selection(0);
	band_choice.set_name(&t("Depth to judge"));
	sizer.add(
		&band_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	sizer.add(&band_choice, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let ok = Button::builder(&dialog).with_label(&t("OK")).build();
	let cancel = Button::builder(&dialog).with_label(&t("Cancel")).build();
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
			&t("Viewability"),
			&t("How the design reads from %s degrees").replacen("%s", &(angle).to_string(), 1),
			&state.drawing.viewability_report(angle as f64, band),
		);
	}
}

/// The wizard frame: one reusable step dialog every wizard in
/// the app is built on (Elshara's wizard frame — the same
/// controls, invocable at any step of the process, adaptively
/// scoped to whatever is in focus). A wizard is a header that
/// always says where you are ("Step 2 of 4: ..."), one body
/// panel per step — only the current step's panel is shown, so
/// Tab walks just this step's controls — and Back, Next, Finish
/// and Cancel. Finish is offered on every step: a wizard may be
/// finished as soon as it has what it needs. Callers build each
/// step's body into the panel `add_step` hands back, and may
/// register an `on_show` hook to refresh a step's body from the
/// steps before it each time it becomes current.
struct Wizard {
	dialog: Dialog,
	header: StaticText,
	sizer: BoxSizer,
	/// The Back/Next/Finish/Cancel row: built at construction,
	/// added to the sizer only when the wizard runs, so it sits
	/// under the step panels instead of between them and the
	/// header.
	buttons: BoxSizer,
	built: std::cell::Cell<bool>,
	steps: RefCell<Vec<(String, Panel)>>,
	current: std::cell::Cell<usize>,
	back_btn: Button,
	next_btn: Button,
	on_show: RefCell<Option<Rc<dyn Fn(usize)>>>,
}

impl Wizard {
	fn new(frame: &Frame, title: &str) -> Rc<Wizard> {
		let dialog = Dialog::builder(frame, title).with_size(680, 560).build();
		bind_escape_cancel(dialog);
		let sizer = BoxSizer::builder(Orientation::Vertical).build();
		let header = StaticText::builder(&dialog)
			.with_label(&t("Step 1 of 1"))
			.build();
		sizer.add(
			&header,
			0,
			SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
			5,
		);
		let back_btn = Button::builder(&dialog).with_label(&t("Back")).build();
		let next_btn = Button::builder(&dialog).with_label(&t("Next")).build();
		let finish_btn = Button::builder(&dialog).with_label(&t("Finish")).build();
		let cancel_btn = Button::builder(&dialog).with_label(&t("Cancel")).build();
		let buttons = BoxSizer::builder(Orientation::Horizontal).build();
		buttons.add(&back_btn, 0, SizerFlag::All, 5);
		buttons.add(&next_btn, 0, SizerFlag::All, 5);
		buttons.add(&finish_btn, 0, SizerFlag::All, 5);
		buttons.add(&cancel_btn, 0, SizerFlag::All, 5);
		dialog.set_sizer(sizer, true);
		let wiz = Rc::new(Wizard {
			dialog,
			header,
			sizer,
			buttons,
			built: std::cell::Cell::new(false),
			steps: RefCell::new(Vec::new()),
			current: std::cell::Cell::new(0),
			back_btn,
			next_btn,
			on_show: RefCell::new(None),
		});
		{
			let w = wiz.clone();
			wiz.back_btn.on_click(move |_| {
				let i = w.current.get();
				if i > 0 {
					w.show_step(i - 1);
				}
			});
		}
		{
			let w = wiz.clone();
			wiz.next_btn.on_click(move |_| {
				let i = w.current.get();
				if i + 1 < w.steps.borrow().len() {
					w.show_step(i + 1);
				}
			});
		}
		{
			let d = wiz.dialog.clone();
			finish_btn.on_click(move |_| d.end_modal(ID_OK));
		}
		{
			let d = wiz.dialog.clone();
			cancel_btn.on_click(move |_| d.end_modal(ID_CANCEL));
		}
		wiz
	}

	/// Add a step and hand back its body panel: build the step's
	/// controls into it (with its own sizer, set on the panel).
	/// The first step starts visible; the rest start hidden.
	fn add_step(self: &Rc<Self>, title: &str) -> Panel {
		let panel = Panel::builder(&self.dialog).build();
		let first = self.steps.borrow().is_empty();
		self.sizer
			.add(&panel, 1, SizerFlag::Expand | SizerFlag::All, 5);
		panel.show(first);
		self.steps.borrow_mut().push((title.to_string(), panel));
		self.refresh_chrome();
		panel
	}

	/// The step-change hook: called with the new step's index
	/// each time a step becomes current, so a step's body can be
	/// built from what the earlier steps settled.
	fn set_on_show(self: &Rc<Self>, hook: impl Fn(usize) + 'static) {
		*self.on_show.borrow_mut() = Some(Rc::new(hook));
	}

	fn refresh_chrome(&self) {
		let steps = self.steps.borrow();
		if steps.is_empty() {
			return;
		}
		let i = self.current.get().min(steps.len() - 1);
		self.header.set_label(
			&t("Step %s of %s: %s")
				.replacen("%s", &(i + 1).to_string(), 1)
				.replacen("%s", &(steps.len()).to_string(), 1)
				.replacen("%s", &(steps[i].0).to_string(), 1),
		);
		self.back_btn.enable(i > 0);
		self.next_btn.enable(i + 1 < steps.len());
	}

	fn show_step(&self, i: usize) {
		{
			let steps = self.steps.borrow();
			for (j, (_, panel)) in steps.iter().enumerate() {
				panel.show(j == i);
			}
		}
		self.current.set(i);
		self.refresh_chrome();
		if let Some(hook) = self.on_show.borrow().clone() {
			hook(i);
		}
		self.dialog.layout();
	}

	/// Run the wizard; true when it was finished, false when
	/// cancelled.
	fn run(&self) -> bool {
		if !self.built.get() {
			self.built.set(true);
			self.sizer.add_sizer(&self.buttons, 0, SizerFlag::All, 5);
			self.dialog.layout();
		}
		self.dialog.show_modal() == ID_OK
	}
}

/// The body panel's own vertical sizer, the shape every wizard
/// step uses: a word label first (screen readers find it), then
/// the step's controls.
fn step_sizer(panel: &Panel) -> BoxSizer {
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	panel.set_sizer(sizer, true);
	sizer
}

/// The Setup Wizard (File, New with Wizard — a new project; and
/// Design, Setup Wizard — restate the current project's goal).
/// Step 1 asks Elshara's first question — what do you want to
/// create or build? — and the kind chosen decides which focus
/// questions step 2 asks; the tools that matter follow the
/// focus, and the answers ARE the goal, confirmed on the last
/// step with the goal report's preview. Finishing also pins
/// the kind's default sliders (see `default_slider_ids`) — a
/// new project's set outright, a restated goal's as a top-up —
/// and the status line names what the wizard did. Finishing a
/// new project seeds its canvas per the kind (see
/// `seed_drawing`): writing
/// starts with words to replace, image creation with a tagged
/// background layer and a grey ground, cosmetic work with the
/// gray ramp in the palette (its work starts from a picture
/// brought in). Finishing on an existing project only states
/// (or restates) its goal.
fn setup_wizard(win: &Rc<Win>, shared: &Shared, new_project: bool) {
	let frame = win.frame;
	let wiz = Wizard::new(
		&frame,
		&(if new_project {
			t("New Project Wizard")
		} else {
			t("Setup Wizard")
		}),
	);
	// Step 1: the kind.
	let p1 = wiz.add_step(&t("What do you want to create or build?"));
	let s1 = step_sizer(&p1);
	let kind_label = StaticText::builder(&p1)
		.with_label(&t("What do you want to create or build?"))
		.build();
	s1.add(
		&kind_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let kind_list = QuietList::<usize>::new(&p1, &t("What do you want to create or build"));
	s1.add(&kind_list, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let mode_note = StaticText::builder(&p1)
		.with_label(if new_project {
			"A new project starts in Slide mode: the sliders are the surface, and the typed fields stay away. Switch the interaction mode any time in Design, Project Options."
		} else {
			"This project's interaction mode — Draw, Slide or Type, one surface at a time — is switched in Design, Project Options."
		})
		.build();
	s1.add(
		&mode_note,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let kind_list = Rc::new(kind_list);
	kind_list.update_labelled(
		GoalKind::ALL
			.iter()
			.enumerate()
			.map(|(i, k)| (i, k.name().to_string()))
			.collect(),
		Select::Start,
	);
	let kind_sel = Rc::new(std::cell::Cell::new(
		shared
			.borrow()
			.goal
			.as_ref()
			.map(|g| g.kind)
			.unwrap_or(GoalKind::Writing),
	));
	// Preselect the current goal's kind when restating one.
	if !new_project {
		if let Some(pos) = GoalKind::ALL.iter().position(|k| *k == kind_sel.get()) {
			kind_list.update_labelled(
				GoalKind::ALL
					.iter()
					.enumerate()
					.map(|(i, k)| (i, k.name().to_string()))
					.collect(),
				Select::Key(pos),
			);
		}
	}
	{
		let kind_sel = kind_sel.clone();
		let list = kind_list.clone();
		kind_list.on_selection_changed(move || {
			if let Some(i) = list.selected() {
				kind_sel.set(GoalKind::ALL[i]);
			}
		});
	}
	// Step 2: the focus questions of the chosen kind — up to
	// four rows, relabelled each time the step is shown, each
	// answer a choice whose last entry is "No fixed answer".
	let p2 = wiz.add_step(&t("Focus questions"));
	let s2 = step_sizer(&p2);
	let mut q_labels: Vec<StaticText> = Vec::new();
	let mut q_choices: Vec<Choice> = Vec::new();
	for _ in 0..4 {
		let label = StaticText::builder(&p2).with_label(&t("")).build();
		let choice = Choice::builder(&p2).build();
		s2.add(
			&label,
			0,
			SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
			5,
		);
		s2.add(&choice, 0, SizerFlag::Expand | SizerFlag::All, 5);
		q_labels.push(label);
		q_choices.push(choice);
	}
	// The answers being built: one slot per question row, and
	// the kind they belong to (changing kind restarts them).
	let answers: Rc<RefCell<Vec<Option<String>>>> = Rc::new(RefCell::new(vec![None; 4]));
	let answers_kind: Rc<std::cell::Cell<Option<GoalKind>>> = Rc::new(std::cell::Cell::new(None));
	// Preload a restated goal's answers on its first showing.
	if !new_project {
		if let Some(goal) = &shared.borrow().goal {
			let mut slot = answers.borrow_mut();
			for (i, (_, a)) in goal.answers.iter().enumerate().take(4) {
				slot[i] = a.clone();
			}
			answers_kind.set(Some(goal.kind));
		}
	}
	let collect_answers: Rc<dyn Fn()> = {
		let (answers, q_choices) = (answers.clone(), q_choices.clone());
		let kind_sel = kind_sel.clone();
		Rc::new(move || {
			let kind = kind_sel.get();
			let questions = kind.questions();
			let mut slot = answers.borrow_mut();
			for (i, choice) in q_choices.iter().enumerate() {
				let Some(q) = questions.get(i) else { break };
				// A row the focus step never populated (the
				// wizard was finished from another step) has no
				// choices at all: leave its stored answer as it
				// was rather than erasing it.
				if choice.get_count() == 0 {
					continue;
				}
				slot[i] = choice
					.get_selection()
					.and_then(|sel| q.options.get(sel as usize).cloned())
					.map(|s| s.to_string());
			}
		})
	};
	// Step 3: the goal, confirmed — the report preview.
	let p3 = wiz.add_step(&t("Your goal"));
	let s3 = step_sizer(&p3);
	let goal_label = StaticText::builder(&p3).with_label(&t("Your goal")).build();
	s3.add(
		&goal_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let goal_body = TextCtrl::builder(&p3)
		.with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::ReadOnly)
		.build();
	s3.add(&goal_body, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let build_goal: Rc<dyn Fn() -> ProjectGoal> = {
		let (kind_sel, answers, answers_kind) =
			(kind_sel.clone(), answers.clone(), answers_kind.clone());
		Rc::new(move || {
			let kind = kind_sel.get();
			let slot = answers.borrow();
			// The stored answers belong to the kind they were
			// collected under; a kind chosen but never taken
			// through the focus step starts with them all open.
			let usable = answers_kind.get() == Some(kind);
			ProjectGoal {
				kind,
				answers: kind
					.questions()
					.iter()
					.enumerate()
					.map(|(i, q)| {
						(
							q.question.to_string(),
							if usable {
								slot.get(i).cloned().flatten()
							} else {
								None
							},
						)
					})
					.collect(),
			}
		})
	};
	{
		let (q_labels, q_choices, answers, answers_kind, kind_sel, collect) = (
			q_labels.clone(),
			q_choices.clone(),
			answers.clone(),
			answers_kind.clone(),
			kind_sel.clone(),
			collect_answers.clone(),
		);
		let (goal_body, build_goal, shared) =
			(goal_body.clone(), build_goal.clone(), shared.clone());
		wiz.set_on_show(move |step| match step {
			1 => {
				let kind = kind_sel.get();
				if answers_kind.get() != Some(kind) {
					*answers.borrow_mut() = vec![None; 4];
					answers_kind.set(Some(kind));
				}
				let slot = answers.borrow();
				for (i, (label, choice)) in q_labels.iter().zip(q_choices.iter()).enumerate() {
					match kind.questions().get(i) {
						Some(q) => {
							label.set_label(&q.question);
							label.show(true);
							choice.show(true);
							choice.clear();
							for opt in &q.options {
								choice.append(opt);
							}
							choice.append(&t("No fixed answer"));
							let sel = slot
								.get(i)
								.cloned()
								.flatten()
								.and_then(|a| q.options.iter().position(|o| *o == a))
								.unwrap_or(q.options.len());
							choice.set_selection(sel as u32);
							choice.set_name(&q.question);
						}
						None => {
							label.set_label("");
							label.show(false);
							choice.show(false);
						}
					}
				}
			}
			2 => {
				collect();
				let goal = build_goal();
				let preview = {
					let state = shared.borrow();
					let mut project = if new_project {
						let mut p = current_project(&state);
						p.drawing = seed_drawing(goal.kind);
						p.palette = if goal.kind == GoalKind::ImageCosmetic {
							gray_ramp()
						} else {
							Vec::new()
						};
						p
					} else {
						current_project(&state)
					};
					project.goal = Some(goal.clone());
					project
				};
				let text = match goal_report(&preview) {
					Some(rep) => rep.full_text(&goal),
					None => t("No goal to report."),
				};
				goal_body.set_value(&text);
			}
			_ => {}
		});
	}
	if !wiz.run() {
		return;
	}
	collect_answers();
	let goal = build_goal();
	let mut state = shared.borrow_mut();
	if new_project {
		state.drawing = seed_drawing(goal.kind);
		state.palette = if goal.kind == GoalKind::ImageCosmetic {
			gray_ramp()
		} else {
			Vec::new()
		};
		state.layer = 0;
		// The wizard stocks the Sliders tab for the kind: a
		// new project starts with the sliders its kind of work
		// reaches for first (see `default_slider_ids`), default
		// target and jump, grayscale start on.
		state.pinned_sliders = default_slider_ids(goal.kind)
			.iter()
			.map(|s| s.to_string())
			.collect();
		state.slider_target = AdjustTarget::BaseColor;
		state.page_jump_percent = DEFAULT_PAGE_JUMP_PERCENT;
		state.grayscale_start = true;
		// A new project starts on the sliders (Elshara's
		// principle: what can be slid is not typed).
		state.interaction_mode = InteractionMode::Slide;
		state.design_path = None;
		reset_adjustment(&mut state);
	} else {
		// Restating a goal tops the pins up with the kind's
		// defaults, keeping every slider already chosen.
		for id in default_slider_ids(goal.kind) {
			if !state.pinned_sliders.iter().any(|p| p == id) {
				state.pinned_sliders.push(id.to_string());
			}
		}
	}
	state.goal = Some(goal.clone());
	let pinned_count = state.pinned_sliders.len();
	let report = goal_report(&current_project(&state));
	if new_project {
		// The wizard's fresh project is the saved baseline:
		// nothing to lose until the designer changes it.
		let fp = project_fingerprint(&state);
		state.saved_fingerprint = fp;
	}
	drop(state);
	// The pins may have changed either way, and a new project
	// starts in Slide mode: apply the mode, which rebuilds
	// the tab when the mode shows it.
	apply_interaction_mode(win, shared, true);
	let state = shared.borrow();
	refresh_lists(win, &state);
	frame.set_status_text(
		&t("%s: goal set — %s.%s %s slider%s pinned for this kind of project.%s%s").replacen("%s", if new_project { "New project" } else { "Setup wizard" }, 1).replacen("%s", &(goal.kind.name()).to_string(), 1).replacen("%s", &(report.map(|r| format!(" {}", r.one_line())).unwrap_or_default()).to_string(), 1).replacen("%s", &(pinned_count).to_string(), 1).replacen("%s", if pinned_count == 1 { "" } else { "s" }, 1).replacen("%s", if new_project { " Canvas seeded for the kind." } else { "" }, 1).replacen("%s", if new_project && goal.kind == GoalKind::ImageCosmetic {
				" Cosmetic work starts from a picture: File, Import, Open Image — the gray ramp is in the palette for the shading work."
			} else if new_project {
				" Grayscale start is on: colors come later, per part, layer, or depth."
			} else {
				""
			}, 1),
		0,
	);
}

/// The Element Wizard (Design, Element Wizard; the Elements
/// list's context menu): the wizard frame scoped to the one
/// element in focus. Step 1 is its stack — what sits on top of
/// it and what it sits on, and why (the viewability core's box
/// maths). Step 2 edits it with sliders (opacity, and
/// roundedness for a rectangle). Step 3 reviews it with the
/// inspector's report — by then the edits are applied, so the
/// review describes the element as it now is.
fn element_wizard(win: &Rc<Win>, shared: &Shared) {
	let frame = win.frame;
	let Some((li, si)) = selected_element_locate(win, shared) else {
		frame.set_status_text(&t("Select an element first."), 0);
		return;
	};
	let flat = {
		let state = shared.borrow();
		state.drawing.layers[..li]
			.iter()
			.map(|l| l.shapes.len())
			.sum::<usize>()
			+ si
	};
	let (stack_text, kind, opacity, corner) = {
		let state = shared.borrow();
		let shape = &state.drawing.layers[li].shapes[si];
		(
			state.drawing.stack_report(flat).unwrap_or_default(),
			shape.kind,
			shape.opacity,
			shape.corner,
		)
	};
	let wiz = Wizard::new(&frame, &t("Element Wizard"));
	// Step 1: the stack.
	let p1 = wiz.add_step(&t("Stack: what sits on what"));
	let s1 = step_sizer(&p1);
	let stack_label = StaticText::builder(&p1).with_label(&t("The stack")).build();
	s1.add(
		&stack_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let stack_body = TextCtrl::builder(&p1)
		.with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::ReadOnly)
		.build();
	stack_body.set_value(&stack_text);
	s1.add(&stack_body, 1, SizerFlag::Expand | SizerFlag::All, 5);
	// Step 2: edit with sliders. The sliders hold pending
	// values; they are applied when the review step is reached
	// or the wizard is finished, never mid-wizard on Cancel.
	let p2 = wiz.add_step(&t("Edit"));
	let s2 = step_sizer(&p2);
	let pending_opacity = Rc::new(std::cell::Cell::new(opacity));
	let pending_corner = Rc::new(std::cell::Cell::new(corner));
	let op_label = StaticText::builder(&p2).with_label(&t("Opacity")).build();
	let op_slider = Slider::builder(&p2)
		.with_min_value(0)
		.with_max_value(100)
		.with_value(opacity as i32)
		.build();
	op_slider.set_name(&t("Opacity"));
	let op_readout = StaticText::builder(&p2)
		.with_label(&t("%s percent opaque").replacen("%s", &(opacity).to_string(), 1))
		.build();
	s2.add(
		&op_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let op_row = BoxSizer::builder(Orientation::Horizontal).build();
	op_row.add(&op_slider, 1, SizerFlag::Expand | SizerFlag::All, 5);
	op_row.add(&op_readout, 0, SizerFlag::All, 5);
	s2.add_sizer(&op_row, 0, SizerFlag::Expand, 0);
	{
		let (pending, readout) = (pending_opacity.clone(), op_readout.clone());
		op_slider.on_slider(move |e| {
			pending.set(e.get_value().clamp(0, 100) as u8);
			readout.set_label(&t("%s percent opaque").replacen(
				"%s",
				&(pending.get()).to_string(),
				1,
			));
		});
	}
	if kind == ShapeKind::Rectangle {
		let c_label = StaticText::builder(&p2)
			.with_label(&t("Roundedness"))
			.build();
		let c_slider = Slider::builder(&p2)
			.with_min_value(0)
			.with_max_value(50)
			.with_value(corner as i32)
			.build();
		c_slider.set_name(&t("Roundedness"));
		let c_readout = StaticText::builder(&p2)
			.with_label(&t("corners %s percent").replacen("%s", &(corner).to_string(), 1))
			.build();
		s2.add(
			&c_label,
			0,
			SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
			5,
		);
		let c_row = BoxSizer::builder(Orientation::Horizontal).build();
		c_row.add(&c_slider, 1, SizerFlag::Expand | SizerFlag::All, 5);
		c_row.add(&c_readout, 0, SizerFlag::All, 5);
		s2.add_sizer(&c_row, 0, SizerFlag::Expand, 0);
		let (pending, readout) = (pending_corner.clone(), c_readout.clone());
		c_slider.on_slider(move |e| {
			pending.set(e.get_value().clamp(0, 50) as u8);
			readout.set_label(&t("corners %s percent").replacen(
				"%s",
				&(pending.get()).to_string(),
				1,
			));
		});
	} else {
		let note = StaticText::builder(&p2)
			.with_label(&t("Roundedness applies to rectangles; this is a %s, so opacity is its only slider here.").replacen("%s", &(kind.name()).to_string(), 1))
			.build();
		s2.add(&note, 0, SizerFlag::All, 5);
	}
	// Step 3: review with the inspector.
	let p3 = wiz.add_step(&t("Review"));
	let s3 = step_sizer(&p3);
	let review_label = StaticText::builder(&p3).with_label(&t("Review")).build();
	s3.add(
		&review_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let review_body = TextCtrl::builder(&p3)
		.with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::ReadOnly)
		.build();
	s3.add(&review_body, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let applied = Rc::new(std::cell::Cell::new(false));
	let apply_edits: Rc<dyn Fn()> = {
		let (shared, win) = (shared.clone(), win.clone());
		let (po, pc, applied) = (
			pending_opacity.clone(),
			pending_corner.clone(),
			applied.clone(),
		);
		Rc::new(move || {
			if applied.get() {
				return;
			}
			applied.set(true);
			let mut state = shared.borrow_mut();
			let mut changed = false;
			if state.drawing.layers[li].shapes[si].opacity != po.get() {
				changed |= state.drawing.set_shape_opacity(li, si, po.get());
			}
			if kind == ShapeKind::Rectangle
				&& state.drawing.layers[li].shapes[si].corner != pc.get()
			{
				changed |= state.drawing.set_shape_corner(li, si, pc.get());
			}
			if changed {
				refresh_lists(&win, &state);
				announce(&win, &state);
			}
		})
	};
	{
		let (apply, review_body, shared) =
			(apply_edits.clone(), review_body.clone(), shared.clone());
		wiz.set_on_show(move |step| {
			if step == 2 {
				apply();
				let state = shared.borrow();
				if let Some(text) = inspect_shape(&state.drawing, flat) {
					review_body.set_value(&text);
				}
			}
		});
	}
	if wiz.run() {
		apply_edits();
		frame.set_status_text(
			&t("Element wizard finished: the element's opacity is now %s percent opaque%s.")
				.replacen("%s", &(pending_opacity.get()).to_string(), 1)
				.replacen(
					"%s",
					&(if kind == ShapeKind::Rectangle {
						format!(", its roundedness {} percent", pending_corner.get())
					} else {
						String::new()
					})
					.to_string(),
					1,
				),
			0,
		);
	}
}

/// The Export Wizard (File, Export, Export Wizard): the wizard
/// frame over the exporters. Step 1 chooses the format, step 2
/// how the result is embedded or accompanied, step 3 confirms
/// in words what will be written — then Finish asks for the
/// main file's name and writes. Embedding honestly means what
/// each format can do: a picture inlined in a page gets an HTML
/// companion that shows it; "with caption text" writes the
/// caption sheet beside the main file; a description is words
/// already, so embedding changes nothing and the confirm step
/// says so.
fn export_wizard(win: &Rc<Win>, shared: &Shared) {
	let frame = win.frame;
	let formats: [String; 5] = [
		t("PNG picture"),
		t("HTML page"),
		t("Raised-line SVG"),
		t("Description (recreation file)"),
		t("Animated GIF"),
	];
	let embeds: [String; 3] = [
		t("A separate file on its own"),
		t("Inline in an HTML page"),
		t("With caption text alongside"),
	];
	let wiz = Wizard::new(&frame, &t("Export Wizard"));
	let p1 = wiz.add_step(&t("Format"));
	let s1 = step_sizer(&p1);
	let f_label = StaticText::builder(&p1)
		.with_label(&t("Export format"))
		.build();
	s1.add(
		&f_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let f_list = QuietList::<usize>::new(&p1, &t("Export format"));
	s1.add(&f_list, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let f_list = Rc::new(f_list);
	f_list.update_labelled(
		formats
			.iter()
			.enumerate()
			.map(|(i, n)| (i, n.to_string()))
			.collect(),
		Select::Start,
	);
	let format_sel = Rc::new(std::cell::Cell::new(0usize));
	{
		let (sel, list) = (format_sel.clone(), f_list.clone());
		f_list.on_selection_changed(move || {
			if let Some(i) = list.selected() {
				sel.set(i);
			}
		});
	}
	let p2 = wiz.add_step(&t("Embedding"));
	let s2 = step_sizer(&p2);
	let e_label = StaticText::builder(&p2)
		.with_label(&t("How it will be embedded or accompanied"))
		.build();
	s2.add(
		&e_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let e_list = QuietList::<usize>::new(&p2, &t("Embedding"));
	s2.add(&e_list, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let e_list = Rc::new(e_list);
	e_list.update_labelled(
		embeds
			.iter()
			.enumerate()
			.map(|(i, n)| (i, n.to_string()))
			.collect(),
		Select::Start,
	);
	let embed_sel = Rc::new(std::cell::Cell::new(0usize));
	{
		let (sel, list) = (embed_sel.clone(), e_list.clone());
		e_list.on_selection_changed(move || {
			if let Some(i) = list.selected() {
				sel.set(i);
			}
		});
	}
	let p3 = wiz.add_step(&t("Confirm"));
	let s3 = step_sizer(&p3);
	let c_label = StaticText::builder(&p3)
		.with_label(&t("Confirm the export"))
		.build();
	s3.add(
		&c_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let c_body = TextCtrl::builder(&p3)
		.with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::ReadOnly)
		.build();
	s3.add(&c_body, 1, SizerFlag::Expand | SizerFlag::All, 5);
	{
		let (c_body, format_sel, embed_sel) =
			(c_body.clone(), format_sel.clone(), embed_sel.clone());
		wiz.set_on_show(move |step| {
			if step != 2 {
				return;
			}
			let (f, e) = (format_sel.get(), embed_sel.get());
			let embed_line = match (f, e) {
				(3, _) => {
					t("A description is words already: embedding changes nothing, and only the description file is written.")
				}
				(_, 0) => t("One file, on its own."),
				(1, 1) => {
					t("An HTML page is inline by nature: the design's shapes are the page. Only the page is written.")
				}
				(2, 1) => {
					t("SVG is a text format pages embed as it is: only the SVG file is written.")
				}
				(_, 1) => {
					t("The picture file, plus an HTML page beside it that shows the picture inline.")
				}
				(_, 2) => t("The main file, plus the caption sheet as a text file beside it."),
				_ => String::new(),
			};
			c_body.set_value(&t("Format: %s.\nEmbedding: %s.\n%s\nPress Finish to choose the file name and write it.").replacen("%s", &(formats[f]).to_string(), 1).replacen("%s", &(embeds[e]).to_string(), 1).replacen("%s", &(embed_line).to_string(), 1));
		});
	}
	if !wiz.run() {
		return;
	}
	let (f, e) = (format_sel.get(), embed_sel.get());
	// The files a caption sheet or an inline page would sit
	// beside the main one, derived from the chosen path.
	let sibling = |path: &str, ext: &str, tag: &str| -> String {
		let p = Path::new(path);
		let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("design");
		let name = if tag.is_empty() {
			format!("{stem}.{ext}")
		} else {
			format!("{stem}-{tag}.{ext}")
		};
		match p.parent() {
			Some(dir) if !dir.as_os_str().is_empty() => {
				dir.join(name).to_string_lossy().into_owned()
			}
			_ => name,
		}
	};
	let write_side = |path: &str, what: &str, text: &str| match std::fs::write(path, text) {
		Ok(()) => frame.set_status_text(
			&t("Exported %s to %s")
				.replacen("%s", what, 1)
				.replacen("%s", path, 1),
			0,
		),
		Err(err) => frame.set_status_text(
			&t("Export of %s failed: %s")
				.replacen("%s", what, 1)
				.replacen("%s", &(err).to_string(), 1),
			0,
		),
	};
	let state = shared.borrow();
	match f {
		0 => {
			let Some(path) = file_dialog(
				&frame,
				&t("Export wizard: PNG picture"),
				&format!("{}|*.png", t("PNG image (*.png)")),
				true,
			) else {
				return;
			};
			match state.drawing.flatten().render().save(&path) {
				Ok(()) => frame.set_status_text(
					&t("Exported PNG to %s").replacen("%s", &(path).to_string(), 1),
					0,
				),
				Err(err) => {
					frame.set_status_text(
						&t("Export failed: %s").replacen("%s", &(err).to_string(), 1),
						0,
					);
					return;
				}
			}
			if e == 1 {
				let name = Path::new(&path)
					.file_name()
					.and_then(|n| n.to_str())
					.unwrap_or("design.png")
					.to_string();
				let html = t("<!DOCTYPE html>\n<html lang=\"en\">\n<head><meta charset=\"utf-8\"><title>Blind Designer design</title></head>\n<body>\n<img src=\"%s\" alt=\"A design exported from Blind Designer\">\n</body>\n</html>\n").replacen("%s", &(name).to_string(), 1);
				write_side(&sibling(&path, "html", ""), &t("inline page"), &html);
			} else if e == 2 {
				let sheet = state.drawing.flatten().caption_sheet();
				write_side(&sibling(&path, "txt", "captions"), &t("captions"), &sheet);
			}
		}
		1 => {
			let Some(path) = file_dialog(
				&frame,
				&t("Export wizard: HTML page"),
				&format!("{}|*.html", t("HTML page (*.html)")),
				true,
			) else {
				return;
			};
			write_side(
				&path,
				&t("HTML page"),
				&state.drawing.flatten().to_html(&t("Blind Designer design")),
			);
			if e == 2 {
				let sheet = state.drawing.flatten().caption_sheet();
				write_side(&sibling(&path, "txt", "captions"), &t("captions"), &sheet);
			}
		}
		2 => {
			let Some(path) = file_dialog(
				&frame,
				&t("Export wizard: raised-line SVG"),
				&format!("{}|*.svg", t("SVG drawing (*.svg)")),
				true,
			) else {
				return;
			};
			let svg = state.drawing.to_tactile_svg();
			write_side(&path, &t("raised-line SVG"), &svg);
			if e == 2 {
				let sheet = state.drawing.flatten().caption_sheet();
				write_side(&sibling(&path, "txt", "captions"), &t("captions"), &sheet);
			}
		}
		3 => {
			let Some(path) = file_dialog(
				&frame,
				&t("Export wizard: description (recreation file)"),
				&format!("{}|*.txt", t("Text file (*.txt)")),
				true,
			) else {
				return;
			};
			let summary = state.drawing.summary();
			write_side(&path, &t("description"), &summary);
		}
		_ => {
			let Some(path) = file_dialog(
				&frame,
				&t("Export wizard: animated GIF"),
				&format!("{}|*.gif", t("GIF image (*.gif)")),
				true,
			) else {
				return;
			};
			match state.drawing.to_gif() {
				Ok(bytes) => match std::fs::write(&path, &bytes) {
					Ok(()) => frame.set_status_text(
						&t("Exported GIF to %s").replacen("%s", &(path).to_string(), 1),
						0,
					),
					Err(err) => {
						frame.set_status_text(
							&t("Export failed: %s").replacen("%s", &(err).to_string(), 1),
							0,
						);
						return;
					}
				},
				Err(err) => {
					frame.set_status_text(
						&t("Export failed: %s").replacen("%s", &(err).to_string(), 1),
						0,
					);
					return;
				}
			}
			if e == 1 {
				let name = Path::new(&path)
					.file_name()
					.and_then(|n| n.to_str())
					.unwrap_or("design.gif")
					.to_string();
				let html = t("<!DOCTYPE html>\n<html lang=\"en\">\n<head><meta charset=\"utf-8\"><title>Blind Designer animation</title></head>\n<body>\n<img src=\"%s\" alt=\"An animation exported from Blind Designer\">\n</body>\n</html>\n").replacen("%s", &(name).to_string(), 1);
				write_side(&sibling(&path, "html", ""), &t("inline page"), &html);
			} else if e == 2 {
				let sheet = state.drawing.animation_description();
				write_side(
					&sibling(&path, "txt", "description"),
					"animation description",
					&sheet,
				);
			}
		}
	}
}

/// The Recognizability report (Design, Recognizability): how
/// clearly someone else would recognize what is being created,
/// per project type — core's rough guide, in the standard
/// report dialog.
fn recognizability_dialog(win: &Win, shared: &Shared) {
	let report = {
		let state = shared.borrow();
		recognizability_report(&current_project(&state))
	};
	report_dialog(
		win,
		&t("Recognizability"),
		&t("How recognizable this design is, per its project type"),
		&report,
	);
}

/// The Colour Treatment dialog (Design, Apply Colour
/// Treatment): grayscale start's second half. Pick a layer, or
/// a whole depth, and its grayscale shapes take the palette's
/// colours in paint order — form first, colour when the part
/// is ready. Locked shapes stay grayscale, and the outcome
/// says both counts.
fn color_treatment_dialog(win: &Rc<Win>, shared: &Shared) {
	let frame = win.frame;
	// The scopes on offer: every layer, then every depth role
	// that has a layer. Built from the design itself, so a scope
	// that does not exist is never offered.
	let scopes: Vec<(String, TreatmentScope)> = {
		let state = shared.borrow();
		let mut v: Vec<(String, TreatmentScope)> = state
			.drawing
			.layers
			.iter()
			.enumerate()
			.map(|(i, l)| {
				(
					t("Layer: %s").replacen("%s", &(l.name).to_string(), 1),
					TreatmentScope::Layer(i),
				)
			})
			.collect();
		for role in DepthRole::ALL {
			if role != DepthRole::Unassigned && state.drawing.layers.iter().any(|l| l.depth == role)
			{
				v.push((
					t("Depth: %s").replacen("%s", &(role.name()).to_string(), 1),
					TreatmentScope::Depth(role),
				));
			}
		}
		v
	};
	let dialog = Dialog::builder(&frame, &t("Apply Colour Treatment")).build();
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let label = StaticText::builder(&dialog)
		.with_label(&t(
			"Give the grayscale shapes in this scope the palette colours, in paint order",
		))
		.build();
	sizer.add(
		&label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let choice = Choice::builder(&dialog).build();
	for (name, _) in &scopes {
		choice.append(name);
	}
	choice.set_selection(0);
	choice.set_name(&t("Colour treatment scope"));
	sizer.add(&choice, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let ok = Button::builder(&dialog)
		.with_label(&t("Apply Treatment"))
		.build();
	let cancel = Button::builder(&dialog).with_label(&t("Cancel")).build();
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
	if dialog.show_modal() != ID_OK {
		return;
	}
	let Some(sel) = choice.get_selection() else {
		return;
	};
	let Some((name, scope)) = scopes.get(sel as usize) else {
		return;
	};
	let mut state = shared.borrow_mut();
	if state.palette.is_empty() {
		frame.set_status_text(&t("Colour treatment takes colours from the palette, and the palette is empty: add colours or load the starter colours first."),
			0,
		);
		return;
	}
	let colors: Vec<Srgb> = state.palette.iter().map(|pc| pc.color).collect();
	let outcome = state.drawing.apply_color_treatment(*scope, &colors);
	if outcome.changed > 0 {
		refresh_lists(win, &state);
		announce(win, &state);
	} else {
		frame.set_status_text(
			&t("Colour treatment on %s: no grayscale shapes to colour%s.")
				.replacen("%s", &(name).to_string(), 1)
				.replacen(
					"%s",
					&(if outcome.locked > 0 {
						format!(" ({} locked stayed grayscale)", outcome.locked)
					} else {
						String::new()
					})
					.to_string(),
					1,
				),
			0,
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
	let dialog = Dialog::builder(&frame, &t("Scheme Settings")).build();
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let wheel_label = StaticText::builder(&dialog)
		.with_label(&t("Harmony wheel"))
		.build();
	sizer.add(
		&wheel_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let wheel = Choice::builder(&dialog).build();
	wheel.append(&t("Screen wheel (RGB hues)"));
	wheel.append(&t("Paint wheel (RYB: red is opposite green)"));
	wheel.set_selection(if wheel_ryb { 1 } else { 0 });
	sizer.add(&wheel, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let spread_label = StaticText::builder(&dialog)
		.with_label(&t("Analogous spread in degrees"))
		.build();
	sizer.add(
		&spread_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let spread_spin = SpinCtrl::builder(&dialog).with_range(5, 90).build();
	spread_spin.set_value(spread as i32);
	sizer.add(&spread_spin, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let ok = Button::builder(&dialog).with_label(&t("OK")).build();
	let cancel = Button::builder(&dialog).with_label(&t("Cancel")).build();
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
			&t("Scheme settings saved: %s wheel, analogous spread %s degrees.")
				.replacen(
					"%s",
					if state.wheel_ryb {
						"paint (RYB)"
					} else {
						"screen (RGB)"
					},
					1,
				)
				.replacen("%s", &format!("{:.0}", state.scheme_spread), 1),
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
	let dialog = Dialog::builder(&frame, &t("Color Tools"))
		.with_size(640, 560)
		.build();
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let first_label = StaticText::builder(&dialog)
		.with_label(&t("First color"))
		.build();
	sizer.add(
		&first_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let first = Choice::builder(&dialog).build();
	sizer.add(&first, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let second_label = StaticText::builder(&dialog)
		.with_label(&t("Second color"))
		.build();
	sizer.add(
		&second_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let second = Choice::builder(&dialog).build();
	sizer.add(&second, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let blend_label = StaticText::builder(&dialog)
		.with_label(&t("Blend: percent of the second color"))
		.build();
	sizer.add(
		&blend_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let blend_row = BoxSizer::builder(Orientation::Horizontal).build();
	let blend_slider = Slider::builder(&dialog)
		.with_min_value(0)
		.with_max_value(100)
		.with_value(50)
		.build();
	blend_slider.set_name(&t("Blend percent of second color"));
	blend_row.add(&blend_slider, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let blend_readout = StaticText::builder(&dialog)
		.with_label(&t("50 percent"))
		.build();
	blend_row.add(&blend_readout, 0, SizerFlag::All, 5);
	sizer.add_sizer(&blend_row, 0, SizerFlag::Expand, 0);
	let blend_btn = Button::builder(&dialog).with_label(&t("Blend")).build();
	let subtract_btn = Button::builder(&dialog)
		.with_label(&t("Subtract Second from First"))
		.build();
	let gradient_btn = Button::builder(&dialog).with_label(&t("Gradient…")).build();
	let row1 = BoxSizer::builder(Orientation::Horizontal).build();
	for b in [&blend_btn, &subtract_btn, &gradient_btn] {
		row1.add(b, 0, SizerFlag::All, 5);
	}
	sizer.add_sizer(&row1, 0, SizerFlag::All, 5);
	let contrast_btn = Button::builder(&dialog)
		.with_label(&t("Contrast Check"))
		.build();
	let wavelength_btn = Button::builder(&dialog)
		.with_label(&t("Dominant Wavelength"))
		.build();
	let random_btn = Button::builder(&dialog)
		.with_label(&t("Random Color"))
		.build();
	let row2 = BoxSizer::builder(Orientation::Horizontal).build();
	for b in [&contrast_btn, &wavelength_btn, &random_btn] {
		row2.add(b, 0, SizerFlag::All, 5);
	}
	sizer.add_sizer(&row2, 0, SizerFlag::All, 5);
	let freq_btn = Button::builder(&dialog)
		.with_label(&t("Frequency to Color (experimental)…"))
		.build();
	let close = Button::builder(&dialog).with_label(&t("Close")).build();
	let row3 = BoxSizer::builder(Orientation::Horizontal).build();
	row3.add(&freq_btn, 0, SizerFlag::All, 5);
	row3.add(&close, 0, SizerFlag::All, 5);
	sizer.add_sizer(&row3, 0, SizerFlag::All, 5);
	let result_label = StaticText::builder(&dialog)
		.with_label(&t("Result"))
		.build();
	sizer.add(
		&result_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
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
			readout.set_label(&t("%s percent").replacen(
				"%s",
				&(for_closure.get_value()).to_string(),
				1,
			));
		});
	}
	// The two picked palette colors, or a note in the Result field.
	let picked = {
		let (first, second, result) = (first.clone(), second.clone(), result.clone());
		move |state: &State| -> Option<(Srgb, Srgb)> {
			let (Some(i), Some(j)) = (first.get_selection(), second.get_selection()) else {
				result.set_value(&t(
					"Pick a first and a second color from the palette first.",
				));
				return None;
			};
			match (state.palette.get(i as usize), state.palette.get(j as usize)) {
				(Some(a), Some(b)) => Some((a.color, b.color)),
				_ => {
					result.set_value(&t(
						"Pick a first and a second color from the palette first.",
					));
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
				scheme: t("blend of %s and %s")
					.replacen("%s", &(a.to_hex()).to_string(), 1)
					.replacen("%s", &(b.to_hex()).to_string(), 1),
			};
			let text = t("Blend at %s percent of the second color: %s. Added to the palette.")
				.replacen("%s", &(percent).to_string(), 1)
				.replacen("%s", &(describe_in(c, state.color_format)).to_string(), 1);
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
				scheme: t("%s minus %s").replacen("%s", &(a.to_hex()).to_string(), 1).replacen("%s", &(b.to_hex()).to_string(), 1),
			};
			let text = t("Subtraction, each channel of the second taken from the first: %s. Added to the palette.").replacen("%s", &(describe_in(c, state.color_format)).to_string(), 1);
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
				&t("Gradient steps"),
				&[(t("Steps in the ramp, both end colors included"), 2, 32, 7)],
			) else {
				return;
			};
			let mut state = shared.borrow_mut();
			let Some((a, b)) = picked(&state) else { return };
			let ramp = gradient(a, b, v[0] as usize);
			let provenance = Provenance::Generated {
				scheme: t("gradient from %s to %s")
					.replacen("%s", &(a.to_hex()).to_string(), 1)
					.replacen("%s", &(b.to_hex()).to_string(), 1),
			};
			let text = t("Gradient of %s steps from %s to %s added to the palette.")
				.replacen("%s", &(ramp.len()).to_string(), 1)
				.replacen("%s", &(describe_in(a, state.color_format)).to_string(), 1)
				.replacen("%s", &(describe_in(b, state.color_format)).to_string(), 1);
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
			result.set_value(
				&t("%s against %s: contrast %s to 1 — %s.")
					.replacen("%s", &(describe_in(a, state.color_format)).to_string(), 1)
					.replacen("%s", &(describe_in(b, state.color_format)).to_string(), 1)
					.replacen("%s", &format!("{:.1}", ratio), 1)
					.replacen("%s", &(contrast_verdict(ratio)).to_string(), 1),
			);
		});
	}
	{
		let (shared, result, first) = (shared.clone(), result.clone(), first.clone());
		wavelength_btn.on_click(move |_| {
			let state = shared.borrow();
			let Some(i) = first.get_selection() else {
				result.set_value(&t("Pick a first color from the palette first."));
				return;
			};
			let Some(pc) = state.palette.get(i as usize) else { return };
			match pc.color.dominant_wavelength() {
				Some(w) => result.set_value(&format!(
					"{}: {}.",
					describe_in(pc.color, state.color_format),
					w.spoken()
				)),
				None => result.set_value(&t("%s has no dominant wavelength: it is achromatic, a gray with no hue direction.").replacen("%s", &(describe_in(pc.color, state.color_format)).to_string(), 1)),
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
			let provenance = Provenance::Generated {
				scheme: "random color".to_string(),
			};
			let text = t("Random color: %s. Added to the palette.").replacen(
				"%s",
				&(describe_in(c, state.color_format)).to_string(),
				1,
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
				&t("Frequency to color (experimental)"),
				&[(t("Sound frequency in hertz"), 1, 1_000_000, 440)],
			) else {
				return;
			};
			let hz = v[0] as f64;
			let Some((c, nm)) = frequency_to_color(hz) else {
				result.set_value(&t("That frequency could not be folded into the visible band."));
				return;
			};
			let mut state = shared.borrow_mut();
			let provenance = Provenance::Generated {
				scheme: t("frequency %s hertz (experimental)").replacen("%s", &format!("{:.0}", hz), 1),
			};
			let text = t("Experimental: %s hertz folded up by octaves lands at %s nanometres: %s. Added to the palette. Octaves of sound are mapped to octaves of light by convention, not by physics.").replacen("%s", &format!("{:.0}", hz), 1).replacen("%s", &format!("{:.0}", nm), 1).replacen("%s", &(describe_in(c, state.color_format)).to_string(), 1);
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
			(
				i,
				t("%s: %s elements on an 800 by 600 canvas")
					.replacen("%s", (name).as_ref(), 1)
					.replacen("%s", &(count).to_string(), 1),
			)
		})
		.collect()
}

/// The Templates dialog (File, New from Template): the built-in
/// starter designs, listed with their element counts. Loading one
/// replaces the current drawing — the template is an ordinary
/// design from then on: it edits, saves and exports like anything
/// drawn by hand. The list is labelled "Templates".
fn templates_dialog(win: &Rc<Win>, shared: &Shared) {
	let frame = win.frame;
	let dialog = Dialog::builder(&frame, &t("New from Template")).build();
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let list = QuietList::new(&dialog, &t("Templates"));
	sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let load = Button::builder(&dialog)
		.with_label(&t("Load Template"))
		.build();
	let close = Button::builder(&dialog).with_label(&t("Close")).build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	buttons.add(&load, 0, SizerFlag::All, 5);
	buttons.add(&close, 0, SizerFlag::All, 5);
	sizer.add_sizer(&buttons, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	let list = Rc::new(list);
	list.update_labelled(template_rows(), Select::Start);
	{
		let (list, shared, dialog, w) = (list.clone(), shared.clone(), dialog.clone(), win.clone());
		load.on_click(move |_| {
			let Some(i) = list.selected() else { return };
			let name = templates::TEMPLATE_NAMES[i];
			if let Some(design) = templates::template(name) {
				let mut state = shared.borrow_mut();
				state.drawing = design;
				state.layer = 0;
				// A new project starts from scratch (Elshara):
				// no pinned sliders, default target and page jump,
				// no goal, grayscale start on.
				state.pinned_sliders = Vec::new();
				state.slider_target = AdjustTarget::BaseColor;
				state.page_jump_percent = DEFAULT_PAGE_JUMP_PERCENT;
				state.goal = None;
				state.grayscale_start = true;
				state.interaction_mode = InteractionMode::Slide;
				state.design_path = None;
				reset_adjustment(&mut state);
				// The fresh template design is the saved baseline.
				let fp = project_fingerprint(&state);
				state.saved_fingerprint = fp;
				drop(state);
				apply_interaction_mode(&w, &shared, true);
				frame.set_status_text(
					&t("Started a new design from the %s template. It is a normal design now: edit it, then Save Design to keep it.").replacen("%s", name, 1),
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
	let dialog = Dialog::builder(&frame, &t("Add Symbol")).build();
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let list = QuietList::new(&dialog, &t("Symbols"));
	sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let add = Button::builder(&dialog)
		.with_label(&t("Add Symbol\u{2026}"))
		.build();
	let close = Button::builder(&dialog).with_label(&t("Close")).build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	buttons.add(&add, 0, SizerFlag::All, 5);
	buttons.add(&close, 0, SizerFlag::All, 5);
	sizer.add_sizer(&buttons, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	let rows: Vec<(usize, String)> = symbols::SYMBOL_NAMES
		.iter()
		.enumerate()
		.map(|(i, name)| (i, t("%s symbol").replacen("%s", (name).as_ref(), 1)))
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
				&t("Add %s symbol").replacen("%s", name, 1),
				&[
					(
						t("Position: numbers like 120, 80, or words like top left, center, a bit right"),
						"60, 60",
					),
					(t("Size: numbers like 200 by 100, or words like a bit wider, a bit taller"), "100 by 100"),
					(t("Color: a name like sky blue, or a hex code"), &color.to_hex()),
				],
			) else {
				return;
			};
			let (Some((x, y)), Some((w, h)), Some(color)) = (
				parse_position(&v[0], canvas.0, canvas.1, Some((60, 60))),
				parse_size(&v[1], Some((100, 100))),
				parse_color(&v[2]),
			) else {
				frame.set_status_text(&t("One of those fields did not parse. Position and size take numbers or plain words; color takes a name like sky blue or a hex code."),
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
				shape.caption = Some(t("%s symbol").replacen("%s", name, 1));
				if state.drawing.add_shape(layer, shape).is_some() {
					added += 1;
				}
			}
			drop(state);
			if added > 0 {
				frame.set_status_text(
					&t("Added %s symbol (%s stroke%s).").replacen("%s", name, 1).replacen("%s", &(added).to_string(), 1).replacen("%s", if added == 1 { "" } else { "s" }, 1),
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
		&t("Choose an image to extract colors from"),
		&format!(
			"{}|*.png;*.jpg;*.jpeg;*.webp",
			t("Image files (*.png;*.jpg;*.jpeg;*.webp)")
		),
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
		&t("How many colors to extract"),
		&[(t("Number of colors"), count_def.min, count_def.max, now)],
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
				&t("Extracted %s colors from %s.%s")
					.replacen("%s", &(entries.len()).to_string(), 1)
					.replacen("%s", &(name).to_string(), 1)
					.replacen(
						"%s",
						if set_background {
							" The largest is now the design background."
						} else {
							" They were added to the palette."
						},
						1,
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
			frame.set_status_text(
				&t("Could not open that image: %s").replacen("%s", &(e).to_string(), 1),
				0,
			);
		}
	}
}

/// The Replacement Variables dialog's "save a substituted copy"
/// result code: a modal code, not a window ID — the compile-time
/// guard covers window IDs, and this value only ever travels
/// from a button to `show_modal`'s caller.
const MODAL_SAVE_COPY: i32 = 4242;

/// Design, Replacement Variables: one value field per {name}
/// token the design uses, then the choice of where the
/// substitution lands — this project, or a copy saved as a new
/// design file (the template workflow: design once with tokens,
/// print many finished copies). An empty field means "no value
/// given": the token stays in the words, and the report names
/// every token left unfilled, either way.
fn replacement_variables_dialog(win: &Rc<Win>, shared: &Shared) {
	let frame = win.frame;
	let project = current_project(&shared.borrow());
	let names = variables::scan(&project);
	if names.is_empty() {
		frame.set_status_text(&t("This design uses no replacement variables. Write a token like {name} in a text element or a caption, and it will be fillable here."),
			0,
		);
		return;
	}
	let dialog = Dialog::builder(&frame, &t("Replacement Variables")).build();
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let intro = StaticText::builder(&dialog)
		.with_label(&t(
			"Give each variable a value. Leave a field empty to leave that token as it is.",
		))
		.build();
	sizer.add(
		&intro,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let mut fields = Vec::new();
	for name in &names {
		let label = StaticText::builder(&dialog)
			.with_label(&t("Value for {%s}").replacen("%s", &(name).to_string(), 1))
			.build();
		sizer.add(
			&label,
			0,
			SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
			5,
		);
		let field = TextCtrl::builder(&dialog).build();
		field.set_name(&t("Value for %s").replacen("%s", &(name).to_string(), 1));
		sizer.add(&field, 0, SizerFlag::Expand | SizerFlag::All, 5);
		fields.push(field);
	}
	let here = Button::builder(&dialog)
		.with_label(&t("Substitute in This Project"))
		.build();
	let copy = Button::builder(&dialog)
		.with_label(&t("Substitute in a Copy and Save As…"))
		.build();
	let cancel = Button::builder(&dialog).with_label(&t("Cancel")).build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	buttons.add(&here, 0, SizerFlag::All, 5);
	buttons.add(&copy, 0, SizerFlag::All, 5);
	buttons.add(&cancel, 0, SizerFlag::All, 5);
	sizer.add_sizer(&buttons, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	{
		let d = dialog.clone();
		here.on_click(move |_| d.end_modal(ID_OK));
	}
	{
		let d = dialog.clone();
		copy.on_click(move |_| d.end_modal(MODAL_SAVE_COPY));
	}
	{
		let d = dialog.clone();
		cancel.on_click(move |_| d.end_modal(ID_CANCEL));
	}
	let code = dialog.show_modal();
	if code == ID_CANCEL {
		return;
	}
	// Empty fields are "no value given", so they never reach
	// the values list: their tokens stay, and are reported.
	let values: Vec<(String, String)> = names
		.iter()
		.zip(fields.iter())
		.map(|(name, field)| (name.clone(), field.get_value()))
		.filter(|(_, value)| !value.is_empty())
		.collect();
	if code == ID_OK {
		let mut state = shared.borrow_mut();
		let report = state.drawing.substitute_variables(&values);
		if report.replaced == 0 {
			frame.set_status_text(
				&t("Nothing was substituted: no value matched a token in this design."),
				0,
			);
			return;
		}
		refresh_lists(win, &state);
		announce(win, &state);
		if !report.unknown.is_empty() {
			drop(state);
			report_dialog(
				win,
				&t("Replacement Variables"),
				&t("Substitution done, with tokens left unfilled"),
				&t("%s values substituted in %s elements.\nStill in the words, unfilled: %s.")
					.replacen("%s", &(report.replaced).to_string(), 1)
					.replacen("%s", &(report.affected).to_string(), 1)
					.replacen(
						"%s",
						&(report
							.unknown
							.iter()
							.map(|n| format!("{{{n}}}"))
							.collect::<Vec<_>>()
							.join(", "))
						.to_string(),
						1,
					),
			);
		}
		return;
	}
	// The copy: the current project is untouched; the copy —
	// substitution in its log, like any other edit — is what
	// gets saved.
	let (copy_project, report) = variables::substitute(&project, &values);
	if report.replaced == 0 {
		frame.set_status_text(&t("Nothing was substituted, so no copy was saved: no value matched a token in this design."),
			0,
		);
		return;
	}
	if let Some(path) = file_dialog(
		&frame,
		&t("Save substituted copy"),
		&format!("{}|*.bdd", t("Blind Designer design (*.bdd)")),
		true,
	) {
		match std::fs::write(&path, copy_project.to_text()) {
			Ok(()) => {
				let mut note = t("Saved a substituted copy to %s: %s values substituted in %s elements. This project is unchanged.").replacen("%s", &(path).to_string(), 1).replacen("%s", &(report.replaced).to_string(), 1).replacen("%s", &(report.affected).to_string(), 1);
				if !report.unknown.is_empty() {
					note.push_str(
						&t(" Still unfilled in the copy: %s.").replacen(
							"%s",
							&(report
								.unknown
								.iter()
								.map(|n| format!("{{{n}}}"))
								.collect::<Vec<_>>()
								.join(", "))
							.to_string(),
							1,
						),
					);
				}
				frame.set_status_text(&note, 0);
			}
			Err(e) => {
				frame.set_status_text(&t("Save failed: %s").replacen("%s", &(e).to_string(), 1), 0)
			}
		}
	}
}

/// The Focus window's two texts: the focus line (the selected
/// element's own row, or the whole project), and the body —
/// the statistics, then the goal report's criterion lines, so
/// what is built and what is not built yet read in one place.
fn focus_texts(win: &Win, state: &State) -> (String, String) {
	let focus = win
		.elements_list
		.borrow()
		.selected()
		.and_then(|flat| {
			element_rows(state)
				.into_iter()
				.find(|(key, _)| *key == flat)
				.map(|(_, row)| row)
		})
		.map(|row| t("Focus: %s").replacen("%s", &(row).to_string(), 1))
		.unwrap_or_else(|| t("Focus: whole project"));
	let mut body = ProjectStats::gather(&state.drawing, state.palette.len()).report();
	match &state.goal {
		Some(goal) => {
			if let Some(report) = goal_report(&current_project(state)) {
				body.push_str(
					"\n\nAgainst your goal — what is built, and what is not built yet:\n",
				);
				body.push_str(&report.full_text(goal));
			}
		}
		None => {
			body.push_str(
				"\n\nNo goal stated for this project: File, New with Wizard or Design, Setup Wizard states one. Without a goal there is nothing to measure against — what you have built is the statistics above.",
			);
		}
	}
	(focus, body)
}

/// Keep the Focus window's texts current. Called from
/// `refresh_lists`, so the window follows every edit, like the
/// main window's own statistics line. A no-op while it is shut.
fn refresh_focus_window(win: &Win, state: &State) {
	if win.focus_frame.borrow().is_none() {
		return;
	}
	let (focus, body) = focus_texts(win, state);
	if let Some(line) = win.focus_line.borrow().as_ref() {
		line.set_label(&focus);
	}
	if let Some(text) = win.focus_body.borrow().as_ref() {
		text.set_value(&body);
	}
}

/// View, Focus Mode, both directions. Elshara's distraction-
/// free ask: a window with ONLY the focus, the statistics and
/// the goal's built / not-yet lines. It is its own top-level
/// window rather than a rebuilt main window for a concrete
/// reason: the Sliders tab and the list pages are owned by the
/// notebook and rebuilt in place there (a wx widget's parent
/// is fixed at birth — the same fact `rebuild_list_pages`
/// documents), so collapsing the notebook to a single focus
/// page would mean destroying and recreating the sliders and
/// both lists on every toggle, slider runs included. A
/// separate window shows the same information and touches
/// nothing. Non-modal, so it never traps anyone.
fn set_focus_mode(win: &Rc<Win>, shared: &Shared, on: bool) {
	{
		let mut state = shared.borrow_mut();
		state.focus_mode = on;
	}
	if let Some(bar) = win.frame.get_menu_bar() {
		bar.check_item(ID_FOCUS_MODE, on);
	}
	if !on {
		// The close handler below does the cleanup (state,
		// menu check, handles) for both paths — the menu and
		// the window's own close box.
		let frame = *win.focus_frame.borrow();
		if let Some(frame) = frame {
			frame.close(false);
		}
		return;
	}
	if win.focus_frame.borrow().is_none() {
		let frame = Frame::builder()
			.with_title(&t("Focus — Blind Designer"))
			.with_size(Size::new(640, 480))
			.build();
		let panel = Panel::builder(&frame).build();
		let sizer = BoxSizer::builder(Orientation::Vertical).build();
		let line = StaticText::builder(&panel)
			.with_label(&t("Focus: whole project"))
			.build();
		line.set_name(&t("Current focus"));
		sizer.add(
			&line,
			0,
			SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
			5,
		);
		let body = TextCtrl::builder(&panel)
			.with_style(TextCtrlStyle::MultiLine | TextCtrlStyle::ReadOnly)
			.build();
		body.set_name(&t("What you have built, and what is not built yet"));
		sizer.add(&body, 1, SizerFlag::Expand | SizerFlag::All, 5);
		let close = Button::builder(&panel)
			.with_label(&t("Close Focus Window"))
			.build();
		sizer.add(&close, 0, SizerFlag::All, 5);
		panel.set_sizer(sizer, true);
		{
			let f = frame.clone();
			close.on_click(move |_| f.close(false));
		}
		{
			let (w, s) = (win.clone(), shared.clone());
			frame.on_close(move |_| {
				*w.focus_frame.borrow_mut() = None;
				*w.focus_line.borrow_mut() = None;
				*w.focus_body.borrow_mut() = None;
				s.borrow_mut().focus_mode = false;
				if let Some(bar) = w.frame.get_menu_bar() {
					bar.check_item(ID_FOCUS_MODE, false);
				}
			});
		}
		*win.focus_frame.borrow_mut() = Some(frame);
		*win.focus_line.borrow_mut() = Some(line);
		*win.focus_body.borrow_mut() = Some(body);
	}
	let state = shared.borrow();
	refresh_focus_window(win, &state);
	drop(state);
	if let Some(frame) = win.focus_frame.borrow().as_ref() {
		frame.show(true);
		// Land in the window (Elshara's report: it opened, but
		// NVDA never arrived there): raise it, then put the
		// keyboard in the body text, so what the screen reader
		// meets is the built / not-yet lines themselves.
		frame.raise();
	}
	if let Some(body) = win.focus_body.borrow().as_ref() {
		body.set_focus();
	}
}

/// One pointer-drawing session: the raw path as travelled, in
/// canvas coordinates, and the brush the stroke is being drawn
/// with (which becomes the current brush on Finish, as with
/// Add Stroke by typed points).
struct DrawSession {
	raw: Vec<(i32, i32)>,
	held: bool,
	width: u32,
	color: Srgb,
	/// Where the session's color sits in the palette, when it
	/// came from there — C cycles from that seat.
	color_pos: Option<usize>,
	scale: f64,
	canvas_w: u32,
	canvas_h: u32,
}

/// Design, Draw with Pointer: press and drag on the surface —
/// mouse, or touch, which Windows delivers as mouse events —
/// and the path becomes ONE stroke in the current brush on
/// Finish. Lifting and pressing again continues the same
/// stroke from there (a straight join), as a stroke typed as
/// points would. While the window is open, keys 1 to 9 set the
/// brush width and C cycles the brush color through the
/// palette; the status line in the window says every change
/// in words, and how many points the stroke will keep.
fn draw_pointer_dialog(win: &Rc<Win>, shared: &Shared) {
	let frame = win.frame;
	let (canvas_w, canvas_h, brush) = {
		let state = shared.borrow();
		(
			state.drawing.width,
			state.drawing.height,
			state.brush.clone(),
		)
	};
	// The surface never upscales; a canvas bigger than the fit
	// box scales down, and positions divide back by the scale.
	let scale = (880.0 / canvas_w as f64)
		.min(620.0 / canvas_h as f64)
		.min(1.0);
	let session = Rc::new(RefCell::new(DrawSession {
		raw: Vec::new(),
		held: false,
		width: brush.width,
		color: brush.color,
		color_pos: None,
		scale,
		canvas_w,
		canvas_h,
	}));
	let dialog = Dialog::builder(&frame, &t("Draw with Pointer")).build();
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let intro = StaticText::builder(&dialog)
		.with_label(&t("Press and drag to draw one stroke. Keys: 1 to 9 set the brush width, C cycles the brush colour, Escape cancels."))
		.build();
	sizer.add(
		&intro,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let surface = Panel::builder(&dialog)
		.with_size(Size::new(
			((canvas_w as f64 * scale).round() as i32).max(50),
			((canvas_h as f64 * scale).round() as i32).max(50),
		))
		.build();
	surface.set_name(
		&t("Drawing surface: the %s by %s canvas")
			.replacen("%s", &(canvas_w).to_string(), 1)
			.replacen("%s", &(canvas_h).to_string(), 1),
	);
	sizer.add(&surface, 0, SizerFlag::All, 5);
	let status = StaticText::builder(&dialog)
		.with_label(
			&t("Brush width %s pixels. Colour %s. No points yet.")
				.replacen("%s", &(brush.width).to_string(), 1)
				.replacen(
					"%s",
					&(describe_in(brush.color, shared.borrow().color_format)).to_string(),
					1,
				),
		)
		.build();
	status.set_name(&t("Draw status"));
	sizer.add(
		&status,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let finish = Button::builder(&dialog).with_label(&t("Finish")).build();
	let cancel = Button::builder(&dialog).with_label(&t("Cancel")).build();
	let buttons = BoxSizer::builder(Orientation::Horizontal).build();
	buttons.add(&finish, 0, SizerFlag::All, 5);
	buttons.add(&cancel, 0, SizerFlag::All, 5);
	sizer.add_sizer(&buttons, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);

	// The session's status line, in words: width, color, and
	// how many points the stroke will keep after sampling.
	let say = |session: &DrawSession, format: ColorFormat| {
		let kept = chroma_core::design::sample_points(&session.raw, 4).len();
		t("Brush width %s pixels. Colour %s. %s")
			.replacen("%s", &(session.width).to_string(), 1)
			.replacen("%s", &(describe_in(session.color, format)).to_string(), 1)
			.replacen(
				"%s",
				&(if kept == 0 {
					t("No points yet.")
				} else {
					format!("{kept} points so far.")
				})
				.to_string(),
				1,
			)
	};
	let to_canvas = |session: &DrawSession, p: Point| {
		let x = (p.x as f64 / session.scale).round() as i32;
		let y = (p.y as f64 / session.scale).round() as i32;
		(
			x.clamp(0, session.canvas_w as i32 - 1),
			y.clamp(0, session.canvas_h as i32 - 1),
		)
	};
	{
		let (session, status, shared2) = (session.clone(), status.clone(), shared.clone());
		surface.on_mouse_left_down(move |data| {
			let WindowEventData::MouseButton(e) = data else {
				return;
			};
			let Some(p) = e.get_position() else { return };
			let mut s = session.borrow_mut();
			s.held = true;
			let pt = to_canvas(&s, p);
			if s.raw.last() != Some(&pt) {
				s.raw.push(pt);
			}
			drop(s);
			let s = session.borrow();
			let format = shared2.borrow().color_format;
			status.set_label(&say(&s, format));
		});
	}
	{
		let (session, status, shared2) = (session.clone(), status.clone(), shared.clone());
		surface.on_mouse_motion(move |data| {
			let WindowEventData::MouseMotion(e) = data else {
				return;
			};
			let Some(p) = e.get_position() else { return };
			let mut s = session.borrow_mut();
			if !s.held {
				return;
			}
			let pt = to_canvas(&s, p);
			if s.raw.last() != Some(&pt) {
				s.raw.push(pt);
			}
			drop(s);
			let s = session.borrow();
			let format = shared2.borrow().color_format;
			status.set_label(&say(&s, format));
		});
	}
	{
		let session = session.clone();
		surface.on_mouse_left_up(move |_| {
			session.borrow_mut().held = false;
		});
	}
	{
		let (session, status, shared2, d) = (
			session.clone(),
			status.clone(),
			shared.clone(),
			dialog.clone(),
		);
		surface.on_key_down(move |data| {
			let WindowEventData::Keyboard(kb) = &data else {
				data.skip(true);
				return;
			};
			let Some(code) = kb.get_key_code() else {
				data.skip(true);
				return;
			};
			if code == WXK_ESCAPE {
				d.end_modal(ID_CANCEL);
				return;
			}
			let mut s = session.borrow_mut();
			if (b'1' as i32..=b'9' as i32).contains(&code) {
				let def = slider_by_id("brush-width").expect("the brush-width slider exists");
				s.width = (code - b'0' as i32).clamp(def.min, def.max) as u32;
			} else if code == b'C' as i32 {
				let state = shared2.borrow();
				if state.palette.is_empty() {
					drop(s);
					status.set_label(
						"No palette colours to cycle: the palette is empty. The brush colour is unchanged.",
					);
					return;
				}
				let next = match s.color_pos {
					Some(i) => (i + 1) % state.palette.len(),
					None => 0,
				};
				s.color_pos = Some(next);
				s.color = state.palette[next].color;
				drop(state);
			} else {
				data.skip(true);
				return;
			}
			let format = shared2.borrow().color_format;
			status.set_label(&say(&s, format));
		});
	}
	{
		let (session, status, d) = (session.clone(), status.clone(), dialog.clone());
		finish.on_click(move |_| {
			if chroma_core::design::sample_points(&session.borrow().raw, 4).len() < 2 {
				status.set_label("Nothing to finish yet: press and drag on the surface to draw the stroke first.");
				return;
			}
			d.end_modal(ID_OK);
		});
	}
	{
		let d = dialog.clone();
		cancel.on_click(move |_| d.end_modal(ID_CANCEL));
	}
	// The surface is the point of this window: focus starts
	// there, so the keys work without a first click. The
	// buttons stay one Tab away.
	surface.set_focus();
	if dialog.show_modal() != ID_OK {
		return;
	}
	let s = session.borrow();
	let mut state = shared.borrow_mut();
	let brush_name = state.brush.name.clone();
	let pattern = state.brush.pattern.clone();
	state.brush = Brush::new(&brush_name, s.width, s.color, &pattern);
	let layer = state.layer;
	let mut shape = Shape::stroke_from_points(&s.raw, 4, s.width, s.color);
	shape.pattern = pattern;
	shape.opacity = state.current_opacity;
	let index = state.drawing.add_shape(layer, shape);
	refresh_lists(win, &state);
	if let Some(index) = index {
		speak_added(win, &state, layer, index);
	}
}

/// Rows for the Text Studio's list: one per text element, in
/// reading order (the order `Drawing::text_positions` walks),
/// keyed by that ordinal: the phrase's first line and its style
/// in words.
fn text_rows(state: &State) -> Vec<(usize, String)> {
	state
		.drawing
		.text_positions()
		.iter()
		.enumerate()
		.map(|(i, &(li, si))| {
			let s = &state.drawing.layers[li].shapes[si];
			let first = s.text.lines().next().unwrap_or("");
			let more = if s.text.contains('\n') { " …" } else { "" };
			let words = s.style.words();
			let style = if words.is_empty() {
				"plain".to_string()
			} else {
				words.join(", ")
			};
			(
				i,
				t("%s: \"%s\"%s — %s")
					.replacen("%s", &(state.drawing.layers[li].name).to_string(), 1)
					.replacen("%s", first, 1)
					.replacen("%s", more, 1)
					.replacen("%s", &(style).to_string(), 1),
			)
		})
		.collect()
}

/// The Text Studio (Design menu): the project's text elements
/// as an editor. Select one to load its words and style; Apply
/// rewrites them as one logged step. The transforms sit below:
/// sort the selected element's lines, spread every text element
/// into columns, and build a grid of text elements from a
/// pasted table (tabs between the columns, as spreadsheets
/// copy them). Accessibility: every control is labelled in
/// words before it appears; the list carries the name "Text
/// elements"; nothing here moves focus on its own.
fn text_studio_dialog(win: &Rc<Win>, shared: &Shared) {
	let frame = win.frame;
	let dialog = Dialog::builder(&frame, &t("Text Studio"))
		.with_size(640, 720)
		.build();
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let list_label = StaticText::builder(&dialog)
		.with_label(&t("Text elements"))
		.build();
	sizer.add(
		&list_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let list = QuietList::new(&dialog, &t("Text elements"));
	sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let phrase_label = StaticText::builder(&dialog)
		.with_label(&t("Words of the selected element (one line per row)"))
		.build();
	sizer.add(
		&phrase_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let phrase = TextCtrl::builder(&dialog)
		.with_style(TextCtrlStyle::MultiLine)
		.build();
	sizer.add(&phrase, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let bold = CheckBox::builder(&dialog).with_label(&t("Bold")).build();
	let italic = CheckBox::builder(&dialog)
		.with_label(&t(
			"Italic (spoken and exported; the picture font has no slanted letters)",
		))
		.build();
	let underline = CheckBox::builder(&dialog)
		.with_label(&t("Underline"))
		.build();
	for b in [&bold, &italic, &underline] {
		sizer.add(b, 0, SizerFlag::All, 5);
	}
	let heading_label = StaticText::builder(&dialog)
		.with_label(&t("Heading level"))
		.build();
	sizer.add(
		&heading_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let heading_choice = Choice::builder(&dialog).build();
	for h in Heading::ALL {
		heading_choice.append(&h.name());
	}
	sizer.add(&heading_choice, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let align_label = StaticText::builder(&dialog)
		.with_label(&t("Alignment in its box"))
		.build();
	sizer.add(
		&align_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let align_choice = Choice::builder(&dialog).build();
	for a in TextAlign::ALL {
		align_choice.append(&a.label());
	}
	sizer.add(&align_choice, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let apply_btn = Button::builder(&dialog)
		.with_label(&t("Apply Words and Style"))
		.build();
	let sort_az_btn = Button::builder(&dialog)
		.with_label(&t("Sort Lines A to Z"))
		.build();
	let sort_za_btn = Button::builder(&dialog)
		.with_label(&t("Sort Lines Z to A"))
		.build();
	let edit_buttons = BoxSizer::builder(Orientation::Horizontal).build();
	for b in [&apply_btn, &sort_az_btn, &sort_za_btn] {
		edit_buttons.add(b, 0, SizerFlag::All, 5);
	}
	sizer.add_sizer(&edit_buttons, 0, SizerFlag::All, 5);
	let columns_label = StaticText::builder(&dialog)
		.with_label(&t(
			"Spread every text element into this many columns, in reading order",
		))
		.build();
	sizer.add(
		&columns_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let columns_spin = SpinCtrl::builder(&dialog).with_range(1, 12).build();
	columns_spin.set_value(2);
	sizer.add(&columns_spin, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let spread_btn = Button::builder(&dialog)
		.with_label(&t("Spread into Columns"))
		.build();
	sizer.add(&spread_btn, 0, SizerFlag::All, 5);
	let grid_label = StaticText::builder(&dialog)
		.with_label(&t(
			"Build a text grid from a pasted table (tabs between columns)",
		))
		.build();
	sizer.add(
		&grid_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let grid_data = TextCtrl::builder(&dialog)
		.with_style(TextCtrlStyle::MultiLine)
		.build();
	sizer.add(&grid_data, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let grid_size_label = StaticText::builder(&dialog)
		.with_label(&t("Grid font size in points"))
		.build();
	sizer.add(
		&grid_size_label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let grid_size_spin = SpinCtrl::builder(&dialog).with_range(6, 72).build();
	grid_size_spin.set_value(14);
	sizer.add(&grid_size_spin, 0, SizerFlag::Expand | SizerFlag::All, 5);
	let grid_btn = Button::builder(&dialog)
		.with_label(&t("Build Text Grid"))
		.build();
	sizer.add(&grid_btn, 0, SizerFlag::All, 5);
	let close = Button::builder(&dialog).with_label(&t("Close")).build();
	sizer.add(&close, 0, SizerFlag::All, 5);
	dialog.set_sizer(sizer, true);
	let list = Rc::new(list);
	{
		let state = shared.borrow();
		list.update_labelled(text_rows(&state), Select::Start);
	}
	let grey = grey_with_selection(&list, &[apply_btn, sort_az_btn, sort_za_btn]);
	// The selected element as (layer, shape), if any.
	let picked = {
		let (list, shared) = (list.clone(), shared.clone());
		move || -> Option<(usize, usize)> {
			let ordinal = list.selected()?;
			shared
				.borrow()
				.drawing
				.text_positions()
				.get(ordinal)
				.copied()
		}
	};
	let picked = Rc::new(picked);
	// Load the selected element's words and style into the editor.
	let load = {
		let (list, shared) = (list.clone(), shared.clone());
		let (phrase, bold, italic, underline) = (
			phrase.clone(),
			bold.clone(),
			italic.clone(),
			underline.clone(),
		);
		let (heading_choice, align_choice) = (heading_choice.clone(), align_choice.clone());
		move || {
			let Some(ordinal) = list.selected() else {
				return;
			};
			let state = shared.borrow();
			let Some(&(li, si)) = state.drawing.text_positions().get(ordinal) else {
				return;
			};
			let s = &state.drawing.layers[li].shapes[si];
			phrase.set_value(&s.text);
			bold.set_value(s.style.bold);
			italic.set_value(s.style.italic);
			underline.set_value(s.style.underline);
			let h = Heading::ALL
				.iter()
				.position(|x| *x == s.style.heading)
				.unwrap_or(0);
			heading_choice.set_selection(h as u32);
			let a = TextAlign::ALL
				.iter()
				.position(|x| *x == s.style.align)
				.unwrap_or(0);
			align_choice.set_selection(a as u32);
		}
	};
	let load = Rc::new(load);
	// Refresh the rows and the main window, speak the newest
	// log entry, and reload the editor from the selection.
	let sync = {
		let (win, list, shared, grey, load) = (
			win.clone(),
			list.clone(),
			shared.clone(),
			grey.clone(),
			load.clone(),
		);
		move || {
			{
				let state = shared.borrow();
				list.update_labelled(text_rows(&state), Select::Keep);
				refresh_lists(&win, &state);
				if let Some(last) = state.drawing.log.last() {
					frame.set_status_text(&last.describe(&state.drawing), 0);
				}
			}
			grey();
			load();
		}
	};
	let sync = Rc::new(sync);
	{
		let load = load.clone();
		list.on_selection_changed(move || load());
	}
	{
		let (shared, sync, picked) = (shared.clone(), sync.clone(), picked.clone());
		let (phrase, bold, italic, underline) = (
			phrase.clone(),
			bold.clone(),
			italic.clone(),
			underline.clone(),
		);
		let (heading_choice, align_choice) = (heading_choice.clone(), align_choice.clone());
		apply_btn.on_click(move |_| {
			let Some((li, si)) = picked() else { return };
			let style = TextStyle {
				bold: bold.is_checked(),
				italic: italic.is_checked(),
				underline: underline.is_checked(),
				heading: heading_choice
					.get_selection()
					.and_then(|i| Heading::ALL.get(i as usize).copied())
					.unwrap_or_default(),
				align: align_choice
					.get_selection()
					.and_then(|i| TextAlign::ALL.get(i as usize).copied())
					.unwrap_or_default(),
			};
			let words = phrase.get_value();
			shared.borrow_mut().drawing.edit_text(li, si, &words, style);
			sync();
		});
	}
	for (btn, ascending) in [(&sort_az_btn, true), (&sort_za_btn, false)] {
		let (shared, sync, picked) = (shared.clone(), sync.clone(), picked.clone());
		btn.on_click(move |_| {
			let Some((li, si)) = picked() else { return };
			if shared
				.borrow_mut()
				.drawing
				.sort_text_lines(li, si, ascending)
			{
				sync();
			} else {
				frame.set_status_text(
					&t("That element has fewer than two lines; there is nothing to sort."),
					0,
				);
			}
		});
	}
	{
		let (shared, sync) = (shared.clone(), sync.clone());
		spread_btn.on_click(move |_| {
			let columns = columns_spin.value().max(1) as usize;
			let placed = shared.borrow_mut().drawing.distribute_texts(columns);
			if placed == 0 {
				frame.set_status_text(&t("This design has no text elements to spread."), 0);
			} else {
				sync();
			}
		});
	}
	{
		let (win, shared, sync) = (win.clone(), shared.clone(), sync.clone());
		grid_btn.on_click(move |_| {
			let data = grid_data.get_value();
			let mut state = shared.borrow_mut();
			let layer = state.layer;
			let fill = selected_color(&win, &state).unwrap_or(Srgb::BLACK);
			let size = grid_size_spin.value().max(6) as u32;
			let added = state.drawing.add_text_grid(layer, &data, fill, size);
			drop(state);
			if added == 0 {
				frame.set_status_text(&t("No words in that table; nothing was built. Paste rows with tabs between the columns."),
					0,
				);
			} else {
				sync();
			}
		});
	}
	{
		let dialog = dialog.clone();
		close.on_click(move |_| dialog.end_modal(ID_CANCEL));
	}
	load();
	grey();
	dialog.show_modal();
}

/// Embed a feed snapshot (Design menu): ask for the feed's
/// address and a count, fetch it, and place the items' titles
/// as one text element — a snapshot, refreshed only by
/// embedding again. The fetch is blocking with a 20-second
/// budget; failures are reported in plain words and never
/// panic. Accessibility: two labelled fields, like the other
/// dialogs; the result is announced by `speak_added`.
fn embed_feed_dialog(win: &Rc<Win>, shared: &Shared) {
	let frame = win.frame;
	let Some(v) = fields_dialog(
		&frame,
		&t("Embed feed snapshot"),
		&[
			(t("Feed address (an RSS or Atom URL)"), "https://"),
			(t("How many of the latest items (1 to 50)"), "5"),
		],
	) else {
		return;
	};
	let url = v[0].trim().to_string();
	if url.is_empty() {
		frame.set_status_text(&t("No feed address given; nothing was embedded."), 0);
		return;
	}
	let count: usize = v[1]
		.trim()
		.parse()
		.ok()
		.filter(|n| (1..=50).contains(n))
		.unwrap_or(5);
	let fetched = (|| -> Result<String, String> {
		let config = ureq::Agent::config_builder()
			.timeout_global(Some(std::time::Duration::from_secs(20)))
			.build();
		let agent = ureq::Agent::new_with_config(config);
		let mut response = agent.get(&url).call().map_err(|e| e.to_string())?;
		response
			.body_mut()
			.read_to_string()
			.map_err(|e| e.to_string())
	})();
	let xml = match fetched {
		Ok(xml) => xml,
		Err(e) => {
			frame.set_status_text(
				&t("That feed could not be fetched: %s. Check the address and your connection, then try again.").replacen("%s", &(e).to_string(), 1),
				0,
			);
			return;
		}
	};
	let titles = parse_feed_titles(&xml, count);
	if titles.is_empty() {
		frame.set_status_text(&t("No items were found at that address. It may not be a feed, or the feed may be empty."),
			0,
		);
		return;
	}
	let host = url
		.split("://")
		.nth(1)
		.unwrap_or(&url)
		.split('/')
		.next()
		.unwrap_or("")
		.to_string();
	let lines = titles.len();
	let phrase = titles.join("\n");
	let mut state = shared.borrow_mut();
	let layer = state.layer;
	let fill = selected_color(win, &state).unwrap_or(Srgb::BLACK);
	let width = state.drawing.width.saturating_sub(20).max(60);
	let height = ((lines as u32 * 29 + 8).max(29)).min(state.drawing.height.max(29));
	let mut shape = Shape::text(&phrase, "", 16, 10, 10, width, height, fill);
	shape.opacity = state.current_opacity;
	shape.caption = Some(
		t("Feed snapshot: %s items from %s")
			.replacen("%s", &(lines).to_string(), 1)
			.replacen("%s", &(host).to_string(), 1),
	);
	let index = state.drawing.add_shape(layer, shape);
	refresh_lists(win, &state);
	if let Some(index) = index {
		speak_added(win, &state, layer, index);
	}
}

/// Embed a pasted post (Design menu): whatever text the user
/// pastes — a Facebook post, words from another Serrebi tool —
/// lands as one text element, captioned as pasted so its
/// origin is never a mystery later.
fn embed_paste_dialog(win: &Rc<Win>, shared: &Shared) {
	let frame = win.frame;
	let dialog = Dialog::builder(&frame, &t("Embed Pasted Post"))
		.with_size(560, 420)
		.build();
	bind_escape_cancel(dialog);
	let sizer = BoxSizer::builder(Orientation::Vertical).build();
	let label = StaticText::builder(&dialog)
		.with_label(&t("Paste the post or text here"))
		.build();
	sizer.add(
		&label,
		0,
		SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
		5,
	);
	let field = TextCtrl::builder(&dialog)
		.with_style(TextCtrlStyle::MultiLine)
		.build();
	sizer.add(&field, 1, SizerFlag::Expand | SizerFlag::All, 5);
	let ok = Button::builder(&dialog).with_label(&t("Embed")).build();
	let cancel = Button::builder(&dialog).with_label(&t("Cancel")).build();
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
	if dialog.show_modal() != ID_OK {
		return;
	}
	let pasted = field.get_value();
	let text = pasted.trim();
	if text.is_empty() {
		frame.set_status_text(&t("Nothing was pasted; nothing was embedded."), 0);
		return;
	}
	let first_line = text.lines().next().unwrap_or("");
	let short: String = first_line.chars().take(50).collect();
	let lines = text.lines().count() as u32;
	let mut state = shared.borrow_mut();
	let layer = state.layer;
	let fill = selected_color(win, &state).unwrap_or(Srgb::BLACK);
	let width = state.drawing.width.saturating_sub(20).max(60);
	let height = ((lines * 29 + 8).max(29)).min(state.drawing.height.max(29));
	let mut shape = Shape::text(text, "", 16, 10, 10, width, height, fill);
	shape.opacity = state.current_opacity;
	shape.caption = Some(t("pasted post: %s").replacen("%s", &(short).to_string(), 1));
	let index = state.drawing.add_shape(layer, shape);
	refresh_lists(win, &state);
	if let Some(index) = index {
		speak_added(win, &state, layer, index);
	}
}

fn dispatch(win: &Rc<Win>, shared: &Shared, id: Id) {
	let frame = win.frame;
	match id {
		ID_EXIT => {
			if confirm_discard(win, shared, "close Blind Designer") {
				frame.close(true);
			}
		}
		ID_NEW_WINDOW => {
			match std::env::current_exe()
				.and_then(|exe| std::process::Command::new(exe).spawn())
			{
				Ok(_) => frame.set_status_text(&t("A second Blind Designer window opened: it starts its own project, and each window saves its own design."),
					0,
				),
				Err(e) => frame.set_status_text(
					&t("A second window could not be opened: %s").replacen("%s", &(e).to_string(), 1),
					0,
				),
			}
		}
		ID_DICTIONARY => dictionary_dialog(win),
		ID_HELP_FILE => help_dialog(win),
		ID_SOUND_CUES => {
			let mut state = shared.borrow_mut();
			state.sound_cues = !state.sound_cues;
			frame.set_status_text(
				&(if state.sound_cues {
					t("Sound cues on: drawing changes play a pitch and position tone.")
				} else {
					t("Sound cues off. Spoken readouts are unchanged.")
				}),
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
				&t("About Blind Designer"),
			)
			.build()
			.show_modal();
		}
		ID_OPEN_IMAGE => extract_image_dialog(win, shared, true),
		ID_EXTRACT_IMAGE => extract_image_dialog(win, shared, false),
		ID_PROJECT_OPTIONS => project_options_dialog(win, shared),
		ID_NEW_WIZARD => {
			if confirm_discard(win, shared, "start a new project") {
				setup_wizard(win, shared, true);
			}
		}
		ID_SETUP_WIZARD => setup_wizard(win, shared, false),
		ID_ELEMENT_WIZARD => element_wizard(win, shared),
		ID_EXPORT_WIZARD => export_wizard(win, shared),
		ID_RECOGNIZABILITY => recognizability_dialog(win, shared),
		ID_COLOR_TREATMENT => color_treatment_dialog(win, shared),
		ID_REPLACEMENT_VARIABLES => replacement_variables_dialog(win, shared),
		ID_DRAW_POINTER => draw_pointer_dialog(win, shared),
		ID_RECOGNIZE_SHAPE => recognize_selected(win, shared),
		ID_FOCUS_MODE => {
			let on = !shared.borrow().focus_mode;
			set_focus_mode(win, shared, on);
			frame.set_status_text(
				&(if on {
					t("Focus mode on: the Focus window shows the current focus, the statistics, and what is built and not built yet.")
				} else {
					t("Focus mode off: the Focus window is closed.")
				}),
				0,
			);
		}
		ID_ANNOUNCE_SLIDERS => {
			let mut state = shared.borrow_mut();
			state.announce_sliders = !state.announce_sliders;
			if let Some(bar) = frame.get_menu_bar() {
				bar.check_item(ID_ANNOUNCE_SLIDERS, state.announce_sliders);
			}
			frame.set_status_text(
				&(if state.announce_sliders {
					t("Slider announcements on: moving a slider says what changed and that everything else stayed the same.")
				} else {
					t("Slider announcements off: moving a slider says only its name and value.")
				}),
				0,
			);
		}
		ID_SAVE_DESIGN => {
			save_design(win, shared, None);
		}
		ID_OPEN_DESIGN => {
			if let Some(path) = file_dialog(
				&frame,
				&t("Open design"),
				&format!("{}|*.bdd", t("Blind Designer design (*.bdd)")),
				false,
			) {
				// Opening replaces the current design: unsaved
				// work gets its say first.
				if !confirm_discard(win, shared, "open a different design") {
					frame.set_status_text(&t("Open cancelled: the current project is unchanged."),
						0,
					);
					return;
				}
				match std::fs::read_to_string(&path) {
					Ok(text) => match Project::from_text(&text) {
						Ok(project) => {
							let mut state = shared.borrow_mut();
							state.drawing = project.drawing;
							state.palette = project.palette;
							state.pinned_sliders = project.pinned_sliders;
							state.slider_target = project.slider_target;
							state.page_jump_percent = project.page_jump_percent;
							state.goal = project.goal;
							state.grayscale_start = project.grayscale_start;
							state.interaction_mode = project.interaction_mode;
							state.design_path = Some(PathBuf::from(&path));
							reset_adjustment(&mut state);
							state.layer = 0;
							// The opened design is the saved baseline.
							let fp = project_fingerprint(&state);
							state.saved_fingerprint = fp;
							let layers = state.drawing.layers.len();
							let shapes: usize =
								state.drawing.layers.iter().map(|l| l.shapes.len()).sum();
							let colors = state.palette.len();
							drop(state);
							// The opened project's own mode rules
							// the window now (a file that does not
							// name one means Type).
							apply_interaction_mode(win, shared, true);
							let state = shared.borrow();
							refresh_lists(win, &state);
							frame.set_status_text(
								&t("Opened design from %s: %s layers, %s shapes, %s palette colors. It replaces the previous design.").replacen("%s", &(path).to_string(), 1).replacen("%s", &(layers).to_string(), 1).replacen("%s", &(shapes).to_string(), 1).replacen("%s", &(colors).to_string(), 1),
								0,
							);
						}
						Err(e) => frame.set_status_text(
							&t("That file could not be opened as a design: %s").replacen("%s", &(e).to_string(), 1),
							0,
						),
					},
					Err(e) => {
						frame.set_status_text(&t("Could not read that file: %s").replacen("%s", &(e).to_string(), 1), 0)
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
			// The channel group has its own format picker now
			// and goes its own way: a View change no longer
			// rebuilds the Sliders tab.
			match current {
				Some(c) => {
					let state = shared.borrow();
					select_color(win, &state, c);
					frame.set_status_text(
						&t("Color values now shown as %s, for example %s.").replacen("%s", &(format.name()).to_string(), 1).replacen("%s", &(c.format(format)).to_string(), 1),
						0,
					);
				}
				None => frame.set_status_text(
					&t("Color values now shown as %s.").replacen("%s", &(format.name()).to_string(), 1),
					0,
				),
			}
		}
		ID_EXPORT_PNG => {
			if let Some(path) = file_dialog(&frame, &t("Export design as PNG"), &format!("{}|*.png", t("PNG image (*.png)")), true) {
				let state = shared.borrow();
				match state.drawing.flatten().render().save(&path) {
					Ok(()) => frame.set_status_text(&t("Exported PNG to %s").replacen("%s", &(path).to_string(), 1), 0),
					Err(e) => frame.set_status_text(&t("Export failed: %s").replacen("%s", &(e).to_string(), 1), 0),
				}
			}
		}
		ID_EXPORT_CAPTIONS => {
			if let Some(path) =
				file_dialog(&frame, &t("Export caption sheet"), &format!("{}|*.txt", t("Text file (*.txt)")), true)
			{
				let state = shared.borrow();
				match std::fs::write(&path, state.drawing.flatten().caption_sheet()) {
					Ok(()) => frame.set_status_text(&t("Exported captions to %s").replacen("%s", &(path).to_string(), 1), 0),
					Err(e) => frame.set_status_text(&t("Export failed: %s").replacen("%s", &(e).to_string(), 1), 0),
				}
			}
		}
		ID_EXPORT_CSS => {
			if let Some(path) =
				file_dialog(&frame, &t("Export palette as CSS"), &format!("{}|*.css", t("CSS file (*.css)")), true)
			{
				let state = shared.borrow();
				let colors: Vec<Srgb> = if state.palette.is_empty() {
					vec![state.drawing.background]
				} else {
					state.palette.iter().map(|pc| pc.color).collect()
				};
				match std::fs::write(&path, css_for_colors(&colors, "color")) {
					Ok(()) => frame.set_status_text(&t("Exported CSS to %s").replacen("%s", &(path).to_string(), 1), 0),
					Err(e) => frame.set_status_text(&t("Export failed: %s").replacen("%s", &(e).to_string(), 1), 0),
				}
			}
		}
		ID_EXPORT_SUMMARY => {
			if let Some(path) = file_dialog(
				&frame,
				&t("Export recreation file (step-by-step summary)"),
				&format!("{}|*.txt", t("Text file (*.txt)")),
				true,
			) {
				let state = shared.borrow();
				match std::fs::write(&path, state.drawing.summary()) {
					Ok(()) => frame.set_status_text(&t("Exported recreation file to %s").replacen("%s", &(path).to_string(), 1), 0),
					Err(e) => frame.set_status_text(&t("Export failed: %s").replacen("%s", &(e).to_string(), 1), 0),
				}
			}
		}
		ID_EXPORT_HTML => {
			if let Some(path) = file_dialog(
				&frame,
				&t("Export design as an HTML page"),
				&format!("{}|*.html", t("HTML page (*.html)")),
				true,
			) {
				let state = shared.borrow();
				match std::fs::write(&path, state.drawing.flatten().to_html(&t("Blind Designer design"))) {
					Ok(()) => frame.set_status_text(&t("Exported HTML page to %s").replacen("%s", &(path).to_string(), 1), 0),
					Err(e) => frame.set_status_text(&t("Export failed: %s").replacen("%s", &(e).to_string(), 1), 0),
				}
			}
		}
		ID_EXPORT_GIF => {
			if let Some(path) = file_dialog(
				&frame,
				&t("Export animation as a GIF"),
				&format!("{}|*.gif", t("GIF image (*.gif)")),
				true,
			) {
				let state = shared.borrow();
				let count =
					if state.drawing.frames.is_empty() { 1 } else { state.drawing.frames.len() };
				match state.drawing.to_gif() {
					Ok(bytes) => match std::fs::write(&path, &bytes) {
						Ok(()) => frame.set_status_text(
							&t("Exported animated GIF (%s frames) to %s").replacen("%s", &(count).to_string(), 1).replacen("%s", &(path).to_string(), 1),
							0,
						),
						Err(e) => {
							frame.set_status_text(&t("Export failed: %s").replacen("%s", &(e).to_string(), 1), 0)
						}
					},
					Err(e) => frame.set_status_text(&t("Export failed: %s").replacen("%s", &(e).to_string(), 1), 0),
				}
			}
		}
		ID_EXPORT_ANIM_HTML => {
			if let Some(path) = file_dialog(
				&frame,
				&t("Export animation as an HTML page"),
				&format!("{}|*.html", t("HTML page (*.html)")),
				true,
			) {
				let state = shared.borrow();
				match std::fs::write(
					&path,
					state.drawing.to_html_animated(&t("Blind Designer animation")),
				) {
					Ok(()) => {
						frame.set_status_text(&t("Exported animated HTML page to %s").replacen("%s", &(path).to_string(), 1), 0)
					}
					Err(e) => frame.set_status_text(&t("Export failed: %s").replacen("%s", &(e).to_string(), 1), 0),
				}
			}
		}
		ID_EXPORT_ANIM_DESC => {
			if let Some(path) = file_dialog(
				&frame,
				&t("Export animation description (frame by frame)"),
				&format!("{}|*.txt", t("Text file (*.txt)")),
				true,
			) {
				let state = shared.borrow();
				match std::fs::write(&path, state.drawing.animation_description()) {
					Ok(()) => frame.set_status_text(
						&t("Exported animation description to %s").replacen("%s", &(path).to_string(), 1),
						0,
					),
					Err(e) => frame.set_status_text(&t("Export failed: %s").replacen("%s", &(e).to_string(), 1), 0),
				}
			}
		}
		ID_IMPORT_CSS => {
			if let Some(path) = file_dialog(
				&frame,
				&t("Import colors from a CSS file"),
				&format!("{}|*.css", t("CSS file (*.css)")),
				false,
			) {
				match std::fs::read_to_string(&path) {
					Ok(text) => {
						let found = parse_css_colors(&text);
						if found.is_empty() {
							frame.set_status_text(&t("No colors found in that CSS file."), 0);
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
								&t("Imported %s colors from CSS: %s").replacen("%s", &(found.len()).to_string(), 1).replacen("%s", &(notes.join("; ")).to_string(), 1),
								0,
							);
						}
					}
					Err(e) => {
						frame.set_status_text(&t("Could not read that CSS file: %s").replacen("%s", &(e).to_string(), 1), 0)
					}
				}
			}
		}
		ID_IMPORT_HTML => {
			if let Some(path) = file_dialog(
				&frame,
				&t("Import an HTML page as a design"),
				&format!("{}|*.html;*.htm", t("HTML file (*.html;*.htm)")),
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
						// The old project's goal described the old
						// design; the imported one starts unstated.
						state.goal = None;
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
							&t("Imported %s: %s shapes on the canvas, %s colors in the palette. The imported design replaces the previous one.").replacen("%s", &(path).to_string(), 1).replacen("%s", &(shapes).to_string(), 1).replacen("%s", &(colors).to_string(), 1),
							0,
						);
					}
					Err(e) => {
						frame.set_status_text(&t("Could not read that HTML file: %s").replacen("%s", &(e).to_string(), 1), 0)
					}
				}
			}
		}
		ID_ADD_COLOR => {
			// Adding a color by hand happens in the hex field on
			// the main window (the palette context menu's way in):
			// take the user there and say so.
			win.hex_field.set_focus();
			frame.set_status_text(&t("Type a hex value in the Hex value field, then press Apply Hex to add the color."),
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
				let mut text =
					ProjectStats::gather(&state.drawing, state.palette.len()).report();
				// A stated goal is measured here too: the two
				// percentages and every criterion behind them.
				if let Some(goal) = &state.goal {
					if let Some(rep) = goal_report(&current_project(&state)) {
						text.push_str("\n\n");
						text.push_str(&rep.full_text(goal));
					}
				}
				text
			};
			report_dialog(win, &t("Project statistics"), &t("What is in this project"), &report);
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
				&t("Text Readability"),
				&t("How readable each text element is"),
				&report,
			);
		}
		ID_COMPLEMENTARY | ID_SPLIT | ID_ANALOGOUS | ID_TRIADIC | ID_TETRADIC | ID_SHADES
		| ID_TINTS | ID_TONES => {
			let mut state = shared.borrow_mut();
			let Some(base) = selected_color(win, &state) else {
				frame.set_status_text(&t("Select a palette color first."), 0);
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
							t("analogous (%s degrees)").replacen("%s", &format!("{:.0}", spread), 1),
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
						t("%s (paint wheel)").replacen("%s", &(label).to_string(), 1)
					} else {
						label
					};
					(colors, label)
				}
			};
			let provenance =
				Provenance::Generated { scheme: t("%s of %s").replacen("%s", &(what).to_string(), 1).replacen("%s", &(base.to_hex()).to_string(), 1) };
			add_colors(win, &mut state, &colors, &what, &provenance);
		}
		ID_REMOVE_COLOR => {
			let mut state = shared.borrow_mut();
			if let Some(i) = win.palette_list.borrow().selected() {
				if i < state.palette.len() {
					let gone = state.palette.remove(i).color;
					let label = describe_in(gone, state.color_format);
					refresh_lists(win, &state);
					frame.set_status_text(&t("Removed %s").replacen("%s", &(label).to_string(), 1), 0);
				}
			} else {
				frame.set_status_text(&t("Select a palette color first."), 0);
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
				frame.set_status_text(&t("Select a palette color first."), 0);
			}
		}
		ID_CANVAS_SIZE => {
			let (w, h) = {
				let state = shared.borrow();
				(state.drawing.width, state.drawing.height)
			};
			if let Some(v) = numbers_dialog(
				&frame,
				&t("Canvas size"),
				&[
					(t("Width in pixels"), 16, 8000, w as i32),
					(t("Height in pixels"), 16, 8000, h as i32),
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
				frame.set_status_text(&t("Select a palette color to fill the shape first."), 0);
				return;
			};
			if let Some((x, y, w, h, color)) = place_dialog(
				win,
				&t("Add %s filled with %s").replacen("%s", &(kind.name()).to_string(), 1).replacen("%s", &(describe_in(fill, format)).to_string(), 1),
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
			let Some(phrase) = text_dialog(&frame, &t("Add text"), &t("Text to place in the design"), "")
			else {
				return;
			};
			if phrase.trim().is_empty() {
				frame.set_status_text(&t("The text was empty; nothing was added."), 0);
				return;
			}
			let Some(font) = text_dialog(&frame, &t("Add text"), &t("Font name"), "Arial") else {
				return;
			};
			let Some(v) = numbers_dialog(
				&frame,
				&t("Add text"),
				&[(t("Font size in points"), 6, 200, 24)],
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
				&t("Add text \"%s\"").replacen("%s", phrase.trim(), 1),
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
				&t("Choose an image to place in the design"),
				&format!("{}|*.png;*.jpg;*.jpeg;*.webp", t("Image files (*.png;*.jpg;*.jpeg;*.webp)")),
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
				&t("Add stroke"),
				&t("Points as x,y pairs, separated by spaces (for example: 10,20 100,20 100,80)"),
				"10,10 200,10 200,100",
			) else {
				return;
			};
			let Some(points) = parse_points(&text).filter(|p| p.len() >= 2) else {
				frame.set_status_text(&t("Points need to be x,y pairs, at least two of them, whole numbers."),
					0,
				);
				return;
			};
			let Some(v) = fields_dialog(
				&frame,
				&t("Stroke brush"),
				&[
					(t("Brush width in pixels"), &brush.width.to_string()),
					(t("Color: a name like sky blue, or a hex code"), &brush.color.to_hex()),
					(t("Pattern: solid, dashed or dotted"), &brush.pattern),
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
					&t("The brush width needs to be a whole number from %s to %s.").replacen("%s", &(width_def.min).to_string(), 1).replacen("%s", &(width_def.max).to_string(), 1),
					0,
				);
				return;
			};
			let Some(color) = parse_color(&v[1]) else {
				frame.set_status_text(&t("I could not understand that color. Use a name like sky blue, or a hex code like 87CEEB."),
					0,
				);
				return;
			};
			let pattern = v[2].trim().to_lowercase();
			if !PATTERNS.contains(&pattern.as_str()) {
				frame.set_status_text(&t("The pattern needs to be solid, dashed or dotted."), 0);
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
				frame.set_status_text(&t("Select a shape first."), 0);
				return;
			};
			let at = {
				let state = shared.borrow();
				locate(&state, flat)
			};
			let Some((li, si)) = at else { return };
			// Slide mode routes the command to the shape's own
			// sliders instead of a dialog (Elshara's principle:
			// what can be slid is not typed).
			let slide = shared.borrow().interaction_mode == InteractionMode::Slide;
			if slide {
				let part = if id == ID_MOVE_SHAPE {
					ShapePart::Across
				} else {
					ShapePart::Rotation
				};
				focus_shape_part(win, part);
				frame.set_status_text(
					&t("Slide mode: adjust %s on the Sliders tab.").replacen("%s", &(part.name()).to_string(), 1),
					0,
				);
				return;
			}
			if id == ID_MOVE_SHAPE {
				let (cur, canvas) = {
					let state = shared.borrow();
					let s = &state.drawing.layers[li].shapes[si];
					((s.x, s.y), (state.drawing.width, state.drawing.height))
				};
				if let Some(v) = fields_dialog(
					&frame,
					&t("Move shape"),
					&[(t("New position: numbers like 120, 80, or words like top left, center, a bit right"),
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
						None => frame.set_status_text(&t("I could not understand that position. Use numbers like 120, 80, or words like top left, center, a bit right."),
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
					&t("Rotate shape"),
					&[(t("Angle in degrees, clockwise"), -360, 360, now as i32)],
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
				frame.set_status_text(&t("Select a shape first."), 0);
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
				frame.set_status_text(&t("A stroke's size is its points; move it or add a new one."), 0);
				return;
			}
			// Slide mode: the Width slider carries this, the
			// way Move rests on Across (see ID_MOVE_SHAPE).
			let slide = shared.borrow().interaction_mode == InteractionMode::Slide;
			if slide {
				focus_shape_part(win, ShapePart::Width);
				frame.set_status_text(&t("Slide mode: adjust Width on the Sliders tab."), 0);
				return;
			}
			if let Some(v) = fields_dialog(
				&frame,
				&t("Resize shape"),
				&[(t("New size: numbers like 200 by 100, or words like a bit wider, a bit taller"),
					&t("%s by %s").replacen("%s", &(w).to_string(), 1).replacen("%s", &(h).to_string(), 1),
				)],
			) {
				match parse_size(&v[0], Some((w, h))) {
					Some((nw, nh)) => {
						let mut state = shared.borrow_mut();
						state.drawing.resize_shape(li, si, nw, nh);
						refresh_lists(win, &state);
						announce(win, &state);
					}
					None => frame.set_status_text(&t("I could not understand that size. Use numbers like 200 by 100, or words like a bit wider."),
						0,
					),
				}
			}
		}
		ID_RECOLOR_SHAPE => {
			let mut state = shared.borrow_mut();
			let (Some(flat), Some(c)) = (win.elements_list.borrow().selected(), selected_color(win, &state))
			else {
				frame.set_status_text(&t("Select a shape and a palette color first."), 0);
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
		ID_TEXT_STUDIO => {
			text_studio_dialog(win, shared);
			// Text edits can recolor nothing, but they move and
			// rewrite shapes; refresh like the Elements dialog.
			reset_adjustment(&mut shared.borrow_mut());
			let state = shared.borrow();
			refresh_lists(win, &state);
		}
		ID_EMBED_FEED => {
			embed_feed_dialog(win, shared);
			let state = shared.borrow();
			refresh_lists(win, &state);
		}
		ID_EMBED_PASTE => {
			embed_paste_dialog(win, shared);
			let state = shared.borrow();
			refresh_lists(win, &state);
		}
		ID_TEMPLATES => {
			if !confirm_discard(win, shared, "start a new design from a template") {
				return;
			}
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
			if !confirm_discard(win, shared, "start a new design from a pattern") {
				return;
			}
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
					report_dialog(win, &t("Inspector: layer"), &t("Layer inspector"), &text);
				}
			}
		}
		ID_INSPECT_COLOR => {
			if let Some(i) = win.palette_list.borrow().selected() {
				let state = shared.borrow();
				if let Some(text) =
					inspect_color(&state.drawing, &state.palette, i)
				{
					report_dialog(win, &t("Inspector: color"), &t("Color inspector"), &text);
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
					&(if now {
						t("Element locked: bulk changes will skip it; direct edits still work.")
					} else {
						t("Element unlocked: bulk changes reach it again.")
					}),
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
				&(if on {
					t("Palette and Elements now sit side by side on the Lists page.")
				} else {
					t("Palette and Elements are back on their own tabs.")
				}),
				0,
			);
		}
		ID_VIEWABILITY => viewability_dialog(win, shared),
		ID_SAVE_SCENE => {
			if let Some(path) = file_dialog(
				&frame,
				&t("Save scene (a design file other designs can insert)"),
				&format!("{}|*.bdd", t("Blind Designer scene (*.bdd)")),
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
					goal: None,
					grayscale_start: false,
					interaction_mode: InteractionMode::Type,
				};
				match std::fs::write(&path, scene.to_text()) {
					Ok(()) => frame.set_status_text(&t("Saved scene to %s").replacen("%s", &(path).to_string(), 1), 0),
					Err(e) => frame.set_status_text(&t("Save failed: %s").replacen("%s", &(e).to_string(), 1), 0),
				}
			}
		}
		ID_INSERT_SCENE => {
			if let Some(path) = file_dialog(
				&frame,
				&t("Insert a scene file"),
				&format!("{}|*.bdd", t("Blind Designer scene (*.bdd)")),
				false,
			) {
				match std::fs::read_to_string(&path) {
					Ok(text) => match Project::from_text(&text) {
						Ok(scene) => {
							let name = Path::new(&path)
								.file_stem()
								.map(|n| n.to_string_lossy().into_owned())
								.unwrap_or_else(|| t("Scene"));
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
									&t("Inserted scene \"%s\": %s elements onto layer \"%s\", captioned as one group.").replacen("%s", &(name).to_string(), 1).replacen("%s", &(count).to_string(), 1).replacen("%s", &(layer_name).to_string(), 1),
									0,
								);
							} else {
								frame.set_status_text(&t("That scene has no painted elements to insert."),
									0,
								);
							}
						}
						Err(e) => frame.set_status_text(
							&t("That file could not be opened as a scene: %s").replacen("%s", &(e).to_string(), 1),
							0,
						),
					},
					Err(e) => {
						frame.set_status_text(&t("Could not read that file: %s").replacen("%s", &(e).to_string(), 1), 0)
					}
				}
			}
		}
		ID_DESCRIBE_AREA => {
			if let Some(area) = text_dialog(
				&frame,
				&t("Describe area"),
				&t("Area: a name like top left, center, bottom right — or a box as across, down, width, height"),
				"center",
			) {
				let state = shared.borrow();
				match state.drawing.describe_area_text(&area) {
					Some(text) => report_dialog(
						win,
						&t("Describe area"),
						&t("Elements in the %s area").replacen("%s", &(area).to_string(), 1),
						&text,
					),
					None => frame.set_status_text(&t("I could not understand that area. Name a part of the canvas, like top left or center, or give a box as four numbers."),
						0,
					),
				}
			}
		}
		ID_EXPORT_TACTILE_SVG => {
			if let Some(path) = file_dialog(
				&frame,
				&t("Export raised-line SVG (one line style per layer)"),
				&format!("{}|*.svg", t("SVG drawing (*.svg)")),
				true,
			) {
				let state = shared.borrow();
				match std::fs::write(&path, state.drawing.to_tactile_svg()) {
					Ok(()) => frame
						.set_status_text(&t("Exported raised-line SVG to %s").replacen("%s", &(path).to_string(), 1), 0),
					Err(e) => frame.set_status_text(&t("Export failed: %s").replacen("%s", &(e).to_string(), 1), 0),
				}
			}
		}
		ID_EXPORT_TACTILE_SHEET => {
			if let Some(path) = file_dialog(
				&frame,
				&t("Export tactile sheet (element by element, in reading order)"),
				&format!("{}|*.txt", t("Text file (*.txt)")),
				true,
			) {
				let state = shared.borrow();
				match std::fs::write(&path, state.drawing.tactile_sheet()) {
					Ok(()) => frame
						.set_status_text(&t("Exported tactile sheet to %s").replacen("%s", &(path).to_string(), 1), 0),
					Err(e) => frame.set_status_text(&t("Export failed: %s").replacen("%s", &(e).to_string(), 1), 0),
				}
			}
		}
		ID_EXPORT_BRAILLE => {
			if let Some(path) = file_dialog(
				&frame,
				&t("Export braille labels (Grade 1)"),
				&format!("{}|*.brf", t("Braille labels (*.brf)")),
				true,
			) {
				let state = shared.borrow();
				match std::fs::write(&path, state.drawing.braille_labels()) {
					Ok(()) => frame
						.set_status_text(&t("Exported braille labels to %s").replacen("%s", &(path).to_string(), 1), 0),
					Err(e) => frame.set_status_text(&t("Export failed: %s").replacen("%s", &(e).to_string(), 1), 0),
				}
			}
		}
		ID_EXPORT_PRACTICE => {
			if let Some(path) = file_dialog(
				&frame,
				&t("Save practice sheet"),
				&format!("{}|*.txt", t("Text files (*.txt)")),
				true,
			) {
				let state = shared.borrow();
				match std::fs::write(&path, state.drawing.practice_sheet()) {
					Ok(()) => {
						frame.set_status_text(&t("Saved the practice sheet to %s").replacen("%s", &(path).to_string(), 1), 0)
					}
					Err(e) => {
						frame.set_status_text(&t("Could not save the practice sheet: %s").replacen("%s", &(e).to_string(), 1), 0)
					}
				}
			}
		}
		ID_EXPORT_PDF => {
			if let Some(path) = file_dialog(
				&frame,
				&t("Save PDF"),
				&format!("{}|*.pdf", t("PDF documents (*.pdf)")),
				true,
			) {
				let state = shared.borrow();
				let pdf = state.drawing.to_pdf();
				match std::fs::write(&path, &pdf) {
					Ok(()) => frame.set_status_text(
						&t("Saved the PDF to %s: page 1 is the picture, page 2 is the design's words as text.").replacen("%s", &(path).to_string(), 1),
						0,
					),
					Err(e) => {
						frame.set_status_text(&t("Could not save the PDF: %s").replacen("%s", &(e).to_string(), 1), 0)
					}
				}
			}
		}
		ID_EXPORT_PRESENTATION => {
			if let Some(path) = file_dialog(
				&frame,
				&t("Save presentation"),
				&format!("{}|*.html", t("Web pages (*.html)")),
				true,
			) {
				let state = shared.borrow();
				let html = state.drawing.to_presentation_html(&t("Blind Designer presentation"));
				match std::fs::write(&path, html) {
					Ok(()) => {
						frame.set_status_text(&t("Saved the presentation to %s").replacen("%s", &(path).to_string(), 1), 0)
					}
					Err(e) => {
						frame.set_status_text(&t("Could not save the presentation: %s").replacen("%s", &(e).to_string(), 1), 0)
					}
				}
			}
		}
		ID_EXPORT_DOCUMENT => {
			if let Some(path) = file_dialog(
				&frame,
				&t("Save document"),
				&format!("{}|*.md", t("Markdown documents (*.md)")),
				true,
			) {
				let state = shared.borrow();
				// The document references a PNG beside it under
				// the same name; write both, and say both.
				let png_path = Path::new(&path).with_extension("png");
				let png_name = png_path
					.file_name()
					.map(|f| f.to_string_lossy().into_owned())
					.unwrap_or_else(|| "design.png".to_string());
				let md = state
					.drawing
					.to_document_markdown(&t("Blind Designer document"), &png_name);
				let md_result = std::fs::write(&path, md);
				let png_result = state.drawing.flatten().render().save(&png_path);
				match (md_result, png_result) {
					(Ok(()), Ok(())) => frame.set_status_text(
						&t("Saved the document to %s and its picture to %s. Keep the two together.").replacen("%s", &(path).to_string(), 1).replacen("%s", &(png_path.display()).to_string(), 1),
						0,
					),
					(Err(e), _) => {
						frame.set_status_text(&t("Could not save the document: %s").replacen("%s", &(e).to_string(), 1), 0)
					}
					(_, Err(e)) => {
						frame.set_status_text(&t("Could not save the document's picture: %s").replacen("%s", &(e).to_string(), 1), 0)
					}
				}
			}
		}
		ID_PRINT_PDF => {
			let state = shared.borrow();
			let pdf = state.drawing.to_pdf();
			drop(state);
			let path = std::env::temp_dir().join("blind-designer-print.pdf");
			match std::fs::write(&path, &pdf) {
				Ok(()) => {
					let shown = path.display().to_string();
					#[cfg(target_os = "windows")]
					let opened = std::process::Command::new("cmd")
						.args(["/C", "start", "", &shown])
						.spawn();
					#[cfg(target_os = "macos")]
					let opened = std::process::Command::new("open").arg(&shown).spawn();
					#[cfg(all(unix, not(target_os = "macos")))]
					let opened = std::process::Command::new("xdg-open").arg(&shown).spawn();
					match opened {
						Ok(_) => frame.set_status_text(
							&t("Opened %s in your PDF viewer. Print from the viewer — Blind Designer does not drive the printer itself.").replacen("%s", &(shown).to_string(), 1),
							0,
						),
						Err(e) => frame.set_status_text(
							&t("The PDF is saved at %s, but the viewer could not be opened: %s. Open the file and print from there.").replacen("%s", &(shown).to_string(), 1).replacen("%s", &(e).to_string(), 1),
							0,
						),
					}
				}
				Err(e) => frame.set_status_text(&t("Could not write the print PDF: %s").replacen("%s", &(e).to_string(), 1), 0),
			}
		}
		ID_COMPARE_IMAGES => {
			let images = format!("{}|*.png;*.jpg;*.jpeg;*.webp", t("Image files (*.png;*.jpg;*.jpeg;*.webp)"));
			if let Some(first_path) = file_dialog(&frame, &t("Choose the first image"), &images, false) {
				if let Some(second_path) =
					file_dialog(&frame, &t("Choose the second image"), &images, false)
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
							&t("Image comparison"),
							&t("How the two images compare"),
							&compare_images(&a, &b).report(&name(&first_path), &name(&second_path)),
						),
						_ => frame.set_status_text(&t("One of those files could not be read as an image."),
							0,
						),
					}
				}
			}
		}
		ID_TACTILE_GRID => {
			let images = format!("{}|*.png;*.jpg;*.jpeg;*.webp", t("Image files (*.png;*.jpg;*.jpeg;*.webp)"));
			if let Some(path) = file_dialog(
				&frame,
				&t("Choose an image to feel as a tactile grid"),
				&images,
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
					&t("Tactile grid size"),
					&[
						(t("Columns"), col_def.min, col_def.max, cols_now),
						(t("Rows"), row_def.min, row_def.max, rows_now),
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
								&t("Tactile grid"),
								&t("The picture as a grid of shades"),
								&grid.sheet(&name),
							);
						}
						Err(e) => frame
							.set_status_text(&t("Could not open that image: %s").replacen("%s", &(e).to_string(), 1), 0),
					}
				}
			}
		}
		ID_SAVE_BRUSH => {
			let current = shared.borrow().brush.clone();
			let default = if current.name.is_empty() {
				t("My brush")
			} else {
				current.name.clone()
			};
			if let Some(name) = text_dialog(
				&frame,
				&t("Save current brush"),
				&t("Name for this brush (%s pixel, %s, %s pattern)").replacen("%s", &(current.width).to_string(), 1).replacen("%s", &(chroma_core::color_words(current.color)).to_string(), 1).replacen("%s", &(current.pattern).to_string(), 1),
				&default,
			) {
				if name.trim().is_empty() {
					frame.set_status_text(&t("The brush was not saved: it needs a name."), 0);
					return;
				}
				let mut state = shared.borrow_mut();
				let brush = Brush::new(name.trim(), current.width, current.color, &current.pattern);
				let spoken = brush.spoken();
				state.brush = brush.clone();
				state.brushes.save(brush);
				persist_brushes(win, &state);
				frame.set_status_text(&t("Saved brush: %s.").replacen("%s", &(spoken).to_string(), 1), 0);
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
				frame.set_status_text(&t("Select a shape first."), 0);
			}
		}
		_ => {}
	}
}

fn build_menus(frame: &Frame) {
	// Menus are grouped by the job being done (Elshara's feedback):
	// File carries Import and Export submenus instead of two dozen
	// flat items (the feed and pasted-post embeds are imports, and
	// live there); the old Palette menu is now Color, with Schemes
	// and Variations submenus; Design groups its Add, Arrange, Text
	// and Review commands; View's fourteen formats sit in one Color
	// Format submenu. Every item's help string says in plain
	// language what it does — the status line shows it while the
	// item is rested on.
	let import = Menu::builder().build();
	import.append(ID_OPEN_IMAGE, &t("&Open Image…\tCtrl+O"), &t("Open an image: its colors replace the palette and its largest color becomes the background."), ItemKind::Normal);
	import.append(
		ID_EXTRACT_IMAGE,
		&t("&Extract Colors from Image…"),
		&t("Add an image's colors to the palette, leaving the design untouched."),
		ItemKind::Normal,
	);
	import.append(
		ID_IMPORT_CSS,
		&t("&CSS Colors…"),
		&t("Add every color found in a CSS file to the palette."),
		ItemKind::Normal,
	);
	import.append(
		ID_IMPORT_HTML,
		&t("&HTML Design…"),
		&t("Turn a simple HTML page (inline styles) into a design."),
		ItemKind::Normal,
	);
	import.append(
		ID_COMPARE_IMAGES,
		&t("Compare I&mages…"),
		&t("Compare two images: their sizes, shared colors, which is brighter, which is blurrier."),
		ItemKind::Normal,
	);
	import.append(
		ID_TACTILE_GRID,
		&t("Describe Image as &Tactile Grid…"),
		&t("Describe any image as a grid of shades, in braille patterns and in words."),
		ItemKind::Normal,
	);
	import.append(ID_EMBED_FEED, &t("Embed Feed &Snapshot…"), &t("Fetch a feed (RSS or Atom) and place its latest item titles in the design as one text element — a snapshot: it changes only when you embed again."), ItemKind::Normal);
	import.append(ID_EMBED_PASTE, &t("Embed &Pasted Post…"), &t("Paste text copied from anywhere — a Facebook post, another Serrebi tool — and place it in the design as a text element captioned as pasted."), ItemKind::Normal);

	let export = Menu::builder().build();
	export.append(ID_EXPORT_WIZARD, &t("Export &Wizard…"), &t("Step through the export: choose the format, how it will be embedded or accompanied, and confirm what will be written before anything is."), ItemKind::Normal);
	export.append(
		ID_EXPORT_PNG,
		&t("&PNG…\tCtrl+E"),
		&t("Save the design as a PNG picture."),
		ItemKind::Normal,
	);
	export.append(
		ID_EXPORT_CAPTIONS,
		&t("&Captions…"),
		&t("Save a text sheet describing every shape in words."),
		ItemKind::Normal,
	);
	export.append(
		ID_EXPORT_CSS,
		&t("C&SS…"),
		&t("Save the palette as CSS variables, ready for a web page."),
		ItemKind::Normal,
	);
	export.append(
		ID_EXPORT_SUMMARY,
		&t("&Summary (recreation file)…"),
		&t("Save the step-by-step log that recreates this design exactly."),
		ItemKind::Normal,
	);
	export.append(ID_EXPORT_HTML, &t("HT&ML Page…"), &t("Save the design as a standalone web page whose shapes carry their captions as accessible names."), ItemKind::Normal);
	export.append(
		ID_EXPORT_GIF,
		&t("Animated &GIF…"),
		&t("Save the animation as a looping GIF; with no frames, the design as one still picture."),
		ItemKind::Normal,
	);
	export.append(
		ID_EXPORT_ANIM_HTML,
		&t("Animated HTML…"),
		&t("Save the animation as a web page that steps through the frames."),
		ItemKind::Normal,
	);
	export.append(
		ID_EXPORT_ANIM_DESC,
		&t("Animation &Description…"),
		&t("Save the film as words: every frame, its duration, and what changed."),
		ItemKind::Normal,
	);
	export.append(
		ID_EXPORT_TACTILE_SVG,
		&t("&Raised-Line SVG…"),
		&t("Save the design as outlines for swell paper or an embosser, one line style per layer."),
		ItemKind::Normal,
	);
	export.append(ID_EXPORT_TACTILE_SHEET, &t("Tactile &Sheet…"), &t("Save the design as a text walk, element by element in reading order, with braille labels."), ItemKind::Normal);
	export.append(
		ID_EXPORT_BRAILLE,
		&t("&Braille Labels…"),
		&t("Save the elements' captions as Grade 1 braille labels."),
		ItemKind::Normal,
	);
	export.append(ID_EXPORT_PRACTICE, &t("Print-Writing &Practice Sheet…"), &t("Save a practice sheet: every text element printed large as block letters to trace or copy, with a legibility line for each."), ItemKind::Normal);
	export.append(ID_EXPORT_PDF, &t("P&DF…"), &t("Save a two-page PDF: the design as a picture, then its text elements as real, readable text."), ItemKind::Normal);
	export.append(ID_EXPORT_PRESENTATION, &t("Presen&tation HTML…"), &t("Save the design as presentation slides — one per animation frame — each laying the picture and its words side by side."), ItemKind::Normal);
	export.append(ID_EXPORT_DOCUMENT, &t("&Document (Markdown)…"), &t("Save a Markdown document: the design's PNG written beside it and referenced, then the design's words with their styles, then every element's caption."), ItemKind::Normal);

	let file = Menu::builder().build();
	file.append(
		ID_OPEN_DESIGN,
		&t("Open &Design…"),
		&t("Open a saved design file, replacing the current design."),
		ItemKind::Normal,
	);
	file.append(
		ID_SAVE_DESIGN,
		&t("&Save Design…"),
		&t("Save the whole design — canvas, layers, palette and log — to a file."),
		ItemKind::Normal,
	);
	file.append(ID_SAVE_SCENE, &t("Save Sc&ene…"), &t("Save the current design as a scene file, ready to insert into other designs as a group."), ItemKind::Normal);
	file.append(ID_INSERT_SCENE, &t("Insert &Scene…"), &t("Insert a saved scene's elements onto the current layer, captioned as one group under the scene's name."), ItemKind::Normal);
	file.append(ID_TEMPLATES, &t("New from &Template…"), &t("Start a new design from a built-in layout: birthday card, house, face, chart, portrait or landscape."), ItemKind::Normal);
	file.append(ID_NEW_WIZARD, &t("New with &Wizard…"), &t("Start a new project through the setup wizard: say what you want to create or build, answer its focus questions (any may be left with no fixed answer), and the goal is set and the canvas seeded for that kind."), ItemKind::Normal);
	file.append(ID_PATTERN_TEMPLATES, &t("New from &Pattern…"), &t("Start a new design as a two-color pattern — checkerboard, stripes or dots — woven from two palette colors you pick."), ItemKind::Normal);
	file.append(ID_NEW_WINDOW, &t("New &Window"), &t("Open a second Blind Designer window — a separate copy of the app — so two projects can be open at once; each window keeps and saves its own design."), ItemKind::Normal);
	file.append_separator();
	file.append_submenu(
		import,
		&t("&Import"),
		&t("Bring images, colors and designs in."),
	);
	file.append_submenu(
		export,
		&t("&Export"),
		&t("Write the design out: pictures, pages, text and touch."),
	);
	file.append(ID_PRINT_PDF, &t("&Print via PDF…"), &t("Write the design's PDF to a temporary file and open it in your PDF viewer; print from the viewer. (Blind Designer does not drive the printer itself.)"), ItemKind::Normal);
	file.append_separator();
	file.append(
		ID_EXIT,
		&t("E&xit"),
		&t("Close Blind Designer."),
		ItemKind::Normal,
	);

	let schemes = Menu::builder().build();
	schemes.append(
		ID_COMPLEMENTARY,
		&t("&Complementary (hue + 180)"),
		&t("Add the color opposite the selected one on the hue circle."),
		ItemKind::Normal,
	);
	schemes.append(
		ID_SPLIT,
		&t("&Split Complementary (hue ± 150)"),
		&t("Add the two colors 150 degrees either side of the selected one."),
		ItemKind::Normal,
	);
	schemes.append(ID_ANALOGOUS, &t("&Analogous (hue ± spread)"), &t("Add the two neighboring colors, the Scheme Settings spread either side (30 degrees to start)."), ItemKind::Normal);
	schemes.append(
		ID_TRIADIC,
		&t("&Triadic (hue ± 120)"),
		&t("Add the two colors a third of the circle away, either side."),
		ItemKind::Normal,
	);
	schemes.append(
		ID_TETRADIC,
		&t("T&etradic"),
		&t("Add the three colors that complete a rectangle on the hue circle."),
		ItemKind::Normal,
	);
	schemes.append_separator();
	schemes.append(
		ID_SCHEME_SETTINGS,
		&t("Scheme &Settings…"),
		&t("Choose the harmony wheel (screen or paint) and the analogous spread in degrees."),
		ItemKind::Normal,
	);

	let variations = Menu::builder().build();
	variations.append(
		ID_SHADES,
		&t("Add 4 S&hades (lightness down)"),
		&t("Add four darker versions of the selected color."),
		ItemKind::Normal,
	);
	variations.append(
		ID_TINTS,
		&t("Add 4 &Tints (lightness up)"),
		&t("Add four lighter versions of the selected color."),
		ItemKind::Normal,
	);
	variations.append(
		ID_TONES,
		&t("Add 4 Tone&s (chroma down)"),
		&t("Add four softer, less saturated versions of the selected color."),
		ItemKind::Normal,
	);

	let color = Menu::builder().build();
	color.append_submenu(
		schemes,
		&t("&Schemes"),
		&t("New colors computed from the selected color."),
	);
	color.append_submenu(
		variations,
		&t("&Shades, Tints and Tones"),
		&t("Darker, lighter and softer versions of the selected color."),
	);
	color.append_separator();
	color.append(ID_COLOR_TOOLS, &t("Color &Tools…"), &t("Blend, subtract and gradient between two palette colors, random colors, contrast checks, dominant wavelength, and sound frequency to color."), ItemKind::Normal);
	color.append(
		ID_TEXT_READABILITY,
		&t("Te&xt Readability…"),
		&t("How readable every text element in the design is against its background."),
		ItemKind::Normal,
	);
	color.append_separator();
	color.append(ID_LOAD_STARTER, &t("&Load Starter Colors"), &t("Add the starter colors to the palette. A new palette starts empty; these join only when you ask."), ItemKind::Normal);
	color.append(
		ID_REMOVE_COLOR,
		&t("&Remove Selected Color"),
		&t("Take the selected color out of the palette."),
		ItemKind::Normal,
	);
	color.append_separator();
	color.append(
		ID_SET_BACKGROUND,
		&t("Set &Background to Selected Color"),
		&t("Make the selected palette color the design's background."),
		ItemKind::Normal,
	);

	let add = Menu::builder().build();
	add.append(
		ID_ADD_RECT,
		&t("&Rectangle…"),
		&t("Place a rectangle, by numbers or in plain words."),
		ItemKind::Normal,
	);
	add.append(
		ID_ADD_ELLIPSE,
		&t("&Ellipse…"),
		&t("Place an ellipse; equal width and height make a circle."),
		ItemKind::Normal,
	);
	add.append(
		ID_ADD_STROKE,
		&t("Stro&ke…"),
		&t("Draw a line through points you list, with the current brush."),
		ItemKind::Normal,
	);
	add.append(
		ID_ADD_TEXT,
		&t("&Text…"),
		&t("Place words in the design, in a named font at a chosen size."),
		ItemKind::Normal,
	);
	add.append(
		ID_ADD_IMAGE,
		&t("&Image…"),
		&t("Place an image file in the design, scaled into a box you give it."),
		ItemKind::Normal,
	);
	add.append(
		ID_ADD_SYMBOL,
		&t("S&ymbol…"),
		&t("Stamp a ready-made symbol: star, heart, arrow, check, cross, sun or moon."),
		ItemKind::Normal,
	);

	let arrange = Menu::builder().build();
	arrange.append(
		ID_MOVE_SHAPE,
		&t("&Move…"),
		&t("Move the selected element, by numbers or in plain words."),
		ItemKind::Normal,
	);
	arrange.append(
		ID_ROTATE_SHAPE,
		&t("R&otate…"),
		&t("Turn the selected element by an angle in degrees."),
		ItemKind::Normal,
	);
	arrange.append(
		ID_RESIZE_SHAPE,
		&t("Resi&ze…"),
		&t("Change the selected element's size, by numbers or in plain words."),
		ItemKind::Normal,
	);
	arrange.append(
		ID_RECOLOR_SHAPE,
		&t("Set to Palette &Color"),
		&t("Give the selected element the selected palette color."),
		ItemKind::Normal,
	);
	arrange.append(
		ID_REMOVE_SHAPE,
		&t("Remove Sha&pe"),
		&t("Delete the selected element from the design."),
		ItemKind::Normal,
	);

	let text = Menu::builder().build();
	text.append(ID_TEXT_STUDIO, &t("Text &Studio…"), &t("The project's text elements as an editor: rewrite their words, set bold, italic, underline, heading level and alignment, sort their lines, spread them into columns, and build text grids from pasted tables."), ItemKind::Normal);
	text.append(ID_REPLACEMENT_VARIABLES, &t("Replacement &Variables…"), &t("Fill in the {name} tokens written in text elements and captions: give each a value, then substitute in this project or in a saved copy."), ItemKind::Normal);

	let review = Menu::builder().build();
	review.append(ID_INSPECT, &t("In&spect Selected…"), &t("The inspector: what the selected element is, where it is used, what can change it and what happens if it is left alone. Enter on the Elements list does the same."), ItemKind::Normal);
	review.append(ID_VIEWABILITY, &t("&Viewability…"), &t("Report how the design reads from a viewing angle: what is viewable, what is hidden behind something else, what is turned away, per depth."), ItemKind::Normal);
	review.append(ID_RECOGNIZABILITY, &t("&Recognizability…"), &t("How clearly someone else would recognize what you are creating, judged for this project's type: a rough guide from element counts, contrast and depth layering, not a measurement."), ItemKind::Normal);
	review.append(
		ID_STATS,
		&t("&Project Statistics…"),
		&t("How many layers, elements, palette colors and animation frames this project has."),
		ItemKind::Normal,
	);

	let design = Menu::builder().build();
	design.append(
		ID_CANVAS_SIZE,
		&t("Canvas &Size…"),
		&t("Change the canvas width and height in pixels."),
		ItemKind::Normal,
	);
	design.append_separator();
	design.append_submenu(add, &t("&Add"), &t("Add a new element to the design."));
	design.append_submenu(
		arrange,
		&t("A&rrange Selected Shape"),
		&t("Move, turn, resize, recolor or remove the selected element."),
	);
	design.append_separator();
	design.append(ID_ELEMENTS, &t("E&lements…"), &t("Every element in plain language, with rename, describe, move, resize, recolor and delete."), ItemKind::Normal);
	design.append_submenu(text, &t("&Text"), &t("The project's words: the text editor, and the replacement variables that fill its tokens."));
	design.append_submenu(review, &t("&Review"), &t("Look at the design: inspect the selection, judge its viewability and recognizability, and read its statistics."));
	design.append(ID_ELEMENT_WIZARD, &t("Element &Wizard…"), &t("The wizard scoped to the selected element: its stack (what sits on it, what it sits on), slider edits, and the inspector's review."), ItemKind::Normal);
	design.append(ID_LOCK_TOGGLE, &t("&Lock/Unlock Selected"), &t("Lock the selected element so bulk changes skip it, or unlock it so they reach it again. Direct edits work either way."), ItemKind::Normal);
	design.append(ID_SEARCH_INSERT, &t("&Search and Insert…\tCtrl+F"), &t("Search palette colors, starter colors, elements, symbols and templates by name, and insert what you find at the cursor."), ItemKind::Normal);
	design.append(
		ID_LAYERS,
		&t("&Layers…"),
		&t("Add, rename, reorder, show or hide layers, and choose where new shapes land."),
		ItemKind::Normal,
	);
	design.append(
		ID_ANIMATION,
		&t("Anim&ation…"),
		&t("Build a film from frames of the canvas, each with its own duration."),
		ItemKind::Normal,
	);
	design.append(
		ID_DESCRIBE_AREA,
		&t("&Describe Area…"),
		&t("Hear only the elements inside one part of the canvas."),
		ItemKind::Normal,
	);
	design.append(ID_PROJECT_OPTIONS, &t("Project &Options…"), &t("Choose which sliders this project pins to its Sliders tab, what the color adjustments act on, how far Page Up and Page Down jump, grayscale start, and load the starter colors or the gray ramp."), ItemKind::Normal);
	design.append(ID_SETUP_WIZARD, &t("Setup &Wizard…"), &t("State (or restate) this project's goal through the wizard: what you want to create or build, and its focus questions — any answer may be left with no fixed answer."), ItemKind::Normal);
	design.append(ID_COLOR_TREATMENT, &t("Apply Colour &Treatment…"), &t("Grayscale start's second half: give the grayscale shapes of one layer, or of a whole depth, the palette colours in paint order. Locked shapes stay grayscale."), ItemKind::Normal);
	design.append(ID_DRAW_POINTER, &t("Draw with &Pointer…"), &t("Draw one stroke with the mouse or touch: press and drag on the surface, release, Finish. Keys 1 to 9 set the brush width while you draw; C cycles the brush colour."), ItemKind::Normal);
	design.append(ID_RECOGNIZE_SHAPE, &t("Recognize Shape from &Stroke"), &t("Read the selected stroke's path and replace it with the known shape it traces — a line, a rectangle, an ellipse or a triangle — in the same color and brush width. A stroke that matches nothing known stays exactly as drawn."), ItemKind::Normal);
	design.append_separator();
	design.append(
		ID_SAVE_BRUSH,
		&t("Save Current Br&ush…"),
		&t("Keep the current brush — width, color, pattern — in the brush library under a name."),
		ItemKind::Normal,
	);
	design.append(
		ID_BRUSHES,
		&t("Brush Li&brary…"),
		&t("The saved brushes; apply one to make it current for new strokes."),
		ItemKind::Normal,
	);

	let formats = Menu::builder().build();
	formats.append(
		ID_FORMAT_HEX,
		&t("Color Values as &Hex"),
		&t("Show and speak color values as hex codes, like #3A7CA5."),
		ItemKind::Check,
	);
	formats.append(
		ID_FORMAT_RGB,
		&t("Color Values as &RGB"),
		&t("Show and speak color values as red, green and blue numbers, 0 to 255."),
		ItemKind::Check,
	);
	formats.append(
		ID_FORMAT_HSL,
		&t("Color Values as HS&L"),
		&t("Show and speak color values as hue, saturation and lightness."),
		ItemKind::Check,
	);
	formats.append(
		ID_FORMAT_HSV,
		&t("Color Values as HS&V"),
		&t("Show and speak color values as hue, saturation and value."),
		ItemKind::Check,
	);
	formats.append(
		ID_FORMAT_CMYK,
		&t("Color Values as CM&YK"),
		&t("Show and speak color values as cyan, magenta, yellow and black ink percents."),
		ItemKind::Check,
	);
	formats.append(
		ID_FORMAT_XYZ,
		&t("Color Values as CIE &XYZ"),
		&t("Show and speak color values as CIE XYZ tristimulus numbers."),
		ItemKind::Check,
	);
	formats.append(
		ID_FORMAT_XYY,
		&t("Color Values as x&yY"),
		&t("Show and speak color values as chromaticity x and y plus luminance Y."),
		ItemKind::Check,
	);
	formats.append(
		ID_FORMAT_LAB,
		&t("Color Values as CIE L&ab"),
		&t("Show and speak color values as CIE lightness and the a and b opponent axes."),
		ItemKind::Check,
	);
	formats.append(
		ID_FORMAT_LCH,
		&t("Color Values as CIE LC&H"),
		&t("Show and speak color values as CIE lightness, chroma and hue."),
		ItemKind::Check,
	);
	formats.append(
		ID_FORMAT_LUV,
		&t("Color Values as CIE LU&V"),
		&t("Show and speak color values as CIE lightness and the u and v axes."),
		ItemKind::Check,
	);
	formats.append(ID_FORMAT_HUNTER, &t("Color Values as &Hunter Lab"), &t("Show and speak color values in the 1948 Hunter Lab system (different numbers from CIE Lab)."), ItemKind::Check);
	formats.append(
		ID_FORMAT_DECIMAL,
		&t("Color Values as &Decimal"),
		&t("Show and speak color values as one decimal number, 0 to 16777215."),
		ItemKind::Check,
	);
	formats.append(
		ID_FORMAT_WEBSAFE,
		&t("Color Values as &Web Safe"),
		&t("Show and speak each color snapped to the nearest web-safe color."),
		ItemKind::Check,
	);
	formats.append(
		ID_FORMAT_BINARY,
		&t("Color Values as &Binary"),
		&t("Show and speak color values as a 24-bit binary string."),
		ItemKind::Check,
	);

	let view = Menu::builder().build();
	view.append_submenu(formats, &t("Color &Format"), &t("How color values are shown and spoken across the app; exactly one format is ticked at a time."));
	view.append_separator();
	view.append(ID_ANNOUNCE_SLIDERS, &t("&Announce Slider Changes"), &t("Moving a slider says what changed and that everything else stayed the same. Off: only the slider's name and new value are said."), ItemKind::Check);
	view.append(ID_LISTS_SIDE_BY_SIDE, &t("Lists &Side by Side"), &t("Show the Palette and Elements lists side by side on one Lists page instead of on separate tabs. This session only."), ItemKind::Check);
	view.append(ID_FOCUS_MODE, &t("&Focus Mode"), &t("Open the Focus window: only the current focus, the statistics, and what you have built against your goal — and what is not built yet. This session only."), ItemKind::Check);

	let help = Menu::builder().build();
	help.append(
		ID_DICTIONARY,
		&t("&Dictionary…"),
		&t("Plain-language definitions of the color and design terms the app uses."),
		ItemKind::Normal,
	);
	help.append(
		ID_HELP_FILE,
		&t("&Help and Worked Examples…"),
		&t("How the app works, what every setting changes, and step-by-step examples."),
		ItemKind::Normal,
	);
	help.append(ID_SOUND_CUES, &t("&Sound Cues"), &t("Play a tone for each drawing change: pitch says height, stereo says left or right, length says size."), ItemKind::Check);
	help.append_separator();
	help.append(
		ID_ABOUT,
		&t("&About Blind Designer"),
		&t("What Blind Designer is."),
		ItemKind::Normal,
	);

	let bar = MenuBar::builder()
		.append(file, &t("&File"))
		.append(color, &t("&Color"))
		.append(design, &t("&Design"))
		.append(view, &t("&View"))
		.append(help, &t("&Help"))
		.build();
	bar.check_item(ID_SOUND_CUES, true); // sound cues start on
	bar.check_item(ID_FORMAT_HEX, true); // color values start as hex
	bar.check_item(ID_ANNOUNCE_SLIDERS, true); // slider announcements start on
	frame.set_menu_bar(bar);
}

fn main() {
	let _ = wxdragon::main(|_| {
		// Translations first, before any UI exists: pick the catalog
		// that best fits the system language (English when none does)
		// for both this app's strings and wxWidgets' stock strings.
		let mut translations = patois::wx::WxTranslationManager::new(env!("CARGO_PKG_NAME"));
		translations.initialize(WxStdCatalogLoader);
		let frame = Frame::builder()
			.with_title(&t("Blind Designer"))
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
		let (palette_page, palette_list) = list_panel(&notebook, &t("Palette"));
		let (elements_page, elements_list) = list_panel(&notebook, &t("Elements"));
		let (layers_page, layers_list) = list_panel(&notebook, &t("Layers"));
		notebook.add_page(&palette_page, &t("Palette"), true, None);
		notebook.add_page(&elements_page, &t("Elements"), false, None);
		notebook.add_page(&layers_page, &t("Layers"), false, None);

		let readout_label = StaticText::builder(&panel)
			.with_label(&t("Selected color"))
			.build();
		let readout = StaticText::builder(&panel)
			.with_label(&t(
				"No color selected yet. Open an image, or add a color by hex.",
			))
			.build();

		// The typed-fields surface (Elshara's interaction
		// modes): the hex row and the L C H spins row live in
		// ONE panel, so a mode switch shows or hides the whole
		// surface as a unit — visible only in Type mode.
		let typed_panel = Panel::builder(&panel).build();
		let hex_label = StaticText::builder(&typed_panel)
			.with_label(&t("Hex value"))
			.build();
		let hex_field = TextCtrl::builder(&typed_panel).build();
		hex_field.set_value("#3A7CA5");
		let apply_hex = Button::builder(&typed_panel)
			.with_label(&t("Apply Hex"))
			.build();

		let l_label = StaticText::builder(&typed_panel)
			.with_label(&t("Lightness (0 to 1)"))
			.build();
		let l_spin = SpinCtrlDouble::builder(&typed_panel)
			.with_range(0.0, 1.0)
			.build();
		let c_label = StaticText::builder(&typed_panel)
			.with_label(&t("Chroma (0 to 0.4)"))
			.build();
		let c_spin = SpinCtrlDouble::builder(&typed_panel)
			.with_range(0.0, 0.4)
			.build();
		let h_label = StaticText::builder(&typed_panel)
			.with_label(&t("Hue (0 to 360 degrees)"))
			.build();
		let h_spin = SpinCtrlDouble::builder(&typed_panel)
			.with_range(0.0, 360.0)
			.build();
		l_spin.set_increment(0.01);
		l_spin.set_digits(2);
		c_spin.set_increment(0.01);
		c_spin.set_digits(2);
		h_spin.set_increment(1.0);
		h_spin.set_digits(0);
		let apply_lch = Button::builder(&typed_panel)
			.with_label(&t("Apply L C H"))
			.build();
		let typed_sizer = BoxSizer::builder(Orientation::Vertical).build();
		let editor = BoxSizer::builder(Orientation::Horizontal).build();
		editor.add(&hex_label, 0, SizerFlag::All, 5);
		editor.add(&hex_field, 1, SizerFlag::All, 5);
		editor.add(&apply_hex, 0, SizerFlag::All, 5);
		typed_sizer.add_sizer(&editor, 0, SizerFlag::Expand, 0);
		let spins = BoxSizer::builder(Orientation::Horizontal).build();
		spins.add(&l_label, 0, SizerFlag::All, 5);
		spins.add(&l_spin, 1, SizerFlag::All, 5);
		spins.add(&c_label, 0, SizerFlag::All, 5);
		spins.add(&c_spin, 1, SizerFlag::All, 5);
		spins.add(&h_label, 0, SizerFlag::All, 5);
		spins.add(&h_spin, 1, SizerFlag::All, 5);
		spins.add(&apply_lch, 0, SizerFlag::All, 5);
		typed_sizer.add_sizer(&spins, 0, SizerFlag::Expand, 0);
		typed_panel.set_sizer(typed_sizer, true);

		// The Draw surface: one line saying the drawing tools
		// lead here, and the button that opens them. Visible
		// only in Draw mode.
		let draw_panel = Panel::builder(&panel).build();
		let draw_note = StaticText::builder(&draw_panel)
			.with_label(&t(
				"Draw mode: the pointer drawing tools lead. Everything else waits in the menus.",
			))
			.build();
		let draw_button = Button::builder(&draw_panel)
			.with_label(&t("Draw with Pointer\u{2026}"))
			.build();
		let draw_sizer = BoxSizer::builder(Orientation::Vertical).build();
		draw_sizer.add(
			&draw_note,
			0,
			SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
			5,
		);
		draw_sizer.add(&draw_button, 0, SizerFlag::All, 5);
		draw_panel.set_sizer(draw_sizer, true);

		let stats_label = StaticText::builder(&panel)
			.with_label(&t("Project statistics"))
			.build();
		let stats_text = StaticText::builder(&panel).build();

		let sizer = BoxSizer::builder(Orientation::Vertical).build();
		sizer.add(&notebook, 1, SizerFlag::Expand | SizerFlag::All, 5);
		sizer.add(
			&readout_label,
			0,
			SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
			5,
		);
		sizer.add(&readout, 0, SizerFlag::Expand | SizerFlag::All, 5);
		sizer.add(&typed_panel, 0, SizerFlag::Expand, 0);
		sizer.add(&draw_panel, 0, SizerFlag::Expand, 0);
		sizer.add(
			&stats_label,
			0,
			SizerFlag::Left | SizerFlag::Right | SizerFlag::Top,
			5,
		);
		sizer.add(&stats_text, 0, SizerFlag::Expand | SizerFlag::All, 5);
		panel.set_sizer(sizer, true);

		// wxID_ANY (-1): "assign an ID for me". This call used to pass -2
		// under a comment claiming it was wxID_ANY — in wx, -2 is
		// wxID_SEPARATOR, "invalid for normal items", and it trips the
		// wincmn.cpp CreateBase() "invalid id value" assert at startup
		// (issue #4).
		frame.create_status_bar(1, 0, ID_ANY as i32, &t("Status"));
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
			channel_format: ColorFormat::Hex,
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
			focus_mode: false,
			// A new project has no goal yet (the setup wizard
			// states one), and starts in grayscale: form first,
			// colour treatment later.
			goal: None,
			grayscale_start: true,
			// The startup project is a new project: it starts
			// in Slide mode (new projects slide; projects
			// opened from files carry their own mode).
			interaction_mode: InteractionMode::Slide,
			// No file yet, and the fresh project is its own
			// baseline (set for real just below, once the
			// fingerprint helper can see the whole state).
			design_path: None,
			saved_fingerprint: (0, 0, 0, false, 0),
		}));
		{
			let mut state = shared.borrow_mut();
			let fp = project_fingerprint(&state);
			state.saved_fingerprint = fp;
		}
		let channel_rebuild_timer = Timer::new(&frame);
		let win = Rc::new(Win {
			frame,
			panel,
			notebook,
			typed_panel,
			draw_panel,
			draw_button,
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
			shape_rows: RefCell::new(Vec::new()),
			channel_rebuild_timer,
			slider_target_choice: RefCell::new(None),
			slider_refreshing: std::cell::Cell::new(false),
			focus_frame: RefCell::new(None),
			focus_line: RefCell::new(None),
			focus_body: RefCell::new(None),
		});

		{
			// The channel-format picker's deferred rebuild: one
			// tick after a pick, rebuild the Sliders tab for the
			// newly chosen format (see build_channel_group).
			let (w, s) = (win.clone(), shared.clone());
			win.channel_rebuild_timer.on_tick(move |_| {
				build_sliders_tab(&w, &s);
			});
		}
		{
			// The Draw surface's button opens the same pointer
			// drawing dialog as Design, Draw with Pointer.
			let (w, s) = (win.clone(), shared.clone());
			win.draw_button
				.on_click(move |_| draw_pointer_dialog(&w, &s));
		}
		// The startup project's mode is Slide: the Sliders tab
		// is built, the typed and Draw panels stay hidden. No
		// focus move — the first-launch wizard opens next and
		// owns the keyboard until it is done.
		apply_interaction_mode(&win, &shared, false);
		{
			let state = shared.borrow();
			refresh_lists(&win, &state);
		}
		frame.set_status_text(&t("A new project: the setup wizard opens with the app — it states the goal, seeds the canvas, and pins the sliders for the kind you choose. Grayscale start is on: colors come later, per part, layer, or depth."),
			0,
		);

		wire_palette_list(&win, &shared);
		wire_elements_list(&win, &shared);
		{
			let (w, s) = (win.clone(), shared.clone());
			win.layers_list.on_key_down(move |data| {
				let WindowEventData::Keyboard(kb) = data else {
					return false;
				};
				if kb.get_key_code() != Some(WXK_RETURN) {
					return false;
				}
				if let Some(i) = w.layers_list.selected() {
					let state = s.borrow();
					if let Some(text) = inspect_layer(&state.drawing, i) {
						report_dialog(&w, &t("Inspector: layer"), &t("Layer inspector"), &text);
					}
				}
				true
			});
		}
		{
			let w = win.clone();
			win.layers_list
				.on_context_menu(move || show_layers_context(&w));
		}
		{
			let (w, s) = (win.clone(), shared.clone());
			win.layers_list.on_selection_changed(move || {
				// Selecting a layer in the main window makes it the
				// current layer: where new shapes land. (Renaming,
				// removing and hiding stay in Design, Layers.)
				let Some(i) = w.layers_list.selected() else {
					return;
				};
				let name = {
					let Ok(mut state) = s.try_borrow_mut() else {
						return;
					};
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
					&t("New shapes now land on layer \"%s\".").replacen(
						"%s",
						&(name).to_string(),
						1,
					),
					0,
				);
			});
		}
		{
			let (w, s) = (win.clone(), shared.clone());
			win.frame
				.on_menu_selected(move |e| dispatch(&w, &s, e.get_id()));
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
						w.frame.set_status_text(
							&t("That is not a hex color. Use RRGGBB, like 3A7CA5."),
							0,
						);
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
		// First launch meets the wizard (Elshara's report: it
		// never appeared at all): a fresh start opens the New
		// Project Wizard on the blank project, so the goal, the
		// seeded canvas and the kind's sliders are chosen up
		// front. Cancelling leaves the blank project exactly as
		// it is.
		setup_wizard(&win, &shared, true);
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
			assert!(
				*id > 0 && *id <= ID_HIGHEST,
				"menu ID {id} out of the valid wx range"
			);
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
