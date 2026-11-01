// Installs a menu on a native window; null for unsupported, invalid or failed creation.
create_native_menu :: (window: WT.Window_Type, items: [] Menu_Item) -> *Native_Menu {
    if !window || !items.count || !valid_menu(items) || !C.initialize_application() return null;
    // AppKit has one application menu; replacing a live installation would leave stale targets.
    if menus.count return null;

    pool := C.autorelease_pool();

    defer C.release(pool);

    if !target_class {
        target_class = O.objc_allocateClassPair(O.objc_getClass("NSObject"), "JacketDesktopMenuTarget", 0);
        if !target_class return null;

        if O.class_addMethod(target_class, O.selector("menuAction:"), cast(O.IMP) menu_action, "v@:@") == 0 {
            O.objc_disposeClassPair(target_class);
            target_class = null;
            return null;
        }

        O.objc_registerClassPair(target_class);
    }

    menu := New(Native_Menu);
    menu.window = window;
    menu.handle = C.result_message(O.objc_getClass("NSMenu"), "new", *void);
    menu.target = C.result_message(target_class, "new", *void);
    // Preserve the previous application menu until this installation is removed.
    app := C.result_message(O.objc_getClass("NSApplication"), "sharedApplication", *void);

    menu.previous = C.result_message(app, "mainMenu", *void);
    C.void_message(menu.previous, "retain");
    // AppKit reserves the first submenu for the application; keep File as a real File menu.
    application_menu := C.result_message(O.objc_getClass("NSMenu"), "new", *void);
    application_item := add_item(menu.handle, "Application", "", false);

    C.argument_message(application_item, "setSubmenu:", application_menu);
    hide_item := add_item(application_menu, "Hide", "h", false);

    C.argument_message(hide_item, "setAction:", O.selector("hide:"));
    C.argument_message(hide_item, "setTarget:", app);
    C.release(application_menu);

    for item, index: items {
        // Group by title, including non-adjacent entries.
        seen := false;
        for earlier: 0..index - 1 if items[earlier].menu == item.menu seen = true;
        if seen continue;

        submenu := C.result_message(O.objc_getClass("NSMenu"), "new", *void);

        C.argument_message(submenu, "setTitle:", C.native_string(item.menu));
        C.argument_message(submenu, "setAutoenablesItems:", cast(s8) 0);
        top := add_item(menu.handle, item.menu, "", false);

        C.argument_message(top, "setSubmenu:", submenu);

        for entry: items if entry.menu == item.menu {
            key: string;
            byte := entry.key + 32;

            if entry.key key = .{ 1, *byte };

            native := add_item(submenu, entry.title, key, true);

            C.argument_message(native, "setTarget:", menu.target);
            C.argument_message(native, "setTag:", cast(s64) entry.action);
            mask: u64 = 1 << 20;
            if entry.shift mask |= 1 << 17;

            C.argument_message(native, "setKeyEquivalentModifierMask:", mask);
        }

        C.release(submenu);
    }

    register_menu(menu);
    C.argument_message(app, "setMainMenu:", menu.handle);

    return menu;
}

// Removes a menu before window destruction and frees its resources.
destroy_native_menu :: (menu: *Native_Menu) {
    if !menu return;

    app := C.result_message(O.objc_getClass("NSApplication"), "sharedApplication", *void);

    C.argument_message(app, "setMainMenu:", menu.previous);
    C.release(menu.previous);
    C.release(menu.handle);
    C.release(menu.target);
    unregister_menu(menu);
}

// Runs a modal file/folder picker. The returned UTF-8 path is owned; free it after use.
native_file_dialog :: (window: WT.Window_Type, kind: Dialog_Kind, directory: string = "", name: string = "") -> Dialog_Result {
    if !C.initialize_application() return .{ status = .FAILED };

    pool := C.autorelease_pool();

    defer C.release(pool);
    panel: *void;

    if kind == .SAVE {
        panel = C.result_message(O.objc_getClass("NSSavePanel"), "savePanel", *void);
        if name.count C.argument_message(panel, "setNameFieldStringValue:", C.native_string(name));
    } else {
        panel = C.result_message(O.objc_getClass("NSOpenPanel"), "openPanel", *void);
        C.argument_message(panel, "setCanChooseFiles:", cast(s8) (kind == .OPEN));
        C.argument_message(panel, "setCanChooseDirectories:", cast(s8) (kind == .FOLDER));
        C.argument_message(panel, "setAllowsMultipleSelection:", cast(s8) 0);
    }

    if !panel return .{ status = .FAILED };

    if directory.count {
        url := C.argument_result_message(O.objc_getClass("NSURL"), "fileURLWithPath:", C.native_string(directory), *void);
        C.argument_message(panel, "setDirectoryURL:", url);
    }

    response := C.result_message(panel, "runModal", s64);
    if response != 1 return .{ status = .CANCELLED };

    url := C.result_message(panel, "URL", *void);
    path := C.result_message(url, "path", *void);
    bytes := C.result_message(path, "UTF8String", *u8);

    if !bytes return .{ status = .FAILED };

    return .{ status = .ACCEPTED, path = copy_string(to_string(bytes)) };
}

#scope_module
C :: #import "internal/window/cocoa";
O :: #import "internal/bindings/objc-api";

target_class: O.Class;

add_item :: (menu: *void, title: string, key: string, action: bool) -> *void {
    Send :: #type (receiver: *void, selector: O.Selector, title: *void, action: O.Selector, key: *void) -> *void #c_call;
    send := cast(Send) desktop_send;
    selector: O.Selector;

    if action selector = O.selector("menuAction:");

    return send(menu, O.selector("addItemWithTitle:action:keyEquivalent:"), C.native_string(title), selector, C.native_string(key));
}

menu_action :: (target: *void, selector: *void, sender: *void) #c_call {
    push_context, defer_pop;

    for menu: menus if menu.target == target {
        queue_action(menu, cast(u16) C.result_message(sender, "tag", s64));
        return;
    }
}

objc :: #library,system "objc"; @native(external)
desktop_send :: () #foreign objc "objc_msgSend";
