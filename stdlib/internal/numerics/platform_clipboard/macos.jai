shutdown :: () {
    if cached_board != null {
        CFRelease(cached_board);
        cached_board = null;
    }
}

write_text :: (text: string, _window: *void = null) -> (success: bool, message: string) {
    board, error := open_board();
    if board == null return false, error;

    flavor := make_cf_string("public.utf8-plain-text");
    if flavor == null return false, "Cannot allocate clipboard flavor string";

    defer CFRelease(flavor);
    data := CFDataCreate(null, text.data, text.count);
    if data == null return false, "Cannot allocate clipboard text data";

    defer CFRelease(data);

    status := PasteboardClear(board);
    if status != 0 return false, sprint("PasteboardClear failed: %", status);

    status = PasteboardPutItemFlavor(board, cast(*void) 1, flavor, data, 0);
    if status != 0 return false, sprint("PasteboardPutItemFlavor failed: %", status);

    return true, "";
}

read_text :: (_window: *void = null) -> (success: bool, text: string, message: string) {
    board, error := open_board();
    if board == null return false, "", error;

    PasteboardSynchronize(board);
    count: u64;
    status := PasteboardGetItemCount(board, *count);

    if status != 0 return false, "", sprint("PasteboardGetItemCount failed: %", status);
    if count == 0 return true, "", "";

    flavor := make_cf_string("public.utf8-plain-text");
    if flavor == null return false, "", "Cannot allocate clipboard flavor string";

    defer CFRelease(flavor);

    for index: 1..count {
        item: *void;

        status = PasteboardGetItemIdentifier(board, cast(s64) index, *item);
        if status != 0 return false, "", sprint("PasteboardGetItemIdentifier failed: %", status);

        data: *void;

        status = PasteboardCopyItemFlavorData(board, item, flavor, *data);
        if status == -25133 continue; // Native badPasteboardFlavorErr: this item is not UTF-8 text.
        if status != 0 return false, "", sprint("PasteboardCopyItemFlavorData failed: %", status);
        if data == null return false, "", "Pasteboard returned a null data reference";

        length := CFDataGetLength(data);
        if length < 0 {
            CFRelease(data);
            return false, "", "Pasteboard returned a negative data length";
        }

        result := copy_string(string.{ length, CFDataGetBytePtr(data) });

        CFRelease(data);

        return true, result, "";
    }

    return false, "", "The native clipboard contains no UTF-8 text flavor";
}

#scope_file
#import "Basic";

core_foundation :: #system_library "CoreFoundation";
application_services :: #system_library "ApplicationServices";

make_cf_string :: (text: string) -> *void {
    return CFStringCreateWithBytes(null, text.data, text.count, 0x08000100, 0);
}

cached_board: *void;

open_board :: () -> (board: *void, error: string) {
    if cached_board != null return cached_board, "";

    name := make_cf_string(PASTEBOARD_NAME);
    if name == null return null, "Cannot allocate native pasteboard name";

    defer CFRelease(name);
    status := PasteboardCreate(name, *board);
    if status != 0 return null, sprint("PasteboardCreate failed: %", status);
    if board == null return null, "PasteboardCreate returned a null board";

    cached_board = board;

    return board, "";
}

CFRelease :: (object: *void) #foreign core_foundation;
CFStringCreateWithBytes :: (allocator: *void, bytes: *u8, length: s64, encoding: u32, external_representation: u8) -> *void #foreign core_foundation;
CFDataCreate :: (allocator: *void, bytes: *u8, length: s64) -> *void #foreign core_foundation;
CFDataGetLength :: (data: *void) -> s64 #foreign core_foundation;
CFDataGetBytePtr :: (data: *void) -> *u8 #foreign core_foundation;
PasteboardCreate :: (name: *void, board: **void) -> s32 #foreign application_services;
PasteboardSynchronize :: (board: *void) -> u32 #foreign application_services;
PasteboardClear :: (board: *void) -> s32 #foreign application_services;
PasteboardGetItemCount :: (board: *void, count: *u64) -> s32 #foreign application_services;
PasteboardGetItemIdentifier :: (board: *void, index: s64, item: **void) -> s32 #foreign application_services;
PasteboardPutItemFlavor :: (board: *void, item: *void, flavor: *void, data: *void, flags: u32) -> s32 #foreign application_services;
PasteboardCopyItemFlavorData :: (board: *void, item: *void, flavor: *void, data: **void) -> s32 #foreign application_services;
