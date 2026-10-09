// rules: compiler.26 compiler.27
// The Compiler procedures that used to stop with "not supported": get_root_type,
// compiler_get_base_path, the no-op reporters and breakpoints, and add_data_segment with
// add_global_data.
#import "Basic";
#import "Compiler";
#import "String";

root_type_of :: (code: Code) -> Get_Root_Type_Status, Type {
    return get_root_type(code);
}

CODE_STATUS :: #run -> Get_Root_Type_Status {
    status := root_type_of(#code 1.5 + 2.5);
    return status;
}

CODE_SIZE :: #run -> s64 {
    status, type := root_type_of(#code cast(u8) 3);
    if status != .SUCCESS return -1;
    return (cast(*Type_Info) type).runtime_size;
}

NULL_STATUS :: #run -> Get_Root_Type_Status {
    code := #code,null;
    status := get_root_type(code);
    return status;
}

TABLE :: #run -> [] u8 {
    segment, created := add_data_segment("probe_section");
    assert(segment != null && !created);
    bytes: [4] u8 = .[1, 2, 3, 4];
    return add_global_data(bytes, .USER_SEGMENT, segment);
}

checks :: () {
    assert(NULL_STATUS == .INPUT_IS_CODE_NULL);
    // Typed nodes exist for code from a compile-time run that has done nothing yet.
    assert(CODE_STATUS == .SUCCESS && CODE_SIZE == 1);

    base := compiler_get_base_path();
    assert(base.count > 0 && (ends_with(base, "/") || ends_with(base, "\\")), "base path: %", base);

    // Accepted and ignored.
    compiler_report_errors_for_unresolved_identifiers("probe.jai");
    compiler_report_errors_for_untyped_declarations_with_these_notes(get_current_workspace(), "a", "b");
    compiler_set_memory_breakpoint(null);

    assert(TABLE.count == 4 && TABLE[0] == 1 && TABLE[3] == 4);
    print("ok\n");
}

// Compile-time only: the Compiler module's procedures need the compiler.
#run checks();

main :: () {}
