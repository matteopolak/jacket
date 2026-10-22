#if CPU == .ARM64 {
    // Reuse the SDK-shaped Windows binding so register layouts have one owner.
    CONTEXT :: struct #align 16 {
        using members: Win.CONTEXT;
    }
} else {
    CONTEXT :: struct {
        using members: struct {
            before_rax: [152] u8 = ---;
            Rsp: u64;
            Rbp: u64;
            other: [10] u64;
            Rip: u64;
        }
        padding: [1232 - size_of(type_of(members))] u8 = ---;
    }

}

EXCEPTION_RECORD :: struct {
    ExceptionCode: u32;
    ExceptionFlags: u32;
    ExceptionRecord: *EXCEPTION_RECORD;
    ExceptionAddress: *void;
    NumberParameters: u32;
    ExceptionInformation: [15] u64;
}

EXCEPTION_POINTERS :: struct {
    ExceptionRecord: *EXCEPTION_RECORD;
    ContextRecord: *CONTEXT;
}

ADDRESS64 :: struct {
    Offset: u64;
    Segment: u16;
    Mode: s32;
}

STACKFRAME64 :: struct {
    using members: struct {
        AddrPC: ADDRESS64;
        AddrReturn: ADDRESS64;
        AddrFrame: ADDRESS64;
        AddrStack: ADDRESS64;
    }
    padding: [264 - size_of(type_of(members))] u8;
}

windows_context: CONTEXT #align 32;
symbols_initialized := false;
CurrentProcess := cast(*void) -1;

#if CPU == .ARM64 #assert size_of(CONTEXT) == 912;
else #assert size_of(CONTEXT) == 1232;
#assert size_of(STACKFRAME64) == 264;
#assert size_of(EXCEPTION_RECORD) == 152;
#assert size_of(EXCEPTION_POINTERS) == 16;

#scope_file
#if CPU == .ARM64 Win :: #import "Windows";
