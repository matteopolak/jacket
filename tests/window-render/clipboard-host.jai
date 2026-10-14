#import "Basic";
C :: #import "Clipboard";

main :: () {
    C.os_clipboard_set_text("clean-room clipboard π 🪟");
    value := C.os_clipboard_get_text();

    assert(value == "clean-room clipboard π 🪟");
    free(value);

    #if OS == .MACOS {
        C.os_clipboard_set_text("a\0b");
        embedded := C.os_clipboard_get_text();
        assert(embedded == "a\0b" && embedded.count == 3);
        free(embedded);
    }

    pixels := u8.[128, 0, 0, 128, 0, 64, 0, 128, 0, 0, 128, 128, 255, 255, 255, 255];

    C.os_clipboard_set_bitmap(2, 2, pixels.data, 4, true);

    #if OS == .MACOS {
        inspection := inspect_bitmap(0);
        if inspection != 0 print("native PNG inspection failed at check %\n", inspection);
        assert(inspection == 0, "native clipboard PNG pixels or orientation differ");
    }

    rgb := u8.[9, 1, 2, 3, 4, 5, 3, 5, 7, 6, 7, 8];

    C.os_clipboard_set_bitmap(2, 2, rgb.data, 3);
    #if OS == .MACOS assert(inspect_bitmap(1) == 0, "native clipboard RGB samples differ");

    C.os_clipboard_empty();
    print("ok\n");
}

#scope_file
#if OS == .MACOS {
    probe :: #library,system "window_clipboard_probe";
    inspect_bitmap :: (test_case: s32) -> s32 #foreign probe "window_clipboard_probe";
}
