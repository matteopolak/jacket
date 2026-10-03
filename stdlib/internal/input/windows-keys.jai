get_key_code :: (wParam: WPARAM) -> Key_Code {
    if wParam >= VK_F1 && wParam <= VK_F24 return cast(Key_Code) (cast(u32) Key_Code.F1 + cast(u32) (wParam - VK_F1));
    if (wParam >= #char "A" && wParam <= #char "Z") || (wParam >= #char "0" && wParam <= #char "9") return cast(Key_Code) wParam;
    if wParam == VK_RWIN return .CMD;
    if wParam == VK_LSHIFT || wParam == VK_RSHIFT return .SHIFT;
    if wParam == VK_LCONTROL || wParam == VK_RCONTROL return .CTRL;
    if wParam == VK_LMENU || wParam == VK_RMENU return .ALT;
    for mapping: windows_keys if mapping.native == wParam return mapping.code;

    if wParam <= 255 {
        value := native_key_character(cast(u32) wParam, 2) & 0xffff;
        if value >= #char "a" && value <= #char "z" value -= 32;
        // Non-ASCII values overlap the canonical special-key ranges. Committed
        // Unicode text is delivered independently through WM_CHAR.
        if value >= 32 && value <= 126 return cast(Key_Code) value;
    }

    return .UNKNOWN;
}

get_vk :: (key: Key_Code) -> VK: u64 {
    value := cast(u32) key;
    if value >= cast(u32) Key_Code.F1 && value <= cast(u32) Key_Code.F24 return VK_F1 + value - cast(u32) Key_Code.F1;
    if (value >= #char "A" && value <= #char "Z") || (value >= #char "0" && value <= #char "9") return value;
    for mapping: windows_keys if mapping.code == key return mapping.native;

    if value >= 32 && value <= 126 {
        translated := native_character_key(cast(u16) value);
        if translated != -1 return cast(u64) (cast(s32) translated & 0xff);
    }

    return 0;
}

#scope_module
#import "Windows";

key_user32 :: #library,system "user32";
native_key_character :: (code: u32, kind: u32) -> u32 #foreign key_user32 "MapVirtualKeyW";
native_character_key :: (character: u16) -> s16 #foreign key_user32 "VkKeyScanW";

Windows_Key :: struct {
    native: WPARAM;
    code: Key_Code;
}

windows_keys := Windows_Key.[
    .{ VK_BACK, .BACKSPACE }, .{ VK_TAB, .TAB }, .{ VK_RETURN, .ENTER },
    .{ VK_ESCAPE, .ESCAPE }, .{ VK_SPACE, .SPACEBAR }, .{ VK_DELETE, .DELETE },
    .{ VK_UP, .ARROW_UP }, .{ VK_DOWN, .ARROW_DOWN }, .{ VK_LEFT, .ARROW_LEFT },
    .{ VK_RIGHT, .ARROW_RIGHT }, .{ VK_PRIOR, .PAGE_UP }, .{ VK_NEXT, .PAGE_DOWN },
    .{ VK_HOME, .HOME }, .{ VK_END, .END }, .{ VK_INSERT, .INSERT },
    .{ VK_PAUSE, .PAUSE }, .{ VK_SCROLL, .SCROLL_LOCK }, .{ VK_SNAPSHOT, .PRINT_SCREEN },
    .{ VK_MENU, .ALT }, .{ VK_CONTROL, .CTRL }, .{ VK_SHIFT, .SHIFT }, .{ VK_LWIN, .CMD }
];
