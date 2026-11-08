//! goals: what a project is for, stated once and measured as
//! you work.
//!
//! The setup wizard's first question — "What do you want to
//! create or build?" — names the project's kind, and its focus
//! questions become the goal: answers the designer confirms as
//! they go. Any answer may be left open ("no fixed answer"): the
//! goal stays flexible there on purpose, and the accounting says
//! so instead of pretending. From the goal and the design, the
//! goal report computes two percentages (Elshara's pair): the
//! goal match — of the criteria that are settled, how many the
//! design meets — and the creation — of every criterion there
//! is, settled or not, how many the design meets. Unanswered
//! criteria count as not-yet in the creation, never in the goal
//! match.
//!
//! Also here: the recognizability report (per project type,
//! plainly labelled a rough guide, not a measurement) and the
//! seed drawing a new wizard project starts from.

use patois::t;

use crate::design::{Shape, ShapeKind};
use crate::drawing::{DepthRole, Drawing};
use crate::project::{Project, ProjectStats};
use crate::Srgb;

/// What the project is for: the setup wizard's first answer.
/// Each kind brings its own focus questions and its own
/// checkable criteria.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalKind {
	Writing,
	ImageCosmetic,
	ImageCreation,
	Mixed,
	AccessibilityReview,
	Conversion,
}

impl GoalKind {
	/// Every kind, in the order the wizard's first step lists
	/// them: the writing project first (it is the one whose
	/// tools are words), then the picture kinds.
	pub const ALL: [GoalKind; 6] = [
		GoalKind::Writing,
		GoalKind::ImageCreation,
		GoalKind::ImageCosmetic,
		GoalKind::Mixed,
		GoalKind::AccessibilityReview,
		GoalKind::Conversion,
	];

	/// The stable id project files store.
	pub fn id(&self) -> &'static str {
		match self {
			GoalKind::Writing => "writing",
			GoalKind::ImageCosmetic => "image-cosmetic",
			GoalKind::ImageCreation => "image-creation",
			GoalKind::Mixed => "mixed",
			GoalKind::AccessibilityReview => "accessibility-review",
			GoalKind::Conversion => "conversion",
		}
	}

	pub fn from_id(id: &str) -> Option<GoalKind> {
		GoalKind::ALL.iter().find(|k| k.id() == id).copied()
	}

	/// The spoken name the wizard lists: "a writing project",
	/// and so on.
	pub fn name(&self) -> String {
		match self {
			GoalKind::Writing => t("a writing project (mostly typing)"),
			GoalKind::ImageCosmetic => t("an image project (cosmetic work on a picture)"),
			GoalKind::ImageCreation => t("creating an image (shapes and colours)"),
			GoalKind::Mixed => t("a mixed project (words and pictures together)"),
			GoalKind::AccessibilityReview => t("an accessibility review"),
			GoalKind::Conversion => t("a conversion (one form into another)"),
		}
	}

	/// The focus questions this kind asks, in order. Every one
	/// of them may be answered "no fixed answer" — see
	/// `ProjectGoal`.
	pub fn questions(&self) -> Vec<GoalQuestion> {
		match self {
			GoalKind::Writing => vec![
				GoalQuestion {
					question: t("What will the writing be?"),
					options: vec![
						t("A document or letter"),
						t("A sign or label"),
						t("Words inside a picture"),
					],
				},
				GoalQuestion {
					question: t("How will it be read?"),
					options: vec![
						t("On screen"),
						t("In print"),
						t("Both on screen and in print"),
					],
				},
			],
			GoalKind::ImageCreation => vec![
				GoalQuestion {
					question: t("What is the picture of?"),
					options: vec![
						t("A scene or place"),
						t("An object"),
						t("A person or character"),
						t("A pattern or decoration"),
					],
				},
				GoalQuestion {
					question: t("What should stand out most?"),
					options: vec![
						t("The foreground"),
						t("The colours"),
						t("The shapes themselves"),
					],
				},
			],
			GoalKind::ImageCosmetic => vec![
				GoalQuestion {
					question: t("What are you improving?"),
					options: vec![
						t("A photo or picture"),
						t("A palette or colour scheme"),
						t("A design made here"),
					],
				},
				GoalQuestion {
					question: t("What matters most?"),
					options: vec![
						t("The colours"),
						t("The contrast"),
						t("The balance of light and dark"),
					],
				},
			],
			GoalKind::Mixed => vec![
				GoalQuestion {
					question: t("How will it be presented?"),
					options: vec![
						t("As a picture"),
						t("As a document"),
						t("As a web page"),
						t("As slides"),
					],
				},
				GoalQuestion {
					question: t("How will people use it?"),
					options: vec![
						t("Just look at it or read it"),
						t("Hear it described"),
						t("Feel it (tactile)"),
					],
				},
			],
			GoalKind::AccessibilityReview => vec![
				GoalQuestion {
					question: t("What are you reviewing?"),
					options: vec![
						t("A design made here"),
						t("An imported design"),
						t("A picture"),
					],
				},
				GoalQuestion {
					question: t("Who is it for?"),
					options: vec![t("Blind readers"), t("Low-vision readers"), t("Everyone")],
				},
			],
			GoalKind::Conversion => vec![
				GoalQuestion {
					question: t("What are you converting from?"),
					options: vec![t("A picture file"), t("A web page"), t("Another design")],
				},
				GoalQuestion {
					question: t("What should it become?"),
					options: vec![
						t("A tactile design"),
						t("A described design (words)"),
						t("A web page"),
					],
				},
			],
		}
	}
}

