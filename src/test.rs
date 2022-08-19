use getargs::{Opt, Options};
use crate::{os, ShortOpt};

#[test]
fn stuff() {
	#[cfg(not(windows))]
		let args = [
		os!("--help"),
		os!("--dfsjdgfasjdk"),
		// -ewehu®\xFF\xFE\xFD
		unsafe { crate::osb!(b"-ewehu\xC2\xAE\xEE\x84\xAB\xEE\x85\x8B\xEE\x83\x9A\xFF\xFE\xFD") }
	];

	#[cfg(windows)]
		// Windows doesn't allow invalid codepoints in its strings!
		let args = [os!("--help"), os!("--dfsjdgfasjdk"), os!("-ewehu®")];

	let mut options = Options::new(args.into_iter());

	// Parse ASCII and stuff fine!
	assert_eq!(options.next_opt(), Ok(Some(Opt::Long(os!("help")))));
	assert_eq!(options.next_opt(), Ok(Some(Opt::Long(os!("dfsjdgfasjdk")))));
	assert_eq!(options.next_opt(), Ok(Some(Opt::Short(ShortOpt::Codepoint('e' as u32)))));
	assert_eq!(options.next_opt(), Ok(Some(Opt::Short(ShortOpt::Codepoint('w' as u32)))));
	assert_eq!(options.next_opt(), Ok(Some(Opt::Short(ShortOpt::Codepoint('e' as u32)))));
	assert_eq!(options.next_opt(), Ok(Some(Opt::Short(ShortOpt::Codepoint('h' as u32)))));
	assert_eq!(options.next_opt(), Ok(Some(Opt::Short(ShortOpt::Codepoint('u' as u32)))));

	// Optimistically parse UTF-8!
	assert_eq!(options.next_opt(), Ok(Some(Opt::Short(ShortOpt::Codepoint('®' as u32)))));
	assert_eq!(options.next_opt(), Ok(Some(Opt::Short(ShortOpt::Codepoint('' as u32)))));
	assert_eq!(options.next_opt(), Ok(Some(Opt::Short(ShortOpt::Codepoint('' as u32)))));
	assert_eq!(options.next_opt(), Ok(Some(Opt::Short(ShortOpt::Codepoint('' as u32)))));

	// Parse invalid bytes on their own!
	#[cfg(not(windows))] {
		assert_eq!(options.next_opt(), Ok(Some(Opt::Short(ShortOpt::Byte(0xFF)))));
		assert_eq!(options.next_opt(), Ok(Some(Opt::Short(ShortOpt::Byte(0xFE)))));
		assert_eq!(options.next_opt(), Ok(Some(Opt::Short(ShortOpt::Byte(0xFD)))));
	}

	assert_eq!(options.next_opt(), Ok(None));
}
