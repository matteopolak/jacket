// Keyboard, mouse, touch and window events for windows made with `Window_Creation`.
//
// Call `update_window_events` once per frame, then read `events_this_frame` for what happened and
// `input_button_states` for what is held. Each state is a `Key_Current_State` bit set indexed by
// `Key_Code`.
//
// ```jai,no_run
// #import "Window_Creation";
// #import "Input";
//
// window := create_window(800, 600, "input");
// while true {
//     update_window_events();
//     for events_this_frame {
//         if it.type == .QUIT return;
//         if it.type == .KEYBOARD && it.key_code == .ESCAPE return;
//     }
//     if input_button_states[Key_Code.MOUSE_BUTTON_LEFT] & .START print("click at %, %\n", mouse_delta_x, mouse_delta_y);
// }
// ```

#load "types.jai";

#if OS == .LINUX {
    WHEEL_DELTA :: 120;

    #load "../internal/input/x11-keys.jai";
    #load "../internal/input/x11-events.jai";
}

#if OS == .WINDOWS {
    #load "../internal/input/windows-keys.jai";
    #load "../internal/input/windows-events.jai";
}

events_this_frame: [..] Event;
input_application_has_focus := false;
input_button_states: [NUM_BUTTON_STATES] Key_Current_State;
mouse_delta_x: int;
mouse_delta_y: int;
mouse_delta_z: int;
touch_pointers: [10] Touch_Pointer_Info;
touch_pointers_active := 0;

#if OS == .MACOS {
    window_minimized := false;
    osx_externally_generated_events: [..] Event;

    #load "../internal/input/cocoa-events.jai";
}

#if OS == .WINDOWS {
    Gamepad_Lib :: #import "Gamepad";

    window_minimized := false;
    the_gamepad_handler: (wide_path: *u16, connect: bool, is_xbox: bool);
}

// The windows resized since the last `update_window_events`.
//
// ```jai
// for get_window_resizes() print("% x %\n", it.width, it.height);
// assert(get_window_resizes().count == 0);
// ```
get_window_resizes :: () -> [] Window_Resize_Record {
    return resize_records;
}

// The windows moved since the last `update_window_events`.
//
// ```jai
// assert(get_window_moves().count == 0);
// ```
get_window_moves :: () -> [] Window_Move_Record {
    return move_records;
}

// Records a resize for `get_window_resizes`. The window backends call it; programs rarely need to.
add_resize_record :: (hwnd: Window_Type, width: s32, height: s32) {
    add_to_frame(*resize_records, Window_Resize_Record.{ hwnd, width, height });
}

// Starts a new frame: drops this frame's events and resize or move records, zeroes the mouse deltas
// and clears the `START` and `END` flags while keeping `DOWN`. `update_window_events` calls it first.
//
// ```jai
// input_button_states[Key_Code.SPACEBAR] = .DOWN | .START;
// input_per_frame_event_and_flag_update();
// assert(input_button_states[Key_Code.SPACEBAR] == .DOWN);
// ```
input_per_frame_event_and_flag_update :: () {
    // A new frame retains held buttons and discards transition flags.
    for *state: input_button_states state.* &= .DOWN;
    for *pointer: touch_pointers pointer.state &= .DOWN;
    for *event: events_this_frame Basic.array_free(event.files);

    Basic.array_reset_keeping_memory(*events_this_frame);
    Basic.array_reset_keeping_memory(*resize_records);
    Basic.array_reset_keeping_memory(*move_records);
    mouse_delta_x = 0;
    mouse_delta_y = 0;
    mouse_delta_z = 0;
}

#scope_module
Window_Type :: WT.Window_Type;

WT :: #import "Window_Type";
Basic :: #import "Basic";

resize_records: [..] Window_Resize_Record;
move_records: [..] Window_Move_Record;

// The per-frame arrays outlive the frame's temporary storage, so they always grow with the
// default allocator, never the caller's context allocator (a program may set it to temp).
add_to_frame :: (array: *[..] $T, item: T) {
    if !array.allocator.proc array.allocator = context.default_allocator;
    Basic.array_add(array, item);
}
