//! Apply checked source procedure policy without consulting debug names or source files.
use inkwell::{
    attributes::{Attribute, AttributeLoc},
    context::Context,
    values::{CallSiteValue, FunctionValue},
};
use jai_types::InlineHint;

pub(crate) fn validate(
    library: &jai_ir::Library,
    procedure: jai_ir::ProcedureId,
    hint: InlineHint,
) -> Result<(), crate::Error> {
    let declaration = library.inline_hint(procedure);
    // Source demonstrates Never overriding Always, but not the reverse.
    if matches!((declaration, hint), (InlineHint::Never, InlineHint::Always)) {
        return Err(crate::Error::UnsupportedInlining {
            procedure,
            reason: "contradictory declaration and call-site policy requires Jai precedence evidence",
        });
    }
    if hint == InlineHint::Always
        && library.procedure_by_id(procedure).is_none()
        && !library.prototypes().iter().any(|prototype| {
            prototype.id == procedure
                && matches!(prototype.origin, jai_ir::PrototypeOrigin::Intrinsic(_))
        })
    {
        return Err(crate::Error::UnsupportedInlining {
            procedure,
            reason: "forced inline target has no available native body",
        });
    }
    Ok(())
}

pub(crate) fn apply(context: &Context, function: FunctionValue<'_>, hint: InlineHint) {
    let name = match hint {
        InlineHint::Automatic => return,
        InlineHint::Always => "alwaysinline",
        InlineHint::Never => "noinline",
    };
    function.add_attribute(
        AttributeLoc::Function,
        context.create_enum_attribute(Attribute::get_named_enum_kind_id(name), 0),
    );
}

pub(crate) fn apply_call(context: &Context, call: CallSiteValue<'_>, hint: InlineHint) {
    let name = match hint {
        InlineHint::Automatic => return,
        InlineHint::Always => "alwaysinline",
        InlineHint::Never => "noinline",
    };
    call.add_attribute(
        AttributeLoc::Function,
        context.create_enum_attribute(Attribute::get_named_enum_kind_id(name), 0),
    );
}
