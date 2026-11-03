//! dictionary: the built-in dictionary of color and design terms (q0).
//!
//! Elshara's ask: "a dictionary of all the values of terms necessary
//! for color design, and support it." This is that dictionary's first
//! edition — the terms this app itself uses, plus the classic color
//! theory vocabulary a designer meets everywhere, each defined in
//! plain language with the numbers the app reports. The dataset is
//! curated in code (no file to lose, no network to reach), and the
//! lookup is plain substring search, so the dictionary dialog stays a
//! thin shell over this tested core.

/// One dictionary entry: a term and its plain-language definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
	pub term: &'static str,
	pub definition: &'static str,
}

const fn e(term: &'static str, definition: &'static str) -> Entry {
	Entry { term, definition }
}

/// The dictionary itself, in alphabetical order.
pub static ENTRIES: &[Entry] = &[
	e("AA", "The standard WCAG contrast level. Normal text passes at a contrast ratio of 4.5 to 1 or more, large text at 3 to 1."),
	e("AAA", "The stricter WCAG contrast level. Normal text passes at a contrast ratio of 7 to 1 or more against its background."),
	e("achromatic", "Having no hue at all: black, white, and the grays between them. In OKLCH an achromatic color has chroma at or near zero, so its hue number means nothing."),
	e("analogous", "A color scheme of neighbors: colors whose hues sit close together, classically within about 30 degrees either side of the base color. Calm and unified, because the hues barely change."),
	e("brightness", "See value. In this app, lightness in OKLCH is the precise measure; brightness is the everyday word for the same idea."),
	e("brush", "The width, in pixels, that a stroke is painted with. A wider brush makes a thicker line along the stroke's points."),
	e("canvas", "The whole picture area, measured in pixels wide by pixels high. Every shape's position is counted from the canvas's top-left corner."),
	e("caption", "A short spoken-style description of a shape or a whole design: what it is, its color in words and numbers, and how it contrasts with the background. A shape with no caption of its own gets one generated; a written caption is never overwritten."),
	e("chroma", "How colorful a color is, apart from how light or what hue it is. Zero chroma is gray; higher chroma is more vivid. In OKLCH, chroma runs from 0 to about 0.4."),
	e("color name", "A plain word for a color family by its hue angle, like teal or magenta. This app names hues by their OKLCH angle, which sits differently from HSL: pure blue is 264 degrees here."),
	e("complementary", "The color exactly opposite on the hue wheel: the base hue plus 180 degrees, with the same lightness and chroma. Complementary pairs have the strongest hue contrast there is."),
	e("contrast ratio", "How strongly two colors differ in luminance, from 1 to 1 (identical) up to 21 to 1 (black against white). It is the number WCAG pass and fail levels are judged by."),
	e("cool color", "A color whose hue leans toward green, blue, or purple. Cool colors tend to feel calm and distant."),
	e("CSS variables", "Named values in a stylesheet, written like --color-1: #3A7CA5;. Exporting a palette as CSS variables hands the exact hex values to a web page, each with its spoken caption as a comment."),
	e("ellipse", "An oval shape, set by the position of its top-left corner and its width and height. Equal width and height make a circle."),
	e("event log", "The numbered list of every change made to a drawing, in order: canvas set, layer added, shape shifted, and so on. Read top to bottom, the log is the recipe that recreates the image exactly."),
	e("gamut", "The full set of colors a system can actually show. A color outside the sRGB gamut cannot be displayed exactly; this app reduces its chroma, holding lightness and hue exact, until it fits."),
	e("gray", "An achromatic color between black and white. Also spelled grey. A color whose chroma is under 0.02 reads as gray whatever its hue number says."),
	e("hex code", "A color written as six hexadecimal digits, like #3A7CA5: red, green, and blue brightness, two digits each, from 00 to FF. A leading # is customary but optional."),
	e("hue", "Which color family a color belongs to, as an angle in degrees from 0 to 360 around the wheel. In OKLCH the pure colors sit at: red 29, orange 57, yellow 110, green 142, teal 195, blue 264, magenta 328."),
	e("image element", "A shape that holds an image file placed in the design, drawn scaled into its box. The project file stores the file's path, not its pixels; if the file is missing when the design is drawn, a framed placeholder in the shape's color shows where it belongs."),
	e("layer", "A named sheet of shapes within a drawing. Layers paint in order, first layer at the back. A hidden layer paints nothing. New shapes land on the current layer."),
	e("lightness", "How light or dark a color is, from 0 (black) to 1 (white), independent of its hue and chroma. In spoken captions it is reported as a percentage."),
	e("luminance", "The measured brightness of a color as screens emit it, from 0.0 for black to 1.0 for white. Contrast ratios are computed from the luminance of the two colors."),
	e("monochrome", "A scheme or image built from one hue, varied only by lightness and chroma: shades, tints, and tones of a single color."),
	e("muted", "A color with low-to-moderate chroma (about 0.02 to 0.06 in OKLCH): softened, grayish, not vivid."),
	e("OKLCH", "The color space this app works in natively: lightness, chroma, and hue angle, designed so equal number steps look like equal color steps. Colors here are never mixed; they are computed by changing these numbers."),
	e("palette", "The working set of colors for a design. Colors enter the palette by hex value, by LCH numbers, by scheme generation from a selected color, or by extraction from an image."),
	e("palette extraction", "Reading an image and reporting its dominant colors as a palette, each with the share of the image it covers. This app quantizes in OKLab space so the split follows perceived differences."),
	e("pastel", "A light, soft color: high lightness with low-to-moderate chroma. Pastels are tints that kept some color."),
	e("position cue", "A sound that tells you where something is: stereo pan follows left-to-right position, pitch follows top-to-bottom position (top of the canvas sounds highest), and a longer tone means a bigger shape."),
	e("primary colors", "In paint, red, yellow, and blue; in light and on screens, red, green, and blue. Screens mix light, so this app's hex values are red-green-blue."),
	e("provenance", "The recorded memory of how a color entered a palette and where it was used: extracted from an image, generated as a complement, typed as hex, and so on."),
	e("recreation file", "The primary export: a step-by-step document generated from the event log that rebuilds the exact image from nothing, followed by the final state's shape list as a check."),
	e("rectangle", "A four-sided shape with right-angle corners, set by its top-left corner position, width, and height. Equal width and height make a square."),
	e("rotation", "Turning a shape around its own center, in degrees clockwise. Rectangles and ellipses rotate; a stroke's points turn with it."),
	e("saturation", "The everyday word for how colorful a color is. In this app chroma is the precise measure; saturation belongs to the HSL and HSV spaces."),
	e("saved brush", "A stroke brush kept under a name in the brush library: its width, its color, and its stamp pattern. Saving a brush means a favorite pen can be applied to new strokes in one choice instead of setting the three values again."),
	e("scheme", "A set of colors derived from one base color by known hue angles: complementary, split complementary, analogous, triadic, tetradic. Because the angles are fixed, every color in a scheme is predictable before it is made."),
	e("shade", "A darker version of a color: same hue and chroma, lightness stepped down toward black."),
	e("shape", "One drawn thing on a layer: a rectangle, an ellipse, or a stroke, with a position, a size, a fill color, and an optional caption and rotation."),
	e("split complementary", "A scheme of the base color plus the two colors 150 degrees either side of its complement: nearly opposite, but with more variety and less tension than a direct complementary pair."),
	e("sRGB", "The standard color system of screens and the web. Every color this app exports is an sRGB color, written as a hex code."),
	e("stroke", "A drawn line through a list of points, painted with a brush of a set width. A stroke's size is its points; it is resized by moving or redrawing, not by width and height."),
	e("tetradic", "A four-color scheme: the base hue plus 60, 180, and 240 degrees — two complementary pairs forming a rectangle on the hue wheel."),
	e("text element", "A shape that holds a phrase of text, placed by measurement like any other shape: a position, a box, a font name, and a point size. Its caption is the phrase itself unless you write a different one. In an exported web page the phrase is real text a screen reader reads."),
	e("tint", "A lighter version of a color: same hue and chroma, lightness stepped up toward white."),
	e("tone", "A softer version of a color: same hue and lightness, chroma scaled down toward gray."),
	e("triadic", "A three-color scheme: the base hue plus 120 and 240 degrees, evenly spaced around the wheel. Bold and balanced at once."),
	e("value", "How light or dark a color is, in painters' language. In this app the precise measure is OKLCH lightness."),
	e("vivid", "A color with high chroma (about 0.12 to 0.20 in OKLCH): strong and saturated-looking. Above that, electric."),
	e("warm color", "A color whose hue leans toward red, orange, or yellow. Warm colors tend to feel close and energetic."),
	e("WCAG", "The Web Content Accessibility Guidelines: the standard behind contrast levels AA and AAA for text against its background."),
];