/// One focus question of a goal kind, with the answers the
/// wizard offers. "No fixed answer" is always offered as well;
/// it is not in `options` because it is the absence of an
/// answer, and the goal model stores it as `None`.
#[derive(Debug, Clone)]
pub struct GoalQuestion {
	pub question: String,
	pub options: Vec<String>,
}

/// A project's stated goal: its kind, and one entry per focus
/// question of that kind — the question's words and the answer
/// chosen, or `None` where the answer was left open ("no fixed
/// answer") on purpose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectGoal {
	pub kind: GoalKind,
	pub answers: Vec<(String, Option<String>)>,
}

impl ProjectGoal {
	/// A goal for `kind` with every answer open.
	pub fn new(kind: GoalKind) -> ProjectGoal {
		ProjectGoal {
			kind,
			answers: kind
				.questions()
				.iter()
				.map(|q| (q.question.to_string(), None))
				.collect(),
		}
	}

	/// The answer recorded for one of the kind's questions, by
	/// the question's words: `Some(Some(answer))` when answered,
	/// `Some(None)` when deliberately open, `None` when the
	/// question is not part of this goal at all.
	pub fn answer_for(&self, question: &str) -> Option<&Option<String>> {
		self.answers
			.iter()
			.find(|(q, _)| q == question)
			.map(|(_, a)| a)
	}

	/// Has this question been given a fixed answer?
	pub fn is_answered(&self, question: &str) -> bool {
		matches!(self.answer_for(question), Some(Some(_)))
	}

	/// The questions left open, in order.
	pub fn open_questions(&self) -> Vec<&str> {
		self.answers
			.iter()
			.filter(|(_, a)| a.is_none())
			.map(|(q, _)| q.as_str())
			.collect()
	}
}

/// The goal report: two percentages and the lines behind them.
/// `goal_match_pct` counts only settled criteria; `creation_pct`
/// counts them all, open answers included as not-yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalReport {
	pub goal_match_pct: u32,
	pub creation_pct: u32,
	pub lines: Vec<String>,
	pub suggestions: Vec<String>,
}

impl GoalReport {
	/// The pair of percentages as one line — the form the main
	/// window's statistics line carries.
	pub fn one_line(&self) -> String {
		t("Goal match %s percent. Creation %s percent.")
			.replacen("%s", &(self.goal_match_pct).to_string(), 1)
			.replacen("%s", &(self.creation_pct).to_string(), 1)
	}

