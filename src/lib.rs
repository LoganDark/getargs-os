#![cfg_attr(feature = "os_str_bytes", feature(os_str_bytes))]

#![allow(clippy::tabs_in_doc_comments)] // what a stupid fucking lint

//! Adds a newtype wrapper ([`OsArgument`]) around [`OsStr`] that allows it to
//! be parsed by [`getargs::Options`].
//!
//! In combination with the [`argv`](https://docs.rs/argv) crate, this allows
//! for lowest-cost argument parsing across all platforms (zero-cost on Linux).
//!
//! This is a separate crate from `getargs` because it requires (wildly) unsafe
//! code. `std` does not want us messing with [`OsStr`]s at all!
//!
//! ## Usage
//!
//! First, obtain an iterator over [`OsStr`]s somehow - I recommend
//! [`argv`](https://docs.rs/argv) once again - then wrap them in [`OsArgument`]
//! and pass that to [`Options::new`][getargs::Options::new].
//!
//! ```compile_only
//! # fn main() {
//! use getargs::Options;
//! use getargs_os::OsArgument;
//!
//! let mut opts = Options::new(argv::iter().skip(1).map(<&OsArgument>::from));
//! # }
//! ```
//!
//! Then use [`Options`][getargs::Options] as normal - check its documentation
//! for more usage examples.
//!
//! You can use the [`os!`] macro to create new OS strings to compare arguments
//! against. This macro works on all operating systems. For example:
//!
//! ```compile_only
//! # fn main() {
//! # use getargs::{Options, Arg};
//! # use getargs_os::{os, OsArgument};
//! # let mut opts = Options::new(argv::iter().skip(1).map(<&OsArgument>::from));
//! while let Some(arg) = opts.next_arg().expect("some ooga booga just happened") {
//! 	if arg == Arg::Long(os!("help")) {
//! 		// print help...
//! 	} else {
//! 		// ...
//! 	}
//! }
//! # }
//! ```
//!
//! ### `os_str_bytes` feature
//!
//! To unlock `From<&str>` and `PartialEq<&str>` impls for `&OsArgument`, you
//! must enable the unstable `os_str_bytes` feature, which depends on Nightly.
//! This is because earlier versions of Rust didn't provide guarantees that OS
//! strings are a superset of UTF-8 (even though `getargs-os` relied on this
//! anyway in the past). Since the feature now exists, I don't want to make
//! `getargs-os` unconditionally require Nightly, but new features relying on
//! this guarantee will be gated behind the `os_str_bytes` feature until it is
//! stabilized.

use std::ffi::OsStr;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use getargs::Argument;

mod utf8_bs;

#[cfg(test)]
mod test;

/// A newtype wrapper around [`OsStr`] that allows it to be parsed by
/// [`Options`][getargs::Options].
///
/// The short option type for this [`Argument`] implementation is *UTF-8
/// codepoints*; however they may not all be valid `char`s.
#[repr(transparent)]
pub struct OsArgument(pub OsStr);

impl<'a> From<&'a OsStr> for &'a OsArgument {
	fn from(from: &'a OsStr) -> Self {
		// SAFETY: `OsArgument` is `repr(transparent)`
		unsafe { std::mem::transmute(from) }
	}
}

impl<'a> From<&'a OsArgument> for &'a OsStr {
	fn from(from: &'a OsArgument) -> Self {
		// SAFETY: `OsArgument` is `repr(transparent)`
		unsafe { std::mem::transmute(from) }
	}
}

#[cfg(feature = "os_str_bytes")]
impl From<&str> for &OsArgument {
	fn from(from: &str) -> Self {
		Self::from(unsafe { OsStr::from_os_str_bytes_unchecked(from.as_bytes()) })
	}
}

impl OsArgument {
	fn as_bytes(&self) -> &[u8] {
		#[cfg(windows)]
		// SAFETY: This relies on representation! This is not future-proof!
		// But there is no other way to do this, OsStr is completely opaque!
		// `std` tries very hard to hide the contents from us!
		unsafe { std::mem::transmute(&self.0) }

		#[cfg(not(windows))]
		// Unix is awesome and `OsStr`s are just byte arrays
		std::os::unix::ffi::OsStrExt::as_bytes(&self.0)
	}

	fn from_bytes(bytes: &[u8]) -> &Self {
		#[cfg(windows)]
		// SAFETY: Ditto above!
		unsafe { std::mem::transmute(bytes) }

		#[cfg(not(windows))]
		// Unix is awesome and `OsStr`s are just byte arrays
		<&Self as From<&OsStr>>::from(std::os::unix::ffi::OsStrExt::from_bytes(bytes))
	}
}

