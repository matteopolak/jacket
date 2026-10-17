// Reviewed Windows-only facade staged on host; implementation bytes are unchanged.
#import "Basic";
#import "Compiler";
C :: #import "developer-tools-check-staged";
N :: #import "developer-tools-check-names";

Original_Check_Record :: struct {
    value: s32;
}

main :: () {
    header: Code_Procedure_Header;
    header.name = "independent_jai_name";
    assert(C.verify_elsewhere_binding(*header, .MACOS, .ARM64));
    header.procedure_flags = .ELSEWHERE;
    header.foreign_function_name = header.name;
    assert(C.verify_elsewhere_binding(*header, .MACOS, .ARM64));
    header.name = "renamed_binding";

    for names, index: N.symbols {
        os := ifx index == 0 then Operating_System_Tag.MACOS else .WINDOWS;

        header.foreign_function_name = names.scalars;
        header.type = ifx index == 0 then type_info(#type (s32, u16, s64, float64, *u8) -> s64 #c_call) else type_info(#type (s32, u16, s32, float64, *u8) -> s64 #c_call);
        assert(C.verify_elsewhere_binding(*header, os, .ARM64));
        header.foreign_function_name = names.zero;
        header.type = type_info(#type () -> s32 #c_call);
        assert(C.verify_elsewhere_binding(*header, os, .ARM64));
        header.foreign_function_name = names.reference;
        header.type = type_info(#type (*s32) #c_call);
        assert(C.verify_elsewhere_binding(*header, os, .ARM64));
        header.foreign_function_name = names.callback;
        header.type = type_info(#type (#type (float64) -> s32 #c_call) #c_call);
        assert(C.verify_elsewhere_binding(*header, os, .ARM64));
        header.foreign_function_name = names.array;
        header.type = type_info(#type (*[4] s32) #c_call);
        assert(C.verify_elsewhere_binding(*header, os, .ARM64));
        header.foreign_function_name = names.record;
        header.type = type_info(#type (Original_Check_Record) -> Original_Check_Record #c_call);
        assert(C.verify_elsewhere_binding(*header, os, .ARM64));
        header.foreign_function_name = names.method;
        header.type = type_info(#type (*void, s32, float64) -> s32 #c_call);
        header.procedure_flags |= .CPP_METHOD;
        assert(C.verify_elsewhere_binding(*header, os, .ARM64));
        header.procedure_flags &= ~header.procedure_flags.CPP_METHOD;
    }

    // Ordinary Itanium function mangling does not encode the return type.
    header.foreign_function_name = N.symbols[0].zero;
    header.type = type_info(#type () -> float64 #c_call);
    assert(C.verify_elsewhere_binding(*header, .MACOS, .ARM64));
    C.finish_error_checking();
    print("ok\n");
}
