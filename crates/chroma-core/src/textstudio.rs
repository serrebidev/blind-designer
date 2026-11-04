//! textstudio: the writing side of the design — feed snapshots
//! in, practice sheets out.
//!
//! The feed parser reads the two syndication shapes the web
//! actually serves: RSS items and Atom entries. It is a small
//! hand parser in the house style (the web module's parsers are
//! the model): it finds each item/entry block, takes its first
//! title, unwraps CDATA, strips any markup inside the title and
//! decodes the common entities. It is deliberately not an XML
//! parser — see the ponytail note in `web`.
//!
//! The practice sheet turns a design's text elements into a
//! print-writing exercise: each phrase printed large, as rows of
//! block letters a finger or a pen can follow, with a legibility
//! line (size and contrast, labeled a rough guide).

use crate::design::ShapeKind;
use crate::drawing::Drawing;
use crate::font;

/// Pull a character reference or named entity's value out of a
/// title. Handles the XML five (&amp; &lt; &gt; &quot; &apos;)
/// plus decimal and hex numeric references; anything unknown
/// passes through untouched.
fn decode_entities(text: &str) -> String {
	let mut out = String::with_capacity(text.len());
	let mut rest = text;
	while let Some(amp) = rest.find('&') {
		out.push_str(&rest[..amp]);
		let after = &rest[amp + 1..];
		let Some(semi) = after.find(';') else {
			out.push('&');
			rest = after;
			continue;
		};
		let entity = &after[..semi];
		let decoded: Option<String> = match entity {
			"amp" => Some("&".to_string()),
			"lt" => Some("<".to_string()),
			"gt" => Some(">".to_string()),
			"quot" => Some("\"".to_string()),
			"apos" => Some("'".to_string()),
			_ => entity
				.strip_prefix("#x")
				.and_then(|h| u32::from_str_radix(h, 16).ok())
				.or_else(|| {
					entity.strip_prefix('#').and_then(|d| d.parse::<u32>().ok())
				})
				.and_then(char::from_u32)
				.map(|c| c.to_string()),
		};
		match decoded {
			Some(d) => {
				out.push_str(&d);
				rest = &after[semi + 1..];
			}
			None => {
				out.push('&');
				rest = after;
			}
		}
	}
	out.push_str(rest);
	out
}

