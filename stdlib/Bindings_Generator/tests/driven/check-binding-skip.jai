// Unsupported targets skip signature parsing and log once per module instance.
#import "Basic";
#import "Compiler";
C :: #import "developer-tools-check-staged";

main :: () {
    header: Code_Procedure_Header;
    header.name = "original_jai_name";
    header.foreign_function_name = "deliberately not mangled";
    header.procedure_flags = .ELSEWHERE;
    assert(C.verify_elsewhere_binding(*header, .WASM, .X64));
    assert(C.verify_elsewhere_binding(*header, .WASM, .X64));
    assert(C.verify_elsewhere_binding(*header, .PS5, .ARM64));
    print("ok\n");
}
