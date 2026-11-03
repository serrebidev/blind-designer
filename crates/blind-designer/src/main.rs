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

use chroma_core::animation::{MAX_FRAME_MS, MIN_FRAME_MS};
use chroma_core::brushes::{Brush, BrushLibrary, PATTERNS};
use chroma_core::design::{css_for_colors, Shape, ShapeKind};
use chroma_core::dictionary;
use chroma_core::drawing::{Drawing, Event};
use chroma_core::palette::{PaletteColor, Provenance};
use chroma_core::plain::{parse_color, parse_position, parse_size};
use chroma_core::project::Project;
use chroma_core::sound::{cue_at, cue_for_shape, tone_wav, Cue};
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
const ID_ABOUT: Id = 401;
const ID_DICTIONARY: Id = 402;
const ID_HELP_FILE: Id = 403;
const ID_SOUND_CUES: Id = 404;
const ID_FORMAT_HEX: Id = 501;
const ID_FORMAT_RGB: Id = 502;
const ID_FORMAT_HSL: Id = 503;
const ID_FORMAT_HSV: Id = 504;

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
	ID_ABOUT,
	ID_DICTIONARY,
	ID_HELP_FILE,
	ID_SOUND_CUES,
	ID_FORMAT_HEX,
	ID_FORMAT_RGB,
	ID_FORMAT_HSL,
	ID_FORMAT_HSV,
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
	/// The current stroke brush: width, color and pattern new strokes
	/// are painted with. Applying a saved brush replaces it.
	brush: Brush,
	/// The saved-brush library, persisted beside the executable.
	brushes: BrushLibrary,
}

struct Win {
	frame: Frame,
	palette_list: QuietList<usize>,
	shapes_list: QuietList<usize>,
	readout: StaticText,
	hex_field: TextCtrl,
	l_spin: SpinCtrlDouble,
	c_spin: SpinCtrlDouble,
	h_spin: SpinCtrlDouble,
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

/// Shape rows across all layers, in paint order. The payload is the
/// flat index over every layer's shapes; `locate` turns it back into
/// (layer, shape) for the edit actions.
fn shape_rows(state: &State) -> Vec<(usize, String)> {
	let design = state.drawing.flatten();
	let mut rows = Vec::new();
	let mut flat = 0;
	for layer in &state.drawing.layers {
		for s in &layer.shapes {
			let mut label = format!("{}: {}", layer.name, design.caption_for(s));
			if !layer.visible {
				label.push_str(" (layer hidden)");
			}
			rows.push((flat, label));
			flat += 1;
		}
	}
	rows
}

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
			play_cue(cue);
		}
	}
}

thread_local! {
	/// The cue currently sounding. wxSound plays asynchronously, so the
	/// object is kept alive here; replacing it stops the previous cue.
	static CURRENT_SOUND: RefCell<Option<Sound>> = const { RefCell::new(None) };
}