/// Look up one term exactly (case-insensitive, surrounding whitespace
/// ignored). `None` when the dictionary has no such term.
pub fn lookup(term: &str) -> Option<&'static Entry> {
	let want = term.trim().to_lowercase();
	ENTRIES.iter().find(|en| en.term.to_lowercase() == want)
}

/// Search the dictionary: every entry whose term or definition
/// contains the query (case-insensitive), in dictionary order. An
/// empty query returns the whole dictionary.
pub fn search(query: &str) -> Vec<&'static Entry> {
	let q = query.trim().to_lowercase();
	if q.is_empty() {
		return ENTRIES.iter().collect();
	}
	ENTRIES
		.iter()
		.filter(|en| {
			en.term.to_lowercase().contains(&q) || en.definition.to_lowercase().contains(&q)
		})
		.collect()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn the_terms_the_app_itself_uses_are_all_defined() {
		for term in [
			"hue", "chroma", "lightness", "OKLCH", "layer", "stroke", "brush", "canvas",
			"caption", "event log", "recreation file", "palette", "gamut", "contrast ratio",
			"complementary", "shade", "tint", "tone", "hex code",
		] {
			assert!(lookup(term).is_some(), "missing term: {term}");
		}
	}

	#[test]
	fn lookup_is_case_and_whitespace_insensitive() {
		assert_eq!(lookup("  Hue ").unwrap().term, "hue");
		assert_eq!(lookup("wcag").unwrap().term, "WCAG");
		assert!(lookup("not a term").is_none());
	}

	#[test]
	fn search_matches_terms_and_definitions() {
		let hits = search("complementary");
		assert!(hits.iter().any(|en| en.term == "complementary"));
		assert!(hits.iter().any(|en| en.term == "split complementary"));
		// Definition text counts too: "wheel" appears in several entries.
		assert!(search("wheel").len() >= 3);
		assert_eq!(search("").len(), ENTRIES.len());
		assert!(search("zzzz").is_empty());
	}

	#[test]
	fn entries_are_alphabetical_with_real_definitions() {
		let terms: Vec<String> = ENTRIES.iter().map(|en| en.term.to_lowercase()).collect();
		let mut sorted = terms.clone();
		sorted.sort();
		assert_eq!(terms, sorted);
		assert!(ENTRIES.len() >= 40, "only {} entries", ENTRIES.len());
		for en in ENTRIES {
			assert!(en.definition.len() > 40, "{} has a stub definition", en.term);
			assert!(en.definition.ends_with('.'), "{} must end a sentence", en.term);
		}
	}

	#[test]
	fn numbers_in_definitions_match_the_app() {
		// The contrast levels and hue anchors the code actually uses.
		assert!(lookup("AA").unwrap().definition.contains("4.5 to 1"));
		assert!(lookup("AAA").unwrap().definition.contains("7 to 1"));
		let hue = lookup("hue").unwrap().definition;
		assert!(hue.contains("blue 264"), "{hue}");
	}
}
