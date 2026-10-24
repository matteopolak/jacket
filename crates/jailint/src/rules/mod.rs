//! The rules. Each lives in its own module, documented with why it exists; [`RULES`] lists
//! them with their default levels.
use crate::config::Category;
use crate::syntax::{Cx, Node};
use crate::{Edit, Finding, Fix, Level};
use jaic::ast::{BinOp, Expr, ExprKind as E, Stmt, StmtKind as S, UnOp};
use jaic::sema::ProcId;
use jaic::source::Span;

mod absurd_comparison;
mod almost_swapped;
mod approx_constant;
mod assert_constant;
mod autocast_in_arg;
mod autocast_used;
mod bad_bit_mask;
mod banner_comment;
mod bitwise_precedence;
mod bool_comparison;
mod bool_to_int_with_if;
pub(crate) mod broken_doc_link;
mod cognitive_complexity;
mod constant_condition;
mod defer_frees_returned;
mod defer_in_loop;
mod detached_doc_comment;
mod double_free;
mod dropped_result;
mod duplicate_condition;
mod element_count_as_bytes;
mod enum_member_prefix;
mod erasing_op;
mod excessive_bools;
mod excessive_nesting;
mod exit_in_module;
mod field_reassign_with_default;
mod float_equality;
mod float_loop_counter;
mod format_push_string;
mod frees;
mod identical_branches;
mod identical_case_bodies;
mod identical_operands;
mod identity_op;
mod if_not_else;
mod impossible_comparison;
mod incomplete_enum_switch;
mod index_only_loop;
mod infinite_loop;
mod int_plus_one;
mod integer_division_in_float;
mod large_stack_array;
mod leaked_allocation;
mod leaked_temporary;
mod loop_with_break_condition;
mod lossy_float_literal;
mod lossy_xx;
mod magic_number;
mod manual_array_find;
mod manual_assign_op;
mod manual_clamp;
mod manual_index_counter;
mod manual_memcpy;
mod min_max;
mod misrefactored_assign_op;
mod missing_doc;
mod mixed_case_hex_literals;
mod modified_range_bound;
mod needless_bool;
mod needless_continue;
mod neg_cmp_float;
mod neg_multiply;
mod no_effect;
mod pointer_into_resizable;
mod print_in_module;
mod range_past_count;
mod redundant_alias;
mod redundant_cast;
mod redundant_deref;
mod remove_in_for;
mod returned_local_address;
mod reversed_range;
mod self_assignment;
mod shadowed_it;
mod shadowed_local;
mod shift_overflow;
mod single_case_switch;
mod string_literal_write;
mod temp_storage_escape;
mod todo_marker;
mod too_many_arguments;
mod unchecked_directive;
mod unclosed_resource;
mod uninit_read;
mod unknown_allow;
mod unnecessary_force_cast;
mod unnecessary_semicolon;
mod unpaired_lock;
mod unreadable_literal;
mod unsigned_range_end_minus_one;
pub(crate) mod unused_import;
mod unused_parameter;
pub(crate) use unused_parameter::RunState;
mod unused_result;
mod unused_suppression;
mod unused_variable;
mod use_after_free;
mod useless_format;
mod useless_resizable;
mod wrapping_constant;
mod zero_ptr;

pub struct RuleInfo {
    pub name: &'static str,
    /// The rule's group; its level is the rule's default.
    pub category: Category,
    /// One line for `--list`.
    pub summary: &'static str,
    pub(crate) check: fn(&Cx, &mut Vec<Finding>),
}

impl RuleInfo {
    /// The level the rule has when nothing configures it: its category's.
    pub fn default(&self) -> Level {
        self.category.level()
    }
}

