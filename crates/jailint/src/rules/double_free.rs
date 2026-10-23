//! `double_free`: `free(buffer); ... free(buffer);`.
//!
//! Freeing the same memory twice corrupts the allocator; closing a closed resource fails or
//! closes whatever reused its handle.
//!
//! Fires on a statement-level call of a `@FreesArgument` procedure whose first argument is a
//! local variable or parameter that an earlier statement of the same statement list already
//! freed the same way, with no assignment to the variable in between. Both calls sit on one
//! straight-line path. Quiet when:
//!
//! - anything mentions the variable between the two calls (that is `use_after_free`'s business,
//!   or a new value), or a `return`, `break` or `continue` comes between;
//! - the second free is a `defer` or nested in a branch;
//! - the variable's address is taken anywhere in the procedure, the procedure uses `using`, or
//!   the code in between holds `#insert`, `#asm` or a macro call.
use super::frees::{After, scan};
use super::suggesting;
use crate::syntax::Cx;
use crate::{Edit, Finding};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    scan(cx, &mut |freed| {
        if let After::Free(span) = freed.after {
            let (start, end) = cx.statement_removal(span);
            out.push(suggesting(
                span,
                format!("`{}` is freed a second time", freed.name),
                format!(
                    "remove the second call, or assign `{}` a new value in between",
                    freed.name
                ),
                "remove the second free",
                vec![Edit {
                    start,
                    end,
                    text: String::new(),
                }],
            ));
        }
    });
}
