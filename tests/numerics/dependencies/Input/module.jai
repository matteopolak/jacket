// Test-only record fixture from reviewed Input spec. No platform pump is modeled.
Event_Type :: enum {
    UNINITIALIZED;
    KEYBOARD;
    TEXT_INPUT;
    WINDOW;
    MOUSE_WHEEL;
    QUIT;
    DRAG_AND_DROP_FILES;
    TOUCH;
}

Window_Event_Type :: enum u8 {
    UNINITIALIZED;
    GAINED_FOCUS;
    LOST_FOCUS;
    MOVED_TO_BACKGROUND;
    MOVED_TO_FOREGROUND;
}

Key_Current_State :: enum_flags u32 {
    NONE :: 0;
    DOWN :: 1;
    START :: 4;
    END :: 8;
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
    ARROW_DOWN;
    ARROW_LEFT;
    ARROW_RIGHT;
    PAGE_UP;
    PAGE_DOWN;
    HOME;
    END;
    INSERT;
    PAUSE;
    SCROLL_LOCK;
    ALT;
    CTRL;
    SHIFT;
    CMD;
    META :: CMD;
    F1;
    F2;
    F3;
    F4;
    F5;
    F6;
    F7;
    F8;
    F9;
    F10;
    F11;
    F12;
    F13;
    F14;
    F15;
    F16;
    F17;
    F18;
    F19;
    F20;
    F21;
    F22;
    F23;
    F24;
    PRINT_SCREEN;
    MOUSE_BUTTON_LEFT;
    MOUSE_BUTTON_MIDDLE;
    MOUSE_BUTTON_RIGHT;
    MOUSE_WHEEL_UP;
    MOUSE_WHEEL_DOWN;
    MOUSE_BUTTON_EXTRA_1;
    MOUSE_BUTTON_EXTRA_2;
}

Event :: struct {
    using modifier_flags: Modifier_Flags;
    Modifier_Flags :: struct {
        packed: u32;
        #overlay(packed) using unpacked: struct {
            shift_pressed: bool;
            ctrl_pressed: bool;
            alt_pressed: bool;
            cmd_meta_pressed: bool;
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
        MOVED;
        PRESSED;
        RELEASED;
    }
    touch_index: s32;
    touch_x, touch_y: float;
    window_event_type: Window_Event_Type;
    window: Window_Type;
}

#scope_file
#import "Window_Type";
