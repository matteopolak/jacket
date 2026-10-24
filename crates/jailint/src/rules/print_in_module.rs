//! `print_in_module`: a module writes to standard output or error.
//!
//! ```jai
//! parse :: (text: string) -> Tree {
//!     print("parsing %\n", text);
//!     ...
//! }
//! ```
//!
//! A library does not know where its output belongs: the program that uses it may be a server,
//! a tool whose output is piped, or a game with no console. Return the text or an error and let
//! the program print it, or log through the logger, which the program can redirect.
//!
//! A file is part of a module when it or a directory above it holds a `module.jai`. A call is
//! reported when the procedure it calls carries the note `@WritesOutput` (the library's `print`,
//! `write_string`, `write_strings` and `write_builder`), whatever it is called where it is used.
//! Macros and generated files are left alone. No fix.
use super::calls_in_module;
use crate::Finding;
use crate::syntax::Cx;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    calls_in_module(
        cx,
        out,
        "WritesOutput",
        "a module should not write to the program's output",
        "return the text or an error, or log it, and let the program decide what to print",
    );
}
