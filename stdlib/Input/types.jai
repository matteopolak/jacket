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

Window_Event_Type :: enum u8 {
    UNINITIALIZED :: 0;
    GAINED_FOCUS :: 1;
    LOST_FOCUS :: 2;
    MOVED_TO_BACKGROUND :: 3;
    MOVED_TO_FOREGROUND :: 4;
}

Key_Current_State :: enum_flags u32 {
    NONE :: 0;
    DOWN :: 1;
    START :: 4;
    END :: 8;
}

GAMEPAD_BUTTON_COUNT :: 32;
TOUCH_POINTERS_COUNT :: 10;
NUM_BUTTON_STATES :: cast(int) Key_Code.TOUCH + 1;

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

Touch_Pointer_Info :: struct {
    state: Key_Current_State;
    window_position_x: float;
    window_position_y: float;
}

Window_Resize_Record :: struct {
    window: Window_Type;
    width: s32;
    height: s32;
}

Window_Move_Record :: struct {
    window: Window_Type;
    x: s32;
    y: s32;
}

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
