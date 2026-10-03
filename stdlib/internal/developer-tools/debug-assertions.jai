attach_to_debugger :: () -> bool {
    if is_debugger_present() return true;
    return os_attach_to_debugger();
}

os_attach_to_debugger :: () -> bool {
    #if OS == .WINDOWS return attach_native(USE_GRAPHICS) != 0;
    else return false;
}

break_assert :: (loc: Source_Code_Location, message: string) -> bool {
    print_assert(loc, message);
    return attach_to_debugger();
}

#if USE_GRAPHICS {
    interactive_assert :: (loc: Source_Code_Location, message: string) -> bool {
        print_assert(loc, message);
        if !State.interactive_reporting return true;
        if Tables.table_contains(*ignored_asserts, loc) return false;

        response := os_show_assert_dialog(loc, message);
        if response == .Ignore {
            // Preserve filename identity independently of the caller's storage.
            State.remember_ignored_assert(loc);

            return false;
        }

        if response == .Debug return attach_to_debugger();

        abort(.Assert);

        return false;
    }

    #if OS == .MACOS {
        os_show_assert_dialog :: (loc: Source_Code_Location, message: string) -> Assert_Response {
            return .Abort;
        }
    }

    #if OS == .WINDOWS {
        os_show_assert_dialog :: (loc: Source_Code_Location, message: string) -> Assert_Response {
            text := tprint("%:%:%: assertion failed: %", loc.fully_pathed_filename, loc.line_number, loc.character_number, message);
            return cast(Assert_Response) dialog_native(text.data, text.count);
        }
    }

    #if OS == .LINUX #load "debug-dialog-linux.jai";
}

#scope_file
#import "Basic";
State :: #import "internal/debug-state";
Tables :: #import "Hash_Table";

#if OS == .WINDOWS {
    native :: #foreign_library "debug-native/developer_tools_debug";
    attach_native :: (graphics: bool) -> s32 #foreign native "developer_debug_attach";
    dialog_native :: (text: *u8, count: s64) -> s32 #foreign native "developer_debug_dialog";
}
