#import "Basic";

main :: () {
    info := Stack_Trace_Procedure_Info.{ name = "walk", location = .{ fully_pathed_filename = "tree.jai", line_number = 2 } };
    node := Stack_Trace_Node.{ info = *info, line_number = 3 };
    print_stack_trace(*node, prefix = "OUT ", to_standard_error = false);
    print_stack_trace(*node, prefix = "ERR ");
}
