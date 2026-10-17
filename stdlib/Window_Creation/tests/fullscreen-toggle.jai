// Window_Creation against the real window system: a window goes fullscreen and back with
// toggle_fullscreen, reporting its client size each time. On macOS swap_view_buffers
// presents a real Simp render target. On Linux the request goes to the window
// manager, which may not exist (CI runs Xvfb bare), so only the request and the measured size are
// checked there. The wasm targets have no window system.
#import "Basic";

#import "Window_Creation";
#import "Input";

#if OS == .MACOS {
    S :: #import "Simp";
}

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
        observer := observe_fullscreen();

        defer stop_observing(observer);
        // Establish a real renderer context through the reviewed public API.
        S.set_render_target(window);
        S.clear_render_target(0.25, 0.5, 0.75, 1);
        swap_view_buffers(window);
    }

    saved: Saved_Window_Info;
    ok, full_width, full_height := toggle_fullscreen(window, true, *saved);

    assert(ok, "toggle_fullscreen on failed");
    assert(full_width > 0 && full_height > 0, "fullscreen size % x %", full_width, full_height);

    #if OS == .WINDOWS {
        assert(full_width >= width && full_height >= height, "fullscreen % x % is smaller than the window % x %", full_width, full_height, width, height);
    }

    #if OS == .MACOS wait_fullscreen(window, true);
    else pump(5);
    restored_ok, restored_width, restored_height := toggle_fullscreen(window, false, *saved);

    assert(restored_ok, "toggle_fullscreen off failed");

    #if OS == .MACOS {
        wait_fullscreen(window, false);
        _, _, restored_width, restored_height, restored_ok = get_dimensions(window, false);
        assert(restored_ok, "restored native dimensions unavailable");
    }

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

#scope_file
#if OS == .MACOS {
    O :: #import "Objective_C";

    entered, exited: Window_Type;

    native_object :: (receiver: *void, name: string) -> *void {
        Call :: #type (receiver: *void, selector: O.Selector) -> *void #c_call;
        call := cast(Call) O.objc_msgSend;
        return call(receiver, O.selector(name));
    }

    observe_fullscreen :: () -> *void {
        cls := O.objc_allocateClassPair(O.objc_getClass("NSObject"), "CleanGraphicsFullScreenTestObserver", 0);

        assert(cls != null);
        assert(O.class_addMethod(cls, O.selector("completed:"), cast(O.IMP) fullscreen_completed, "v@:@") != 0);
        O.objc_registerClassPair(cls);
        observer := native_object(cls, "new");

        assert(observer != null);
        center := O.NSNotificationCenter.defaultCenter();

        for name: string.["NSWindowDidEnterFullScreenNotification", "NSWindowDidExitFullScreenNotification"] {
            O.NSNotificationCenter.addObserver(center, cast(O.id) observer, O.selector("completed:"), O.NSString.getTempString(name), null);
        }

        return observer;
    }

    stop_observing :: (observer: *void) {
        Remove :: #type (receiver: *void, selector: O.Selector, observer: *void) #c_call;
        unregister := cast(Remove) O.objc_msgSend;

        unregister(O.NSNotificationCenter.defaultCenter(), O.selector("removeObserver:"), observer);
        Void :: #type (receiver: *void, selector: O.Selector) #c_call;
        release := cast(Void) O.objc_msgSend;

        release(observer, O.selector("release"));
    }

    fullscreen_completed :: (#discard receiver: *void, #discard selector: *void, notification: *void) #c_call {
        push_context, defer_pop;
        name := O.to_string(cast(*O.NSString) native_object(notification, "name"));
        window := cast(Window_Type) native_object(notification, "object");

        if name == "NSWindowDidEnterFullScreenNotification" entered = window;
        else if name == "NSWindowDidExitFullScreenNotification" exited = window;
    }

    wait_fullscreen :: (window: Window_Type, desired: bool) {
        for 1..8000 {
            update_window_events();
            if (desired && entered == window) || (!desired && exited == window) return;

            sleep_milliseconds(5);
        }

        assert(false, "native fullscreen transition did not complete");
    }
}
