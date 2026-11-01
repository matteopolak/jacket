// Installs a menu on a native window; null for unsupported, invalid or failed creation.
create_native_menu :: (window: WT.Window_Type, items: [] Menu_Item) -> *Native_Menu {
    if !window || !items.count || !valid_menu(items) return null;
    for installed: menus if installed.window == window return null;

    handle := win_create_menu();
    if !handle return null;

    menu := New(Native_Menu);
    menu.window = window;
    menu.handle = handle;
    accelerators: [..] Accelerator;

    defer array_free(accelerators);

    for item, index: items {
        seen := false;
        for earlier: 0..index - 1 if items[earlier].menu == item.menu seen = true;
        if seen continue;

        submenu := win_create_popup_menu();
        title := Text.to_utf16(item.menu);

        defer free(title);

        if !submenu || !title || win_append_menu(handle, 0x10, cast(u64) submenu, title) == 0 {
            if submenu win_destroy_menu(submenu);
            win_destroy_menu(handle);
            free(menu);
            return null;
        }

        for entry: items if entry.menu == item.menu {
            label := entry.title;
            if entry.key label = tprint("%\tCtrl+%%", label, ifx entry.shift then "Shift+" else "", string.{ 1, *entry.key });

            wide := Text.to_utf16(label);

            defer free(wide);

            if !wide || win_append_menu(submenu, 0, entry.action, wide) == 0 {
                win_destroy_menu(handle);
                free(menu);
                return null;
            }

            if entry.key array_add(*accelerators, .{ flags = cast(u8) (0x09 | (ifx entry.shift then 0x04 else 0)), key = entry.key, command = entry.action });
        }
    }

    if accelerators.count {
        menu.accelerators = win_create_accelerators(accelerators.data, cast(s32) accelerators.count);
        if !menu.accelerators {
            win_destroy_menu(handle);
            free(menu);
            return null;
        }
    }

    menu.previous = win_get_menu(window);

    if win_set_menu(window, handle) == 0 {
        if menu.accelerators win_destroy_accelerators(menu.accelerators);
        win_destroy_menu(handle);
        free(menu);
        return null;
    }

    if !install_window_hook(window) {
        win_set_menu(window, menu.previous);
        win_destroy_menu(handle);
        if menu.accelerators win_destroy_accelerators(menu.accelerators);

        free(menu);

        return null;
    }

    register_menu(menu);
    win_draw_menu(window);

    return menu;
}

// Removes a menu before window destruction and frees its resources.
destroy_native_menu :: (menu: *Native_Menu) {
    if !menu return;

    win_set_menu(menu.window, menu.previous);
    win_draw_menu(menu.window);
    win_destroy_menu(menu.handle);
    if menu.accelerators win_destroy_accelerators(menu.accelerators);

    unregister_menu(menu);
}

// Called before TranslateMessage; consumed accelerators must not also become keyboard/text input.
translate_accelerator :: (message: *W.MSG) -> bool {
    for menu: menus if message.hwnd == menu.window && menu.accelerators {
        if win_translate_accelerator(menu.window, menu.accelerators, message) != 0 return true;
    }

    return false;
}

// WM_COMMAND carries a menu/accelerator ID in LOWORD; controls have a nonzero lParam.
menu_command :: (window: W.HWND, command: u64, control: s64) -> bool {
    if control != 0 return false;

    for menu: menus if menu.window == window {
        queue_action(menu, cast(u16) (command & 0xffff));
        return true;
    }

    return false;
}