	/// The full report text: the percentages, every criterion
	/// line, the suggestions, and the answers still open.
	pub fn full_text(&self, goal: &ProjectGoal) -> String {
		let mut out = t(
			"Goal: %s.\nGoal match: %s percent of the settled criteria the design meets.\n\
			 Creation: %s percent of all criteria, open answers counted as not yet.\n",
		)
		.replacen("%s", &(goal.kind.name()).to_string(), 1)
		.replacen("%s", &(self.goal_match_pct).to_string(), 1)
		.replacen("%s", &(self.creation_pct).to_string(), 1);
		for (question, answer) in &goal.answers {
			match answer {
				Some(a) => out.push_str(
					&t("%s %s.\n")
						.replacen("%s", &(question).to_string(), 1)
						.replacen("%s", &(a).to_string(), 1),
				),
				None => out.push_str(&t("%s No fixed answer — left open.\n").replacen(
					"%s",
					&(question).to_string(),
					1,
				)),
			}
		}
		for line in &self.lines {
			out.push_str(&format!("{line}\n"));
		}
		if !self.suggestions.is_empty() {
			out.push_str(&t("Suggested improvements:\n"));
			for s in &self.suggestions {
				out.push_str(&format!("- {s}\n"));
			}
		}
		out
	}
}

/// One judged criterion: whether it is settled (its question,
/// when it has one, was given a fixed answer), whether the
/// design meets it, the line the report speaks, and the
/// suggestion an unmet settled criterion earns.
struct Criterion {
	settled: bool,
	met: bool,
	line: String,
	suggestion: Option<String>,
}

/// The facts the criteria judge, gathered once from the project.
struct Facts {
	elements: usize,
	texts: usize,
	images: usize,
	colors: usize,
	/// Text elements whose contrast against the background is
	/// under 3 to 1 — the readability report's "unreadable".
	unreadable_texts: usize,
	/// Elements standing out from the background at 3 to 1+.
	standout_elements: usize,
	has_background: bool,
	has_depth_layer: bool,
}

fn facts(project: &Project) -> Facts {
	let d = &project.drawing;
	let stats = ProjectStats::gather(d, project.palette.len());
	let mut unreadable_texts = 0;
	let mut standout_elements = 0;
	for layer in &d.layers {
		for shape in &layer.shapes {
			if shape.fill.contrast(d.background) >= 3.0 {
				standout_elements += 1;
			}
			if shape.kind == ShapeKind::Text && shape.fill.contrast(d.background) < 3.0 {
				unreadable_texts += 1;
			}
		}
	}
	Facts {
		elements: stats.elements,
		texts: stats.texts,
		images: stats.images,
		colors: stats.colors,
		unreadable_texts,
		standout_elements,
		has_background: d.background != Srgb::WHITE
			|| d.layers.iter().any(|l| l.depth == DepthRole::Background),
		has_depth_layer: d.layers.iter().any(|l| l.depth != DepthRole::Unassigned),
	}
}

