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

get_window_resizes :: () -> [] Window_Resize_Record {
    return resize_records;
}

get_window_moves :: () -> [] Window_Move_Record {
    return move_records;
}

add_resize_record :: (hwnd: Window_Type, width: s32, height: s32) {
    if !resize_records.allocator.proc resize_records.allocator = context.default_allocator;
    Basic.array_add(*resize_records, Window_Resize_Record.{ hwnd, width, height });
}

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
