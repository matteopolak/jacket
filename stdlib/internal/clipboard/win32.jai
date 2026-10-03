os_clipboard_get_text :: () -> string {
    open_clipboard();
    defer close_clipboard();
    if format_available(CF_UNICODETEXT) == .FALSE return alloc_string(0);

    handle := GetClipboardData(CF_UNICODETEXT);

    assert(handle != null, "native clipboard text retrieval failed");
    size := global_size(handle);

    assert(size >= 2 && size % 2 == 0, "native clipboard text storage is invalid");
    wide := cast(*u16) GlobalLock(handle);

    assert(wide != null, "native clipboard text lock failed");
    defer unlock_memory(handle);
    count: int = 0;
    while cast(u64) count < size / 2 && wide[count] != 0 count += 1;

    assert(cast(u64) count < size / 2 && count <= 0x7fffffff, "native clipboard text is unterminated or oversized");
    if count == 0 return alloc_string(0);

    bytes := WideCharToMultiByte(CP_UTF8, 0x80, wide, cast(s32) count, null, 0, null, null);

    assert(bytes > 0, "native clipboard UTF-16 conversion failed");
    result := alloc_string(bytes);
    written := WideCharToMultiByte(CP_UTF8, 0x80, wide, cast(s32) count, result.data, bytes, null, null);

    assert(written == bytes, "native clipboard UTF-8 conversion failed");

    return result;
}

os_clipboard_set_text :: (text: string) {
    assert(text.count <= 0x7fffffff);
    for byte: text assert(byte != 0, "Windows CF_UNICODETEXT cannot represent embedded NUL characters");

    count: s32 = 0;

    if text.count != 0 {
        count = MultiByteToWideChar(CP_UTF8, 8, text.data, cast(s32) text.count, null, 0);
        assert(count > 0, "clipboard text is not valid UTF-8");
    }

    handle := GlobalAlloc(GMEM_MOVEABLE, (cast(u64) count + 1) * 2);

    assert(handle != null, "native clipboard text allocation failed");
    transferred := false;

    defer if !transferred release_memory(handle);
    wide := cast(*u16) GlobalLock(handle);

    assert(wide != null, "native clipboard text lock failed");

    if count != 0 {
        written := MultiByteToWideChar(CP_UTF8, 8, text.data, cast(s32) text.count, wide, count);
        assert(written == count, "native clipboard text conversion failed");
    }

    wide[count] = 0;
    unlock_memory(handle);
    open_clipboard();
    defer close_clipboard();
    assert(EmptyClipboard() != .FALSE, "native clipboard clear failed");
    assert(SetClipboardData(CF_UNICODETEXT, handle) != null, "native clipboard text write failed");
    transferred = true;
}

os_clipboard_empty :: () {
    open_clipboard();
    defer close_clipboard();
    assert(EmptyClipboard() != .FALSE, "native clipboard clear failed");
}

os_clipboard_set_bitmap :: (width: s32, height: s32, pixels: *u8, channels: s32, flip := false) {
    assert(width > 0 && height > 0 && pixels != null);
    assert(channels == 3 || channels == 4, "clipboard bitmap accepts RGB or RGBA pixels");
    size := cast(u64) width * cast(u64) height * 4;

    assert(size <= 0xffffffff, "clipboard bitmap exceeds native DIB storage limits");
    handle := GlobalAlloc(GMEM_MOVEABLE, size_of(Native_DIB_Header) + size);

    assert(handle != null, "native clipboard bitmap allocation failed");
    transferred := false;

    defer if !transferred release_memory(handle);
    header := cast(*Native_DIB_Header) GlobalLock(handle);

    assert(header != null, "native clipboard bitmap lock failed");
    header.* = .{};
    header.size = cast(u32) size_of(Native_DIB_Header);
    header.width = width;
    header.height = -height;
    header.planes = 1;
    header.bits = 32;
    header.compression = BI_BITFIELDS;
    header.image_bytes = cast(u32) size;
    header.red_mask = 0x00ff0000;
    header.green_mask = 0x0000ff00;
    header.blue_mask = 0x000000ff;
    header.alpha_mask = 0xff000000;
    header.color_space = 0x73524742;
    header.intent = 4;
    dest := cast(*u8) (header + 1);

    for row: 0..height - 1 {
        source_row := row;
        if flip source_row = height - 1 - row;

        for column: 0..width - 1 {
            source := pixels + (cast(int) source_row * width + column) * channels;
            offset := (cast(int) row * width + column) * 4;
            dest[offset] = source[2];
            dest[offset + 1] = source[1];
            dest[offset + 2] = source[0];
            if channels == 4 dest[offset + 3] = source[3];
            else dest[offset + 3] = 255;
        }
    }

    unlock_memory(handle);
    open_clipboard();
    defer close_clipboard();
    assert(EmptyClipboard() != .FALSE, "native clipboard clear failed");
    assert(SetClipboardData(CF_DIBV5, handle) != null, "native clipboard bitmap write failed");
    transferred = true;
}

#scope_module
#import "Windows";

kernel32_native :: #library,system "kernel32";
user32_native :: #library,system "user32";
global_size :: (handle: HGLOBAL) -> SIZE_T #foreign kernel32_native "GlobalSize";
global_free :: (handle: HGLOBAL) -> HGLOBAL #foreign kernel32_native "GlobalFree";
set_last_error :: (code: u32) #foreign kernel32_native "SetLastError";
format_available :: (format: u32) -> BOOL #foreign user32_native "IsClipboardFormatAvailable";
owner: HWND;

// BITMAPV5HEADER, from the public Windows SDK. C field order is ABI-significant.
Native_DIB_Header :: struct {
    size: u32;
    width, height: s32;
    planes, bits: u16;
    compression, image_bytes: u32;
    x_pixels_per_meter, y_pixels_per_meter: s32;
    colors_used, colors_important: u32;
    red_mask, green_mask, blue_mask, alpha_mask: u32;
    color_space: u32;
    endpoints: [9] s32;
    gamma_red, gamma_green, gamma_blue: u32;
    intent, profile_data, profile_size, reserved: u32;
}

#assert size_of(Native_DIB_Header) == 124;

open_clipboard :: () {
    if !owner {
        // A real, invisible native owner avoids OpenClipboard(NULL)'s ownership trap.
        owner = CreateWindowExA(0, "STATIC", "", 0, 0, 0, 0, 0, null, null, GetModuleHandleW(null), null);
        assert(owner != null, "native clipboard owner creation failed");
    }

    assert(OpenClipboard(owner) != .FALSE, "native clipboard is unavailable or busy");
}

close_clipboard :: () {
    assert(CloseClipboard() != .FALSE, "native clipboard close failed");
}

unlock_memory :: (handle: HGLOBAL) {
    set_last_error(0);
    result := GlobalUnlock(handle);
    assert(result != .FALSE || cast(u32) GetLastError() == 0, "native clipboard memory unlock failed");
}

release_memory :: (handle: HGLOBAL) {
    assert(global_free(handle) == null, "native clipboard memory release failed");
}
