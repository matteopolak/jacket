use crate::{
    Context,
    optimization::{BitcodeOptimization, Optimization},
    target::{NativeTarget, TargetOptions},
};
use inkwell::attributes::{Attribute, AttributeLoc};
use jai_llvm::{DebugEmission, DebugPrimitive, DebugSession, DebugSource, DebugVariableKind};
use std::num::NonZeroU32;

#[test]
fn line_zero_caller_location_is_valid_and_does_not_import_inlined_callee_lines() {
    for level in [BitcodeOptimization::O0, BitcodeOptimization::O2] {
        let context = Context::create();
        let target = NativeTarget::select(&TargetOptions {
            optimization: Optimization {
                bitcode: level,
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap();
        let module = context.create_module("own.artificial.call");
        let debug = DebugSession::new(
            &module,
            &context,
            "caller.jai",
            "/own-source",
            "jai-rs",
            level != BitcodeOptimization::O0,
            DebugEmission::Variables,
        )
        .unwrap();
        let builder = context.create_builder();
        let function_type = context
            .i64_type()
            .fn_type(&[context.i64_type().into()], false);
        let callee = module.add_function("step", function_type, None);
        builder.position_at_end(context.append_basic_block(callee, "entry"));
        let source = |file, line| DebugSource {
            file,
            directory: "/own-source",
            line: NonZeroU32::new(line).unwrap(),
            column: NonZeroU32::new(1).unwrap(),
        };
        let callee_scope = debug
            .function_scope(&builder, callee, source("callee.jai", 2), "step")
            .unwrap();
        let argument_storage = builder
            .build_alloca(context.i64_type(), "argument")
            .unwrap();
        builder
            .build_store(argument_storage, callee.get_nth_param(0).unwrap())
            .unwrap();
        let argument = debug
            .variable(
                callee_scope,
                source("callee.jai", 2),
                "argument",
                DebugVariableKind::Parameter(NonZeroU32::new(1).unwrap()),
                DebugPrimitive::Signed64,
                8,
            )
            .unwrap();
        debug.declare(&builder, argument_storage, argument).unwrap();
        let result = builder
            .build_int_add(
                callee.get_nth_param(0).unwrap().into_int_value(),
                context.i64_type().const_int(1, false),
                "result",
            )
            .unwrap();
        builder.build_return(Some(&result)).unwrap();
        let caller = module.add_function("visible_run", function_type, None);
        builder.position_at_end(context.append_basic_block(caller, "entry"));
        let scope = debug
            .function_scope(&builder, caller, source("caller.jai", 3), "visible_run")
            .unwrap();
        builder.unset_current_debug_location();
        debug.set_artificial_location(&builder, scope).unwrap();
        let call = builder
            .build_call(
                callee,
                &[caller.get_nth_param(0).unwrap().into()],
                "silent.call",
            )
            .unwrap();
        call.add_attribute(
            AttributeLoc::Function,
            context.create_enum_attribute(Attribute::get_named_enum_kind_id("alwaysinline"), 0),
        );
        debug
            .set_location(&builder, scope, source("caller.jai", 4))
            .unwrap();
        builder
            .build_return(Some(&call.try_as_basic_value().basic().unwrap()))
            .unwrap();
        let ordinary = module.add_function("ordinary", function_type, None);
        builder.position_at_end(context.append_basic_block(ordinary, "entry"));
        debug
            .function_scope(&builder, ordinary, source("caller.jai", 10), "ordinary")
            .unwrap();
        let call = builder
            .build_call(
                callee,
                &[ordinary.get_nth_param(0).unwrap().into()],
                "ordinary.call",
            )
            .unwrap();
        call.add_attribute(
            AttributeLoc::Function,
            context.create_enum_attribute(Attribute::get_named_enum_kind_id("alwaysinline"), 0),
        );
        builder
            .build_return(Some(&call.try_as_basic_value().basic().unwrap()))
            .unwrap();
        debug.finish();
        module.verify().unwrap();
        target.prepare(&module).unwrap();
        let ir = module.print_to_string().to_string();
        let caller = ir
            .lines()
            .skip_while(|line| !line.starts_with("define ") || !line.contains("@visible_run("))
            .take_while(|line| *line != "}")
            .collect::<Vec<_>>();
        assert!(
            !caller.iter().any(|line| line.contains("call i64 @step")),
            "{ir}"
        );
        assert!(
            !caller.iter().any(|line| line.contains("#dbg_")),
            "suppressed argument survived: {ir}"
        );
        for instruction in caller {
            if let Some((_, id)) = instruction.split_once("!dbg !") {
                let id: String = id.chars().take_while(char::is_ascii_digit).collect();
                let location = ir
                    .lines()
                    .find(|line| line.starts_with(&format!("!{id} = ")))
                    .unwrap();
                assert!(
                    !location.contains("inlinedAt:"),
                    "suppressed call imported callee location: {ir}"
                );
            }
        }
        let ordinary: Vec<_> = ir
            .lines()
            .skip_while(|line| !line.starts_with("define ") || !line.contains("@ordinary("))
            .take_while(|line| *line != "}")
            .collect();
        assert!(
            ordinary.iter().any(|line| line.contains("#dbg_")),
            "ordinary inline argument was removed: {ir}"
        );
    }
}
