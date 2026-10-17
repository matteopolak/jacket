// Clipboard: text set on the system clipboard comes back unchanged (ASCII, UTF-8, empty), emptying
// it leaves no text, and a bitmap can be published. The program's previous clipboard text is put
// back at the end. On Linux the clipboard goes through wl-copy/wl-paste or xclip; without either
// helper (or without a display) nothing can be stored, and the calls must still return cleanly.
// The browser build keeps the clipboard in memory.
#import "Basic";
#import "Clipboard";

#if OS == .LINUX {
    #import "File_Utilities";
}

main :: () {
    saved := os_clipboard_get_text();

    defer free(saved);

    defer {
        if saved.count os_clipboard_set_text(saved);
    }

    if !clipboard_available() {
        // Nothing is stored, so every read is empty, whatever was set.
        os_clipboard_set_text("unreachable");
        assert(os_clipboard_get_text() == "", "a clipboard without a helper program returned text");
        empty_clipboard();
        pixels: [4] u8 = .[1, 2, 3, 4];

        os_clipboard_set_bitmap(1, 1, pixels.data, 4);
        print("ok\n");

        return;
    }

    samples := string.["plain ascii text", "naïve ✓ 日本", "tab\tand\nnewline", ""];
    for samples {
        os_clipboard_set_text(it);
        back := os_clipboard_get_text();
        assert(back == it, "clipboard text round trip failed");
        free(back);
    }

    os_clipboard_set_text("about to be emptied");
    empty_clipboard();
    after_empty := os_clipboard_get_text();

    assert(after_empty == "", "the emptied clipboard still holds text");
    free(after_empty);

    // A 2x2 image in both pixel layouts the clipboard accepts. Publishing it replaces the text.
    rgba: [16] u8;
    for 0..15 rgba[it] = cast(u8) (it * 16);

    rgb: [12] u8;
    for 0..11 rgb[it] = cast(u8) (255 - it * 16);

    os_clipboard_set_text("replaced by the image");
    os_clipboard_set_bitmap(2, 2, rgba.data, 4);
    #if OS == .MACOS inspect_bitmap(rgba.data, 4, false);
    os_clipboard_set_bitmap(2, 2, rgb.data, 3, flip = true);
    #if OS == .MACOS inspect_bitmap(rgb.data, 3, true);

    #if OS != .WASM {
        // The browser build keeps text only; the native clipboards now hold an image and no text.
        text := os_clipboard_get_text();
        assert(text != "replaced by the image", "set_bitmap left the old text on the clipboard");
        free(text);
    }

    // Invalid pointers/extents/channel counts have no silent-success contract.
    // Valid image samples, alpha and flipped orientation are checked natively.
    print("ok\n");
}

#scope_file
// The Linux clipboard has no separate empty operation: storing empty text clears it.
empty_clipboard :: () {
    #if OS == .LINUX os_clipboard_set_text("");
    else os_clipboard_empty();
}

#if OS == .MACOS {
    O :: #import "Objective_C";

    object :: (receiver: *void, name: *u8) -> *void {
        Call :: #type (receiver: *void, selector: O.SEL) -> *void #c_call;
        call := cast(Call) O.objc_msgSend;
        return call(receiver, O.sel_registerName(name));
    }

    inspect_bitmap :: (pixels: *u8, channels: int, flipped: bool) {
        pool := object(O.objc_getClass("NSAutoreleasePool"), "new");
        Void :: #type (receiver: *void, selector: O.SEL) #c_call;
        release := cast(Void) O.objc_msgSend;

        defer release(pool, O.sel_registerName("release"));
        Argument :: #type (receiver: *void, selector: O.SEL, value: *void) -> *void #c_call;
        argument := cast(Argument) O.objc_msgSend;
        board := object(O.objc_getClass("NSPasteboard"), "generalPasteboard");
        png := argument(board, O.sel_registerName("dataForType:"), O.NSString.getTempString("public.png"));

        assert(png != null, "clipboard contains no native PNG payload");
        bitmap := argument(object(O.objc_getClass("NSBitmapImageRep"), "alloc"), O.sel_registerName("initWithData:"), png);

        assert(bitmap != null, "native PNG decoder rejected clipboard image");
        defer release(bitmap, O.sel_registerName("release"));
        Integer :: #type (receiver: *void, selector: O.SEL) -> s64 #c_call;
        integer := cast(Integer) O.objc_msgSend;

        assert(integer(bitmap, O.sel_registerName("pixelsWide")) == 2);
        assert(integer(bitmap, O.sel_registerName("pixelsHigh")) == 2);
        assert(integer(bitmap, O.sel_registerName("bitsPerSample")) == 8);
        Pixel :: #type (receiver: *void, selector: O.SEL, samples: *u64, x: s64, y: s64) #c_call;
        pixel := cast(Pixel) O.objc_msgSend;

        for y: 0..1 {
            source_y := ifx flipped then 1 - y else y;

            for x: 0..1 {
                samples: [4] u64;

                pixel(bitmap, O.sel_registerName("getPixel:atX:y:"), samples.data, x, y);

                for component: 0..channels - 1 {
                    assert(samples[component] == pixels[(source_y * 2 + x) * channels + component], "native clipboard PNG sample mismatch");
                }
            }
        }
    }
}

// Linux has no clipboard of its own: one of the helper programs and a display must be present.
clipboard_available :: () -> bool {
    #if OS == .LINUX {
        wayland := get_env("WAYLAND_DISPLAY").count > 0 && on_path("wl-copy") && on_path("wl-paste");
        x11 := get_env("DISPLAY").count > 0 && on_path("xclip");
        return wayland || x11;
    } else {
        return true;
    }
}

#if OS == .LINUX {
    POSIX :: #import "POSIX";

    get_env :: (name: string) -> string {
        value := POSIX.getenv(temp_c_string(name));
        if !value return "";
        return to_string(value);
    }

    on_path :: (program: string) -> bool {
        for directory: string.["/usr/bin", "/bin", "/usr/local/bin"] {
            if file_exists(tprint("%/%", directory, program)) return true;
        }

        return false;
    }
}