// Runs a modal file/folder picker. The returned UTF-8 path is owned; free it after use.
native_file_dialog :: (window: WT.Window_Type, kind: Dialog_Kind, directory: string = "", name: string = "") -> Dialog_Result {
    buffer: [32768] u16;

    if kind == .FOLDER {
        // The shell browser avoids requiring COM interface bindings or changing apartment state.
        browse: Browse_Info;
        browse.owner = window;
        browse.display_name = buffer.data;
        browse.flags = 1; // BIF_RETURNONLYFSDIRS
        pidl := win_browse_folder(*browse);
        if !pidl return .{ status = .CANCELLED };

        defer win_free_task(pidl);
        if win_path_from_id(pidl, buffer.data, cast(u32) buffer.count, 0) == 0 return .{ status = .FAILED };
    } else {
        initial := Text.to_utf16(directory);

        defer free(initial);
        suggested := Text.to_utf16(name);

        defer free(suggested);
        if !initial || !suggested return .{ status = .FAILED };

        i := 0;
        while suggested[i] && i < buffer.count - 1 {
            buffer[i] = suggested[i];
            i += 1;
        }

        if suggested[i] return .{ status = .FAILED };

        options: W.OPENFILENAMEW;
        options.lStructSize = size_of(W.OPENFILENAMEW);
        options.hwndOwner = window;
        options.lpstrFile = buffer.data;
        options.nMaxFile = cast(u32) buffer.count;
        options.lpstrInitialDir = initial;
        options.Flags = 0x00080000 | 0x00000008 | 0x00000800; // EXPLORER, NOCHANGEDIR, PATHMUSTEXIST
        success: W.BOOL;

        if kind == .SAVE {
            options.Flags |= 0x00000002; // OVERWRITEPROMPT
            success = W.GetSaveFileNameW(*options);
        } else {
            options.Flags |= 0x00001000; // FILEMUSTEXIST
            success = W.GetOpenFileNameW(*options);
        }

        if success == 0 return .{ status = ifx win_dialog_error() == 0 then .CANCELLED else .FAILED };
    }

    path, ok := Text.from_utf16(buffer.data);

    return .{ status = ifx ok then .ACCEPTED else .FAILED, path = path };
}

#scope_module
W :: #import "Windows";
Text :: #import "internal/files/windows-text";

Accelerator :: struct {
    flags: u8;
    key: u16;
    command: u16;
}

Browse_Info :: struct {
    owner: W.HWND;
    root: *void;
    display_name: *u16;
    title: *u16;
    flags: u32;
    callback: *void;
    parameter: s64;
    image: s32;
}

user32 :: #library,system "user32"; @native(external)
shell32 :: #library,system "shell32"; @native(external)
ole32 :: #library,system "ole32"; @native(external)
comdlg32 :: #library,system "comdlg32"; @native(external)
win_create_menu :: () -> *void #foreign user32 "CreateMenu";
win_create_popup_menu :: () -> *void #foreign user32 "CreatePopupMenu";
win_append_menu :: (menu: *void, flags: u32, id: u64, title: *u16) -> s32 #foreign user32 "AppendMenuW";
win_set_menu :: (window: W.HWND, menu: *void) -> s32 #foreign user32 "SetMenu";
win_draw_menu :: (window: W.HWND) -> s32 #foreign user32 "DrawMenuBar";
win_destroy_menu :: (menu: *void) -> s32 #foreign user32 "DestroyMenu";
win_create_accelerators :: (entries: *Accelerator, count: s32) -> *void #foreign user32 "CreateAcceleratorTableW";
win_destroy_accelerators :: (table: *void) -> s32 #foreign user32 "DestroyAcceleratorTable";
win_translate_accelerator :: (window: W.HWND, table: *void, message: *W.MSG) -> s32 #foreign user32 "TranslateAcceleratorW";
win_dialog_error :: () -> u32 #foreign comdlg32 "CommDlgExtendedError";
win_browse_folder :: (info: *Browse_Info) -> *void #foreign shell32 "SHBrowseForFolderW";
win_path_from_id :: (pidl: *void, path: *u16, count: u32, flags: u32) -> s32 #foreign shell32 "SHGetPathFromIDListEx";
win_free_task :: (memory: *void) #foreign ole32 "CoTaskMemFree";

win_get_menu :: (window: W.HWND) -> *void #foreign user32 "GetMenu";

#assert size_of(Accelerator) == 6;
#assert offset_of(Accelerator.key) == 2;
#assert offset_of(Accelerator.command) == 4;
#assert size_of(Browse_Info) == 64;
#assert offset_of(Browse_Info.parameter) == 48;
