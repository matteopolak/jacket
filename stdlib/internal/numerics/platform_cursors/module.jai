// Native system cursors. Windows handles are borrowed shared cursors; macOS
// objects are retained explicitly and released by destroy. No private selectors.
// One shared borrowed-object identity is used by every UI module instance.
// No concrete NSCursor layout is supplied; never allocate/dereference.
NSCursor :: struct {}

create :: () -> (handles: [8] *void, success: bool, message: string) {
    #if OS == .WINDOWS {
        ids: [8] u64 = .[32512, 32649, 32648, 32644, 32645, 32646, 32642, 32643];
        for id, i: ids {
            handles[i] = LoadCursorW(null, cast(*u16) id);
            if handles[i] == null return handles, false, "LoadCursorW failed for a native system cursor";
        }

        return handles, true, "";
    } else #if OS == .MACOS {
        if NSApplicationLoad() == 0 return handles, false, "AppKit initialization failed";

        cls := objc_getClass("NSCursor\0".data);
        if cls == null return handles, false, "AppKit NSCursor class is unavailable";

        names: [6] string = .["arrowCursor", "pointingHandCursor", "operationNotAllowedCursor", "resizeLeftRightCursor", "resizeUpDownCursor", "closedHandCursor"];
        for name, i: names {
            selector := selector_for(name);
            cursor := message_object(cls, selector);

            if cursor == null {
                destroy(handles);
                return handles, false, "AppKit returned a null system cursor";
            }

            handles[i] = objc_retain(cursor);
        }

        selector := selector_for("frameResizeCursorFromPosition:inDirections:");
        if message_bool(cls, selector_for("respondsToSelector:"), selector) == 0 {
            destroy(handles);
            return handles, false, "Native diagonal cursors require macOS 15; provide the documented cursor callback on earlier systems";
        }

        handles[6] = objc_retain(message_resize(cls, selector, 3, 3));
        handles[7] = objc_retain(message_resize(cls, selector, 9, 3));
        if handles[6] == null || handles[7] == null {
            destroy(handles);
            return handles, false, "AppKit returned a null diagonal cursor";
        }

        return handles, true, "";
    } else {
        return handles, false, "Native cursor creation is unavailable on this target; provide the documented cursor callback";
    }
}

destroy :: (handles: [8] *void) {
    #if OS == .MACOS for handle: handles if handle != null objc_release(handle);
    // Windows LoadCursorW returns shared system handles: DestroyCursor is forbidden.
} @Releases

set :: (handle: *void) -> (success: bool, message: string) {
    if handle == null return false, "Cannot set a null native cursor";

    #if OS == .WINDOWS {
        SetCursor(handle);
        return true, "";
    } else #if OS == .MACOS {
        message_void(handle, selector_for("set"));
        return true, "";
    } else return false, "Native cursor updates require a caller callback on this target";
}

#scope_file
#import "Basic";

#if OS == .WINDOWS {
    user32 :: #system_library "user32";
    LoadCursorW :: (instance: *void, name: *u16) -> *void #foreign user32;
    SetCursor :: (cursor: *void) -> *void #foreign user32;
} else #if OS == .MACOS {
    appkit :: #system_library "AppKit";
    objc :: #system_library "objc";
    // NSApplicationLoad ensures AppKit is linked and initialized before class lookup.
    NSApplicationLoad :: () -> u8 #foreign appkit;
    objc_getClass :: (name: *u8) -> *void #foreign objc;
    sel_registerName :: (name: *u8) -> *void #foreign objc;
    objc_retain :: (object: *void) -> *void #foreign objc;
    objc_release :: (object: *void) #foreign objc;
    message_object :: (receiver: *void, selector: *void) -> *void #foreign objc "objc_msgSend";
    message_bool :: (receiver: *void, selector: *void, argument: *void) -> u8 #foreign objc "objc_msgSend";
    message_resize :: (receiver: *void, selector: *void, position: u64, directions: u64) -> *void #foreign objc "objc_msgSend";
    message_void :: (receiver: *void, selector: *void) #foreign objc "objc_msgSend";

    selector_for :: (name: string) -> *void {
        c_name := to_c_string(name);
        defer free(c_name);
        return sel_registerName(c_name);
    }
}
