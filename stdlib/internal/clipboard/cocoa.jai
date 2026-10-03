os_clipboard_get_text :: () -> string {
    pool := new_object("NSAutoreleasePool");
    if !pool return alloc_string(0);

    defer release_object(pool);
    pb := pasteboard();
    if !pb return alloc_string(0);

    value := send_object_arg(pb, "stringForType:", native_string("public.utf8-plain-text"));

    if !value return alloc_string(0);

    bytes := cast(*u8) send_object(value, "UTF8String");
    Length :: #type (receiver: *void, selector: *void, encoding: u64) -> u64 #c_call;
    query_length := cast(Length) objc_send;
    length := query_length(value, selector("lengthOfBytesUsingEncoding:"), 4);

    if !check_native(length <= 0x7fffffffffffffff, "native clipboard text exceeds string storage limits") return alloc_string(0);
    if length == 0 return alloc_string(0);

    if !check_native(bytes != null, "native clipboard UTF-8 conversion failed") return alloc_string(0);

    return copy_string(.{ cast(int) length, bytes });
}

os_clipboard_set_text :: (text: string) {
    pool := new_object("NSAutoreleasePool");
    if !pool return;

    defer release_object(pool);
    pb := pasteboard();
    if !pb return;

    value := native_string(text);

    if !check_native(value != null, "clipboard text is not valid UTF-8") return;
    if !clear_contents(pb) return;

    Send :: #type (receiver: *void, selector: *void, text: *void, type: *void) -> s8 #c_call;
    send := cast(Send) objc_send;
    ok := send(pb, selector("setString:forType:"), value, native_string("public.utf8-plain-text"));

    if !check_native(ok != 0, "native clipboard text write failed") return;
}

os_clipboard_empty :: () {
    pool := new_object("NSAutoreleasePool");
    if !pool return;

    defer release_object(pool);
    pb := pasteboard();
    if !pb return;
    if !clear_contents(pb) return;
}

os_clipboard_set_bitmap :: (width: s32, height: s32, pixels: *u8, channels: s32, flip := false) {
    if !check_native(width > 0 && height > 0 && pixels != null, "invalid clipboard bitmap dimensions or pixels") return;
    if !check_native(channels == 3 || channels == 4, "clipboard bitmap accepts RGB or RGBA pixels") return;
    if !check_native(cast(u64) width * cast(u64) height * cast(u64) channels <= 0x7fffffffffffffff, "clipboard bitmap size exceeds storage limits") return;

    pool := new_object("NSAutoreleasePool");
    if !pool return;

    defer release_object(pool);
    Init :: #type (receiver: *void, selector: *void, planes: **u8, width: s64, height: s64, bits: s64, samples: s64, alpha: s8, planar: s8, color_space: *void, bitmap_format: u64, row_bytes: s64, pixel_bits: s64) -> *void #c_call;
    init := cast(Init) objc_send;
    allocated := send_object(objc_class("NSBitmapImageRep"), "alloc");

    if !check_native(allocated != null, "native bitmap allocation failed") return;

    row_bytes := cast(s64) width * channels;
    bitmap := init(allocated, selector("initWithBitmapDataPlanes:pixelsWide:pixelsHigh:bitsPerSample:samplesPerPixel:hasAlpha:isPlanar:colorSpaceName:bitmapFormat:bytesPerRow:bitsPerPixel:"), null, width, height, 8, channels, cast(s8) (channels == 4), 0, native_string("NSDeviceRGBColorSpace"), NSBitmapFormatAlphaNonpremultiplied, row_bytes, channels * 8);

    if !check_native(bitmap != null, "native bitmap initialization failed") return;

    defer release_object(bitmap);
    destination := cast(*u8) send_object(bitmap, "bitmapData");

    if !check_native(destination != null, "native bitmap has no pixel storage") return;

    // NSBitmapImageRep owns its destination; caller pixels remain borrowed.
    for row: 0..height - 1 {
        source_row := row;
        if flip source_row = height - 1 - row;
        memcpy(destination + row * row_bytes, pixels + source_row * row_bytes, row_bytes);
    }

    Encode :: #type (receiver: *void, selector: *void, file_type: u64, properties: *void) -> *void #c_call;
    encode := cast(Encode) objc_send;
    properties := send_object(objc_class("NSDictionary"), "dictionary");
    png := encode(bitmap, selector("representationUsingType:properties:"), 4, properties);

    if !check_native(png != null, "native clipboard PNG encoding failed") return;

    pb := pasteboard();
    if !pb return;

    if !clear_contents(pb) return;

    Write :: #type (receiver: *void, selector: *void, data: *void, type: *void) -> s8 #c_call;
    write := cast(Write) objc_send;
    ok := write(pb, selector("setData:forType:"), png, native_string("public.png"));

    if !check_native(ok != 0, "native clipboard bitmap write failed") return;
}

#scope_module
O :: #import "Objective_C";

NSBitmapFormatAlphaNonpremultiplied :: 1 << 1;
objc :: #library,system "objc";
appkit :: #library,system "AppKit";
application_load :: () -> s8 #foreign appkit "NSApplicationLoad";
objc_send :: () #foreign objc "objc_msgSend";
objc_class :: (name: *u8) -> *void #foreign objc "objc_getClass";
selector :: (name: *u8) -> *void #foreign objc "sel_registerName";

send_object :: (receiver: *void, name: *u8) -> *void {
    Send :: #type (receiver: *void, selector: *void) -> *void #c_call;
    send := cast(Send) objc_send;
    return send(receiver, selector(name));
}

send_object_arg :: (receiver: *void, name: *u8, arg: *void) -> *void {
    Send :: #type (receiver: *void, selector: *void, arg: *void) -> *void #c_call;
    send := cast(Send) objc_send;
    return send(receiver, selector(name), arg);
}

new_object :: (name: *u8) -> *void {
    loaded := application_load();
    if loaded == 0 {
        context.logger("native AppKit initialization failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        assert(false, "native AppKit initialization failed");
        return null;
    }

    result := send_object(objc_class(name), "new");
    if !result {
        context.logger("native Objective-C allocation failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        assert(false, "native Objective-C allocation failed");
    }

    return result;
}

release_object :: (object: *void) {
    Send :: #type (receiver: *void, selector: *void) #c_call;
    send := cast(Send) objc_send;
    send(object, selector("release"));
}

native_string :: (text: string) -> *void {
    return O.NSString.getTempString(text);
}

pasteboard :: () -> *void {
    pb := send_object(objc_class("NSPasteboard"), "generalPasteboard");
    if !check_native(pb != null, "native clipboard service is unavailable") return null;
    return pb;
}

clear_contents :: (pb: *void) -> bool {
    Send :: #type (receiver: *void, selector: *void) -> s64 #c_call;
    send := cast(Send) objc_send;
    changed := send(pb, selector("clearContents"));
    return check_native(changed >= 0, "native clipboard clear failed");
}

check_native :: (success: bool, message: string) -> bool {
    if success return true;
    context.logger(message, context.logger_data, Log_Info.{ common_flags = .ERROR });
    assert(false, message);
    return false;
}
