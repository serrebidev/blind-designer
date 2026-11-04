//! pdf: the design as a two-page PDF, written by hand.
//!
//! Page one is the picture: the design's own raster, embedded
//! as a FlateDecode image. Page two is the words: every text
//! element's phrase as real PDF text (Helvetica), so a reader
//! can select, search and hear it — not a picture of words.
//! The writer is deliberately minimal — eight objects, one font,
//! no compression but the image stream — and its cross-reference
//! offsets are computed from the bytes actually written.
//! ponytail: hand-rolled PDF by design — upgrade path is a real
//! PDF crate if documents ever need more than two pages or
//! embedded fonts.

use crate::design::ShapeKind;
use crate::drawing::Drawing;
use std::io::Write;

/// One object under construction: its number and its bytes
/// (without the "N 0 obj" / "endobj" frame).
struct Obj {
	num: usize,
	body: Vec<u8>,
}

fn obj(num: usize, body: impl Into<Vec<u8>>) -> Obj {
	Obj { num, body: body.into() }
}

fn stream_obj(num: usize, dict: &str, data: &[u8]) -> Obj {
	let mut body = format!("{dict} /Length {} >>\nstream\n", data.len()).into_bytes();
	body.extend_from_slice(data);
	body.extend_from_slice(b"\nendstream");
	Obj { num, body }
}

/// Escape a string for a PDF text literal: backslash and the
/// parentheses are escaped, and anything outside printable
/// ASCII becomes '?' — Helvetica here is WinAnsi, and the
/// design's phrases are the app's own plain text.
fn pdf_escape(text: &str) -> String {
	let mut out = String::with_capacity(text.len());
	for c in text.chars() {
		match c {
			'\\' => out.push_str("\\\\"),
			'(' => out.push_str("\\("),
			')' => out.push_str("\\)"),
			' '..='~' => out.push(c),
			_ => out.push('?'),
		}
	}
	out
}

