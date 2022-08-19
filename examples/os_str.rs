use getargs::Options;
use getargs_os::OsArgument;

fn main() {
	let mut opts = Options::new(argv::iter().skip(1).map(<&OsArgument>::from));

	while let Ok(Some(whatever)) = opts.next_arg() {
		println!("{:?}", whatever);
	}
}
