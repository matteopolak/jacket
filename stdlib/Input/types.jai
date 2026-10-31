// What kind of thing an `Event` reports.
Event_Type :: enum u32 {
    UNINITIALIZED :: 0;
    KEYBOARD :: 1;
    TEXT_INPUT :: 2;
    WINDOW :: 3;
    MOUSE_WHEEL :: 4;
    QUIT :: 5;
    DRAG_AND_DROP_FILES :: 6;
    TOUCH :: 7;
}

// What happened to the window, for events of type `.WINDOW`.
Window_Event_Type :: enum u8 {
    UNINITIALIZED :: 0;
    GAINED_FOCUS :: 1;
    LOST_FOCUS :: 2;
    MOVED_TO_BACKGROUND :: 3;
    MOVED_TO_FOREGROUND :: 4;
}

// The state of a key or button this frame. `DOWN` lasts while it is held; `START` and `END` are set
// only on the frame it went down or up.
//
// ```jai
// state := Key_Current_State.DOWN | .START;
// assert((state & .START) != 0);
// assert((state & .END) == 0);
// ```
Key_Current_State :: enum_flags u32 {
    NONE :: 0;
    DOWN :: 1;
    START :: 4;
    END :: 8;
}

GAMEPAD_BUTTON_COUNT :: 32;
TOUCH_POINTERS_COUNT :: 10;
NUM_BUTTON_STATES :: cast(int) Key_Code.TOUCH + 1;

// One input event. Check `type` first; it decides which other fields mean anything: `key_code` and
// `repeat` for `.KEYBOARD`, `utf32` for `.TEXT_INPUT`, `wheel_delta` for `.MOUSE_WHEEL`, `files` for
// `.DRAG_AND_DROP_FILES`, and so on. The modifier flags (`shift_pressed`, `ctrl_pressed`,
// `alt_pressed`, `cmd_meta_pressed`) apply to every event.
//
// ```jai
// event: Event;
// event.type = .KEYBOARD;
// event.key_code = .ENTER;
// event.shift_pressed = true;
// assert(event.key_code == .ENTER && event.packed != 0);
// ```
Event :: struct {
    using modifier_flags: Modifier_Flags;
    Modifier_Flags :: struct {
        packed: u32 = 0;
        #overlay(packed) using unpacked: struct {
            shift_pressed := false;
            ctrl_pressed := false;
            alt_pressed := false;
            cmd_meta_pressed := false;
        }
    }
    type: Event_Type = .UNINITIALIZED;
    key_pressed: u32;
    key_code := Key_Code.UNKNOWN;
    utf32: u32;
    repeat := false;
    text_input_count: u16;
    typical_wheel_delta: s32;
    wheel_delta: s32;
    files: [..] string;
    touch_type: enum u8 {
        MOVED :: 0;
        PRESSED :: 1;
        RELEASED :: 2;
    }
    touch_index: s32;
    touch_x, touch_y: float;
    window_event_type: Window_Event_Type;
    window: Window_Type;
}

// The state and position of one finger on a touch screen.
Touch_Pointer_Info :: struct {
    state: Key_Current_State;
    window_position_x: float;
    window_position_y: float;
}

// A window's new size, from `get_window_resizes`.
Window_Resize_Record :: struct {
    window: Window_Type;
    width: s32;
    height: s32;
}

// A window's new position, from `get_window_moves`.
Window_Move_Record :: struct {
    window: Window_Type;
    x: s32;
    y: s32;
}

// Keys, mouse buttons and gamepad buttons. Printable keys use their ASCII value, upper case for
// letters (`#char "A"` is the `A` key); the rest have names. Index `input_button_states` with it.
//
// ```jai
// assert(cast(u32) Key_Code.ENTER == 13);
// assert(cast(Key_Code) #char "A" != .UNKNOWN);
// ```
Key_Code :: enum u32 {
    UNKNOWN :: 0;
    BACKSPACE :: 8;
    TAB :: 9;
    LINEFEED :: 10;
    ENTER :: 13;
    ESCAPE :: 27;
    SPACEBAR :: 32;
    DELETE :: 127;
    ARROW_UP :: 128;
    ARROW_DOWN :: 129;
    ARROW_LEFT :: 130;
    ARROW_RIGHT :: 131;
    PAGE_UP :: 132;
    PAGE_DOWN :: 133;
    HOME :: 134;
    END :: 135;
    INSERT :: 136;
    PAUSE :: 137;
    SCROLL_LOCK :: 138;
    ALT :: 139;
    CTRL :: 140;
    SHIFT :: 141;
    CMD :: 142;
    META :: CMD;
    F1 :: 143;
    F2 :: 144;
    F3 :: 145;
    F4 :: 146;
    F5 :: 147;
    F6 :: 148;
    F7 :: 149;
    F8 :: 150;
    F9 :: 151;
    F10 :: 152;
    F11 :: 153;
    F12 :: 154;
    F13 :: 155;
    F14 :: 156;
    F15 :: 157;
    F16 :: 158;
    F17 :: 159;
    F18 :: 160;
    F19 :: 161;
    F20 :: 162;
    F21 :: 163;
    F22 :: 164;
    F23 :: 165;
    F24 :: 166;
    PRINT_SCREEN :: 167;
    MOUSE_BUTTON_LEFT :: 168;
    MOUSE_BUTTON_MIDDLE :: 169;
    MOUSE_BUTTON_RIGHT :: 170;
    MOUSE_WHEEL_UP :: 171;
    MOUSE_WHEEL_DOWN :: 172;
    MOUSE_BUTTON_EXTRA_1 :: 173;
    MOUSE_BUTTON_EXTRA_2 :: 174;
    GAMEPAD_0_BEGIN :: 175;
    GAMEPAD_0_END :: 207;
    GAMEPAD_1_BEGIN :: 208;
    GAMEPAD_1_END :: 240;
    GAMEPAD_2_BEGIN :: 241;
    GAMEPAD_2_END :: 273;
    GAMEPAD_3_BEGIN :: 274;
    GAMEPAD_3_END :: 306;
    TOUCH :: 307;
}
