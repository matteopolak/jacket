// DEBUG_BINDINGS=true is explicitly staged test configuration, not a public default.
#import "Basic";
#import "Compiler";
C :: #import "developer-tools-check-debug-staged";
N :: #import "developer-tools-check-names";

main :: () {
    header: Code_Procedure_Header;
    header.name = "renamed_binding";
    header.procedure_flags = .ELSEWHERE;
    header.foreign_function_name = N.symbols[0].scalars;
    header.type = type_info(#type (s32, u16, s64, float64, *u8) -> s64 #c_call);
    assert(C.verify_elsewhere_binding(*header, .MACOS, .ARM64));
    assert(C.verify_elsewhere_binding(*header, .MACOS, .ARM64));
    C.print_binding_check_memory_stats();
    C.finish_error_checking();
    print("ok\n");
}