pub static RULES: &[RuleInfo] = &[
    RuleInfo {
        name: "absurd_comparison",
        category: Category::Correctness,
        summary: "a comparison an unsigned value always passes or fails (`u >= 0`)",
        check: absurd_comparison::check,
    },
    RuleInfo {
        name: "autocast_in_arg",
        category: Category::Restriction,
        summary: "`xx`, the automatic cast, written as an argument of a call",
        check: autocast_in_arg::check,
    },
    RuleInfo {
        name: "almost_swapped",
        category: Category::Correctness,
        summary: "`a = b; b = a;`, a swap that sets both to `b`",
        check: almost_swapped::check,
    },
    RuleInfo {
        name: "autocast_used",
        category: Category::Restriction,
        summary: "`xx`, the automatic cast, outside a call's arguments",
        check: autocast_used::check,
    },
    RuleInfo {
        name: "approx_constant",
        category: Category::Style,
        summary: "a hand-typed `3.14159` or `1.4142` where `Math` has the constant",
        check: approx_constant::check,
    },
    RuleInfo {
        name: "bad_bit_mask",
        category: Category::Correctness,
        summary: "a mask test that can never match (`(x & 4) == 1`, `(x | 2) == 0`)",
        check: bad_bit_mask::check,
    },
    RuleInfo {
        name: "assert_constant",
        category: Category::Style,
        summary: "`assert(true)` or `#assert true`, which can never fail",
        check: assert_constant::check,
    },
    RuleInfo {
        name: "banner_comment",
        category: Category::Style,
        summary: "a comment that is only a line of `-`, `=` or `*` fencing off a section",
        check: banner_comment::check,
    },
    RuleInfo {
        name: "bitwise_precedence",
        category: Category::Suspicious,
        summary: "a bitwise operator that binds differently than in C (`1 << n - 1`)",
        check: bitwise_precedence::check,
    },
    RuleInfo {
        name: "bool_comparison",
        category: Category::Complexity,
        summary: "comparing a bool with `true` or `false`",
        check: bool_comparison::check,
    },
    RuleInfo {
        name: "bool_to_int_with_if",
        category: Category::Pedantic,
        summary: "`ifx c then 1 else 0`, a `bool` turned into an integer with a branch",
        check: bool_to_int_with_if::check,
    },
    RuleInfo {
        name: "broken_doc_link",
        category: Category::Style,
        summary: "a link in a doc comment (`[name]`) that names no declaration",
        check: broken_doc_link::check,
    },
    RuleInfo {
        name: "cognitive_complexity",
        category: Category::Pedantic,
        summary: "a procedure whose branching scores above `max_complexity` (default 25)",
        check: cognitive_complexity::check,
    },
    RuleInfo {
        name: "constant_condition",
        category: Category::Suspicious,
        summary: "an `if` or `while` whose condition the compiler shows is always true or false",
        check: constant_condition::check,
    },
    RuleInfo {
        name: "defer_frees_returned",
        category: Category::Correctness,
        summary: "`defer free(p);` followed by `return p;`, which hands back freed memory",
        check: defer_frees_returned::check,
    },
    RuleInfo {
        name: "defer_in_loop",
        category: Category::Suspicious,
        summary: "a `defer` in a loop body that cleans up something from outside the loop",
        check: defer_in_loop::check,
    },
    RuleInfo {
        name: "detached_doc_comment",
        category: Category::Pedantic,
        summary: "a comment that reads as a declaration's docs but has a blank line between it and the declaration",
        check: detached_doc_comment::check,
    },
    RuleInfo {
        name: "double_free",
        category: Category::Correctness,
        summary: "two `@FreesArgument` calls on the same variable with nothing in between",
        check: double_free::check,
    },
    RuleInfo {
        name: "dropped_result",
        category: Category::Suspicious,
        summary: "a call drops the `bool` that follows the values it keeps (`n := to_integer(s);`)",
        check: dropped_result::check,
    },
    RuleInfo {
        name: "duplicate_condition",
        category: Category::Suspicious,
        summary: "an `else if` that repeats an earlier condition",
        check: duplicate_condition::check,
    },
    RuleInfo {
        name: "element_count_as_bytes",
        category: Category::Correctness,
        summary: "`memcpy(d, xs.data, xs.count)` with elements wider than a byte",
        check: element_count_as_bytes::check,
    },
    RuleInfo {
        name: "enum_member_prefix",
        category: Category::Style,
        summary: "every member of an enum starts with the enum's name",
        check: enum_member_prefix::check,
    },
    RuleInfo {
        name: "erasing_op",
        category: Category::Suspicious,
        summary: "an operation that is always `0` (`x * 0`, `x & 0`)",
        check: erasing_op::check,
    },
    RuleInfo {
        name: "excessive_bools",
        category: Category::Restriction,
        summary: "a procedure with more `bool` parameters than `max_bools` (default 3)",
        check: excessive_bools::check,
    },
    RuleInfo {
        name: "excessive_nesting",
        category: Category::Pedantic,
        summary: "blocks nested deeper than `max_nesting` (default 6)",
        check: excessive_nesting::check,
    },
    RuleInfo {
        name: "field_reassign_with_default",
        category: Category::Pedantic,
        summary: "`x: T; x.a = 1; x.b = 2;` where a struct literal `x := T.{ a = 1, b = 2 }` sets the fields at the declaration",
        check: field_reassign_with_default::check,
    },
    RuleInfo {
        name: "float_loop_counter",
        category: Category::Pedantic,
        summary: "a `while` loop counted in floats (`while t < 1.0 { t += 0.1; }`)",
        check: float_loop_counter::check,
    },
    RuleInfo {
        name: "exit_in_module",
        category: Category::Restriction,
        summary: "a module ends the process",
        check: exit_in_module::check,
    },
    RuleInfo {
        name: "format_push_string",
        category: Category::Perf,
        summary: "`append(*b, tprint(...))`, which formats into a new string to add it to a builder",
        check: format_push_string::check,
    },
    RuleInfo {
        name: "float_equality",
        category: Category::Pedantic,
        summary: "`==` or `!=` between two computed floats",
        check: float_equality::check,
    },
    RuleInfo {
        name: "identical_branches",
        category: Category::Suspicious,
        summary: "an `if`/`else` or `ifx` whose branches are the same",
        check: identical_branches::check,
    },
    RuleInfo {
        name: "identical_case_bodies",
        category: Category::Pedantic,
        summary: "two cases of a `switch` with the same body, which could list their values together",
        check: identical_case_bodies::check,
    },
    RuleInfo {
        name: "identical_operands",
        category: Category::Suspicious,
        summary: "an operator with the same expression on both sides (`a == a`)",
        check: identical_operands::check,
    },
    RuleInfo {
        name: "identity_op",
        category: Category::Complexity,
        summary: "an operation that changes nothing (`x + 0`, `x * 1`)",
        check: identity_op::check,
    },
    RuleInfo {
        name: "if_not_else",
        category: Category::Pedantic,
        summary: "an `if` or `ifx` with a negated condition and an `else`",
        check: if_not_else::check,
    },
    RuleInfo {
        name: "incomplete_enum_switch",
        category: Category::Pedantic,
        summary: "a `switch` on an enum that neither covers every member nor has a default case",
        check: incomplete_enum_switch::check,
    },
    RuleInfo {
        name: "impossible_comparison",
        category: Category::Correctness,
        summary: "comparisons of one integer that never pass together (`x > 5 && x < 3`)",
        check: impossible_comparison::check,
    },
    RuleInfo {
        name: "index_only_loop",
        category: Category::Complexity,
        summary: "`for i: 0..xs.count-1` where `i` only indexes `xs`",
        check: index_only_loop::check,
    },
    RuleInfo {
        name: "infinite_loop",
        category: Category::Suspicious,
        summary: "a `while` whose condition nothing in the loop changes",
        check: infinite_loop::check,
    },
    RuleInfo {
        name: "int_plus_one",
        category: Category::Complexity,
        summary: "`x >= y + 1`, which is `x > y`",
        check: int_plus_one::check,
    },
    RuleInfo {
        name: "large_stack_array",
        category: Category::Pedantic,
        summary: "a local fixed array bigger than `max_stack_array_bytes` (default 64 KiB)",
        check: large_stack_array::check,
    },
    RuleInfo {
        name: "integer_division_in_float",
        category: Category::Suspicious,
        summary: "an integer division whose result is used as a float",
        check: integer_division_in_float::check,
    },
    RuleInfo {
        name: "leaked_allocation",
        category: Category::Pedantic,
        summary: "a local holding memory from an `@Allocates` call that is never freed, returned or handed on",
        check: leaked_allocation::check,
    },
    RuleInfo {
        name: "leaked_temporary",
        category: Category::Suspicious,
        summary: "an `@Allocates` result that is only compared, measured, indexed or printed, so nothing can free it",
        check: leaked_temporary::check,
    },
    RuleInfo {
        name: "lossy_float_literal",
        category: Category::Style,
        summary: "a `float32` literal with more digits than a `float32` keeps (`31415.92653`)",
        check: lossy_float_literal::check,
    },
    RuleInfo {
        name: "loop_with_break_condition",
        category: Category::Complexity,
        summary: "`while true { if !c break; ... }`, a loop whose condition is its first statement",
        check: loop_with_break_condition::check,
    },
    RuleInfo {
        name: "lossy_xx",
        category: Category::Pedantic,
        summary: "`xx` that narrows a number to a smaller type",
        check: lossy_xx::check,
    },
    RuleInfo {
        name: "manual_array_find",
        category: Category::Complexity,
        summary: "a loop that searches an array and breaks with a found flag or index, which `array_find` does",
        check: manual_array_find::check,
    },
    RuleInfo {
        name: "magic_number",
        category: Category::Pedantic,
        summary: "an unexplained number in arithmetic, a comparison or an assignment",
        check: magic_number::check,
    },
    RuleInfo {
        name: "manual_assign_op",
        category: Category::Complexity,
        summary: "`a = a + b` instead of `a += b`",
        check: manual_assign_op::check,
    },
    RuleInfo {
        name: "manual_clamp",
        category: Category::Complexity,
        summary: "a value kept between two constant bounds by hand instead of with `clamp`",
        check: manual_clamp::check,
    },
    RuleInfo {
        name: "manual_memcpy",
        category: Category::Perf,
        summary: "`for i: 0..n-1 { dst[i] = src[i]; }`, an element-by-element copy `memcpy` does at once",
        check: manual_memcpy::check,
    },
    RuleInfo {
        name: "manual_index_counter",
        category: Category::Complexity,
        summary: "a counter incremented in a `for` loop that always equals `it_index`",
        check: manual_index_counter::check,
    },
    RuleInfo {
        name: "min_max",
        category: Category::Correctness,
        summary: "`min(0, max(100, x))`, a clamp that is always one bound",
        check: min_max::check,
    },
    RuleInfo {
        name: "misrefactored_assign_op",
        category: Category::Correctness,
        summary: "`a += a + b`, an update that applies the operator to `a` twice",
        check: misrefactored_assign_op::check,
    },
    RuleInfo {
        name: "missing_doc",
        category: Category::Restriction,
        summary: "an exported procedure, struct or enum with no doc comment",
        check: missing_doc::check,
    },
    RuleInfo {
        name: "mixed_case_hex_literals",
        category: Category::Style,
        summary: "a hex literal with both upper and lower case digits (`0xDeadbeef`)",
        check: mixed_case_hex_literals::check,
    },
    RuleInfo {
        name: "modified_range_bound",
        category: Category::Suspicious,
        summary: "a variable bounding a `for` range assigned in the loop, which does not change it",
        check: modified_range_bound::check,
    },
    RuleInfo {
        name: "needless_bool",
        category: Category::Complexity,
        summary: "`ifx c then true else false` and `if c return true; else return false;`",
        check: needless_bool::check,
    },
    RuleInfo {
        name: "neg_cmp_float",
        category: Category::Pedantic,
        summary: "`!(a < b)` on floats, which differs from `a >= b` when a value is NaN",
        check: neg_cmp_float::check,
    },
    RuleInfo {
        name: "needless_continue",
        category: Category::Style,
        summary: "a `continue` as the last thing a loop body does",
        check: needless_continue::check,
    },
    RuleInfo {
        name: "neg_multiply",
        category: Category::Style,
        summary: "`x * -1`, which is `-x`",
        check: neg_multiply::check,
    },
    RuleInfo {
        name: "no_effect",
        category: Category::Suspicious,
        summary: "a statement that computes a value and drops it (`x == 5;`)",
        check: no_effect::check,
    },
    RuleInfo {
        name: "pointer_into_resizable",
        category: Category::Suspicious,
        summary: "a pointer into a resizable array used after a `@GrowsArray` call may have moved the array (`p := *xs[i]; array_add(*xs, v); use(p);`)",
        check: pointer_into_resizable::check,
    },
    RuleInfo {
        name: "print_in_module",
        category: Category::Restriction,
        summary: "a module writes to the program's output",
        check: print_in_module::check,
    },
    RuleInfo {
        name: "range_past_count",
        category: Category::Correctness,
        summary: "`for i: 0..xs.count` indexing `xs[i]`, one past the end",
        check: range_past_count::check,
    },
    RuleInfo {
        name: "redundant_alias",
        category: Category::Complexity,
        summary: "a local that only exists to be copied into another local or returned",
        check: redundant_alias::check,
    },
    RuleInfo {
        name: "redundant_deref",
        category: Category::Complexity,
        summary: "`p.*.field` where `p.field` means the same",
        check: redundant_deref::check,
    },
    RuleInfo {
        name: "redundant_cast",
        category: Category::Complexity,
        summary: "a cast to the type the value already has",
        check: redundant_cast::check,
    },
    RuleInfo {
        name: "remove_in_for",
        category: Category::Correctness,
        summary: "removing from an array while a `for` walks it forward",
        check: remove_in_for::check,
    },
    RuleInfo {
        name: "returned_local_address",
        category: Category::Correctness,
        summary: "`return *x;` with `x` a local, a pointer into a stack frame that is gone",
        check: returned_local_address::check,
    },
    RuleInfo {
        name: "reversed_range",
        category: Category::Correctness,
        summary: "a range whose start is above its end, which never runs",
        check: reversed_range::check,
    },
    RuleInfo {
        name: "self_assignment",
        category: Category::Suspicious,
        summary: "a value assigned to itself (`x = x;`)",
        check: self_assignment::check,
    },
    RuleInfo {
        name: "shift_overflow",
        category: Category::Suspicious,
        summary: "a shift by at least the bit width of its operand (`x << 40` with `x: s32`)",
        check: shift_overflow::check,
    },
    RuleInfo {
        name: "shadowed_it",
        category: Category::Suspicious,
        summary: "a nested `for` hides an `it` the enclosing loop uses",
        check: shadowed_it::check,
    },
    RuleInfo {
        name: "shadowed_local",
        category: Category::Pedantic,
        summary: "a local variable named like a parameter or another local in an enclosing scope of the same procedure",
        check: shadowed_local::check,
    },
    RuleInfo {
        name: "single_case_switch",
        category: Category::Style,
        summary: "a `switch` with a single case, which is an `if`",
        check: single_case_switch::check,
    },
    RuleInfo {
        name: "string_literal_write",
        category: Category::Correctness,
        summary: "a write into a string literal, which is read-only (`s := \"abc\"; s[0] = x;`)",
        check: string_literal_write::check,
    },
    RuleInfo {
        name: "temp_storage_escape",
        category: Category::Pedantic,
        summary: "a `@TempAllocates` result (`tprint`, `talloc`) stored in a global or through a pointer, or returned",
        check: temp_storage_escape::check,
    },
    RuleInfo {
        name: "todo_marker",
        category: Category::Restriction,
        summary: "a comment with a `TODO`, `FIXME`, `XXX` or `HACK` marker",
        check: todo_marker::check,
    },
    RuleInfo {
        name: "too_many_arguments",
        category: Category::Restriction,
        summary: "a procedure with more parameters than `max_arguments` (default 7)",
        check: too_many_arguments::check,
    },
    RuleInfo {
        name: "unchecked_directive",
        category: Category::Restriction,
        summary: "`#no_abc`, `#no_aoc` or `#asm` with no comment saying why",
        check: unchecked_directive::check,
    },
    RuleInfo {
        name: "unclosed_resource",
        category: Category::Suspicious,
        summary: "a local opened by an `@Opens` call (`file_open`, `append(*builder, ..)`) that nothing closes",
        check: unclosed_resource::check,
    },
    RuleInfo {
        name: "uninit_read",
        category: Category::Correctness,
        summary: "a variable declared `= ---` read before anything is written to it",
        check: uninit_read::check,
    },
    RuleInfo {
        name: "unnecessary_force_cast",
        category: Category::Complexity,
        summary: "`cast,force` where the value already has the type or the conversion widens",
        check: unnecessary_force_cast::check,
    },
    RuleInfo {
        name: "unknown_allow",
        category: Category::Suspicious,
        summary: "a `// jailint: allow(...)` comment naming a rule that does not exist",
        check: unknown_allow::check,
    },
    RuleInfo {
        name: "unnecessary_semicolon",
        category: Category::Style,
        summary: "a `;` that ends nothing (`x = 1;;`, `if c { ... };`)",
        check: unnecessary_semicolon::check,
    },
    RuleInfo {
        name: "unpaired_lock",
        category: Category::Suspicious,
        summary: "a `@Locks` call with a path that returns or ends without a matching `@Unlocks`",
        check: unpaired_lock::check,
    },
    RuleInfo {
        name: "unsigned_range_end_minus_one",
        category: Category::Suspicious,
        summary: "`for 0..n-1` with an unsigned `n`, which wraps around when `n` is 0",
        check: unsigned_range_end_minus_one::check,
    },
    RuleInfo {
        name: "unreadable_literal",
        category: Category::Pedantic,
        summary: "a long number without `_` separators (`1000000`, `0xdeadbeef`)",
        check: unreadable_literal::check,
    },
    RuleInfo {
        name: "unused_import",
        category: Category::Style,
        summary: "an `#import` nothing in its scope uses",
        check: unused_import::check,
    },
    RuleInfo {
        name: "unused_parameter",
        category: Category::Style,
        summary: "a parameter the procedure never uses",
        check: unused_parameter::check,
    },
    RuleInfo {
        name: "unused_result",
        category: Category::Suspicious,
        summary: "a call to a `@UseResult` procedure, as a statement",
        check: unused_result::check,
    },
    RuleInfo {
        name: "unused_suppression",
        category: Category::Suspicious,
        summary: "a name in a `// jailint: allow(...)` comment or `@jailint_allow` note that silences nothing",
        check: unused_suppression::check,
    },
    RuleInfo {
        name: "unused_variable",
        category: Category::Style,
        summary: "a local variable that is never used",
        check: unused_variable::check,
    },
    RuleInfo {
        name: "useless_format",
        category: Category::Complexity,
        summary: "`tprint(\"%\", s)` or `sprint(\"%\", s)` with a string `s`, which only copies it",
        check: useless_format::check,
    },
    RuleInfo {
        name: "useless_resizable",
        category: Category::Perf,
        summary: "a local `[..] T` that is only filled with a known number of `array_add` calls and never resized",
        check: useless_resizable::check,
    },
    RuleInfo {
        name: "use_after_free",
        category: Category::Correctness,
        summary: "a variable read after a `@FreesArgument` call freed it (`free(p); use(p);`)",
        check: use_after_free::check,
    },
    RuleInfo {
        name: "wrapping_constant",
        category: Category::Correctness,
        summary: "a constant that wraps to the type the other operand gives it (`(0xffff_ffff - 40) / h`)",
        check: wrapping_constant::check,
    },
    RuleInfo {
        name: "zero_ptr",
        category: Category::Style,
        summary: "`cast(*T) 0`, which is `null`",
        check: zero_ptr::check,
    },
];

