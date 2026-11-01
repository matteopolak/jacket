// test: skip_on(.WASM, reason = "native desktop services")

#load "../module.jai";

main :: () {
    items := Menu_Item.[
        .{ menu = "File", title = "Open", action = 1, key = #char "O" },
        .{ menu = "File", title = "Save As", action = 2, key = #char "S", shift = true },
        .{ menu = "Edit", title = "Copy", action = 3, key = #char "C" },
    ];

    assert(valid_menu(items));
    assert(valid_menu(.[]));
    items[2].action = 1;
    assert(!valid_menu(items));
    items[2].action = 3;
    items[2].key = #char "O";
    assert(!valid_menu(items));
    items[2].shift = true;
    assert(valid_menu(items));
    items[2].key = #char "c";
    assert(!valid_menu(items));
    items[2].key = 0;
    items[2].title = "";
    assert(!valid_menu(items));
    assert(poll_native_menu(null) == 0);
    destroy_native_menu(null);
    menu: Native_Menu;

    defer array_free(menu.pending);
    array_add(*menu.pending, 7);
    array_add(*menu.pending, 9);
    assert(poll_native_menu(*menu) == 7);
    assert(poll_native_menu(*menu) == 9);
    assert(poll_native_menu(*menu) == 0);
    window: WT.Window_Type;

    assert(create_native_menu(window, items) == null);

    #if OS == .LINUX {
        result := native_file_dialog(window, .OPEN);
        assert(result.status == .UNSUPPORTED && result.path.count == 0);
    }
}
