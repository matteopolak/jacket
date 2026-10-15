#import "Basic";
#if OS == .WINDOWS G :: #import "developer-tools-cpp-workflow-windows";
else G :: #import "developer-tools-cpp-workflow-linux";

#run {
    procedure := cast(*Type_Info_Procedure) type_info(type_of(G.original_cpp_workflow_long));
    expected := ifx OS == .WINDOWS then 4 else 8;
    assert(procedure.argument_types.count == 1 && procedure.return_types.count == 1);
    assert(procedure.argument_types[0].runtime_size == expected && procedure.return_types[0].runtime_size == expected);
}

main :: () {}
