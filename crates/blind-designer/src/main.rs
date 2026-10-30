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
use std::rc::Rc;

use chroma_core::design::{css_for_colors, Design, Shape, ShapeKind};
use chroma_core::{describe, extract_palette, Oklch, Srgb};
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
const ID_ABOUT: Id = 401;

struct State {
	/// Palette colors, with image coverage when extracted from an image.
	palette: Vec<(Srgb, Option<f32>)>,
	design: Design,
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
		.map(|(i, (c, cov))| {
			let mut label = describe(*c);
			if let Some(cov) = cov {
				label.push_str(&format!(", {:.0} percent of the image", cov * 100.0));
			}
			(i, label)
		})
		.collect()
}

fn shape_rows(state: &State) -> Vec<(usize, String)> {
	state
		.design
		.shapes
		.iter()
		.enumerate()
		.map(|(i, s)| (i, state.design.caption_for(s)))
		.collect()
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
		describe(color),
		comp.to_hex(),
		chroma_core::hue_family(comp.to_oklch().h),
		wcag(color.contrast(state.design.background))
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
	win.frame
		.set_status_text(&format!("Selected: {}", describe(color)), 0);
}

fn selected_color(win: &Win, state: &State) -> Option<Srgb> {
	win.palette_list
		.selected()
		.and_then(|i| state.palette.get(i))
		.map(|(c, _)| *c)
}

