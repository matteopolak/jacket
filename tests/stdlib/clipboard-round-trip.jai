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
    defer {
        if saved.count os_clipboard_set_text(saved);
    }

    if !clipboard_available() {
        // Nothing is stored, so every read is empty, whatever was set.
        os_clipboard_set_text("unreachable");
        assert(os_clipboard_get_text() == "", "a clipboard without a helper program returned text");
        os_clipboard_empty();
        pixels: [4] u8 = .[1, 2, 3, 4];
        os_clipboard_set_bitmap(1, 1, pixels.data, 4);
        print("ok\n");
        return;
    }

    samples := string.["plain ascii text", "naïve ✓ 日本", "tab\tand\nnewline", ""];
    for samples {
        os_clipboard_set_text(it);
        back := os_clipboard_get_text();
        assert(back == it, "clipboard round trip of \"%\" gave \"%\"", it, back);
        free(back);
    }

    os_clipboard_set_text("about to be emptied");
    os_clipboard_empty();
    after_empty := os_clipboard_get_text();
    assert(after_empty == "", "the emptied clipboard still holds \"%\"", after_empty);

    // A 2x2 image in both pixel layouts the clipboard accepts. Publishing it replaces the text.
    rgba: [16] u8;
    for 0..15 rgba[it] = cast(u8) (it * 16);
    rgb: [12] u8;
    for 0..11 rgb[it] = cast(u8) (255 - it * 16);
    os_clipboard_set_text("replaced by the image");
    os_clipboard_set_bitmap(2, 2, rgba.data, 4);
    os_clipboard_set_bitmap(2, 2, rgb.data, 3, flip = true);
    #if OS != .WASM {
        // The browser build keeps text only; the native clipboards now hold an image and no text.
        text := os_clipboard_get_text();
        assert(text != "replaced by the image", "set_bitmap left the old text on the clipboard");
    }
    // Shapes the clipboard cannot hold are refused without touching it.
    os_clipboard_set_text("kept");
    os_clipboard_set_bitmap(0, 2, rgba.data, 4);
    os_clipboard_set_bitmap(2, 2, null, 4);
    os_clipboard_set_bitmap(2, 2, rgba.data, 2);
    kept := os_clipboard_get_text();
    assert(kept == "kept", "a refused bitmap changed the clipboard to \"%\"", kept);
    print("ok\n");
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
