// Window_Creation against the real window system: a window goes fullscreen and back with
// toggle_fullscreen, reporting its client size each time, and (on macOS) presenting its view with
// swap_view_buffers is harmless before any drawing. On Linux the request goes to the window
// manager, which may not exist (CI runs Xvfb bare), so only the request and the measured size are
// checked there. The wasm targets have no window system.
#import "Basic";

#if OS == .WASM {
    main :: () {
        print("ok\n");
    }
} else {
    #import "Window_Creation";
    #import "Input";

    pump :: (frames: int) {
        for 1..frames {
            update_window_events();
            sleep_milliseconds(5);
        }
    }

    main :: () {
        window := create_window(320, 200, "jaic fullscreen test");
        assert(cast(u64) window != 0, "create_window failed");
        pump(20);
        _, _, width, height, measured := get_dimensions(window, false);
        assert(measured && width > 0 && height > 0, "window size % x % (%)", width, height, measured);

        #if OS == .MACOS {
            // The content view is an OpenGL view with no context yet: presenting it does nothing.
            swap_view_buffers(window);
            swap_view_buffers(null);
        }

        saved: Saved_Window_Info;
        ok, full_width, full_height := toggle_fullscreen(window, true, *saved);
        assert(ok, "toggle_fullscreen on failed");
        assert(full_width > 0 && full_height > 0, "fullscreen size % x %", full_width, full_height);
        #if OS != .LINUX {
            assert(full_width >= width && full_height >= height, "fullscreen % x % is smaller than the window % x %", full_width, full_height, width, height);
        }
        pump(5);
        restored_ok, restored_width, restored_height := toggle_fullscreen(window, false, *saved);
        assert(restored_ok, "toggle_fullscreen off failed");
        #if OS != .LINUX {
            assert(restored_width == width && restored_height == height, "restored % x %, was % x %", restored_width, restored_height, width, height);
        }
        pump(5);

        // A missing window is refused.
        #if OS == .LINUX {
            none_ok := toggle_fullscreen(0, true, *saved);
        } else {
            none_ok := toggle_fullscreen(null, true, *saved);
        }
        assert(!none_ok, "toggle_fullscreen accepted a null window");
        print("ok\n");
    }
}
