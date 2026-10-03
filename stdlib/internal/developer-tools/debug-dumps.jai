#if OS == .WINDOWS {
    write_minidump :: (pExceptionInfo: *Windows.EXCEPTION_POINTERS, large_dump := false) -> string {
        result := dump_native(pExceptionInfo, large_dump);
        if !result return "";
        defer free_dump_native(result);
        return copy_string(<<result);
    }

    write_minidump :: (pExceptionInfo: *void, large_dump := false) -> string {
        return write_minidump(cast(*Windows.EXCEPTION_POINTERS) pExceptionInfo, large_dump);
    }

    set_minidump_state :: (state: Minidump_State) {
        assert(set_dump_native(cast(s32) state) != 0, "invalid minidump state");
    }

    get_written_minidump_filename :: () -> string {
        result := latest_dump_native();
        return <<result;
    }
} else {
    write_minidump :: (pExceptionInfo: *void, large_dump := false) -> string {
        return "";
    }

    set_minidump_state :: (state: Minidump_State) {
        // Automatic minidump state is a Windows-only facility.
    }

    get_written_minidump_filename :: () -> string {
        return "";
    }
}

#scope_file
#import "Basic";

#if OS == .WINDOWS {
    Windows :: #import "Windows";

    native :: #foreign_library "debug-native/developer_tools_debug";
    dump_native :: (exception: *void, full: bool) -> *string #foreign native "developer_debug_dump";
    free_dump_native :: (value: *string) #foreign native "developer_debug_free_symbols";
    set_dump_native :: (state: s32) -> s32 #foreign native "developer_debug_set_dump_state";
    latest_dump_native :: () -> *string #foreign native "developer_debug_latest_dump";
}
