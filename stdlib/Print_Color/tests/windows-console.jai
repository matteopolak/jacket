#import "Basic";
#import "File";

// Native console colors against the real Windows console API. This needs a Windows host: the
// first half redirects stdout to a file (not a console) and the second half attaches a real
// console screen buffer and reads its attributes back.
#if OS == .WINDOWS {
    W :: #import "Windows";
    Keep :: #import "Print_Color"(SAVE_RESTORE_ON_WINDOWS = false);
    Save :: #import "Print_Color"(SAVE_RESTORE_ON_WINDOWS = true);

    host :: #library,system "kernel32";
    SetStdHandle :: (which: s32, handle: W.HANDLE) -> W.BOOL #foreign host;

    open_wide :: (name: string, access: u32, share: u32, creation: u32) -> W.HANDLE {
        wide: [64] u16;
        for 0..name.count - 1 wide[it] = name[it];
        return W.CreateFileW(wide.data, access, share, null, creation);
    }

    attributes_of :: (handle: W.HANDLE) -> u16 {
        info: W.CONSOLE_SCREEN_BUFFER_INFO;
        assert(W.GetConsoleScreenBufferInfo(handle, *info) != 0, "not a console screen buffer");
        return info.wAttributes;
    }

    early_return :: (handle: W.HANDLE) -> u16 {
        Keep.with_console_color(.GREEN, #code {
            `return attributes_of(handle);
        });
        return 0;
    }

    main :: () {
        original := W.GetStdHandle(W.STD_OUTPUT_HANDLE);

        // Redirected: no escape codes, no attribute calls, pushes stay balanced.
        redirected := open_wide("print-color-redirect.tmp", 0x40000000, 1, 2);
        assert(redirected != W.INVALID_HANDLE_VALUE, "could not create the redirect target");
        SetStdHandle(W.STD_OUTPUT_HANDLE, redirected);
        Keep.with_console_color(.RED, #code { write_string("plain text"); }, .BOLD);
        assert(Keep.console_color_depth() == 0);
        SetStdHandle(W.STD_OUTPUT_HANDLE, original);
        W.CloseHandle(redirected);

        written, found := read_entire_file("print-color-redirect.tmp");
        assert(found && written == "plain text", "redirected output must be untouched: %", written);
        file_delete("print-color-redirect.tmp");

        // A real console screen buffer.
        W.AllocConsole();
        console := open_wide("CONOUT$", 0xc0000000, 3, 3);
        assert(console != W.INVALID_HANDLE_VALUE, "no console screen buffer is available");
        SetStdHandle(W.STD_OUTPUT_HANDLE, console);

        W.SetConsoleTextAttribute(console, 0x07);
        Keep.push_console_color(.RED);
        assert(attributes_of(console) == 0x04, "red: %", attributes_of(console));
        Keep.push_console_color(.BG_BLUE);
        assert(attributes_of(console) == 0x14, "nested: %", attributes_of(console));
        Keep.pop_console_color();
        assert(attributes_of(console) == 0x04, "inner pop restores the enclosing block");
        Keep.pop_console_color();
        assert(attributes_of(console) == 0x07, "outer pop restores the default");

        assert(early_return(console) == 0x02, "inside the block");
        assert(attributes_of(console) == 0x07, "early return restores the console");

        // The two outermost-pop policies differ once the program changed the console itself.
        W.SetConsoleTextAttribute(console, 0x07);
        Save.push_console_color(.RED);
        Save.pop_console_color();
        W.SetConsoleTextAttribute(console, 0x1e);
        Save.push_console_color(.GREEN);
        Save.pop_console_color();
        assert(attributes_of(console) == 0x1e, "save/restore: %", attributes_of(console));

        W.SetConsoleTextAttribute(console, 0x07);
        Keep.push_console_color(.RED);
        Keep.pop_console_color();
        W.SetConsoleTextAttribute(console, 0x1e);
        Keep.push_console_color(.GREEN);
        Keep.pop_console_color();
        assert(attributes_of(console) == 0x07, "default policy: %", attributes_of(console));

        W.SetConsoleTextAttribute(console, 0x07);
        SetStdHandle(W.STD_OUTPUT_HANDLE, original);
        print("ok\n");
    }
} else {
    main :: () {
        print("ok\n");
    }
}