/// How an expression that denotes a place is used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlaceUse {
    Read,
    /// Assigned to, or a part of it is.
    Write,
    /// Its address (or the address of a part) is taken: it may be written through.
    Address,
}

/// How the expression `node` (whose ancestors are `chain`, outermost first) is used: the
/// access path it heads (`x`, `x.a`, `x[1].b`) decides.
pub(crate) fn place_use(node: &Expr, chain: &[Node<'_>]) -> PlaceUse {
    let mut cur = node;
    let mut i = chain.len();
    while i > 0 {
        i -= 1;
        match chain[i] {
            Node::Expr(p) => match &p.kind {
                E::Member(base, _) if base.span == cur.span => cur = p,
                E::Index(base, _) if base.span == cur.span => cur = p,
                E::Unary(UnOp::Star, inner) if inner.span == cur.span => return PlaceUse::Address,
                // `x.*`: the pointer's target; writes go elsewhere.
                _ => return PlaceUse::Read,
            },
            Node::Stmt(s) => {
                return match &s.kind {
                    S::Assign {
                        lhs, ..
                    } if lhs.iter().any(|l| l.span == cur.span) => PlaceUse::Write,
                    S::For(f) if f.by_pointer || f.pointer_if.is_some() => match &f.over {
                        jaic::ast::ForOver::Collection(c) if c.span == cur.span => {
                            PlaceUse::Address
                        }
                        _ => PlaceUse::Read,
                    },
                    S::Using(using) if using.value.span == cur.span => PlaceUse::Address,
                    _ => PlaceUse::Read,
                };
            }
        }
    }
    PlaceUse::Read
}

/// `e` is a call argument (directly) in `chain`'s last node.
pub(crate) fn is_call_argument(e: &Expr, chain: &[Node<'_>]) -> bool {
    matches!(chain.last(), Some(Node::Expr(Expr { kind: E::Call { args, .. }, .. }))
        if args.iter().any(|a| a.value.span == e.span))
}

/// Evaluating `e` has no side effects and gives the same value twice in a row: names, member
/// paths, literals, and indexing, operators and casts over them. Calls never qualify.
pub(crate) fn pure(e: &Expr) -> bool {
    match &e.kind {
        E::Ident(_)
        | E::Int(_)
        | E::Float(_)
        | E::Bool(_)
        | E::Null
        | E::Str(_)
        | E::Char(_)
        | E::InferredMember(_) => true,
        E::Member(base, _) => pure(base),
        E::Index(a, b) | E::Binary(_, a, b) => pure(a) && pure(b),
        E::Unary(_, a) => pure(a),
        E::Cast {
            value, ..
        } => pure(value),
        _ => false,
    }
}

/// `a` and `b` are the same pure expression, written the same way.
pub(crate) fn same_pure(cx: &Cx, a: &Expr, b: &Expr) -> bool {
    pure(a) && pure(b) && cx.same_tokens(a.span, b.span)
}

/// An integer literal, possibly negated: its value.
pub(crate) fn int_literal(e: &Expr) -> Option<i128> {
    match &e.kind {
        E::Int(v) => i128::try_from(v.get()).ok(),
        E::Unary(UnOp::Neg, inner) => int_literal(inner).map(|v| -v),
        E::Unary(UnOp::Plus, inner) => int_literal(inner),
        _ => None,
    }
}

/// The expression is written inside its own parentheses: `(a & b)`.
pub(crate) fn parenthesized(cx: &Cx, e: &Expr) -> bool {
    let (start, end) = cx.whole(e.span);
    start < e.span.start as usize || end > e.span.end as usize
}

/// The procedures a call's callee named, when it is a plain name or `Module.name`.
pub(crate) fn callee_procs<'c>(cx: &'c Cx, callee: &Expr) -> &'c [ProcId] {
    let span = match &callee.kind {
        E::Ident(_) => callee.span,
        E::Member(_, name) => name.span,
        _ => return &[],
    };
    cx.facts.procs_at(span)
}

