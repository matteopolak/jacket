// Reviewed continuable diagnostics and false results; actual SDK-produced names.
#import "Basic";
#import "Compiler";
C :: #import "developer-tools-check-staged";
N :: #import "developer-tools-check-names";

#run {
    header: Code_Procedure_Header;
    header.name = "original_jai_declaration";
    header.procedure_flags = .ELSEWHERE;

    for names, index: N.symbols {
        os := ifx index == 0 then Operating_System_Tag.MACOS else .WINDOWS;

        header.foreign_function_name = names.scalars;
        header.type = type_info(#type () #c_call);
        assert(!C.verify_elsewhere_binding(*header, os, .ARM64));
        header.type = type_info(#type (s64, u16, s64, float64, *u8) -> s64 #c_call);
        assert(!C.verify_elsewhere_binding(*header, os, .ARM64));
        header.foreign_function_name = names.reference;
        header.type = type_info(#type (*u32) #c_call);
        assert(!C.verify_elsewhere_binding(*header, os, .ARM64));
        header.foreign_function_name = names.array;
        header.type = type_info(#type (*[5] s32) #c_call);
        assert(!C.verify_elsewhere_binding(*header, os, .ARM64));
        header.foreign_function_name = "invalid_foreign_name";
        assert(!C.verify_elsewhere_binding(*header, os, .ARM64));
    }

    header.foreign_function_name = N.symbols[1].zero;
    header.type = type_info(#type () -> float64 #c_call);
    assert(!C.verify_elsewhere_binding(*header, .WINDOWS, .X64));
    header.procedure_flags |= .CPP_METHOD;
    assert(!C.verify_elsewhere_binding(*header, .WINDOWS, .X64));
}
