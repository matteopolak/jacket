# Input

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

### Android-specific declarations

- [OS == .ANDROID] `window_minimized :: false`
- `update_window_events :: ()`
- `android_process_events :: ()`
- `set_custom_cursor_handling :: (is_custom: bool)`
- `get_key_code :: (key_code: AKEYCODE) -> Key_Code`
- `android_handle_command :: (app: *Android_App, cmd: s32)`
- `android_handle_input_event :: (event: *AInputEvent) -> handled: bool`
- `Android_KeyEvent :: struct`
- `android_get_key_event :: (event: *AInputEvent) -> Android_KeyEvent`
- `android_show_software_keyboard :: #bake_arguments android_show_or_hide_software_keyboard(show_keyboard = true)`
- `android_hide_software_keyboard :: #bake_arguments android_show_or_hide_software_keyboard(show_keyboard = false)`
- `android_show_or_hide_software_keyboard :: (show_keyboard: bool)`

### macOS-specific declarations

- `set_custom_cursor_handling :: (is_custom: bool)`
- `update_window_events :: ()`
- `add_resize_record :: (hwnd: Window_Type, width: s32, height: s32)`

- `Event_Type :: enum u32`
- `Key_Current_State :: enum_flags u32`
- `GAMEPAD_BUTTON_COUNT :: 32`
- `Key_Code :: enum u32`
- `Event :: struct`
- `Window_Event_Type :: enum u8`
- `Touch_Pointer_Info :: struct`
- `TOUCH_POINTERS_COUNT :: 10`
- `NUM_BUTTON_STATES :: #run enum_highest_value(Key_Code) + 1`
- `Window_Resize_Record :: struct`
- `Window_Move_Record :: struct`
- `get_window_resizes :: () -> [] Window_Resize_Record`
- `get_window_moves :: () -> [] Window_Move_Record`
- `input_per_frame_event_and_flag_update :: ()`
- `set_custom_cursor_handling :: (is_custom: bool)`
- `get_key_code :: (key: KeySym) -> Key_Code`
- `WHEEL_DELTA :: 120`
- `update_window_events :: ()`
- `add_resize_record :: (hwnd: Window, width: s32, height: s32)`

### Windows-specific declarations

- `Gamepad_Lib :: #import "Gamepad"`
- `set_custom_cursor_handling :: (is_custom: bool)`
- `get_key_code :: (wParam: WPARAM) -> Key_Code`
- `get_vk :: (key: Key_Code) -> VK: u64`
- `MyWindowProc :: (hwnd: HWND, message: u32, wParam: WPARAM, lParam: LPARAM) -> s64 #c_call`
- `update_window_events :: ()`
- `the_gamepad_handler: (wide_path: *u16, connect: bool, is_xbox: bool);`

- [OS == .ANDROID] `android_event_processing_block_mode: enum`
- [OS == .WINDOWS] `window_minimized := false`
- [OS == .MACOS] `window_minimized := false`
- [OS == .ANDROID] `android_generate_mouse_clicks_for_touch_events := true`
- [OS == .ANDROID] `android_in_focus: bool`
- [OS == .ANDROID] `android_in_foreground: bool`
- [OS == .ANDROID] `android_using_onscreen_keyboard := false`
- `events_this_frame: [..] Event`
- `input_application_has_focus := false`
- `input_button_states: [NUM_BUTTON_STATES] Key_Current_State`
- `mouse_delta_x: int`
- `mouse_delta_y: int`
- `mouse_delta_z: int`
- [OS == .MACOS] `osx_externally_generated_events: [..] Event`
- `touch_pointers: [10] Touch_Pointer_Info`
- `touch_pointers_active := 0`

## Public type members

#### `Window_Event_Type` (enum u8)

- `UNINITIALIZED`
- `GAINED_FOCUS`
- `LOST_FOCUS`
- `MOVED_TO_BACKGROUND`
- `MOVED_TO_FOREGROUND`

### Android-specific type declarations

#### `Android_KeyEvent` (struct)

- `action:       AKEY_EVENT_ACTION`
- `key_code:     AKEYCODE`
- `scan_code:    s32`
- `meta_state:   AMETA`
- `flags:        AKEY_EVENT_FLAG`
- `repeat_count: s32`
- `down_time:    s64`
- `event_time:   s64`

#### `Event_Type` (enum)

- `UNINITIALIZED :: 0`
- `KEYBOARD    :: 1`
- `TEXT_INPUT  :: 2`
- `WINDOW      :: 3`
- `MOUSE_WHEEL :: 4`
- `QUIT        :: 5`
- `DRAG_AND_DROP_FILES :: 6`
- `TOUCH       :: 7`

#### `Key_Current_State` (enum_flags)

- `NONE  :: 0x0`
- `DOWN  :: 0x1`
- `START :: 0x4`
- `END   :: 0x8`

#### `Key_Code` (enum)

