#import "Basic";
#if OS == .WINDOWS && CPU == .X64 C :: #import "Runtime_Support_Crash_Handler";

main :: () {
    #if OS == .WINDOWS && CPU == .X64 {
        assert(type_info(C.CONTEXT).name == "CONTEXT" && type_info(C.STACKFRAME64).name == "STACKFRAME64");
        assert(size_of(C.CONTEXT) == 1232 && size_of(C.STACKFRAME64) == 264);
        assert(size_of(C.ADDRESS64) == 16 && size_of(C.EXCEPTION_RECORD) == 152);
        previous_filter := set_filter(null);

        defer set_filter(previous_filter);
        C.init();
        C.init();
        assert(C.symbols_initialized && C.CurrentProcess == cast(*void) -1);
        installed := set_filter(null);

        assert(installed == cast(*void) C.handle_exception);
        assert(set_filter(installed) == null);
        C.init();
        assert(set_filter(installed) == installed);
        // Ordinary interpreted procedures/globals are IDs/heap allocations;
        // native builds place these symbols in actual executable PE sections.
        check_symbol_mapping(cast(*void) main);
        check_symbol_mapping(*C.windows_context);
        check_symbol_mapping(cast(*void) C.handle_exception);

        for writable: 0..1 {
            probe := sdk_image_probe(cast(s32) writable);

            assert(probe != null, "SDK fixture found no requested executable section");
            expected_readonly: s32;
            expected_name: *u8;
            expected_count: u64;

            assert(sdk_section(probe, *expected_readonly, *expected_name, *expected_count) == 1);
            found, readonly, name := C.is_ptr_in_program_section(probe);

            assert(found && readonly == (writable == 0) && readonly == (expected_readonly != 0));
            assert(name == string.{ cast(s64) expected_count, expected_name } && name.count > 0);
        }

        local: s64;

        check_symbol_mapping(*local);
        in_stack, stack_read_only, stack_name := C.is_ptr_in_program_section(*local);

        assert(!in_stack && !stack_read_only && stack_name.count == 0);
        null_found, null_readonly, null_name := C.is_ptr_in_program_section(null);

        assert(!null_found && !null_readonly && null_name.count == 0);
    }

    print("ok\n");
}

#scope_file
#if OS == .WINDOWS && CPU == .X64 {
    check_symbol_mapping :: (pointer: *void) {
        expected_readonly: s32;
        expected := sdk_main_image_mapping(pointer, *expected_readonly);

        assert(expected >= 0, "native SDK memory query failed");
        found, readonly, name := C.is_ptr_in_program_section(pointer);

        assert(found == (expected == 1));
        assert(readonly == (expected_readonly != 0));
        assert((name.count > 0) == found);
    }

    sdk :: #library "../../target/platform-io-tests/platform_io_section_sdk";
    kernel32 :: #system_library "kernel32";
    set_filter :: (callback: *void) -> *void #foreign kernel32 "SetUnhandledExceptionFilter";
    sdk_image_probe :: (writable: s32) -> *void #foreign sdk "platform_io_sdk_image_probe";
    sdk_main_image_mapping :: (pointer: *void, readonly: *s32) -> s32 #foreign sdk "platform_io_sdk_main_image_mapping";
    sdk_section :: (pointer: *void, readonly: *s32, name: **u8, count: *u64) -> s32 #foreign sdk "platform_io_sdk_section";
}
