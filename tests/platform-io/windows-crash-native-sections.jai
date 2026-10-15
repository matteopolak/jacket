// Native-executable assertions: compiled code/globals must really be PE sections.
#assert OS == .WINDOWS && CPU == .X64;

#import "Basic";
C :: #import "Runtime_Support_Crash_Handler";

main :: () {
    in_code, read_only, code_name := C.is_ptr_in_program_section(cast(*void) main);

    assert(in_code && read_only && code_name.count > 0);
    in_data, writable, data_name := C.is_ptr_in_program_section(*C.windows_context);

    assert(in_data && !writable && data_name.count > 0);
    local: s64;
    in_stack, stack_read_only, stack_name := C.is_ptr_in_program_section(*local);

    assert(!in_stack && !stack_read_only && stack_name.count == 0);
    print("ok\n");
}
