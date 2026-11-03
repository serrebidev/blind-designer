//! braille: uncontracted (Grade 1) braille translation.
//!
//! ASCII letters, digits and common punctuation become Unicode
//! braille patterns (U+2800 block), one cell per character, the way
//! a braille label reads. Capitals take the capital sign before the
//! letter; a run of digits takes one number sign before it, and a
//! letter from a to j straight after digits takes the letter sign so
//! it is not misread as another digit. Anything with no cell here —
//! symbols outside the table — becomes a blank cell, so the output
//! is always one cell per input character (signs aside).

/// The braille pattern for a set of dot numbers (1 to 8).
fn cell(dots: &[u8]) -> char {
	let mut bits: u32 = 0;
	for d in dots {
		bits |= 1 << (d - 1);
	}
	char::from_u32(0x2800 + bits).unwrap_or('\u{2800}')
}

/// The cell for a lowercase letter (also the digit cells: 1=a … 9=i,
/// 0=j).
fn letter_cell(c: char) -> Option<char> {
	let dots: &[u8] = match c {
		'a' => &[1],
		'b' => &[1, 2],
		'c' => &[1, 4],
		'd' => &[1, 4, 5],
		'e' => &[1, 5],
		'f' => &[1, 2, 4],
		'g' => &[1, 2, 4, 5],
		'h' => &[1, 2, 5],
		'i' => &[2, 4],
		'j' => &[2, 4, 5],
		'k' => &[1, 3],
		'l' => &[1, 2, 3],
		'm' => &[1, 3, 4],
		'n' => &[1, 3, 4, 5],
		'o' => &[1, 3, 5],
		'p' => &[1, 2, 3, 4],
		'q' => &[1, 2, 3, 4, 5],
		'r' => &[1, 2, 3, 5],
		's' => &[2, 3, 4],
		't' => &[2, 3, 4, 5],
		'u' => &[1, 3, 6],
		'v' => &[1, 2, 3, 6],
		'w' => &[2, 4, 5, 6],
		'x' => &[1, 3, 4, 6],
		'y' => &[1, 3, 4, 5, 6],
		'z' => &[1, 3, 5, 6],
		_ => return None,
	};
	Some(cell(dots))
}

/// The cell for a punctuation mark, where this table has one.
fn punctuation_cell(c: char) -> Option<char> {
	let dots: &[u8] = match c {
		',' => &[2],
		';' => &[2, 3],
		':' => &[2, 5],
		'.' => &[2, 5, 6],
		'!' => &[2, 3, 5],
		'?' => &[2, 3, 6],
		'\'' => &[3],
		'-' => &[3, 6],
		'"' => &[3, 5, 6],
		'(' | ')' => &[2, 3, 5, 6],
		'/' => &[3, 4],
		_ => return None,
	};
	Some(cell(dots))
}

/// Translate text to Grade 1 braille patterns.
pub fn to_braille(text: &str) -> String {
	let capital = cell(&[6]);
	let number = cell(&[3, 4, 5, 6]);
	let letter_sign = cell(&[5, 6]);
	let mut out = String::new();
	let mut in_number = false;
	for ch in text.chars() {
		if ch.is_ascii_digit() {
			if !in_number {
				out.push(number);
				in_number = true;
			}
			let as_letter = if ch == '0' { 'j' } else { (b'a' + (ch as u8 - b'1')) as char };
			out.push(letter_cell(as_letter).unwrap_or('\u{2800}'));
			continue;
		}
		if ch.is_ascii_alphabetic() {
			let lower = ch.to_ascii_lowercase();
			if in_number && ('a'..='j').contains(&lower) {
				out.push(letter_sign);
			}
			in_number = false;
			if ch.is_ascii_uppercase() {
				out.push(capital);
			}
			out.push(letter_cell(lower).unwrap_or('\u{2800}'));
			continue;
		}
		in_number = false;
		if ch.is_whitespace() {
			out.push(' ');
		} else if let Some(c) = punctuation_cell(ch) {
			out.push(c);
		} else {
			out.push('\u{2800}'); // no cell in this table: a blank cell
		}
	}
	out
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn letters_translate_cell_for_cell() {
		assert_eq!(to_braille("abc"), "⠁⠃⠉");
		assert_eq!(to_braille("xyz"), "⠭⠽⠵");
		assert_eq!(to_braille("a b"), "⠁ ⠃"); // space stays a space
	}

	#[test]
	fn capitals_take_the_capital_sign() {
		assert_eq!(to_braille("A"), "⠠⠁");
		assert_eq!(to_braille("Hi"), "⠠⠓⠊");
	}

	#[test]
	fn digits_take_one_number_sign_per_run() {
		assert_eq!(to_braille("1"), "⠼⠁");
		assert_eq!(to_braille("12"), "⠼⠁⠃");
		assert_eq!(to_braille("90"), "⠼⠊⠚"); // 9=i, 0=j
		assert_eq!(to_braille("a 2 b"), "⠁ ⠼⠃ ⠃");
	}

	#[test]
	fn a_letter_after_digits_takes_the_letter_sign() {
		// Without the sign, "1a" would read as "11".
		assert_eq!(to_braille("1a"), "⠼⠁⠰⠁");
		// Letters past j need no sign.
		assert_eq!(to_braille("1z"), "⠼⠁⠵");
	}

	#[test]
	fn punctuation_has_cells_and_unknowns_go_blank() {
		assert_eq!(to_braille("End."), "⠠⠑⠝⠙⠲");
		assert_eq!(to_braille("a,b"), "⠁⠂⠃");
		assert_eq!(to_braille("go!"), "⠛⠕⠖");
		assert_eq!(to_braille("a~b"), "⠁\u{2800}⠃");
	}
}