/// The procedure is declared with the note `@name`, on its header or after its body.
pub(crate) fn has_note(cx: &Cx, proc: ProcId, name: &str) -> bool {
    cx.compiler.proc_has_note(proc, name)
}

/// Every procedure the callee named carries the note `@name`: the declaration, not the
/// spelling of the name, says what the call means.
pub(crate) fn calls_noted(cx: &Cx, callee: &Expr, name: &str) -> bool {
    let procs = callee_procs(cx, callee);
    let span = match &callee.kind {
        E::Member(_, name) => name.span,
        _ => callee.span,
    };
    // Of an overload set, the one this call chose says what the call means.
    if let Some(chosen) = cx.compiler.ide_resolved_overload(span, procs) {
        return has_note(cx, chosen, name);
    }
    !procs.is_empty() && procs.iter().all(|&p| has_note(cx, p, name))
}

/// Some statement inside `stmt` (itself included, nested procedures not entered) satisfies `f`.
pub(crate) fn any_stmt(stmt: &Stmt, f: &mut dyn FnMut(&Stmt) -> bool) -> bool {
    let mut found = false;
    let mut stack = Vec::new();
    crate::syntax::walk_node(Node::Stmt(stmt), &mut stack, &mut |n, _| {
        if !found
            && let Node::Stmt(s) = n
            && f(s)
        {
            found = true;
        }
        !found
    });
    found
}