/// Judge the project's goal against its design. `None` when the
/// project has no goal — a project without one is not failing
/// anything; it simply has nothing to measure against.
pub fn goal_report(project: &Project) -> Option<GoalReport> {
	let goal = project.goal.as_ref()?;
	let f = facts(project);
	let mut criteria: Vec<Criterion> = Vec::new();
	// A criterion no question gates: always settled.
	let plain = |met: bool, label: &str, suggestion: &str, criteria: &mut Vec<Criterion>| {
		criteria.push(Criterion {
			settled: true,
			met,
			line: t("%s: %s.")
				.replacen("%s", if met { "Met" } else { "Not yet" }, 1)
				.replacen("%s", label, 1),
			suggestion: (!met).then(|| suggestion.to_string()),
		});
	};
	// A criterion gated by one focus question: settled only
	// when that question has a fixed answer.
	let gated = |question: &str, label: &str, criteria: &mut Vec<Criterion>| {
		let settled = goal.is_answered(question);
		criteria.push(Criterion {
			settled,
			met: settled,
			line: if settled {
				t("Met: %s.").replacen("%s", label, 1)
			} else {
				t("Open: %s — that answer has no fixed answer yet.").replacen(
					"%s",
					label,
					1,
				)
			},
			suggestion: None,
		});
	};
	match goal.kind {
		GoalKind::Writing => {
			plain(
				f.texts >= 1,
				&t("the design has at least one text element"),
				&t("Add a text element: Design, Add, Text."),
				&mut criteria,
			);
			plain(
				f.texts >= 1 && f.unreadable_texts == 0,
				&t("every text element is readable against the background (none unreadable)"),
				&t("Raise the contrast of the unreadable words: Color, Text Readability names them."),
				&mut criteria,
			);
			gated(
				&t("What will the writing be?"),
				&t("you have said what the writing will be"),
				&mut criteria,
			);
		}
		GoalKind::ImageCreation => {
			plain(
				f.elements >= 1,
				&t("the design has at least one shape"),
				&t("Add a shape: Design, Add offers rectangles, ellipses, strokes and symbols."),
				&mut criteria,
			);
			plain(
				f.has_background,
				&t("the design has a background (a chosen background colour, or a layer with the background depth role)"),
				&t("Set a background colour from the palette, or give a layer the background depth role in Design, Layers."),
				&mut criteria,
			);
			gated(
				&t("What is the picture of?"),
				&t("you have said what the picture is of"),
				&mut criteria,
			);
		}
		GoalKind::ImageCosmetic => {
			plain(
				f.images >= 1 || f.colors >= 2,
				&t("the project has an image element, or at least two palette colours to work between"),
				&t("Bring a picture in (File, Import, Open Image) or add a second palette colour."),
				&mut criteria,
			);
			gated(
				&t("What are you improving?"),
				&t("you have said what you are improving"),
				&mut criteria,
			);
		}
		GoalKind::Mixed => {
			gated(
				&t("How will it be presented?"),
				&t("you have said how it will be presented"),
				&mut criteria,
			);
			plain(
				f.elements >= 1,
				&t("the design has at least one element"),
				&t("Add the first element: Design, Add."),
				&mut criteria,
			);
		}
		GoalKind::AccessibilityReview => {
			plain(
				f.elements >= 1,
				&t("the design has at least one element to review"),
				&t("Bring in or draw the design to review: File, Import, or Design, Add."),
				&mut criteria,
			);
			plain(
				f.unreadable_texts == 0,
				&t("no text element is unreadable against the background"),
				&t("Raise the contrast of the unreadable words: Color, Text Readability names them."),
				&mut criteria,
			);
			plain(
				f.elements >= 1 && f.standout_elements >= 1,
				&t("at least one element stands out from the background (contrast 3 to 1 or more)"),
				&t("Give one element a colour at least 3 to 1 against the background."),
				&mut criteria,
			);
			gated(
				&t("What are you reviewing?"),
				&t("you have said what you are reviewing"),
				&mut criteria,
			);
		}
		GoalKind::Conversion => {
			plain(
				f.elements >= 1,
				&t("the design has at least one element (something arrived to convert)"),
				&t("Bring the source in: File, Import."),
				&mut criteria,
			);
			plain(
				f.colors >= 1,
				&t("the project has at least one palette colour"),
				&t("Add a colour by hex, or load the starter colours."),
				&mut criteria,
			);
			gated(
				&t("What should it become?"),
				&t("you have said what it should become"),
				&mut criteria,
			);
		}
	}
	let total = criteria.len() as u32;
	let settled = criteria.iter().filter(|c| c.settled).count() as u32;
	let met = criteria.iter().filter(|c| c.met).count() as u32;
	let met_settled = criteria.iter().filter(|c| c.settled && c.met).count() as u32;
	let pct = |part: u32, whole: u32| -> u32 {
		if whole == 0 {
			100
		} else {
			((part as f32 / whole as f32) * 100.0).round() as u32
		}
	};
	Some(GoalReport {
		goal_match_pct: pct(met_settled, settled),
		creation_pct: pct(met, total),
		lines: criteria.iter().map(|c| c.line.clone()).collect(),
		suggestions: criteria
			.iter()
			.filter_map(|c| c.suggestion.clone())
			.collect(),
	})
}