impl Drawing {
	/// The two-page PDF of this drawing, as bytes. The picture
	/// page shows the flattened design fitted inside a letter
	/// page with margins; the text page lists the text elements
	/// in reading order — hidden layers' words are excluded,
	/// like everywhere else. A design with no text elements
	/// still gets a text page, saying so.
	pub fn to_pdf(&self) -> Vec<u8> {
		let design = self.flatten();
		let img = design.render();
		let (w, h) = (img.width(), img.height());
		let raw: Vec<u8> = img.as_raw().clone();
		let compressed = {
			let mut enc = flate2::write::ZlibEncoder::new(
				Vec::new(),
				flate2::Compression::default(),
			);
			// Writing to a Vec cannot fail; a failure here would
			// be a flate2 bug, and the empty stream still yields
			// a structurally valid (if blank) PDF.
			let _ = enc.write_all(&raw);
			enc.finish().unwrap_or_default()
		};

		// Fit the picture inside a 612x792 letter page, 36pt
		// margins, keeping its shape.
		let scale = (540.0 / w.max(1) as f64).min(720.0 / h.max(1) as f64);
		let (dw, dh) = (w as f64 * scale, h as f64 * scale);
		let (ox, oy) = ((612.0 - dw) / 2.0, (792.0 - dh) / 2.0);
		let picture_ops = format!(
			"q {dw:.2} 0 0 {dh:.2} {ox:.2} {oy:.2} cm /Im1 Do Q\n"
		);

		// The words page: a heading, then every text element's
		// lines, in reading order. Lines stop at the page's
		// bottom margin — what does not fit is counted in a
		// closing line rather than silently dropped.
		let mut lines: Vec<String> = vec!["Text in this design".to_string(), String::new()];
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
		if texts.is_empty() {
			lines.push("(This design has no text elements.)".to_string());
		}
		for shape in &texts {
			for line in shape.text.split('\n') {
				lines.push(line.to_string());
			}
		}
		// 48 lines fit between the margins at 15pt leading.
		let total = lines.len();
		lines.truncate(48);
		if total > lines.len() {
			lines.push(format!("(... and {} more lines in the design.)", total - lines.len()));
		}
		let mut text_ops = String::from("BT /F1 13 Tf 50 742 Td 15 TL\n");
		for (i, line) in lines.iter().enumerate() {
			if i > 0 {
				text_ops.push_str("T*\n");
			}
			text_ops.push_str(&format!("({}) Tj\n", pdf_escape(line)));
		}
		text_ops.push_str("ET\n");

		let objects = vec![
			obj(1, "<< /Type /Catalog /Pages 2 0 R >>"),
			obj(2, "<< /Type /Pages /Kids [3 0 R 6 0 R] /Count 2 >>"),
			obj(
				3,
				"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] \
				 /Resources << /XObject << /Im1 5 0 R >> >> /Contents 4 0 R >>",
			),
			stream_obj(4, "<<", picture_ops.as_bytes()),
			stream_obj(
				5,
				&format!(
					"<< /Type /XObject /Subtype /Image /Width {w} /Height {h} \
					 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode"
				),
				&compressed,
			),
			obj(
				6,
				"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] \
				 /Resources << /Font << /F1 8 0 R >> >> /Contents 7 0 R >>",
			),
			stream_obj(7, "<<", text_ops.as_bytes()),
			obj(8, "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>"),
		];

		let mut out: Vec<u8> = Vec::new();
		out.extend_from_slice(b"%PDF-1.4\n");
		let mut offsets: Vec<usize> = vec![0; objects.len() + 1];
		for o in &objects {
			offsets[o.num] = out.len();
			out.extend_from_slice(format!("{} 0 obj\n", o.num).as_bytes());
			out.extend_from_slice(&o.body);
			out.extend_from_slice(b"\nendobj\n");
		}
		let xref = out.len();
		out.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
		out.extend_from_slice(b"0000000000 65535 f \n");
		for n in 1..=objects.len() {
			out.extend_from_slice(format!("{:010} 00000 n \n", offsets[n]).as_bytes());
		}
		out.extend_from_slice(
			format!(
				"trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
				objects.len() + 1
			)
			.as_bytes(),
		);
		out
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::design::{Shape, ShapeKind};
	use crate::Srgb;
	use std::io::Read;

	fn sample() -> Drawing {
		let mut d = Drawing::new(40, 20, Srgb::WHITE);
		d.add_shape(0, Shape::new(ShapeKind::Rectangle, 0, 0, 10, 10, Srgb::BLACK));
		d.add_shape(0, Shape::text("Hello (PDF) \\ test", "Arial", 14, 2, 2, 30, 8, Srgb::BLACK));
		d
	}

	/// Read the xref table back: (object number -> offset).
	fn xref_offsets(pdf: &[u8]) -> Vec<(usize, usize)> {
		let text = String::from_utf8_lossy(pdf);
		let start = text.find("\nxref\n").unwrap() + 1;
		let mut lines = text[start..].lines();
		assert_eq!(lines.next(), Some("xref"));
		let header = lines.next().unwrap();
		let count: usize = header.split_whitespace().nth(1).unwrap().parse().unwrap();
		(0..count)
			.map(|n| {
				let line = lines.next().unwrap();
				(n, line[..10].parse::<usize>().unwrap())
			})
			.collect()
	}

	#[test]
	fn structure_and_xref_offsets_are_real() {
		let pdf = sample().to_pdf();
		assert!(pdf.starts_with(b"%PDF-1.4\n"));
		assert!(pdf.ends_with(b"%%EOF\n"));
		let text = String::from_utf8_lossy(&pdf).into_owned();
		assert!(text.contains("/Type /Catalog"), "{text}");
		assert!(text.contains("/Count 2"), "{text}");
		// Every xref offset points at its object's first byte.
		// (Compared on the raw bytes: the compressed image
		// stream is not text, and a lossy read would shift
		// every offset after it.)
		for (n, off) in xref_offsets(&pdf) {
			if n == 0 {
				continue;
			}
			let expect = format!("{n} 0 obj");
			assert_eq!(
				&pdf[off..off + expect.len()],
				expect.as_bytes(),
				"xref offset for object {n} is wrong"
			);
		}
		// startxref points at the xref keyword itself.
		let tail = &pdf[pdf.len() - 64..];
		let tail = String::from_utf8_lossy(tail).into_owned();
		let sx: usize = tail
			.rsplit("startxref\n")
			.next()
			.unwrap()
			.trim()
			.trim_end_matches("%%EOF")
			.trim()
			.parse()
			.unwrap();
		assert_eq!(&pdf[sx..sx + 4], b"xref");
	}

	#[test]
	fn the_image_stream_decodes_to_the_raster() {
		let d = sample();
		let pdf = d.to_pdf();
		let text = String::from_utf8_lossy(&pdf).into_owned();
		// Find the image stream and pull its bytes out by the
		// declared length.
		let dict_at = text.find("/Subtype /Image").unwrap();
		let stream_at = text[dict_at..].find("stream\n").unwrap() + dict_at + 7;
		let len: usize = text[dict_at..stream_at]
			.split("/Length ")
			.nth(1)
			.unwrap()
			.split_whitespace()
			.next()
			.unwrap()
			.parse()
			.unwrap();
		let mut decoded = Vec::new();
		flate2::read::ZlibDecoder::new(&pdf[stream_at..stream_at + len])
			.read_to_end(&mut decoded)
			.unwrap();
		let img = d.flatten().render();
		assert_eq!(decoded.len(), (img.width() * img.height() * 3) as usize);
		assert_eq!(decoded, img.as_raw().clone());
	}

	#[test]
	fn the_words_page_carries_real_escaped_text() {
		let pdf = sample().to_pdf();
		let text = String::from_utf8_lossy(&pdf).into_owned();
		assert!(text.contains("(Text in this design) Tj"), "{text}");
		// Parentheses and the backslash in the phrase are escaped.
		assert!(text.contains("(Hello \\(PDF\\) \\\\ test) Tj"), "{text}");
		// A design with no text still gets its words page.
		let bare = Drawing::new(10, 10, Srgb::WHITE).to_pdf();
		let bare_text = String::from_utf8_lossy(&bare).into_owned();
		assert!(bare_text.contains("(\\(This design has no text elements.\\)) Tj"), "{bare_text}");
	}
}
