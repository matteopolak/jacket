#import "Compiler";

// Behavior of the small Compiler-module helpers: operator spellings, #as subclass checks,
// path splitting, null-safe locations and the set_optimization flavors.

Base :: struct {
    x: int;
}

Derived :: struct {
    #as using base: Base;
    y: int;
}

Via_Ptr :: struct {
    #as using inner: *Derived;
}

Not_As :: struct {
    z: int;
    #as using base: Base;
}

main :: () -> int {
    expected := string.["==", "!=", "&&", "||", "<=", ">=", "<<", ">>", "<<<", ">>>"];
    for expected if operator_to_string(cast(Operator_Type) (131 + it_index)) != it return 1;
    if operator_to_string(.PLUS_ASSIGN) != "+=" || operator_to_string(.ROTATE_RIGHT_ASSIGN) != ">>>=" || operator_to_string(.LOGICAL_OR_ASSIGN) != "||=" return 2;
    if operator_to_string(.POINTER_DEREFERENCE) != "(.*)" || operator_to_string(.POSTFIX_DEREFERENCE) != ".*" || operator_to_string(.ARRAY_SUBSCRIPT) != "[]" return 3;
    if operator_to_string(cast(Operator_Type) #char "+") != "" || operator_to_string(cast(Operator_Type) 141) != "" || operator_to_string(cast(Operator_Type) 159) != "" return 4;

    if !is_subclass_of(type_info(Derived), "Base") return 5;
    if is_subclass_of(type_info(Base), "Base") return 6;
    if !is_subclass_of(type_info(Base), "Base", false) return 7;
    if !is_subclass_of(type_info(Via_Ptr), "Base") || !is_subclass_of(type_info(Via_Ptr), "Derived") return 8;
    if is_subclass_of(type_info(Not_As), "Base") || is_subclass_of(type_info(int), "Base") return 9;

    name, dir := get_basename_and_path("a/b/c/d.foo.txt");
    if name != "d.foo" || dir != "a/b/c/" return 10;
    name, dir = get_basename_and_path("x\\y.z");
    if name != "y" || dir != "x\\" return 11;
    name, dir = get_basename_and_path("plain");
    if name != "plain" || dir != "" return 12;
    name, dir = get_basename_and_path("dir/.hidden");
    if name != "" || dir != "dir/" return 13;

    loc := make_location(null);
    if loc.fully_pathed_filename != "" || loc.line_number != 0 return 14;

    o: Build_Options;
    set_optimization(*o, .VERY_OPTIMIZED);
    if o.llvm_options.bitcode_optimization_setting != .O3 || o.llvm_options.machine_code_optimization_setting != .AGGRESSIVE return 15;
    if !o.llvm_options.enable_slp_vectorization || !o.llvm_options.enable_tail_calls || o.llvm_options.disable_inlining return 16;
    if o.array_bounds_check != .OFF || o.cast_bounds_check != .OFF || !o.enable_frame_pointers return 17;
    set_optimization(*o, .OPTIMIZED_SMALL);
    if o.llvm_options.bitcode_optimization_setting != .OS || o.llvm_options.enable_loop_vectorization return 18;
    set_optimization(*o, .DEBUG, preserve_debug_info = false);
    if o.llvm_options.bitcode_optimization_setting != .O0 || o.llvm_options.machine_code_optimization_setting != .NONE return 19;
    if !o.llvm_options.disable_inlining || o.llvm_options.enable_tail_calls || o.cast_bounds_check != .FATAL || o.emit_debug_info != .NONE return 20;
    if !o.x64_options.enable_unix_runtime_frame_information return 21;
    return 0;
}
