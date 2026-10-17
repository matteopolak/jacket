//! The standard-library procedures some rules recognize.
//!
//! A rule never trusts a bare name: a callee counts only when every procedure it names is
//! declared in one of the listed library modules (found through the compiler's import
//! directories), so a program's own `trim` or `min` is left alone. The name tables below are the
//! one place that says which library procedures a rule is about.
use super::callee_procs;
use crate::syntax::Cx;
use jaic::ast::{Expr, ExprKind as E};
use jaic::sema::ProcId;

/// Procedures with these `names`, declared in one of these library `modules`.
pub(crate) struct LibraryProcs {
    pub modules: &'static [&'static str],
    pub names: &'static [&'static str],
}

impl LibraryProcs {
    /// The callee is one of these procedures.
    pub(crate) fn called_by(&self, cx: &Cx, callee: &Expr) -> bool {
        calls_library(cx, callee, self.modules, self.names)
    }

    /// The callee is the procedure `name`, which must be one of these.
    pub(crate) fn calls(&self, cx: &Cx, callee: &Expr, name: &str) -> bool {
        self.names.contains(&name) && calls_library(cx, callee, self.modules, &[name])
    }
}

/// `min` and `max`.
pub(crate) const MIN_MAX: LibraryProcs = LibraryProcs {
    modules: &["Basic", "Math"],
    names: &["min", "max"],
};

/// `array_*_remove_*`: removal from an array by index or value.
pub(crate) const ARRAY_REMOVERS: LibraryProcs = LibraryProcs {
    modules: &["Basic"],
    names: &[
        "array_unordered_remove_by_index",
        "array_ordered_remove_by_index",
        "array_unordered_remove_by_value",
        "array_ordered_remove_by_value",
    ],
};

/// Procedures whose only effect is their result (or an allocation for it).
pub(crate) const VALUE_ONLY: LibraryProcs = LibraryProcs {
    modules: &["Basic", "String", "Math"],
    names: &[
        // String
        "trim",
        "trim_left",
        "trim_right",
        "to_lower_copy",
        "to_upper_copy",
        "slice",
        "join",
        "replace",
        "split",
        "path_strip_extension",
        "path_filename",
        "path_extension",
        "begins_with",
        "ends_with",
        "contains",
        "compare",
        "equal",
        "find_index_from_left",
        "find_index_from_right",
        // Basic
        "copy_string",
        "sprint",
        "tprint",
        "to_lower",
        "to_upper",
        "is_digit",
        "is_alpha",
        "is_alnum",
        "is_space",
        "string_to_int",
        "string_to_float",
        "to_integer",
        "array_copy",
        // Math (and Basic's min/max/clamp)
        "min",
        "max",
        "clamp",
        "abs",
        "sqrt",
        "sin",
        "cos",
        "tan",
        "floor",
        "ceil",
        "round",
        "lerp",
        "normalize",
        "unit_vector",
        "length",
        "dot",
        "cross",
    ],
};

/// The callee is one of `names` from one of the library `modules`: every procedure it named
/// is declared in such a module of an import directory.
fn calls_library(cx: &Cx, callee: &Expr, modules: &[&str], names: &[&str]) -> bool {
    let name = match &callee.kind {
        E::Ident(n) => *n,
        E::Member(_, n) => n.name,
        _ => return false,
    };
    if !names.contains(&name.as_str()) {
        return false;
    }
    let procs = callee_procs(cx, callee);
    !procs.is_empty()
        && procs.iter().all(|&p| {
            let file = cx.compiler.procs[p.0 as usize].lit.header.span.file;
            library_module(cx, &cx.compiler.sources.get(file).path)
                .is_some_and(|m| modules.contains(&m))
                || cx.compiler.modules.iter().any(|module| {
                    let Some(path) = module.path.as_ref().and_then(|path| path.to_str()) else {
                        return false;
                    };
                    if !library_module(cx, path).is_some_and(|module| modules.contains(&module)) {
                        return false;
                    }
                    cx.compiler
                        .scope(module.scope)
                        .names
                        .get(&name)
                        .is_some_and(|ids| {
                            ids.iter().any(|&id| {
                                use jaic::sema::scope::{EntityState, Resolved};
                                let entity = cx.compiler.entity(id);
                                if !entity.exported {
                                    return false;
                                }
                                match &entity.state {
                                    EntityState::Done(Resolved::Proc(candidate))
                                    | EntityState::Done(Resolved::Const {
                                        value: jaic::sema::Value::Proc(candidate),
                                        ..
                                    }) => procedure_or_instance(cx, *candidate, p),
                                    EntityState::Done(Resolved::ProcSet(candidates)) => candidates
                                        .iter()
                                        .any(|&candidate| procedure_or_instance(cx, candidate, p)),
                                    _ => false,
                                }
                            })
                        })
                })
        })
}

/// Exact declared procedure identity and its explicit generic instance tree;
/// equal source text/spans in independently inserted code do not imply identity.
fn procedure_or_instance(cx: &Cx, candidate: ProcId, actual: ProcId) -> bool {
    let mut pending = vec![candidate];
    let mut seen = std::collections::HashSet::new();
    while let Some(candidate) = pending.pop() {
        if candidate == actual {
            return true;
        }
        if seen.insert(candidate) {
            pending.extend(
                cx.compiler.procs[candidate.0 as usize]
                    .instances
                    .values()
                    .copied(),
            );
        }
    }
    false
}

/// The module a file in an import directory belongs to: `Basic` for `<dir>/Basic/Array.jai`
/// or `<dir>/Basic.jai`.
fn library_module<'p>(cx: &Cx, path: &'p str) -> Option<&'p str> {
    cx.compiler.options.import_paths.iter().find_map(|dir| {
        let rest = std::path::Path::new(path).strip_prefix(dir).ok()?;
        let first = rest.components().next()?.as_os_str().to_str()?;
        Some(first.strip_suffix(".jai").unwrap_or(first))
    })
}
