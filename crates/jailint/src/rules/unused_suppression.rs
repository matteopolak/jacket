//! `unused_suppression`: a `// jailint: allow(...)` comment or `@jailint_allow(...)` note that
//! silences nothing.
//!
//! ```jai
//! count := 3; // jailint: allow(float_equality)
//! ```
//!
//! Code changes, the finding a suppression was written for goes away, and the comment stays,
//! ready to hide the next real problem on that line. Reports each name in an `allow(...)`,
//! `allow-file(...)` or `@jailint_allow(...)` list that no finding was silenced by.
//!
//! The work is done by the engine (`lint_files`), which sees every other rule's findings and
//! which suppressions covered them; this module only registers the rule. A name is judged only
//! when it proves something:
//!
//! - every rule it names (all of a category's rules for a category name) ran on the file at
//!   an enabled level, since a rule that is off or overridden to `allow` reports nothing;
//! - the compile finished (`--allow-incomplete` runs, and files whose bodies failed to check,
//!   leave rules quiet);
//! - its file has no `#if`: another branch, compiled only for another target, may be what
//!   needs it (an import only the Windows branch uses);
//! - it is a real rule, category or `all` (`unknown_allow` reports the rest).
//!
//! `all` and category names are only judged when every rule they stand for is on. No fix.
//! `[[lint.overrides]]` entries in `jacket.toml` are not checked: telling that one is stale
//! means running the rule it turns off on every file it matches, across the whole run.
use crate::{Finding, syntax::Cx};

pub(crate) fn check(_cx: &Cx, _out: &mut Vec<Finding>) {
}
