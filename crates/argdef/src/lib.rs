//! Command-line options declared as a struct.
//!
//! ```
//! use argdef::Args;
//!
//! /// demo: a tool
//! #[derive(Args, Default)]
//! #[args(name = "demo", usage = "demo [options] file")]
//! struct Demo {
//!     /// print more
//!     #[arg(name = "-verbose", section = "Options")]
//!     verbose: bool,
//!     /// the output file
//!     #[arg(name = "-out", value = "file", needs = "a file name")]
//!     out: Option<String>,
//!     #[arg(positional)]
//!     files: Vec<String>,
//! }
//!
//! let args = ["-verbose", "-out=a.txt", "in.txt"].map(String::from);
//! let demo = Demo::parse(&args).unwrap();
//! assert!(demo.verbose && demo.out.as_deref() == Some("a.txt") && demo.files == ["in.txt"]);
//! ```
//!
//! Every option is one field with an `#[arg(...)]` attribute; its doc comment is its help.
//! Options are spelt as written (`-check`, `--color`), so single-dash long flags work.
//!
//! Field type decides what an option takes: `bool` is a switch, `Option<T>` a value stored when
//! given, `Vec<T>` a repeatable value, any other `T` a value assigned. A value type is read with
//! [`FromArg`], or with `parse = path` (`fn(&str, &str) -> Result<T, Error>`, option name then
//! text).
//!
//! `#[arg(...)]` keys: `name` and `alias` (spellings; only names are suggested and listed),
//! `value` (placeholder in help), `needs` (what a missing value should have been), `synopsis`
//! (the help label, replacing the spellings), `help` (replaces the doc comment), `section`,
//! `hint = "file"|"dir"` (completion), `tag` (a label [`Spec::scan`] reports), `set = expr`
//! (a switch that stores `expr`), `conflict = "message"` (with `set` on an `Option` field: two
//! different constants are refused), `skip` (a field that is not an option), `with = path` (`fn(&mut Struct, &str, &str) -> Result<(),
//! Error>` stores a value itself), `os` (the value is kept as the OS gave it, for an `OsString` or `PathBuf` field: `FromOsArg`), `attached` (a short option may be glued to its value:
//! `-Dwarnings`), `hidden` (accepted, not listed), `exit` (stop parsing there), `positional`
//! (`Vec<String>`), `rest = "-"` with `until = "--"` (`Option<Vec<String>>`: everything after the
//! marker), `group = "+"` (`Vec<impl Group>`: `+Name` and the arguments after it) and `flatten`
//! (the fields of another [`Args`] struct). A field may carry several `#[arg]`s, each a spelling
//! group of its own; the doc comment goes to the first.
//!
//! The struct's doc comment is the tool's one-line description (first line) and a longer
//! paragraph; `#[args(name, version, usage, sections)]` add the rest of the help.
mod complete;
mod error;
mod help;
mod parse;
mod spec;

pub use argdef_derive::Args;
pub use complete::Shell;
pub use error::{Error, ErrorKind};
pub use help::{Help, Row};
pub use spec::{Args, FromArg, FromOsArg, Group, Input, Kind, Meta, Opt, Scan, Spec, Token};
