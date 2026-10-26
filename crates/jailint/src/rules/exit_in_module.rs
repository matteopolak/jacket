//! `exit_in_module`: a module ends the process.
//!
//! ```jai
//! open_config :: () -> Config {
//!     ...
//!     if !ok exit(1);
//! }
//! ```
//!
//! A library that calls `exit` takes the decision away from the program that uses it: it skips
//! the program's cleanup and cannot be tested. Return an error and let the caller decide.
//!
//! A file is part of a module when it or a directory above it holds a `module.jai` and no
//! `tests` folder lies between them. A call is
//! reported when the procedure it calls carries the note `@Exits` (the library's `exit`),
//! whatever it is called where it is used. Macros and generated files are left alone. No fix.
use super::calls_in_module;
use crate::Finding;
use crate::syntax::Cx;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "exit_in_module",
    category: crate::config::Category::Restriction,
    summary: "a module ends the process",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    calls_in_module(
        cx,
        out,
        "Exits",
        "a module should not end the process",
        "return an error and let the program decide whether to exit",
    );
}