/// Every statement list in a procedure body: the body, nested blocks, `case` bodies and `#if`
/// branches.
pub(crate) fn statement_lists<'a>(body: &'a [Stmt], f: &mut dyn FnMut(&'a [Stmt])) {
    f(body);
    crate::syntax::walk(body, &mut |n, _| {
        if let Some(s) = n.stmt() {
            match &s.kind {
                S::Block(b) => f(&b.stmts),
                S::Switch(boxed) | S::StaticSwitch(boxed) => {
                    let jaic::ast::SwitchStmt {
                        cases, ..
                    } = &**boxed;
                    for c in cases {
                        f(&c.body);
                    }
                }
                S::StaticIf(boxed) => {
                    let jaic::ast::StaticIfStmt {
                        then_branch,
                        else_branch,
                        ..
                    } = &**boxed;
                    f(then_branch);
                    f(else_branch);
                }
                _ => {}
            }
        }
        true
    });
}

/// Visit every statement and expression the file's own code holds: its top-level statements
/// and the bodies of its procedures, macros and generated files left out. For rules that read
/// only syntax, which a macro's expansion site or a generator may write differently.
pub(crate) fn walk_code<'a>(cx: &Cx<'a>, visit: &mut dyn FnMut(Node<'a>, &[Node<'a>])) {
    if cx.generated() {
        return;
    }
    crate::syntax::walk(cx.stmts, &mut |n, up| {
        visit(n, up);
        true
    });
    for p in cx.procs.iter().filter(|p| !p.is_macro) {
        crate::syntax::walk(&p.body.stmts, &mut |n, up| {
            visit(n, up);
            true
        });
    }
}