fn play_cue(cue: Cue) {
	let sound = Sound::from_data(&tone_wav(&cue));
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
	win.palette_list.update_labelled(palette_rows(state), Select::Keep);
	win.shapes_list.update_labelled(shape_rows(state), Select::Keep);
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
	win.palette_list
		.selected()
		.and_then(|i| state.palette.get(i))
		.map(|pc| pc.color)
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
		play_cue(cue_for_shape(&shape, state.drawing.width, state.drawing.height));
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

fn layer_rows(state: &State) -> Vec<(usize, String)> {
	state
		.drawing
		.layers
		.iter()
		.enumerate()
		.map(|(i, l)| {
			let mut label = format!("{} ({} shapes)", l.name, l.shapes.len());
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
	{
		let state = shared.borrow();
		list.update_labelled(layer_rows(&state), Select::Start);
	}
	// Refresh the dialog list and speak the newest log entry.
	let sync = {
		let (list, shared) = (list.clone(), shared.clone());
		move || {
			let state = shared.borrow();
			list.update_labelled(layer_rows(&state), Select::Keep);
			if let Some(last) = state.drawing.log.last() {
				frame.set_status_text(&last.describe(&state.drawing), 0);
			}
		}
	};
	let sync = Rc::new(sync);
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
	// Refresh the dialog list and speak the newest log entry.
	let sync = {
		let (list, shared) = (list.clone(), shared.clone());
		move || {
			let state = shared.borrow();
			list.update_labelled(frame_rows(&state), Select::Keep);
			if let Some(last) = state.drawing.log.last() {
				frame.set_status_text(&last.describe(&state.drawing), 0);
			}
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
			if let Some(v) = numbers_dialog(
				&frame,
				"Frame duration",
				&[(
					"Duration in milliseconds",
					MIN_FRAME_MS as i32,
					MAX_FRAME_MS as i32,
					current as i32,
				)],
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
	// Refresh the list and speak the newest log entry.
	let sync = {
		let (list, shared) = (list.clone(), shared.clone());
		move || {
			let state = shared.borrow();
			list.update_labelled(element_rows(&state), Select::Keep);
			if let Some(last) = state.drawing.log.last() {
				frame.set_status_text(&last.describe(&state.drawing), 0);
			}
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
			let cur = {
				let state = shared.borrow();
				let s = &state.drawing.layers[li].shapes[si];
				if s.kind == ShapeKind::Stroke {
					frame.set_status_text(
						"A stroke's size is its points; move it or add a new one.",
						0,
					);
					return;
				}
				(s.w, s.h)
			};
			let Some(v) = fields_dialog(
				&frame,
				"Resize element",
				&[(
					"New size: numbers like 200 by 100, or words like a bit wider, a bit taller",
					&format!("{} by {}", cur.0, cur.1),
				)],
			) else {
				return;
			};
			match parse_size(&v[0], Some(cur)) {
				Some((w, h)) => {
					shared.borrow_mut().drawing.resize_shape(li, si, w, h);
					sync();
				}
				None => frame.set_status_text(
					"I could not understand that size. Use numbers like 200 by 100, or words like a bit wider.",
					0,
				),
			}
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
		let (list, shared) = (list.clone(), shared.clone());
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
	let Some(v) = numbers_dialog(
		&frame,
		"How many colors to extract",
		&[("Number of colors", 1, 64, 8)],
	) else {
		return;
	};
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

fn dispatch(win: &Win, shared: &Shared, id: Id) {
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
							state.layer = 0;
							let layers = state.drawing.layers.len();
							let shapes: usize =
								state.drawing.layers.iter().map(|l| l.shapes.len()).sum();
							let colors = state.palette.len();
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
		ID_FORMAT_HEX | ID_FORMAT_RGB | ID_FORMAT_HSL | ID_FORMAT_HSV => {
			let format = match id {
				ID_FORMAT_HEX => ColorFormat::Hex,
				ID_FORMAT_RGB => ColorFormat::Rgb,
				ID_FORMAT_HSL => ColorFormat::Hsl,
				_ => ColorFormat::Hsv,
			};
			let current;
			{
				let mut state = shared.borrow_mut();
				state.color_format = format;
				if let Some(bar) = frame.get_menu_bar() {
					for (fid, on) in [
						(ID_FORMAT_HEX, format == ColorFormat::Hex),
						(ID_FORMAT_RGB, format == ColorFormat::Rgb),
						(ID_FORMAT_HSL, format == ColorFormat::Hsl),
						(ID_FORMAT_HSV, format == ColorFormat::Hsv),
					] {
						bar.check_item(fid, on);
					}
				}
				refresh_lists(win, &state);
				current = selected_color(win, &state);
			}
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
		ID_COMPLEMENTARY | ID_SPLIT | ID_ANALOGOUS | ID_TRIADIC | ID_TETRADIC | ID_SHADES
		| ID_TINTS | ID_TONES => {
			let mut state = shared.borrow_mut();
			let Some(base) = selected_color(win, &state) else {
				frame.set_status_text("Select a palette color first.", 0);
				return;
			};
			let o = base.to_oklch();
			let (colors, what): (Vec<Srgb>, &str) = match id {
				ID_COMPLEMENTARY => (vec![o.complementary().to_srgb()], "complementary"),
				ID_SPLIT => (
					o.split_complementary().iter().map(|c| c.to_srgb()).collect(),
					"split complementary",
				),
				ID_ANALOGOUS => (
					o.analogous(30.0).iter().map(|c| c.to_srgb()).collect(),
					"analogous (30 degrees)",
				),
				ID_TRIADIC => (o.triadic().iter().map(|c| c.to_srgb()).collect(), "triadic"),
				ID_TETRADIC => (o.tetradic().iter().map(|c| c.to_srgb()).collect(), "tetradic"),
				ID_SHADES => (o.shades(4).iter().map(|c| c.to_srgb()).collect(), "shades"),
				ID_TINTS => (o.tints(4).iter().map(|c| c.to_srgb()).collect(), "tints"),
				_ => (o.tones(4).iter().map(|c| c.to_srgb()).collect(), "tones"),
			};
			let provenance =
				Provenance::Generated { scheme: format!("{what} of {}", base.to_hex()) };
			add_colors(win, &mut state, &colors, what, &provenance);
		}
		ID_REMOVE_COLOR => {
			let mut state = shared.borrow_mut();
			if let Some(i) = win.palette_list.selected() {
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
				let index = state.drawing.add_shape(layer, Shape::new(kind, x, y, w, h, color));
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
				let index = state.drawing.add_shape(
					layer,
					Shape::text(phrase.trim(), font.trim(), size, x, y, w, h, color),
				);
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
				let index =
					state.drawing.add_shape(layer, Shape::image(&path, x, y, w, h, color));
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
			let Some(width) = v[0].trim().parse::<u32>().ok().filter(|w| (1..=200).contains(w))
			else {
				frame.set_status_text("The brush width needs to be a whole number from 1 to 200.", 0);
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
			let index = state.drawing.add_shape(layer, shape);
			refresh_lists(win, &state);
			if let Some(index) = index {
				speak_added(win, &state, layer, index);
			}
		}
		ID_MOVE_SHAPE | ID_ROTATE_SHAPE => {
			let Some(flat) = win.shapes_list.selected() else {
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
			let Some(flat) = win.shapes_list.selected() else {
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
			let (Some(flat), Some(c)) = (win.shapes_list.selected(), selected_color(win, &state))
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
			let state = shared.borrow();
			refresh_lists(win, &state);
		}
		ID_ELEMENTS => {
			elements_dialog(win, shared);
			let state = shared.borrow();
			refresh_lists(win, &state);
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
			if let Some(flat) = win.shapes_list.selected() {
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
	let file = Menu::builder()
		.append_item(ID_OPEN_DESIGN, "Open &Design…", "")
		.append_item(ID_SAVE_DESIGN, "&Save Design…", "")
		.append_separator()
		.append_item(ID_OPEN_IMAGE, "&Open Image…\tCtrl+O", "")
		.append_item(ID_EXTRACT_IMAGE, "&Extract Colors from Image…", "")
		.append_item(ID_IMPORT_CSS, "Import &CSS Colors…", "")
		.append_item(ID_IMPORT_HTML, "Import &HTML Design…", "")
		.append_separator()
		.append_item(ID_EXPORT_PNG, "Export &PNG…\tCtrl+E", "")
		.append_item(ID_EXPORT_CAPTIONS, "Export &Captions…", "")
		.append_item(ID_EXPORT_CSS, "Export C&SS…", "")
		.append_item(ID_EXPORT_SUMMARY, "Export &Summary (recreation file)…", "")
		.append_item(ID_EXPORT_HTML, "Export HT&ML Page…", "")
		.append_item(ID_EXPORT_GIF, "Export Animated &GIF…", "")
		.append_item(ID_EXPORT_ANIM_HTML, "Export Animated HTML…", "")
		.append_item(ID_EXPORT_ANIM_DESC, "Export Animation &Description…", "")
		.append_separator()
		.append_item(ID_EXIT, "E&xit", "")
		.build();
	let palette = Menu::builder()
		.append_item(ID_COMPLEMENTARY, "&Complementary (hue + 180)", "")
		.append_item(ID_SPLIT, "&Split Complementary (hue ± 150)", "")
		.append_item(ID_ANALOGOUS, "&Analogous (hue ± 30)", "")
		.append_item(ID_TRIADIC, "&Triadic (hue ± 120)", "")
		.append_item(ID_TETRADIC, "T&etradic", "")
		.append_separator()
		.append_item(ID_SHADES, "Add 4 S&hades (lightness down)", "")
		.append_item(ID_TINTS, "Add 4 &Tints (lightness up)", "")
		.append_item(ID_TONES, "Add 4 Tone&s (chroma down)", "")
		.append_separator()
		.append_item(ID_REMOVE_COLOR, "&Remove Selected Color", "")
		.build();
	let design = Menu::builder()
		.append_item(ID_CANVAS_SIZE, "Canvas &Size…", "")
		.append_item(ID_ADD_RECT, "Add &Rectangle…", "")
		.append_item(ID_ADD_ELLIPSE, "Add &Ellipse…", "")
		.append_item(ID_ADD_STROKE, "Add Stro&ke…", "")
		.append_item(ID_ADD_TEXT, "Add &Text…", "")
		.append_item(ID_ADD_IMAGE, "Add &Image…", "")
		.append_item(ID_SET_BACKGROUND, "Set &Background to Selected Color", "")
		.append_separator()
		.append_item(ID_MOVE_SHAPE, "&Move Selected Shape…", "")
		.append_item(ID_ROTATE_SHAPE, "R&otate Selected Shape…", "")
		.append_item(ID_RESIZE_SHAPE, "Resi&ze Selected Shape…", "")
		.append_item(ID_RECOLOR_SHAPE, "Set Selected Shape to Palette &Color", "")
		.append_item(ID_REMOVE_SHAPE, "Remove Selected Sha&pe", "")
		.append_separator()
		.append_item(ID_ELEMENTS, "E&lements…", "")
		.append_item(ID_LAYERS, "&Layers…", "")
		.append_item(ID_ANIMATION, "Anim&ation…", "")
		.append_separator()
		.append_item(ID_SAVE_BRUSH, "Save Current Br&ush…", "")
		.append_item(ID_BRUSHES, "Brush Li&brary…", "")
		.build();
	let view = Menu::builder()
		.append_check_item(ID_FORMAT_HEX, "Color Values as &Hex", "")
		.append_check_item(ID_FORMAT_RGB, "Color Values as &RGB", "")
		.append_check_item(ID_FORMAT_HSL, "Color Values as HS&L", "")
		.append_check_item(ID_FORMAT_HSV, "Color Values as HS&V", "")
		.build();
	let help = Menu::builder()
		.append_item(ID_DICTIONARY, "&Dictionary…", "")
		.append_item(ID_HELP_FILE, "&Help and Worked Examples…", "")
		.append_check_item(ID_SOUND_CUES, "&Sound Cues", "")
		.append_separator()
		.append_item(ID_ABOUT, "&About Blind Designer", "")
		.build();
	let bar = MenuBar::builder()
		.append(file, "&File")
		.append(palette, "&Palette")
		.append(design, "&Design")
		.append(view, "&View")
		.append(help, "&Help")
		.build();
	bar.check_item(ID_SOUND_CUES, true); // sound cues start on
	bar.check_item(ID_FORMAT_HEX, true); // color values start as hex
	frame.set_menu_bar(bar);
}

fn main() {
	let _ = wxdragon::main(|_| {
		let frame = Frame::builder()
			.with_title("Blind Designer")
			.with_size(Size::new(760, 640))
			.build();
		let panel = Panel::builder(&frame).build();

		let palette_label = StaticText::builder(&panel).with_label("Palette").build();
		let palette_list = QuietList::new(&panel, "Palette");
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

		let shapes_label = StaticText::builder(&panel).with_label("Shapes").build();
		let shapes_list = QuietList::new(&panel, "Shapes");

		let sizer = BoxSizer::builder(Orientation::Vertical).build();
		sizer.add(&palette_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
		sizer.add(&palette_list, 1, SizerFlag::Expand | SizerFlag::All, 5);
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
		sizer.add(&shapes_label, 0, SizerFlag::Left | SizerFlag::Right | SizerFlag::Top, 5);
		sizer.add(&shapes_list, 1, SizerFlag::Expand | SizerFlag::All, 5);
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
			palette: vec![PaletteColor::starter(start)],
			drawing: Drawing::new(800, 600, Srgb::WHITE),
			layer: 0,
			sound_cues: true,
			color_format: ColorFormat::Hex,
			brush: Brush::new("", 3, start, "solid"),
			brushes: load_brushes(),
		}));
		let win = Rc::new(Win {
			frame,
			palette_list,
			shapes_list,
			readout,
			hex_field,
			l_spin,
			c_spin,
			h_spin,
		});

		{
			let state = shared.borrow();
			refresh_lists(&win, &state);
			win.palette_list.select(Some(&0));
			select_color(&win, &state, start);
		}

		{
			let (w, s) = (win.clone(), shared.clone());
			win.palette_list.on_selection_changed(move || {
				let state = s.borrow();
				if let Some(c) = selected_color(&w, &state) {
					select_color(&w, &state, c);
				}
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
						if let Some(i) = w.palette_list.selected() {
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
				if let Some(i) = w.palette_list.selected() {
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
