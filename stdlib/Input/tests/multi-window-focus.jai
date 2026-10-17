// Reviewed native focus and global key state, with public Event.window identity.
// Run only under the serialized macOS native-runtime lease.
#assert OS == .MACOS, "Input fixture requires macOS AppKit native focus injection";

Input :: #import "Input";
#import "Basic";
#import "Window_Type";

is_down :: (key: Input.Key_Code) -> bool {
    return (Input.input_button_states[cast(u32) key] & .DOWN) != 0;
}

main :: () {
    defer dispose_native_windows();
    first := owned_native_window(200, 140, "Input focus first");

    assert(first != null);
    Input.update_window_events();
    second := owned_native_window(180, 120, "Input focus second");

    assert(second != null && second != first);
    Input.update_window_events();
    key := cast(Input.Key_Code) #char "A";

    native_message(first, "makeKeyWindow");
    Input.update_window_events();
    assert(has_window_event(first, .GAINED_FOCUS));
    assert(Input.input_application_has_focus);
    native_key(first, true);
    Input.update_window_events();
    assert(has_key(first, key, true));
    assert(!has_key(second, key, true));
    assert(is_down(key));
    assert((Input.input_button_states[cast(u32) key] & .START) != 0);
    Input.update_window_events();
    assert(is_down(key));
    assert((Input.input_button_states[cast(u32) key] & .START) == 0);
    native_key(first, false);
    Input.update_window_events();
    assert(has_key(first, key, false));
    assert(!is_down(key));
    assert((Input.input_button_states[cast(u32) key] & .END) != 0);

    // A real focus handover has one key window; no unreviewed owner map.
    native_message(second, "makeKeyWindow");
    Input.update_window_events();
    assert(has_window_event(first, .LOST_FOCUS));
    assert(has_window_event(second, .GAINED_FOCUS));
    assert(Input.input_application_has_focus);

    native_key(second, true);
    Input.update_window_events();
    assert(has_key(second, key, true) && !has_key(first, key, true));
    assert(is_down(key));
    native_key(second, false);
    Input.update_window_events();
    assert(has_key(second, key, false));
    assert(!is_down(key));
    native_message(first, "makeKeyWindow");
    Input.update_window_events();
    assert(has_window_event(second, .LOST_FOCUS));
    assert(has_window_event(first, .GAINED_FOCUS));
    assert(Input.input_application_has_focus);
    close_native_window(second);
    close_native_window(first);
}

#scope_file
#load "support/macos-fixtures.jai";
