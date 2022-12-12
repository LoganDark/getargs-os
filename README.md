# getargs-os

Adds a newtype wrapper (`OsArgument`) around `OsStr` that allows it to be parsed
by `getargs::Options`.

In combination with the `argv`(https://docs.rs/argv) crate, this allows for
lowest-cost argument parsing across all platforms (zero-cost on Linux).

This is a separate crate from `getargs` because it requires (wildly) unsafe
code. `std` does not want us messing with `OsStr`s at all!

## Usage

First, obtain an iterator over `OsStr`s somehow - I recommend
[`argv`](https://docs.rs/argv) once again - then wrap them in `OsArgument` and
pass that to `Options::new`.

```rust
use getargs::Options;
use getargs_os::OsArgument;
let mut opts = Options::new(argv::iter().skip(1).map(<&OsArgument>::from));
```

Then use `Options`getargs::Options as normal - check its documentation for more
usage examples.

You can use the `os!` macro to create new OS strings to compare arguments
against. This macro works on all operating systems. For example:

```rust
while let Some(arg) = opts.next_arg().expect("some ooga booga just happened") {
	if arg == Arg::Long(os!("help")) {
		// print help...
	} else {
		// ...
	}
}
```
