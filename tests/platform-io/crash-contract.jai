#import "Basic";
#if OS == .WINDOWS && CPU == .X64 C :: #import "Runtime_Support_Crash_Handler";

main :: () {
    #if OS == .WINDOWS && CPU == .X64 {
        assert(type_info(C.CONTEXT).name == "CONTEXT" && type_info(C.STACKFRAME64).name == "STACKFRAME64");
        assert(size_of(C.CONTEXT) == 1232 && size_of(C.STACKFRAME64) == 264);
        assert(size_of(C.ADDRESS64) == 16 && size_of(C.EXCEPTION_RECORD) == 152);
        C.init();
        C.init();
        assert(C.symbols_initialized && C.CurrentProcess == cast(*void) -1);
        in_code, read_only, code_name := C.is_ptr_in_program_section(cast(*void) main);

        assert(in_code && read_only && code_name.count > 0);
        in_data, writable, data_name := C.is_ptr_in_program_section(*C.windows_context);

        assert(in_data && !writable && data_name.count > 0);
        local: s64;
        in_stack, stack_read_only, stack_name := C.is_ptr_in_program_section(*local);

        assert(!in_stack && !stack_read_only && stack_name.count == 0);
    }

    print("ok\n");
}