/// How clearly someone else would recognize what is being
/// created, judged per project type. The report says what it
/// is in its own first lines: a rough guide from counts,
/// contrast and layering — not a measurement of any real
/// viewer's experience.
pub fn recognizability_report(project: &Project) -> String {
	let kind = project
		.goal
		.as_ref()
		.map(|g| g.kind)
		.unwrap_or(GoalKind::ImageCreation);
	let f = facts(project);
	let d = &project.drawing;
	let mut out = String::from(
		"Recognizability — a rough guide, not a measurement. It judges element counts, \
		 contrast against the background, and depth layering; it cannot judge what a \
		 real viewer would make of the result.\n",
	);
	match kind {
		GoalKind::Writing => {
			if f.texts == 0 {
				out.push_str(&t("There are no words in the design yet, so there is nothing for a reader to recognize.\n"));
			} else {
				let (mut large, mut faint) = (0, 0);
				for layer in &d.layers {
					for shape in &layer.shapes {
						if shape.kind != ShapeKind::Text {
							continue;
						}
						let ratio = shape.fill.contrast(d.background);
						if ratio >= 4.5 {
							// Clearly readable; the all-clear
							// branch below is what speaks for it.
						} else if ratio >= 3.0 {
							large += 1;
						} else {
							faint += 1;
						}
					}
				}
				if faint > 0 {
					out.push_str(&t("Some words would be hard to recognize: %s of %s text elements sit under 3 to 1 contrast against the background.\n").replacen("%s", &(faint).to_string(), 1).replacen("%s", &(f.texts).to_string(), 1));
				} else if large > 0 {
					out.push_str(&t("The words are recognizable; %s of %s read best at large sizes (between 3 and 4.5 to 1 contrast), the rest stand out clearly.\n").replacen("%s", &(large).to_string(), 1).replacen("%s", &(f.texts).to_string(), 1));
				} else {
					out.push_str(&t("Every text element stands out clearly (4.5 to 1 contrast or more): a reader would recognize the words at once.\n"));
				}
			}
		}
		GoalKind::AccessibilityReview => {
			if f.elements == 0 {
				out.push_str(&t(
					"The design is empty: there is nothing here to recognize or review yet.\n",
				));
			} else {
				out.push_str(&t("%s of %s elements stand out from the background at 3 to 1 contrast or more.\n").replacen("%s", &(f.standout_elements).to_string(), 1).replacen("%s", &(f.elements).to_string(), 1));
				if f.texts == 0 {
					out.push_str(&t(
						"There are no text elements, so word recognition has nothing to check.\n",
					));
				} else if f.unreadable_texts == 0 {
					out.push_str(&t(
						"Every text element is readable against the background (3 to 1 or more).\n",
					));
				} else {
					out.push_str(&t("%s of %s text elements are unreadable against the background (under 3 to 1).\n").replacen("%s", &(f.unreadable_texts).to_string(), 1).replacen("%s", &(f.texts).to_string(), 1));
				}
			}
		}
		_ => {
			if f.elements == 0 {
				out.push_str(&t(
					"The canvas is empty: there is nothing to recognize yet.\n",
				));
			} else {
				if f.elements <= 2 {
					out.push_str(&t("Only %s element%s so far: a viewer would recognize the pieces, but they do not yet add up to a whole picture.\n").replacen("%s", &(f.elements).to_string(), 1).replacen("%s", if f.elements == 1 { "" } else { "s" }, 1));
				} else {
					out.push_str(&t("%s elements: enough pieces on the canvas to form a recognizable whole.\n").replacen("%s", &(f.elements).to_string(), 1));
				}
				if f.standout_elements == 0 {
					out.push_str(&t("No element reaches 3 to 1 contrast against the background: the picture would read as a wash of similar tones.\n"));
				} else if f.standout_elements == f.elements {
					out.push_str(&t(
						"Every element stands out from the background at 3 to 1 or more.\n",
					));
				} else {
					out.push_str(&t("%s of %s elements stand out from the background at 3 to 1 or more; the rest sit close to it in tone.\n").replacen("%s", &(f.standout_elements).to_string(), 1).replacen("%s", &(f.elements).to_string(), 1));
				}
				if f.has_depth_layer {
					out.push_str(&t("Layers carry depth roles, which give a viewer near and far to read: the picture has structure.\n"));
				} else {
					out.push_str(&t("No layer carries a depth role: the picture reads flat — fine for a pattern or a chart, harder for a scene.\n"));
				}
			}
		}
	}
	out
}

