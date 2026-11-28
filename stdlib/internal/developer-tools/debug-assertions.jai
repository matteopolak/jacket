// Returns true when a debugger is attached, attaching one first if none is. Only Windows can
// attach; elsewhere it returns false when no debugger is running.
attach_to_debugger :: () -> bool {
    if is_debugger_present() return true;
    return os_attach_to_debugger();
}

// Asks the system to attach a debugger to this process. Returns false when it can't, which is
// always on systems other than Windows.
os_attach_to_debugger :: () -> bool {
    #if OS == .WINDOWS return attach_native(USE_GRAPHICS) != 0;
    else return false;
}

// An assert handler that prints the failure and then attaches a debugger. [init] installs it
// when `USE_GRAPHICS` is false.
break_assert :: (loc: Source_Code_Location, message: string) -> bool {
    print_assert(loc, message);
    return attach_to_debugger();
}

#if USE_GRAPHICS {
    // An assert handler that prints the failure and asks in a dialog what to do: Retry attaches a
    // debugger, Ignore skips this assert from now on, Abort exits with `Exit_Code.Assert`. With
    // [set_report_mode] off it only prints. [init] installs it when `USE_GRAPHICS` is true.
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
        // Shows no dialog and returns `.Abort`.
        os_show_assert_dialog :: (loc: Source_Code_Location, message: string) -> Assert_Response {
            return .Abort;
        }
    }

    #if OS == .WINDOWS {
        // Shows a dialog with the failure and returns the button the user pressed.
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
    native :: #library "debug-native/developer_tools_debug"; @native(developer)
    attach_native :: (graphics: bool) -> s32 #foreign native "developer_debug_attach";
    dialog_native :: (text: *u8, count: s64) -> s32 #foreign native "developer_debug_dialog";
}
