// An owned composition update, ordered before Input.events_this_frame[position]. Text remains
// valid until the next update_events call. Cursor counts UTF-8 bytes; -1 means the text's end.
Composition_Event :: struct {
    window: WT.Window_Type;
    position: int;
    text: string;
    commit: bool;
    cancel: bool;
    cursor: int = -1;
}

// Native text messages from the last update_events call, interleaved by their position field.
composition_events: [..] Composition_Event;

// Pumps Input while enabling this extension's menu shortcuts and IME decoding. Use in place of
// Input.update_window_events, once per frame, on the window thread. Other Input APIs are unchanged.
update_events :: () {
    begin_desktop_events();
    defer pumping_events = false;
    #if OS == .MACOS pump_cocoa();
    else #if OS == .WINDOWS pump_windows();
    else I.update_window_events();
}

// Enables composition and positions candidate UI at a caret in top-left window points. Disabling
// cancels marked text. Linux currently leaves native composition unsupported.
set_text_input_rect :: (window: WT.Window_Type, enabled: bool, x: float, y: float, height: float) {
    #if OS == .MACOS set_text_input(window, enabled, x, y, height);
    else #if OS == .WINDOWS {
        hook := install_window_hook(window);
        if !hook return;
        hook.enabled = enabled;
        set_rect(window, enabled, x, y, height);
    }
}

// Restores the original native view class/window procedure before destroying the window. Also
// releases queued composition text belonging to it. Call on the window thread after the last frame.
detach_text_input :: (window: WT.Window_Type) {
    #if OS == .MACOS {
        view := result_message(window, "contentView", *void);
        original := get_association(view, *original_class_key);

        if original {
            set_text_input(window, false, 0, 0, 0);
            object_set_class(view, original);
            set_association(view, *original_class_key, null, 0);
        }
    } else #if OS == .WINDOWS remove_window_hook(window);

    for composition_events if it.window == window {
        free(it.text);
        remove it;
    }

    if !composition_events.count {
        array_free(composition_events);
        composition_events = .{};
    }

    for pending_composition if it.window == window {
        free(it.text);
        remove it;
    }

    if !pending_composition.count {
        array_free(pending_composition);
        pending_composition = .{};
    }
}

#scope_module
I :: #import "Input";
Basic :: #import "Basic";

pending_composition: [..] Composition_Event;
pumping_events: bool;

begin_desktop_events :: () {
    for composition_events free(it.text);

    array_reset(*composition_events);
    if !composition_events.allocator.proc composition_events.allocator = context.default_allocator;

    for pending_composition {
        event := it;
        event.position = 0;
        array_add(*composition_events, event);
    }

    array_free(pending_composition);
    pending_composition = .{};
    pumping_events = true;
}

queue_composition :: (window: WT.Window_Type, text: string, commit: bool, cancel: bool, cursor: int = -1) {
    event := Composition_Event.{ window, I.events_this_frame.count, text, commit, cancel, cursor };

    if pumping_events {
        if !composition_events.allocator.proc composition_events.allocator = context.default_allocator;
        array_add(*composition_events, event);
    } else {
        if !pending_composition.allocator.proc pending_composition.allocator = context.default_allocator;
        array_add(*pending_composition, event);
    }
}