/// The seed drawing a new wizard project of this kind starts
/// from — small and honest: a writing project starts with words
/// to replace, an image creation with a background layer tagged
/// as the background and a grey ground to build on, a mixed
/// project with the tagged layer alone. Kinds whose work starts
/// from something brought in (cosmetic work, conversion) start
/// from the plain canvas, and the wizard says why.
pub fn seed_drawing(kind: GoalKind) -> Drawing {
	let mut d = Drawing::new(800, 600, Srgb::WHITE);
	match kind {
		GoalKind::Writing | GoalKind::AccessibilityReview => {
			d.add_shape(
				0,
				Shape::text("Write here", "Arial", 36, 40, 40, 400, 60, Srgb::BLACK),
			);
		}
		GoalKind::ImageCreation => {
			d.set_layer_depth(0, DepthRole::Background);
			d.add_shape(
				0,
				Shape::new(
					ShapeKind::Rectangle,
					0,
					0,
					800,
					600,
					Srgb::new(232, 232, 232),
				),
			);
		}
		GoalKind::Mixed => {
			d.set_layer_depth(0, DepthRole::Background);
		}
		GoalKind::ImageCosmetic | GoalKind::Conversion => {}
	}
	d
}

/// The sliders the setup wizard pins for a project of this
/// kind: the ones that kind of work reaches for first. Chosen
/// per kind from the slider registry — words lean on the
/// channel and readability adjustments, picture-making on the
/// brush, element and colour tools, cosmetic work on the
/// adjustment family, a review on the tactile grid and the
/// legibility adjustments. Finishing the wizard applies this
/// set (a restated goal tops the project's pins up with any of
/// these it lacks), so the Sliders tab is working from the
/// first minute instead of waiting to be stocked by hand.
pub fn default_slider_ids(kind: GoalKind) -> Vec<&'static str> {
	match kind {
		GoalKind::Writing => vec![
			"color-channels",
			"element-opacity",
			"transparency",
			"adjust-brightness",
			"adjust-clarity",
			"adjust-tone",
			"sound-volume",
		],
		GoalKind::ImageCreation => vec![
			"brush-width",
			"stroke-width",
			"color-channels",
			"transparency",
			"element-opacity",
			"element-scale",
			"element-roundedness",
			"sound-volume",
		],
		GoalKind::ImageCosmetic => vec![
			"extract-count",
			"color-channels",
			"adjust-shade",
			"adjust-tint",
			"adjust-tone",
			"adjust-chroma",
			"adjust-brightness",
			"adjust-warmth",
		],
		GoalKind::Mixed => vec![
			"brush-width",
			"color-channels",
			"transparency",
			"element-opacity",
			"element-scale",
			"frame-duration",
			"sound-volume",
		],
		GoalKind::AccessibilityReview => vec![
			"color-channels",
			"tactile-columns",
			"tactile-rows",
			"adjust-brightness",
			"adjust-clarity",
			"adjust-fog",
			"element-opacity",
			"sound-volume",
		],
		GoalKind::Conversion => vec![
			"extract-count",
			"color-channels",
			"tactile-columns",
			"tactile-rows",
			"adjust-tone",
			"adjust-brightness",
			"sound-volume",
		],
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::palette::PaletteColor;
	use crate::project::DEFAULT_PAGE_JUMP_PERCENT;
	use crate::sliders::AdjustTarget;

	fn project(drawing: Drawing, goal: Option<ProjectGoal>) -> Project {
		Project {
			drawing,
			palette: Vec::new(),
			pinned_sliders: Vec::new(),
			slider_target: AdjustTarget::BaseColor,
			page_jump_percent: DEFAULT_PAGE_JUMP_PERCENT,
			goal,
			grayscale_start: false,
			interaction_mode: crate::project::InteractionMode::Type,
		}
	}

	fn answered(kind: GoalKind, question: &str, answer: &str) -> ProjectGoal {
		let mut g = ProjectGoal::new(kind);
		for (q, a) in &mut g.answers {
			if q == question {
				*a = Some(answer.to_string());
			}
		}
		g
	}

	#[test]
	fn no_goal_no_report() {
		let p = project(Drawing::new(800, 600, Srgb::WHITE), None);
		assert!(goal_report(&p).is_none());
	}

	#[test]
	fn writing_goal_counts_text_and_readability() {
		// Empty design, question answered: all three criteria
		// are settled; only the gated one is met (the saying of
		// it), so both percentages are 1 of 3.
		let goal = answered(
			GoalKind::Writing,
			"What will the writing be?",
			"A sign or label",
		);
		let p = project(Drawing::new(800, 600, Srgb::WHITE), Some(goal));
		let r = goal_report(&p).unwrap();
		assert_eq!(r.goal_match_pct, 33); // 1 of 3 settled criteria met
		assert_eq!(r.creation_pct, 33);
		assert_eq!(r.suggestions.len(), 2);
		// Add clear black text: everything met.
		let mut d = Drawing::new(800, 600, Srgb::WHITE);
		d.add_shape(
			0,
			Shape::text("Hello", "Arial", 36, 40, 40, 300, 60, Srgb::BLACK),
		);
		let goal = answered(
			GoalKind::Writing,
			"What will the writing be?",
			"A sign or label",
		);
		let p = project(d, Some(goal));
		let r = goal_report(&p).unwrap();
		assert_eq!(r.goal_match_pct, 100);
		assert_eq!(r.creation_pct, 100);
		assert!(r.suggestions.is_empty());
	}

	#[test]
	fn no_fixed_answer_counts_in_creation_not_in_goal_match() {
		// Mixed: the presentation question is open, and the
		// design has an element. Goal match judges only the
		// settled criterion (elements) = 100; creation counts
		// the open one as not yet = 1 of 2 = 50.
		let mut d = Drawing::new(800, 600, Srgb::WHITE);
		d.add_shape(
			0,
			Shape::new(ShapeKind::Rectangle, 10, 10, 50, 50, Srgb::BLACK),
		);
		let p = project(d, Some(ProjectGoal::new(GoalKind::Mixed)));
		let r = goal_report(&p).unwrap();
		assert_eq!(r.goal_match_pct, 100, "{}", r.lines.join(" | "));
		assert_eq!(r.creation_pct, 50);
		assert!(r.lines.iter().any(|l| l.starts_with("Open:")));
		assert!(r
			.full_text(p.goal.as_ref().unwrap())
			.contains("No fixed answer"));
		// Answer it and both percentages agree.
		let mut d = Drawing::new(800, 600, Srgb::WHITE);
		d.add_shape(
			0,
			Shape::new(ShapeKind::Rectangle, 10, 10, 50, 50, Srgb::BLACK),
		);
		let goal = answered(GoalKind::Mixed, "How will it be presented?", "As a picture");
		let p = project(d, Some(goal));
		let r = goal_report(&p).unwrap();
		assert_eq!(r.goal_match_pct, 100);
		assert_eq!(r.creation_pct, 100);
	}

	#[test]
	fn image_creation_wants_a_shape_and_a_background() {
		let mut d = Drawing::new(800, 600, Srgb::WHITE);
		let p = project(d.clone(), Some(ProjectGoal::new(GoalKind::ImageCreation)));
		let r = goal_report(&p).unwrap();
		// Settled: shape and background, both unmet; gated open.
		assert_eq!(r.goal_match_pct, 0);
		assert_eq!(r.creation_pct, 0);
		// A shape on a chosen background meets both settled ones.
		d.set_background(Srgb::new(30, 30, 60));
		d.add_shape(0, Shape::new(ShapeKind::Ellipse, 5, 5, 40, 40, Srgb::WHITE));
		let p = project(d, Some(ProjectGoal::new(GoalKind::ImageCreation)));
		let r = goal_report(&p).unwrap();
		assert_eq!(r.goal_match_pct, 100);
		assert_eq!(r.creation_pct, 67); // 2 of 3: the gated one is open
	}

	#[test]
	fn image_cosmetic_accepts_an_image_or_two_colors() {
		let goal = ProjectGoal::new(GoalKind::ImageCosmetic);
		let mut p = project(Drawing::new(800, 600, Srgb::WHITE), Some(goal));
		let r = goal_report(&p).unwrap();
		assert_eq!(r.goal_match_pct, 0);
		p.palette = vec![
			PaletteColor::typed(Srgb::BLACK),
			PaletteColor::typed(Srgb::WHITE),
		];
		let r = goal_report(&p).unwrap();
		assert_eq!(r.goal_match_pct, 100);
	}

	#[test]
	fn recognizability_names_its_limits_and_branches() {
		// No goal: judged as image creation; empty canvas.
		let p = project(Drawing::new(800, 600, Srgb::WHITE), None);
		let text = recognizability_report(&p);
		assert!(text.contains("rough guide, not a measurement"), "{text}");
		assert!(text.contains("nothing to recognize yet"), "{text}");
		// Writing with faint text: the faint branch speaks.
		let mut d = Drawing::new(800, 600, Srgb::WHITE);
		d.add_shape(
			0,
			Shape::text(
				"Faint",
				"Arial",
				36,
				40,
				40,
				300,
				60,
				Srgb::new(200, 200, 200),
			),
		);
		let p = project(d, Some(ProjectGoal::new(GoalKind::Writing)));
		let text = recognizability_report(&p);
		assert!(text.contains("hard to recognize"), "{text}");
		// Writing with clear text.
		let mut d = Drawing::new(800, 600, Srgb::WHITE);
		d.add_shape(
			0,
			Shape::text("Clear", "Arial", 36, 40, 40, 300, 60, Srgb::BLACK),
		);
		let p = project(d, Some(ProjectGoal::new(GoalKind::Writing)));
		assert!(recognizability_report(&p).contains("recognize the words at once"));
		// Image with depth layering vs flat.
		let mut d = Drawing::new(800, 600, Srgb::WHITE);
		d.set_layer_depth(0, DepthRole::Background);
		for i in 0..3 {
			d.add_shape(
				0,
				Shape::new(ShapeKind::Rectangle, i * 60, 10, 50, 50, Srgb::BLACK),
			);
		}
		let p = project(d, Some(ProjectGoal::new(GoalKind::ImageCreation)));
		let text = recognizability_report(&p);
		assert!(text.contains("near and far to read"), "{text}");
		let mut d = Drawing::new(800, 600, Srgb::WHITE);
		for i in 0..3 {
			d.add_shape(
				0,
				Shape::new(ShapeKind::Rectangle, i * 60, 10, 50, 50, Srgb::BLACK),
			);
		}
		let p = project(d, Some(ProjectGoal::new(GoalKind::ImageCreation)));
		assert!(recognizability_report(&p).contains("reads flat"));
	}

	#[test]
	fn seeds_are_small_and_as_documented() {
		let d = seed_drawing(GoalKind::Writing);
		assert_eq!(d.layers[0].shapes.len(), 1);
		assert_eq!(d.layers[0].shapes[0].kind, ShapeKind::Text);
		let d = seed_drawing(GoalKind::ImageCreation);
		assert_eq!(d.layers[0].depth, DepthRole::Background);
		assert_eq!(d.layers[0].shapes.len(), 1);
		let d = seed_drawing(GoalKind::Mixed);
		assert_eq!(d.layers[0].depth, DepthRole::Background);
		assert!(d.layers[0].shapes.is_empty());
		let d = seed_drawing(GoalKind::Conversion);
		assert!(d.layers[0].shapes.is_empty());
		assert_eq!(d.layers[0].depth, DepthRole::Unassigned);
	}

	#[test]
	fn default_sliders_resolve_and_cover_every_kind() {
		for kind in GoalKind::ALL {
			let ids = default_slider_ids(kind);
			assert!(
				(6..=10).contains(&ids.len()),
				"{} gets {} default sliders",
				kind.id(),
				ids.len()
			);
			for (i, id) in ids.iter().enumerate() {
				assert!(
					crate::sliders::slider_by_id(id).is_some(),
					"{} default {id} is not in the slider registry",
					kind.id()
				);
				assert!(!ids[i + 1..].contains(id), "duplicate default {id}");
			}
		}
	}

	#[test]
	fn goal_kinds_round_trip_their_ids_and_questions() {
		for kind in GoalKind::ALL {
			assert_eq!(GoalKind::from_id(kind.id()), Some(kind));
			assert!(!kind.questions().is_empty());
			let g = ProjectGoal::new(kind);
			assert_eq!(g.answers.len(), kind.questions().len());
			assert_eq!(g.open_questions().len(), kind.questions().len());
		}
	}
}
