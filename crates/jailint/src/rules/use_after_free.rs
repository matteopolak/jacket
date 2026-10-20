//! `use_after_free`: `free(buffer); print("%\n", buffer[0]);`.
//!
//! Once a call frees its argument, the memory or resource behind a variable is gone; reading
//! the variable again reads freed memory, or hands a dead resource to another call.
//!
//! Fires on a local variable or parameter that is the first argument of a statement-level call
//! of a `@FreesArgument` procedure and is read by a later statement of the same statement list,
//! before anything gives it a new value. Because the read is in the same list, it runs after the
//! free. Quiet when:
//!
//! - the first mention after the free assigns the variable (`p = null;`), takes its address, is
//!   a `==`/`!=` test, or is another free (`double_free` covers a second free);
//! - a `return`, `break` or `continue` comes first;
//! - the variable's address is taken anywhere in the procedure, the procedure uses `using`, or
//!   the code in between holds `#insert`, `#asm` or a macro call, which can touch it unseen;
//! - the freeing call is a `defer`, or is nested in a branch (only straight-line code is
//!   followed).
use super::finding;
use super::frees::{After, scan};
use crate::Finding;
use crate::syntax::Cx;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    scan(cx, &mut |freed| {
        if let After::Use(span) = freed.after {
            out.push(finding(
                span,
                format!("`{}` is used after the call above freed it", freed.name),
                Some(
                    "free it after its last use, or give it a new value before using it again"
                        .into(),
                ),
            ));
        }
    });
}