/// Report each call, in a file that belongs to a module, to a procedure with the note `note`.
/// For rules about what a library should leave to the program that uses it.
pub(crate) fn calls_in_module(
    cx: &Cx,
    out: &mut Vec<Finding>,
    note: &str,
    message: &str,
    help: &str,
) {
    if cx.generated() || !cx.in_module() {
        return;
    }
    walk_code(cx, &mut |n, _| {
        let Node::Expr(e) = n else {
            return;
        };
        if let E::Call {
            callee, ..
        } = &e.kind
            && calls_noted(cx, callee, note)
        {
            out.push(finding(e.span, message.into(), Some(help.into())));
        }
    });
}

/// Where a finding about a procedure goes, and how its message names it: the declared name,
/// else the start of the header.
pub(crate) fn proc_label(p: &crate::syntax::ProcSite) -> (Span, String) {
    match p.decl.and_then(|d| d.names.first()) {
        Some(name) => (name.span, format!("`{}`", name.name.as_str())),
        None => (
            Span {
                end: p.header.span.start + 1,
                ..p.header.span
            },
            "this procedure".to_string(),
        ),
    }
}

/// A finding without a fix.
pub(crate) fn finding(span: Span, message: String, help: Option<String>) -> Finding {
    Finding {
        start: span.start as usize,
        end: span.end as usize,
        message,
        label: None,
        help,
        fix: None,
    }
}

