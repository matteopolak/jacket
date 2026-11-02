// Native menus, file dialogs and opt-in text services for desktop windows.
// @experimental

// test: skip_on(.WASM, reason = "Extensions/Desktop drives native windows")

#assert OS != .WASM "Extensions/Desktop drives native windows, and WebAssembly has none";

#import "Basic";
WT :: #import "Window_Type";

// A flat native menu entry. `menu` groups entries into top-level menus; `action` must be unique
// and nonzero. `key` is an uppercase ASCII letter (0: none), with Cmd on macOS or Ctrl on Windows.
// `shift` adds Shift. Strings are borrowed only during creation.
//
// ```jai
// item := Menu_Item.{ menu = "File", title = "Open…", action = 1, key = #char "O" };
// assert(item.action == 1);
// ```
Menu_Item :: struct {
    menu: string;
    title: string;
    action: u16; // nonzero; unique in the menu
    key: u8; // uppercase ASCII letter; 0 means no shortcut
    shift: bool;
}

// An owned menu installation, used on the window's thread. macOS allows one installation per
// application; Windows allows one per window. Remove it before destroying its window.
Native_Menu :: struct {
    window: WT.Window_Type;
    handle: *void;
    target: *void;
    accelerators: *void;
    previous: *void;
    pending: [..] u16;
}

// Checks IDs, titles and shortcut uniqueness without contacting a window server.
valid_menu :: (items: [] Menu_Item) -> bool {
    for item, index: items {
        if !item.menu.count || !item.title.count || !item.action return false;
        if item.key != 0 && (item.key < #char "A" || item.key > #char "Z") return false;

        for earlier: 0..index - 1 {
            if items[earlier].action == item.action return false;
            if item.key && items[earlier].key == item.key && items[earlier].shift == item.shift return false;
        }
    }

    return true;
}

// OPEN selects an existing file; SAVE selects an output path; FOLDER selects a directory.
Dialog_Kind :: enum {
    OPEN;
    SAVE;
    FOLDER;
}

// ACCEPTED, CANCELLED, UNSUPPORTED or FAILED. Cancellation must not invoke a fallback prompt.
Dialog_Status :: enum {
    ACCEPTED;
    CANCELLED;
    UNSUPPORTED;
    FAILED;
}

// A dialog outcome with an owned UTF-8 path. Release `path` with `Basic.free` after using it.
Dialog_Result :: struct {
    status: Dialog_Status = .UNSUPPORTED;
    path: string; // owned; free even when empty
}

// Takes the next selected action ID, or zero when empty.
poll_native_menu :: (menu: *Native_Menu) -> u16 {
    if !menu || !menu.pending.count return 0;
    action := menu.pending[0];
    array_ordered_remove_by_index(*menu.pending, 0);
    return action;
}

#if OS == .MACOS {
    #load "cocoa.jai";
    #load "cocoa-text.jai";
    #load "cocoa-events.jai";
} else #if OS == .WINDOWS {
    #load "windows.jai";
    #load "windows-text.jai";
    #load "windows-events.jai";
} else {
    // Linux has no native menu backend; returns null.
    create_native_menu :: (window: WT.Window_Type, items: [] Menu_Item) -> *Native_Menu {
        return null;
    }

    // Releases a menu; a no-op on Linux.
    destroy_native_menu :: (menu: *Native_Menu) {}

    // Linux returns UNSUPPORTED without opening a dialog.
    native_file_dialog :: (window: WT.Window_Type, kind: Dialog_Kind, directory: string = "", name: string = "") -> Dialog_Result {
        return .{};
    }
}

#load "events.jai";

#scope_module
menus: [..] *Native_Menu;

register_menu :: (menu: *Native_Menu) {
    if !menus.allocator.proc menus.allocator = context.default_allocator;
    array_add(*menus, menu);
}

unregister_menu :: (menu: *Native_Menu) {
    for menus if it == menu {
        remove it;
        break;
    }

    if !menus.count {
        array_free(menus);
        menus = .{};
    }

    array_free(menu.pending);
    free(menu);
}

queue_action :: (menu: *Native_Menu, action: u16) {
    if !menu.pending.allocator.proc menu.pending.allocator = context.default_allocator;
    array_add(*menu.pending, action);
}
