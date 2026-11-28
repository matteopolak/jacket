#if OS == .WINDOWS {
    // Writes a crash dump for the exception and returns its file name, allocated with
    // `context.allocator`, or an empty string when it fails.
    //
    // - large_dump: include the whole process memory, not only the stacks.
    write_minidump :: (pExceptionInfo: *Windows.EXCEPTION_POINTERS, large_dump := false) -> string {
        result := dump_native(pExceptionInfo, large_dump);
        if !result return "";
        defer free_dump_native(result);
        return copy_string(<<result);
    }

    // [write_minidump] for an untyped exception pointer.
    write_minidump :: (pExceptionInfo: *void, large_dump := false) -> string {
        return write_minidump(cast(*Windows.EXCEPTION_POINTERS) pExceptionInfo, large_dump);
    }

    // Sets what the crash handler writes when the program crashes: no dump, a small one or a full one.
    set_minidump_state :: (state: Minidump_State) {
        assert(set_dump_native(cast(s32) state) != 0, "invalid minidump state");
    }

    // Returns the name of the last dump written, or an empty string. The string belongs to the
    // library; don't free it.
    get_written_minidump_filename :: () -> string {
        result := latest_dump_native();
        return <<result;
    }
} else {
    // Writes nothing and returns an empty string: dumps are Windows only.
    write_minidump :: (pExceptionInfo: *void, large_dump := false) -> string {
        return "";
    }

    // Does nothing: dumps are Windows only.
    set_minidump_state :: (state: Minidump_State) {
        // Automatic minidump state is a Windows-only facility.
    }

    // Returns an empty string: dumps are Windows only.
    get_written_minidump_filename :: () -> string {
        return "";
    }
}

#scope_file
#import "Basic";

#if OS == .WINDOWS {
    Windows :: #import "internal/bindings/windows-api";

    native :: #library "debug-native/developer_tools_debug"; @native(developer)
    dump_native :: (exception: *void, full: bool) -> *string #foreign native "developer_debug_dump";
    free_dump_native :: (value: *string) #foreign native "developer_debug_free_symbols"; @FreesArgument
    set_dump_native :: (state: s32) -> s32 #foreign native "developer_debug_set_dump_state";
    latest_dump_native :: () -> *string #foreign native "developer_debug_latest_dump";
}
