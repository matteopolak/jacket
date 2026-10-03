init :: () {
    error := native_prepare(CurrentProcess);
    assert(error == 0, "Windows exception-handler initialization failed: %", error);
    symbols_initialized = true;
    native_install(cast(*void) handle_exception);
}

backtrace :: (ctx: *CONTEXT, skips: s32) #no_context {
    native_backtrace(ctx, skips, CurrentProcess);
}

is_ptr_in_program_section :: (ptr: *void) -> in_program_section: bool, read_only: bool, section_name: string #no_context {
    readonly: s32;
    name: *u8;
    count: u64;
    found := native_section(ptr, *readonly, *name, *count);

    return found != 0, readonly != 0, string.{ cast(s64) count, name };
}

handle_exception :: (using pExceptionInfo: *EXCEPTION_POINTERS) -> s32 #c_call {
    return native_handle(pExceptionInfo, *windows_context, CurrentProcess);
}

#scope_file
#import "Basic";

native_runtime :: #library "crash-native/platform_io_windows_crash";
native_prepare :: (process: *void) -> u32 #foreign native_runtime "platform_io_windows_crash_prepare";
native_install :: (handler: *void) #foreign native_runtime "platform_io_windows_crash_install";
native_backtrace :: (ctx: *void, skips: s32, process: *void) #foreign native_runtime "platform_io_windows_crash_backtrace";
native_handle :: (info: *void, snapshot: *void, process: *void) -> s32 #foreign native_runtime "platform_io_windows_crash_handle";
native_section :: (pointer: *void, readonly: *s32, name: **u8, count: *u64) -> s32 #foreign native_runtime "platform_io_windows_crash_section";