/// Clean one raw title body: unwrap CDATA, strip any inner
/// tags, decode entities, and fold all whitespace runs (titles
/// arrive wrapped and indented) into single spaces.
fn clean_title(raw: &str) -> String {
	let trimmed = raw.trim();
	let inner = if let Some(cdata) = trimmed
		.strip_prefix("<![CDATA[")
		.and_then(|s| s.strip_suffix("]]>"))
	{
		cdata.to_string()
	} else {
		// Drop inner markup: anything between < and >.
		let mut out = String::with_capacity(trimmed.len());
		let mut in_tag = false;
		for c in trimmed.chars() {
			match c {
				'<' => in_tag = true,
				'>' => in_tag = false,
				_ if !in_tag => out.push(c),
				_ => {}
			}
		}
		out
	};
	decode_entities(&inner).split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The titles of a feed's items, in feed order, at most `max`
/// of them: RSS `<item>` blocks and Atom `<entry>` blocks are
/// both read, each contributing its first `<title>`. The feed's
/// own channel/feed title is never included — it is not inside
/// an item. Empty titles are skipped. A document with no items
/// (or one that is not a feed at all) yields an empty list,
/// which the caller reports as "no items found", never a guess.
pub fn parse_feed_titles(xml: &str, max: usize) -> Vec<String> {
	let lower = xml.to_ascii_lowercase();
	let mut titles = Vec::new();
	let mut pos = 0;
	while titles.len() < max {
		// The next item or entry block, whichever opens first.
		let next = ["<item", "<entry"]
			.iter()
			.filter_map(|tag| {
				let mut from = pos;
				loop {
					let at = lower[from..].find(tag)? + from;
					// The tag name must end here (">", space, "/"),
					// so "<items>" or "<entryway>" never match.
					let after = lower.as_bytes().get(at + tag.len()).copied();
					if matches!(after, Some(b'>' | b' ' | b'/' | b'\t' | b'\n' | b'\r')) {
						return Some((at, *tag));
					}
					from = at + tag.len();
				}
			})
			.min_by_key(|(at, _)| *at);
		let Some((start, tag)) = next else { break };
		let close_tag = if tag == "<item" { "</item>" } else { "</entry>" };
		let end = lower[start..]
			.find(close_tag)
			.map(|e| start + e)
			.unwrap_or(lower.len());
		let block = &lower[start..end];
		if let Some(tstart) = block.find("<title") {
			if let Some(gt) = block[tstart..].find('>') {
				let body_start = start + tstart + gt + 1;
				if let Some(tend) = lower[body_start..end].find("</title>") {
					let title = clean_title(&xml[body_start..body_start + tend]);
					if !title.is_empty() {
						titles.push(title);
					}
				}
			}
		}
		pos = end + close_tag.len();
	}
	titles
}

/// One text line as rows of block letters: every glyph pixel
/// becomes two characters wide and one row tall, `#` for a set
/// pixel, spaces for a clear one, one space between letters —
/// the tactile-sheet way of printing words large enough to
/// trace. Characters the bitmap font does not know draw as '?',
/// exactly as the design's own renderer draws them.
fn block_line(line: &str) -> String {
	let fallback = font::glyph('?').unwrap();
	let mut out = String::new();
	for row in 0..font::GLYPH_H {
		for ch in line.chars() {
			let g = font::glyph(ch).unwrap_or(fallback);
			for col in 0..font::GLYPH_W {
				let on = (g[row] >> (font::GLYPH_W - 1 - col)) & 1 == 1;
				out.push_str(if on { "##" } else { "  " });
			}
			out.push(' ');
		}
		out.push('\n');
	}
	out
}

/// How legible one text element is, as a rough guide: its point
/// size in words and its contrast against the design
/// background in the readability bands the Text Readability
/// report uses. A heuristic, and the sheet says so.
fn legibility_line(size: u32, ratio: f32) -> String {
	let size_word = if size >= 28 {
		"big"
	} else if size >= 16 {
		"medium-sized"
	} else {
		"small"
	};
	let contrast_word = match ratio {
		x if x >= 4.5 => "easy to read",
		x if x >= 3.0 => "hard to read",
		_ => "unreadable",
	};
	format!(
		"Legibility (rough guide): {size_word} letters at {size} point; contrast {ratio:.1} to 1 against the background, {contrast_word}."
	)
}

impl Drawing {
	/// The print-writing practice sheet: every text element on
	/// the visible layers, in reading order, its phrase printed
	/// large as block letters to trace or copy, with a legibility
	/// line under it. A design with no text gets a sheet saying
	/// so, not an empty file.
	pub fn practice_sheet(&self) -> String {
		let mut texts: Vec<&crate::design::Shape> = Vec::new();
		for layer in &self.layers {
			if !layer.visible {
				continue;
			}
			let mut mine: Vec<&crate::design::Shape> = layer
				.shapes
				.iter()
				.filter(|s| s.kind == ShapeKind::Text)
				.collect();
			mine.sort_by_key(|s| (s.y, s.x));
			texts.extend(mine);
		}
		let mut out = String::from(
			"Print-writing practice sheet.\n\
			 Each text element in the design, printed large as block letters: \
			 trace the letters, or copy them underneath in your own hand. \
			 The legibility lines are a rough guide, not a measurement.\n",
		);
		if texts.is_empty() {
			out.push_str("\nThis design has no text elements to practise with.\n");
			return out;
		}
		for (i, shape) in texts.iter().enumerate() {
			out.push_str(&format!(
				"\n{}. \"{}\"\n{}\n",
				i + 1,
				shape.text.replace('\n', " / "),
				legibility_line(shape.font_size, shape.fill.contrast(self.background))
			));
			for line in shape.text.split('\n') {
				out.push_str(&block_line(line));
				out.push('\n');
			}
		}
		out
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::drawing::Drawing;
	use crate::Srgb;

	const RSS: &str = r#"<?xml version="1.0"?>
		<rss version="2.0"><channel>
		<title>The Channel Title</title>
		<item><title>First Post</title><link>https://example.com/1</link></item>
		<item><title><![CDATA[Second & Special]]></title></item>
		<item><title>Third &#8212; with <b>markup</b> &amp; entity</title></item>
		<item><title>   </title></item>
		</channel></rss>"#;

	#[test]
	fn rss_titles_come_from_items_not_the_channel() {
		let titles = parse_feed_titles(RSS, 10);
		assert_eq!(
			titles,
			vec![
				"First Post".to_string(),
				"Second & Special".to_string(),
				"Third — with markup & entity".to_string(),
			]
		);
		// The cap is honored, in feed order.
		assert_eq!(parse_feed_titles(RSS, 2).len(), 2);
		assert_eq!(parse_feed_titles(RSS, 0), Vec::<String>::new());
	}

	const ATOM: &str = r#"<feed xmlns="http://www.w3.org/2005/Atom">
		<title>Feed Name</title>
		<entry><title>Atom One</title><id>urn:1</id></entry>
		<entry><title type="html">Atom &lt;Two&gt;</title></entry>
		</feed>"#;

	#[test]
	fn atom_titles_come_from_entries() {
		assert_eq!(
			parse_feed_titles(ATOM, 10),
			vec!["Atom One".to_string(), "Atom <Two>".to_string()]
		);
	}

	#[test]
	fn non_feeds_and_empty_feeds_yield_nothing() {
		assert!(parse_feed_titles("<html><body>hello</body></html>", 5).is_empty());
		assert!(parse_feed_titles("", 5).is_empty());
		assert!(parse_feed_titles("<rss><channel><title>Only Channel</title></channel></rss>", 5).is_empty());
	}

	#[test]
	fn practice_sheet_prints_block_letters_and_legibility() {
		let mut d = Drawing::new(200, 100, Srgb::WHITE);
		d.add_shape(0, crate::design::Shape::text("Hi", "Arial", 32, 0, 0, 60, 40, Srgb::BLACK));
		let sheet = d.practice_sheet();
		assert!(sheet.contains("rough guide"), "{sheet}");
		assert!(sheet.contains("\"Hi\""), "{sheet}");
		assert!(sheet.contains("big letters at 32 point"), "{sheet}");
		assert!(sheet.contains("contrast 21.0 to 1"), "{sheet}");
		assert!(sheet.contains("easy to read"), "{sheet}");
		// The block letters are really there: seven glyph rows,
		// and the 'H' stem paints its rows with ## pairs.
		assert!(sheet.contains("##"), "{sheet}");
		// A design with no text says so.
		let empty = Drawing::new(10, 10, Srgb::WHITE).practice_sheet();
		assert!(empty.contains("no text elements"), "{empty}");
	}

	#[test]
	fn practice_sheet_skips_hidden_layers() {
		let mut d = Drawing::new(200, 100, Srgb::WHITE);
		d.add_shape(0, crate::design::Shape::text("Shown", "Arial", 16, 0, 0, 60, 20, Srgb::BLACK));
		let top = d.add_layer("Hidden words");
		d.add_shape(top, crate::design::Shape::text("Secret", "Arial", 16, 0, 30, 60, 20, Srgb::BLACK));
		d.set_layer_visible(top, false);
		let sheet = d.practice_sheet();
		assert!(sheet.contains("Shown"), "{sheet}");
		assert!(!sheet.contains("Secret"), "{sheet}");
	}
}
