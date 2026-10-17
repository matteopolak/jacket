// Reviewed Mac external-event publication, public frame state and native queries.
// External events preserve payloads; they are not an unreviewed state adapter.
#assert OS == .MACOS, "Input fixture requires reviewed macOS osx_externally_generated_events";

#import "Basic";
#import "Window_Type";
Input :: #import "Input";

key_event :: (key: Input.Key_Code, pressed: bool) -> Input.Event {
    event: Input.Event;
    event.type = .KEYBOARD;
    event.key_code = key;
    event.key_pressed = xx pressed;

    return event;
}

state_of :: (key: Input.Key_Code) -> Input.Key_Current_State {
    return Input.input_button_states[cast(u32) key];
}

main :: () {
    defer dispose_native_windows();
    first := owned_native_window(160, 120, "Input publication first");
    second := owned_native_window(180, 140, "Input publication second");

    assert(first != null && second != null && first != second);
    Input.update_window_events();
    press := key_event(.F5, true);
    press.window = first;
    release := key_event(.F6, false);
    release.window = second;
    wheel: Input.Event;
    wheel.type = .MOUSE_WHEEL;
    wheel.window = second;
    wheel.typical_wheel_delta = 1;
    wheel.wheel_delta = -240;
    text: Input.Event;
    text.type = .TEXT_INPUT;
    text.window = first;
    text.utf32 = 0x1f642;
    array_add(*Input.osx_externally_generated_events, press);
    array_add(*Input.osx_externally_generated_events, release);
    array_add(*Input.osx_externally_generated_events, wheel);
    array_add(*Input.osx_externally_generated_events, text);
    Input.update_window_events();
    assert(Input.osx_externally_generated_events.count == 0);
    assert(has_key(first, .F5, true) && !has_key(second, .F5, true));
    assert(has_key(second, .F6, false));
    wheels, texts: int;

    for event: Input.events_this_frame {
        if event.type == .MOUSE_WHEEL && event.window == second {
            assert(event.wheel_delta == -240 && event.typical_wheel_delta == 1);
            wheels += 1;
        }

        if event.type == .TEXT_INPUT && event.window == first {
            assert(event.utf32 == 0x1f642);
            texts += 1;
        }
    }

    assert(wheels == 1 && texts == 1);

    // These are reviewed mutable fields and the public frame transition, not
    // a promise that external publication synthesizes button/motion state.
    Input.input_button_states[cast(u32) Input.Key_Code.F5] = .DOWN | .START;
    Input.mouse_delta_x = 13;
    Input.mouse_delta_y = -3;
    Input.mouse_delta_z = -240;
    Input.add_resize_record(first, 160, 120);
    found_resize := false;

    for resize: Input.get_window_resizes() {
        if resize.window == first && resize.width == 160 && resize.height == 120 found_resize = true;
    }

    assert(found_resize);
    Input.input_per_frame_event_and_flag_update();
    assert(Input.events_this_frame.count == 0);
    assert(state_of(.F5) == .DOWN);
    assert(Input.mouse_delta_x == 0 && Input.mouse_delta_y == 0 && Input.mouse_delta_z == 0);
    assert(Input.get_window_resizes().count == 0);
    Input.input_button_states[cast(u32) Input.Key_Code.F5] = .END;
    Input.input_per_frame_event_and_flag_update();
    assert(state_of(.F5) == .NONE);

    _, _, width, height, measured := W.get_dimensions(first, false);

    assert(measured && width > 0 && height > 0);
    _, _, known := W.get_mouse_pointer_position(first, false);

    assert(known);
    close_native_window(second);
    close_native_window(first);
}

#scope_file
#load "support/macos-fixtures.jai";
