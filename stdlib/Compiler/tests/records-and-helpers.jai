#import "Basic";
C :: #import "Compiler";

main :: () {
    number := C.make_integer_literal(-47);

    assert(number.kind == .LITERAL && number.value_type == .NUMBER);
    assert(number._s64 == -47 && (number.value_flags & .MINUS_SIGN) != 0);
    text := C.make_string_literal("orchid");

    assert(text.kind == .LITERAL && text.value_type == .STRING && text._string == "orchid");
    assert(type_info(C.Code_Binary_Operator).name == "Code_Binary_Operator");
    assert(C.operator_to_string(.ROTATE_LEFT) == "<<<");
    assert(C.operator_to_string(.LOGICAL_OR_ASSIGN) == "||=");
    base, path := C.get_basename_and_path("garden/flowers.jai");

    assert(base == "flowers.jai" && path == "garden/");
    a: C.Build_Options;
    b: C.Build_Options;

    a.user_data_u64 = 17;
    b.user_data_u64 = 23;
    a.context_size_max = 8192;
    C.copy_commonly_propagated_fields(a, *b);
    assert(b.context_size_max == 8192 && b.user_data_u64 == 23);
    C.set_optimization(*b, .OPTIMIZED_SMALL, preserve_debug_info = false);
    assert(b.llvm_options.bitcode_optimization_setting == .OS);
    assert(b.llvm_options.machine_code_optimization_setting == .DEFAULT);
    assert(b.llvm_options.enable_tail_calls && b.llvm_options.enable_loop_unrolling);
    assert(!b.llvm_options.enable_loop_vectorization && !b.llvm_options.enable_split_modules);
    print("ok\n");
}
