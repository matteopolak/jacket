get_key_code :: (key: KeySym) -> Key_Code {
    if key >= XK_F1 && key <= XK_F24 return cast(Key_Code) (cast(u32) Key_Code.F1 + cast(u32) (key - XK_F1));
    if key >= #char "a" && key <= #char "z" return cast(Key_Code) (key - #char "a" + #char "A");
    if key >= 32 && key <= 126 return cast(Key_Code) key;
    for mapping: x11_keys if mapping.native == key return mapping.code;

    return .UNKNOWN;
}

#scope_module
#import "X11";

X11_Key :: struct {
    native: KeySym;
    code: Key_Code;
}

x11_keys := X11_Key.[
    .{ XK_BackSpace, .BACKSPACE }, .{ XK_Tab, .TAB }, .{ XK_Linefeed, .LINEFEED },
    .{ XK_Return, .ENTER }, .{ XK_Escape, .ESCAPE }, .{ XK_Delete, .DELETE },
    .{ XK_Up, .ARROW_UP }, .{ XK_Down, .ARROW_DOWN }, .{ XK_Left, .ARROW_LEFT },
    .{ XK_Right, .ARROW_RIGHT }, .{ XK_Page_Up, .PAGE_UP }, .{ XK_Page_Down, .PAGE_DOWN },
    .{ XK_Home, .HOME }, .{ XK_End, .END }, .{ XK_Insert, .INSERT },
    .{ XK_Pause, .PAUSE }, .{ XK_Scroll_Lock, .SCROLL_LOCK }, .{ XK_Print, .PRINT_SCREEN },
    .{ XK_Alt_L, .ALT }, .{ XK_Alt_R, .ALT }, .{ XK_Control_L, .CTRL },
    .{ XK_Control_R, .CTRL }, .{ XK_Shift_L, .SHIFT }, .{ XK_Shift_R, .SHIFT },
    .{ XK_Super_L, .CMD }, .{ XK_Super_R, .CMD }, .{ XK_Meta_L, .CMD }, .{ XK_Meta_R, .CMD }
];
