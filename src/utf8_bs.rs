//! `std` is stupid and keeps holding implementation details away from us...
//! As a result, there is no way to do this correctly, as there are no APIs...
//! But we need to do it anyway! So rip implementation details from std

#[cfg(windows)]
#[must_use]
#[inline]
pub unsafe fn next_code_point<'a, I: Iterator<Item = &'a u8>>(bytes: &mut I) -> Option<u32> {
	const CONT_MASK: u8 = 0b0011_1111;

	#[inline]
	const fn utf8_first_byte(byte: u8, width: u32) -> u32 {
		(byte & (0x7F >> width)) as u32
	}

	#[inline]
	const fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
		(ch << 6) | (byte & CONT_MASK) as u32
	}

	let x = *bytes.next()?;
	if x < 128 { return Some(x as u32); }

	let init = utf8_first_byte(x, 2);
	let y = *bytes.next().unwrap_unchecked();
	let mut ch = utf8_acc_cont_byte(init, y);

	if x >= 0xE0 {
		let z = *bytes.next().unwrap_unchecked();
		let y_z = utf8_acc_cont_byte((y & CONT_MASK) as u32, z);

		ch = init << 12 | y_z;

		if x >= 0xF0 {
			let w = *bytes.next().unwrap_unchecked();
			ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, w);
		}
	}

	Some(ch)
}

#[cfg(not(windows))]
const UTF8_CHAR_WIDTH: &[u8; 256] = &[
	// 1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
	1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 0
	1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 1
	1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 2
	1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 3
	1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 4
	1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 5
	1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 6
	1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, // 7
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 8
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 9
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // A
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // B
	0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // C
	2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // D
	3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // E
	4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // F
];

#[cfg(not(windows))]
#[must_use]
#[inline]
pub const fn utf8_char_width(b: u8) -> usize {
	UTF8_CHAR_WIDTH[b as usize] as usize
}
