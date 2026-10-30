//! Warnings for forms the checker accepts and the official compiler rejects, found while
//! checking rather than while parsing (the parser's are in `lenient.rs`).
//!
//! Each builds its warning through `leniency_warning`, so it is silent outside the program's own
//! files; the callers sit where jaic takes the lenient path.

use super::{Compiler, ScopeId, Span, TypeId};
use crate::leniency::Leniency;

impl Compiler {
    /// `#caller_location` somewhere other than the default value of a parameter.
    pub(in crate::sema) fn warn_caller_location_outside_default(
        &mut self,
        scope: ScopeId,
        span: Span,
    ) {
        let Some(d) = self.leniency_warning(
            self.scope(scope).module,
            span,
            Leniency::CallerLocationOutsideDefault,
            "`#caller_location` is only allowed as the default value of a parameter",
        ) else {
            return;
        };
        self.warn(
            d.with_label("this is the location of the line itself")
                .with_help(
                    "write `#location()` for the place of this line, or take `loc := #caller_location` as a parameter and pass `loc` on",
                ),
        );
    }

    /// `..xs` next to other arguments for the same variadic parameter.
    pub(in crate::sema) fn warn_spread_with_other_arguments(&mut self, scope: ScopeId, span: Span) {
        let Some(d) = self.leniency_warning(
            self.scope(scope).module,
            span,
            Leniency::SpreadWithOtherArguments,
            "`..` spreads into a variadic parameter by itself",
        ) else {
            return;
        };
        self.warn(
            d.with_label("other arguments also fill this parameter")
                .with_help(
                    "pass the spread alone, or collect all the values into one array first and spread that",
                ),
        );
    }

    /// A procedure cast straight to an integer type.
    pub(in crate::sema) fn warn_procedure_cast_to_integer(
        &mut self,
        scope: ScopeId,
        span: Span,
        target: TypeId,
    ) {
        let name = self.ty.types.name(target);
        let Some(d) = self.leniency_warning(
            self.scope(scope).module,
            span,
            Leniency::ProcedureCastToInteger,
            format!("a procedure cannot be cast to `{name}` directly"),
        ) else {
            return;
        };
        self.warn(
            d.with_label("this casts a procedure to a number")
                .with_help(format!(
                    "cast it to a pointer first: `cast({name}) cast(*void) procedure`"
                )),
        );
    }

    /// `*x` of a macro parameter that the call binds to a number or bool constant.
    pub(in crate::sema) fn warn_address_of_macro_constant(
        &mut self,
        scope: ScopeId,
        span: Span,
        name: &str,
    ) {
        let Some(d) = self.leniency_warning(
            self.scope(scope).module,
            span,
            Leniency::AddressOfMacroConstant,
            format!("`{name}` is a constant here, and a constant has no address"),
        ) else {
            return;
        };
        self.warn(
            d.with_label("a call gives this macro parameter a constant")
                .with_help(format!(
                    "copy it into a variable first: `copy := {name};`, then take `*copy`"
                )),
        );
    }

    /// An untyped `.{..}` literal where a dynamic array is expected.
    pub(in crate::sema) fn warn_struct_literal_as_dynamic_array(
        &mut self,
        scope: ScopeId,
        span: Span,
        ty: TypeId,
    ) {
        let name = self.ty.types.name(ty);
        let Some(d) = self.leniency_warning(
            self.scope(scope).module,
            span,
            Leniency::ArrayFromStructLiteral,
            "an untyped `.{..}` literal is not a dynamic array",
        ) else {
            return;
        };
        self.warn(
            d.with_label(format!("a literal of {name} is wanted here"))
                .with_help(format!("name the type: `{name}.{{ .. }}`")),
        );
    }
}
