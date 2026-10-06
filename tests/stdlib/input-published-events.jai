// Input without a window system: events published after the pump (as Gamepad does), mouse motion
// and window moves queued by adapters, the custom-cursor flag, and the pointer-position adapters.
// Window handles here are plain tokens; nothing native runs, so every target runs all of it.
#import "Basic";
#import "Window_Type";
Input :: #import "Input"(ENABLE_NATIVE_INPUT_ADAPTERS = false);

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

pointer_calls: int;
pointer_at :: (window: Window_Type, client_area: bool) -> float, float {
    pointer_calls += 1;
    if client_area return 12.5, 34.25;
    return cast(float) window, -1;
}
query_at :: (window: Window_Type, client_area: bool) -> float, float, bool {
    return 7, 8, window != cast(Window_Type) 99;
}

main :: () {
    window := cast(Window_Type) 41;
    other := cast(Window_Type) 42;
    Input.update_window_events();

    // Published events land in this frame right away, unrouted or routed to their window, and
    // drive the button states exactly like pumped events.
    Input.publish_input_event(key_event(.F5, true));
    assert(Input.events_this_frame.count == 1 && Input.unrouted_events_this_frame.count == 1, "unrouted publish: % events", Input.events_this_frame.count);
    assert(Input.routed_events_this_frame.count == 0);
    assert(state_of(.F5) == .DOWN | .START, "F5 after its press: %", state_of(.F5));
    Input.publish_input_event(window, key_event(.F6, true));
    assert(Input.routed_events_this_frame.count == 1 && Input.routed_events_this_frame[0].window == window);
    assert(Input.routed_events_this_frame[0].event.key_code == .F6);
    assert(Input.window_key_is_down(window, .F6) && !Input.window_key_is_down(other, .F6));
    wheel: Input.Event;
    wheel.type = .MOUSE_WHEEL;
    wheel.wheel_delta = 240;
    Input.publish_input_event(wheel);
    assert(Input.mouse_delta_z == 240, "wheel published: %", Input.mouse_delta_z);
    assert(Input.events_this_frame.count == 3);

    // The next frame keeps only "held", and a published release ends the key.
    Input.update_window_events();
    assert(Input.events_this_frame.count == 0 && Input.mouse_delta_z == 0);
    assert(state_of(.F5) == .DOWN, "F5 held: %", state_of(.F5));
    Input.publish_input_event(key_event(.F5, false));
    assert(state_of(.F5) == .END, "F5 after its release: %", state_of(.F5));
    Input.publish_input_event(window, key_event(.F6, false));
    assert(!Input.window_key_is_down(window, .F6) && (state_of(.F6) & .END) != 0);

    // Mouse motion queued by an adapter accumulates until the next pump, then lasts one frame.
    Input.queue_mouse_delta(3, -4);
    Input.queue_mouse_delta(10, 1, 120);
    assert(Input.mouse_delta_x == 0, "motion shows before the pump");
    Input.update_window_events();
    assert(Input.mouse_delta_x == 13 && Input.mouse_delta_y == -3 && Input.mouse_delta_z == 120, "motion % % %", Input.mouse_delta_x, Input.mouse_delta_y, Input.mouse_delta_z);
    Input.update_window_events();
    assert(Input.mouse_delta_x == 0 && Input.mouse_delta_y == 0 && Input.mouse_delta_z == 0);

    // Window moves coalesce per window; reading them takes them.
    Input.queue_window_move(window, 10, 20);
    Input.queue_window_move(other, 5, 6);
    Input.queue_window_move(window, 30, 40);
    moves := Input.get_window_moves();
    assert(moves.count == 2, "% moves", moves.count);
    for moves {
        if it.window == window assert(it.x == 30 && it.y == 40, "window moved to % %", it.x, it.y);
        else assert(it.window == other && it.x == 5 && it.y == 6);
    }
    assert(Input.get_window_moves().count == 0, "moves were not taken by the first read");
    Input.queue_window_move(other, 1, 2);
    moves = Input.get_window_moves();
    assert(moves.count == 1 && moves[0].window == other && moves[0].x == 1 && moves[0].y == 2);

    // The flag is module state with nothing to observe here; toggling it must not disturb input.
    Input.set_custom_cursor_handling(true);
    Input.set_custom_cursor_handling(false);
    Input.update_window_events();
    assert(Input.events_this_frame.count == 0);

    // Pointer position: nothing installed, a plain source, then a query source that can fail.
    Input.input_pointer_source = null;
    Input.input_pointer_query_source = null;
    x, y, known := Input.get_input_pointer_position();
    assert(!known && x == 0 && y == 0, "pointer without an adapter: % % %", x, y, known);
    Input.input_pointer_source = pointer_at;
    x, y, known = Input.get_input_pointer_position();
    assert(known && x == cast(float) INVALID_WINDOW && y == -1, "unscoped pointer: % % %", x, y, known);
    x, y = Input.get_input_pointer_position(window, true);
    assert(x == 12.5 && y == 34.25, "client pointer: % %", x, y);
    x, y = Input.get_input_pointer_position(window);
    assert(x == 41 && y == -1, "screen pointer: % %", x, y);
    assert(pointer_calls == 3);
    Input.input_pointer_query_source = query_at;
    x, y, known = Input.get_input_pointer_position();
    assert(known && x == 7 && y == 8, "queried pointer: % % %", x, y, known);
    x, y = Input.get_input_pointer_position(window, true);
    assert(x == 7 && y == 8 && pointer_calls == 3, "the query source takes precedence");
    x, y, known = Input.input_pointer_query_source(cast(Window_Type) 99, false);
    assert(!known, "the query source reports failure");
    print("ok\n");
}
