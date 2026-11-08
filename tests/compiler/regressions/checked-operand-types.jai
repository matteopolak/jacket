#import "Basic";
#import "Compiler";
#import "String";

#run {
    workspace := compiler_create_workspace("checked operands");

    set_build_options_dc(.{ do_output = false }, workspace);
    compiler_begin_intercept(workspace);
    add_build_file(tprint("%/support/checked-operands-target.jai", #filepath), workspace);
    count := 0;

    while true {
        message := compiler_wait_for_message();
        if message.kind == .TYPECHECKED {
            checked := cast(*Message_Typechecked) message;
            for group: checked.all {
                for group.subexpressions {
                    if it.kind != .BINARY_OPERATOR continue;

                    op := cast(*Code_Binary_Operator) it;
                    if cast(Operator_Type) op.operator_type != .IS_EQUAL continue;

                    location := make_location(it);
                    if !contains(location.fully_pathed_filename, "checked-operands-target.jai") continue;

                    assert(op.left.type != null && op.right.type != null);
                    assert(op.left.type == op.right.type);
                    assert(op.type != null && op.type.type == .BOOL);
                    count += 1;
                }
            }
        }

        if message.kind == .COMPLETE break;
    }

    compiler_end_intercept(workspace);
    assert(count == 6);
}

main :: () {}

quoted_checks: int;

quoted_effect :: () -> int {
    quoted_checks += 1;
    return 7;
}

#run {
    root := compiler_get_nodes(#code (#run quoted_effect()) + 1);

    assert(root.kind == .BINARY_OPERATOR);
    assert(quoted_checks == 0);
    literal := compiler_get_nodes(#code 7);

    assert(literal.type == type_info(s64));
}
