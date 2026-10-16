#import "Compiler";

same_arguments :: (a: *Code_Procedure_Call, b: *Code_Procedure_Call) -> bool {
    if !a || !b return a == b;
    if a.arguments_unsorted.count != b.arguments_unsorted.count return false;

    for arg, i: a.arguments_unsorted {
        other := b.arguments_unsorted[i];
        if (!arg.name) != (!other.name) return false;
        if arg.name && arg.name.name != other.name.name return false;

        left := cast(*Code_Literal) arg.expression;
        right := cast(*Code_Literal) other.expression;

        if left.kind != .LITERAL || right.kind != .LITERAL return false;
        if left.value_type != right.value_type || left._u64 != right._u64 return false;
    }

    return true;
}

check :: (code: Code, first: s64, second: s64) -> bool {
    original := cast(*Code_Block) compiler_get_nodes(code);
    copy := cast(*Code_Block) compiler_get_nodes(compiler_get_code(original, code));

    if copy.statements.count != 1 return false;

    node := cast(*Code_Directive_Import) copy.statements[0];
    before := cast(*Code_Directive_Import) original.statements[0];

    if node.kind != .DIRECTIVE_IMPORT return false;
    if node.name != before.name return false;
    if !same_arguments(before.module_parameters_call, node.module_parameters_call) return false;
    if !same_arguments(before.program_parameters_call, node.program_parameters_call) return false;

    if first < 0 {
        if node.module_parameters_call return false;
    } else {
        if !node.module_parameters_call return false;
        if node.module_parameters_call.arguments_unsorted.count != first return false;
    }

    if second < 0 {
        if node.program_parameters_call return false;
    } else {
        if !node.program_parameters_call return false;
        if node.program_parameters_call.arguments_unsorted.count != second return false;
    }

    return true;
}

#assert check(#code {
    #import "M";
}, -1, -1);

#assert check(#code {
    #import "M"();
}, 0, -1);

#assert check(#code {
    #import "M"()();
}, 0, 0);

#assert check(#code {
    #import "M"()(enabled = false);
}, 0, 1);

#assert check(#code {
    #import "M"(local = 9)();
}, 1, 0);

#assert check(#code {
    #import "M"(local = 9)(enabled = false);
}, 1, 1);

program_only :: (code: Code) -> bool {
    original := cast(*Code_Block) compiler_get_nodes(code);
    node := cast(*Code_Directive_Import) original.statements[0];
    node.module_parameters_call = null;
    copy := cast(*Code_Block) compiler_get_nodes(compiler_get_code(original, code));
    result := cast(*Code_Directive_Import) copy.statements[0];

    return result.module_parameters_call != null && result.module_parameters_call.arguments_unsorted.count == 0 && same_arguments(node.program_parameters_call, result.program_parameters_call);
}

#assert program_only(#code {
    #import "M"()(enabled = false);
});

main :: () {}
