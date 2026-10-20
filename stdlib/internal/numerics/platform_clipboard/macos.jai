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

    flavor := make_cf_string("public.utf8-plain-text");
    if flavor == null return false, "", "Cannot allocate clipboard flavor string";

    defer CFRelease(flavor);

    // Another process may change the pasteboard between synchronizing and reading: the native calls
    // then report it out of sync or name an item or index it no longer has. Synchronize again and start over.
    for 1..8 {
        success, text, message, out_of_sync := read_synchronized(board, flavor);
        if !out_of_sync return success, text, message;
    }

    return false, "", "The native clipboard kept changing while it was read";
}

#scope_file
#import "Basic";

PASTEBOARD_OUT_OF_SYNC :: -25130; // Native badPasteboardSyncErr.
PASTEBOARD_NO_SUCH_INDEX :: -25131; // Native badPasteboardIndexErr.
PASTEBOARD_NO_SUCH_ITEM :: -25132; // Native badPasteboardItemErr.
PASTEBOARD_NO_SUCH_FLAVOR :: -25133; // Native badPasteboardFlavorErr.

changed_under_us :: (status: s32) -> bool {
    return status == PASTEBOARD_OUT_OF_SYNC || status == PASTEBOARD_NO_SUCH_INDEX || status == PASTEBOARD_NO_SUCH_ITEM;
}

read_synchronized :: (board: *void, flavor: *void) -> (success: bool, text: string, message: string, out_of_sync: bool) {
    PasteboardSynchronize(board);
    count: u64;
    status := PasteboardGetItemCount(board, *count);

    if changed_under_us(status) return false, "", "", true;
    if status != 0 return false, "", sprint("PasteboardGetItemCount failed: %", status), false;
    if count == 0 return true, "", "", false;

    for index: 1..count {
        item: *void;

        status = PasteboardGetItemIdentifier(board, cast(s64) index, *item);
        if changed_under_us(status) return false, "", "", true;
        if status != 0 return false, "", sprint("PasteboardGetItemIdentifier failed: %", status), false;

        data: *void;

        status = PasteboardCopyItemFlavorData(board, item, flavor, *data);
        if status == PASTEBOARD_NO_SUCH_FLAVOR continue; // This item is not UTF-8 text.
        if changed_under_us(status) return false, "", "", true;
        if status != 0 return false, "", sprint("PasteboardCopyItemFlavorData failed: %", status), false;
        if data == null return false, "", "Pasteboard returned a null data reference", false;

        length := CFDataGetLength(data);
        if length < 0 {
            CFRelease(data);
            return false, "", "Pasteboard returned a negative data length", false;
        }

        result := copy_string(string.{ length, CFDataGetBytePtr(data) });

        CFRelease(data);

        return true, result, "", false;
    }

    return false, "", "The native clipboard contains no UTF-8 text flavor", false;
}

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