- `UNKNOWN :: 0`
- `BACKSPACE   :: 8`
- `TAB         :: 9`
- `LINEFEED    :: 10`
- `ENTER       :: 13`
- `ESCAPE      :: 27`
- `SPACEBAR    :: 32`
- `DELETE      :: 127`
- `ARROW_UP    :: 128`
- `ARROW_DOWN  :: 129`
- `ARROW_LEFT  :: 130`
- `ARROW_RIGHT :: 131`
- `PAGE_UP     :: 132`
- `PAGE_DOWN   :: 133`
- `HOME        :: 134`
- `END         :: 135`
- `INSERT      :: 136`
- `PAUSE       :: 137`
- `SCROLL_LOCK :: 138`
- `ALT`
- `CTRL`
- `SHIFT`
- `CMD`
- `META :: CMD`
- `F1`
- `F2`
- `F3`
- `F4`
- `F5`
- `F6`
- `F7`
- `F8`
- `F9`
- `F10`
- `F11`
- `F12`
- `F13`
- `F14`
- `F15`
- `F16`
- `F17`
- `F18`
- `F19`
- `F20`
- `F21`
- `F22`
- `F23`
- `F24`
- `PRINT_SCREEN`
- `MOUSE_BUTTON_LEFT`
- `MOUSE_BUTTON_MIDDLE`
- `MOUSE_BUTTON_RIGHT`
- `MOUSE_WHEEL_UP`
- `MOUSE_WHEEL_DOWN`
- `MOUSE_BUTTON_EXTRA_1`
- `MOUSE_BUTTON_EXTRA_2`
- `GAMEPAD_0_BEGIN`
- `GAMEPAD_0_END :: GAMEPAD_0_BEGIN + xx GAMEPAD_BUTTON_COUNT`
- `GAMEPAD_1_BEGIN`
- `GAMEPAD_1_END :: GAMEPAD_1_BEGIN + xx GAMEPAD_BUTTON_COUNT`
- `GAMEPAD_2_BEGIN`
- `GAMEPAD_2_END :: GAMEPAD_2_BEGIN + xx GAMEPAD_BUTTON_COUNT`
- `GAMEPAD_3_BEGIN`
- `GAMEPAD_3_END :: GAMEPAD_3_BEGIN + xx GAMEPAD_BUTTON_COUNT`
- `TOUCH`

#### `Event` (struct)

- `using modifier_flags: Modifier_Flags;`

- `Modifier_Flags :: struct { packed: u32 = 0; overlay unpacked: { shift_pressed := false; ctrl_pressed := false; alt_pressed := false; cmd_meta_pressed := false; } }`
- `type: Event_Type = Event_Type.UNINITIALIZED`
- `key_pressed: u32`
- `key_code := Key_Code.UNKNOWN`
- `utf32: u32`
- `repeat := false`
- `text_input_count: u16`
- `typical_wheel_delta: s32`
- `wheel_delta: s32`
- `files: [..] string`
- `touch_type: enum u8 { MOVED :: 0; PRESSED :: 1; RELEASED :: 2; }`
- `touch_index:   s32`
- `touch_x, touch_y: float`
- `window_event_type: Window_Event_Type`
- `window: Window_Type`

#### `Touch_Pointer_Info` (struct)

- `state: Key_Current_State`
- `window_position_x: float`
- `window_position_y: float`

#### `Window_Resize_Record` (struct)

- `window: Window_Type`
- `width:  s32`
- `height: s32`

#### `Window_Move_Record` (struct)

- `window: Window_Type`
- `x: s32`
- `y: s32`

## Behavioral contract and constraints

The platform-independent input layer normalizes keyboard, pointer, and touch input. On Android, the module maps touch contacts to mouse-click events, and pointer-position queries use the window layer. Event ordering and available controls depend on the platform event pump; consumers should process the generated events during the application's update cycle.

On Windows, `the_gamepad_handler` is a mutable optional callback. It receives a null-terminated UTF-16 device path, a `connect` flag that is true for arrival and false for removal, and an `is_xbox` flag. It has no explicit initializer; the window procedure checks whether it is set before invoking it, so assign a handler to receive notifications.

The exposed event/state aggregates do not define an external-event publication contract: the public declarations do not promise that inserting or publishing an `Event` updates `input_button_states`, mouse deltas, or other synthesized state. The `Event.files` array shape likewise does not specify who owns its strings, whether a producer's buffers are copied, or when they may be released. Those guarantees are unspecified and must not be inferred from the aggregate layouts.

`osx_externally_generated_events` is available only under `OS == .MACOS`. Its presence does not imply equivalent native-hook storage on Windows, Linux, or WASM. Common no-window tests may be portable only when they use APIs available under each target's declarations; tests of this macOS hook must be macOS-scoped, and native input-hook execution on other targets is not specified by this module contract.

### Text input and native keyboard composition

`TEXT_INPUT` events carry a platform-delivered character value in `Event.utf32`; they are distinct from `KEYBOARD` press/release events. On macOS, `text_input_count` is populated on the keyboard event with the UTF-32 value count from the native `characters` string; subsequent text events may be fewer because control values, DEL, and U+F700 function-key values are filtered. Windows derives text events from `WM_CHAR` values above 31 and stores the message value directly; because `WM_CHAR` delivers UTF-16 code units, supplementary characters may arrive as surrogate values rather than a single Unicode scalar. macOS emits one text event per remaining UTF-32 code point; a dead key with no characters emits no text event. Android derives a value from the native key event through Java `KeyEvent.getUnicodeChar`; values above 31 produce a text event on key-down. The Windows and Android handlers do not set `text_input_count`. Android's `android_show_software_keyboard` and `android_hide_software_keyboard` request showing or hiding the software keyboard through the platform window-insets controller.

This API exposes committed character events only. It has no composition-start/update/end event, marked-text range, candidate selection, or replacement-range contract. In particular, macOS dead-key composition is not completed by this module, and Android software-keyboard visibility is an explicit request rather than a text-composition protocol. Applications needing full IME composition must integrate the platform text-input client API separately.


Platform coverage note: exports for Nintendo and PS5 targets are unspecified.