impl Deref for OsArgument {
	type Target = OsStr;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for OsArgument {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl PartialEq for OsArgument {
	fn eq(&self, other: &Self) -> bool { self.0 == other.0 }
}

#[cfg(feature = "os_str_bytes")]
impl PartialEq<&str> for &OsArgument {
	fn eq(&self, other: &str) -> bool {
		self == other.into()
	}
}

#[cfg(feature = "os_str_bytes")]
impl PartialEq<&OsArgument> for &str {
	fn eq(&self, other: &OsArgument) -> bool {
		self.into() == other
	}
}

impl Eq for OsArgument {}

impl Debug for OsArgument {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { self.0.fmt(f) }
}

impl Hash for OsArgument {
	fn hash<H: Hasher>(&self, state: &mut H) { self.0.hash(state) }
}

/// Represents either a Unicode codepoint or an arbitrary byte. Used by
/// [`OsArgument`] to represent short options.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum ShortOpt {
	/// A Unicode codepoint. On Windows, short options will always be valid
	/// codepoints (but may be invalid characters, such as unpaired surrogates).
	Codepoint(u32),

	/// An arbitrary byte, which can happen if the [`OsStr`] is invalid Unicode.
	/// Windows always has valid codepoints, but this may be encountered on Unix
	/// or Linux systems.
	Byte(u8)
}

impl From<char> for ShortOpt {
	fn from(codepoint: char) -> Self {
		Self::Codepoint(codepoint as u32)
	}
}

impl From<u32> for ShortOpt {
	fn from(codepoint: u32) -> Self {
		Self::Codepoint(codepoint)
	}
}

impl From<u8> for ShortOpt {
	fn from(byte: u8) -> Self {
		Self::Byte(byte)
	}
}

impl Argument for &'_ OsArgument {
	type ShortOpt = ShortOpt;

	#[inline]
	fn ends_opts(self) -> bool {
		self.as_bytes() == b"--"
	}

	#[inline]
	fn parse_long_opt(self) -> Option<(Self, Option<Self>)> {
		// WTF-8 makes this fine (this is in hideous implementation-detail land)
		self.as_bytes().parse_long_opt().map(|(name, value)| (OsArgument::from_bytes(name), value.map(OsArgument::from_bytes)))
	}

	#[inline]
	fn parse_short_cluster(self) -> Option<Self> {
		// WTF-8 makes this fine again!
		self.as_bytes().parse_short_cluster().map(OsArgument::from_bytes)
	}

	#[cfg_attr(not(windows), inline)] // UTF-8/WTF-8 codepoint parser included, it big!
	fn consume_short_opt(self) -> (Self::ShortOpt, Option<Self>) {
		#[cfg(windows)] {
			// This is horrible and relies on WTF-8 again!
			let mut iter = self.as_bytes().iter();
			let codepoint = unsafe { utf8_bs::next_code_point(&mut iter).unwrap_unchecked() };
			(ShortOpt::Codepoint(codepoint), Some(iter.as_slice()).filter(|&slice| !slice.is_empty()).map(OsArgument::from_bytes))
		}

		#[cfg(not(windows))] {
			let bytes = self.as_bytes();

			// Optimistically try to parse as UTF-8!
			let first = unsafe { *bytes.get_unchecked(0) };
			let encoded_length = utf8_bs::utf8_char_width(first);

			let (codepoint, rest) = if let Some(Ok(Some(char))) = bytes.get(0..encoded_length).map(|slice| std::str::from_utf8(slice).map(|str| str.chars().next())) {
				// SAFETY: We know all of `encoded_length` exists!
				(ShortOpt::Codepoint(char as u32), unsafe { bytes.get_unchecked(encoded_length..) })
			} else {
				// Fall back to one byte at a time if UTF-8 parsing fails!
				(ShortOpt::Byte(first), unsafe { bytes.get_unchecked(1..) })
			};

			(codepoint, Some(OsArgument::from_bytes(rest)).filter(|s| !s.is_empty()))
		}
	}

	#[inline]
	fn consume_short_val(self) -> Self {
		self
	}
}

/// Creates an OS string from a literal string (`"whatever"`).
///
/// For an unsafe version of this macro that permits invalid UTF-8, see [`osb`].
/// Note that [`osb`] causes immediate Undefined Behavior with invalid UTF-8 on
/// on Windows.
#[macro_export]
macro_rules! os {
	($string:literal) => { <&$crate::OsArgument as From<&::std::ffi::OsStr>>::from(unsafe { std::mem::transmute(str::as_bytes($string as &str)) }) }
}

/// Creates an [`OsStr`] from a literal byte string (`b"whatever"`).
///
/// This macro is **unsafe** because creating an [`OsStr`] from invalid UTF-8 is
/// Undefined Behavior on Windows (but not Unix or Linux).
#[macro_export]
macro_rules! osb {
	($bytes:literal) => { <&$crate::OsArgument as From<&::std::ffi::OsStr>>::from(std::mem::transmute($bytes as &[u8])) }
}