/// A statement's bytes with the `;` the span may stop before.
pub(crate) fn statement_range(cx: &Cx, span: Span) -> (usize, usize) {
    let start = span.start as usize;
    let mut end = span.end as usize;
    let rest = &cx.text[end..];
    let trimmed = rest.trim_start_matches([' ', '\t']);
    if trimmed.starts_with(';') && !cx.text[..end].ends_with(';') {
        end += rest.len() - trimmed.len() + 1;
    }
    (start, end)
}

/// A finding with a fix the editor offers and `--fix` leaves alone: the change may not be what
/// was meant, or it changes what the program does. `title` names the change.
pub(crate) fn suggesting(
    span: Span,
    message: String,
    help: String,
    title: impl Into<String>,
    edits: Vec<Edit>,
) -> Finding {
    Finding {
        start: span.start as usize,
        end: span.end as usize,
        message,
        label: None,
        help: Some(help),
        fix: Some(Fix {
            title: title.into(),
            edits,
            machine_applicable: false,
        }),
    }
}

/// A finding whose fix replaces bytes `start..end` with `text`.
pub(crate) fn replacing(
    span: Span,
    message: String,
    help: String,
    (start, end): (usize, usize),
    text: String,
    machine_applicable: bool,
) -> Finding {
    Finding {
        start: span.start as usize,
        end: span.end as usize,
        message,
        label: None,
        help: Some(help.clone()),
        fix: Some(Fix {
            title: help,
            edits: vec![Edit {
                start,
                end,
                text,
            }],
            machine_applicable,
        }),
    }
}

/// How the operator is written.
pub(crate) fn op_text(op: BinOp) -> &'static str {
    match op {
        BinOp::Eq => "==",
        BinOp::Ne => "!=",
        BinOp::Lt => "<",
        BinOp::Le => "<=",
        BinOp::Gt => ">",
        BinOp::Ge => ">=",
        BinOp::Sub => "-",
        BinOp::Div => "/",
        BinOp::Rem => "%",
        BinOp::BitAnd => "&",
        BinOp::BitOr => "|",
        BinOp::BitXor => "^",
        BinOp::And => "&&",
        BinOp::Or => "||",
        BinOp::Add => "+",
        BinOp::Mul => "*",
        BinOp::Shl => "<<",
        BinOp::Shr => ">>",
        BinOp::Rotl => "<<<",
        BinOp::Rotr => ">>>",
    }
}