/// Append colors to the palette and say what was added.
fn add_colors(win: &Win, state: &mut State, colors: &[Srgb], what: &str) {
	for c in colors {
		state.palette.push((*c, None));
	}
	refresh_lists(win, state);
	let names: Vec<String> = colors.iter().map(|c| describe(*c)).collect();
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

fn dispatch(win: &Win, shared: &Shared, id: Id) {
	let frame = win.frame;
	match id {
		ID_EXIT => frame.close(true),
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
		ID_OPEN_IMAGE => {
			if let Some(path) = file_dialog(
				&frame,
				"Open an image to extract its palette",
				"Image files (*.png;*.jpg;*.jpeg;*.webp)|*.png;*.jpg;*.jpeg;*.webp",
				false,
			) {
				match image::open(&path) {
					Ok(img) => {
						let entries = extract_palette(&img, 8);
						let mut state = shared.borrow_mut();
						state.palette =
							entries.iter().map(|e| (e.color, Some(e.coverage))).collect();
						if let Some(first) = entries.first() {
							state.design.background = first.color;
						}
						refresh_lists(win, &state);
						frame.set_status_text(
							&format!(
								"Extracted {} colors from the image. The largest is now the design background.",
								entries.len()
							),
							0,
						);
						if let Some(first) = entries.first() {
							select_color(win, &state, first.color);
						}
					}
					Err(e) => {
						frame.set_status_text(&format!("Could not open that image: {e}"), 0);
					}
				}
			}
		}
		ID_EXPORT_PNG => {
			if let Some(path) = file_dialog(&frame, "Export design as PNG", "PNG image (*.png)|*.png", true) {
				let state = shared.borrow();
				match state.design.render().save(&path) {
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
				match std::fs::write(&path, state.design.caption_sheet()) {
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
					vec![state.design.background]
				} else {
					state.palette.iter().map(|(c, _)| *c).collect()
				};
				match std::fs::write(&path, css_for_colors(&colors, "color")) {
					Ok(()) => frame.set_status_text(&format!("Exported CSS to {path}"), 0),
					Err(e) => frame.set_status_text(&format!("Export failed: {e}"), 0),
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
			add_colors(win, &mut state, &colors, what);
		}
		ID_REMOVE_COLOR => {
			let mut state = shared.borrow_mut();
			if let Some(i) = win.palette_list.selected() {
				if i < state.palette.len() {
					let gone = state.palette.remove(i).0;
					refresh_lists(win, &state);
					frame.set_status_text(&format!("Removed {}", describe(gone)), 0);
				}
			} else {
				frame.set_status_text("Select a palette color first.", 0);
			}
		}
		ID_SET_BACKGROUND => {
			let mut state = shared.borrow_mut();
			if let Some(c) = selected_color(win, &state) {
				state.design.background = c;
				refresh_lists(win, &state);
				frame.set_status_text(
					&format!("Design background is now {}", describe(c)),
					0,
				);
				select_color(win, &state, c);
			} else {
				frame.set_status_text("Select a palette color first.", 0);
			}
		}
		ID_CANVAS_SIZE => {
			let (w, h) = {
				let state = shared.borrow();
				(state.design.width, state.design.height)
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
				state.design.width = v[0] as u32;
				state.design.height = v[1] as u32;
				frame.set_status_text(
					&format!("Canvas is now {} by {} pixels.", v[0], v[1]),
					0,
				);
			}
		}
		ID_ADD_RECT | ID_ADD_ELLIPSE => {
			let kind = if id == ID_ADD_RECT {
				ShapeKind::Rectangle
			} else {
				ShapeKind::Ellipse
			};
			let fill = {
				let state = shared.borrow();
				selected_color(win, &state)
			};
			let Some(fill) = fill else {
				frame.set_status_text("Select a palette color to fill the shape first.", 0);
				return;
			};
			if let Some(v) = numbers_dialog(
				&frame,
				&format!("Add {} filled with {}", kind.name(), describe(fill)),
				&[
					("X position", 0, 8000, 0),
					("Y position", 0, 8000, 0),
					("Width", 1, 8000, 200),
					("Height", 1, 8000, 120),
				],
			) {
				let mut state = shared.borrow_mut();
				state
					.design
					.shapes
					.push(Shape::new(kind, v[0], v[1], v[2] as u32, v[3] as u32, fill));
				refresh_lists(win, &state);
				frame.set_status_text(
					&format!("Added {}", state.design.caption_for(state.design.shapes.last().unwrap())),
					0,
				);
			}
		}
		ID_REMOVE_SHAPE => {
			let mut state = shared.borrow_mut();
			if let Some(i) = win.shapes_list.selected() {
				if i < state.design.shapes.len() {
					state.design.shapes.remove(i);
					refresh_lists(win, &state);
					frame.set_status_text("Shape removed.", 0);
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
		.append_item(ID_OPEN_IMAGE, "&Open Image…\tCtrl+O", "")
		.append_item(ID_EXPORT_PNG, "Export &PNG…\tCtrl+E", "")
		.append_item(ID_EXPORT_CAPTIONS, "Export &Captions…", "")
		.append_item(ID_EXPORT_CSS, "Export C&SS…", "")
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
		.append_item(ID_SET_BACKGROUND, "Set &Background to Selected Color", "")
		.append_item(ID_REMOVE_SHAPE, "Remove Selected &Shape", "")
		.build();
	let help = Menu::builder().append_item(ID_ABOUT, "&About Blind Designer", "").build();
	let bar = MenuBar::builder()
		.append(file, "&File")
		.append(palette, "&Palette")
		.append(design, "&Design")
		.append(help, "&Help")
		.build();
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

		frame.create_status_bar(1, 0, -2, "Status"); // -2 is wxID_ANY
		build_menus(&frame);

		let start = Srgb::new(58, 124, 165);
		let shared: Shared = Rc::new(RefCell::new(State {
			palette: vec![(start, None)],
			design: Design::new(800, 600, Srgb::WHITE),
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
								state.palette[i] = (c, None);
								refresh_lists(&w, &state);
								select_color(&w, &state, c);
								return;
							}
						}
						state.palette.push((c, None));
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
						state.palette[i] = (c, None);
						refresh_lists(&w, &state);
						select_color(&w, &state, c);
						return;
					}
				}
				state.palette.push((c, None));
				refresh_lists(&w, &state);
				select_color(&w, &state, c);
			});
		}

		frame.show(true);
		frame.centre();
	});
}
