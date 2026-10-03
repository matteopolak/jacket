Change_Proc :: #type (wide_path: *u16, connect: bool, is_xbox: bool);

decode :: (kind: u64, payload: *W.DEV_BROADCAST_HDR, proc: Change_Proc) -> bool {
    if !proc || !payload return false;
    if kind != W.DBT_DEVICEARRIVAL && kind != W.DBT_DEVICEREMOVECOMPLETE return false;
    if payload.dbch_size < size_of(W.DEV_BROADCAST_HDR) + size_of(W.GUID) + 2 return false;
    if payload.dbch_devicetype != W.DBT_DEVTYP_DEVICEINTERFACE return false;

    native := cast(*W.DEV_BROADCAST_DEVICEINTERFACE_W) payload;
    xbox := same_guid(native.dbcc_classguid, W.XBOX_GUID);

    if !xbox && !same_guid(native.dbcc_classguid, W.GUID_DEVINTERFACE_HID) return false;

    name := native.dbcc_name.data;
    offset := cast(*u8) name - cast(*u8) payload;
    bytes := cast(int) payload.dbch_size - offset;

    if bytes < 2 || (bytes & 1) != 0 return false;

    terminated := false;

    for index: 0..bytes / 2 - 1 if name[index] == 0 {
        terminated = true;
        break;
    }

    if !terminated return false;

    // The native message owns this UTF-16 path until the synchronous callback
    // returns. Never keep its pointer in a module registry.
    proc(name, kind == W.DBT_DEVICEARRIVAL, xbox);

    return true;
}

open :: (window: W.HWND) -> bool {
    registration: Registration;
    registration.window = window;
    filter: W.DEV_BROADCAST_DEVICEINTERFACE_W;
    filter.dbcc_size = size_of(W.DEV_BROADCAST_DEVICEINTERFACE_W);
    filter.dbcc_devicetype = W.DBT_DEVTYP_DEVICEINTERFACE;
    filter.dbcc_classguid = W.GUID_DEVINTERFACE_HID;
    registration.hid = W.RegisterDeviceNotificationW(window, *filter, W.DEVICE_NOTIFY_WINDOW_HANDLE);
    if !registration.hid return false;

    filter.dbcc_classguid = W.XBOX_GUID;
    registration.xbox = W.RegisterDeviceNotificationW(window, *filter, W.DEVICE_NOTIFY_WINDOW_HANDLE);
    if !registration.xbox {
        Basic.assert(unregister(registration.hid) != 0, "native HID registration rollback failed");
        return false;
    }

    if !registrations.allocator.proc registrations.allocator = context.default_allocator;

    Basic.array_add(*registrations, registration);

    return true;
}

close :: (window: W.HWND) {
    for entry, index: registrations if entry.window == window {
        Basic.assert(unregister(entry.hid) != 0, "native HID notification unregistration failed");
        Basic.assert(unregister(entry.xbox) != 0, "native Xbox notification unregistration failed");
        Basic.array_unordered_remove_by_index(*registrations, index);
        return;
    }
}

#scope_module
W :: #import "Windows";
Basic :: #import "Basic";

Registration :: struct {
    window: W.HWND;
    hid, xbox: W.HDEVNOTIFY;
}

registrations: [..] Registration;

user32 :: #library,system "user32";
unregister :: (handle: W.HDEVNOTIFY) -> W.BOOL #foreign user32 "UnregisterDeviceNotification";

same_guid :: (a: W.GUID, b: W.GUID) -> bool {
    if a.Data1 != b.Data1 || a.Data2 != b.Data2 || a.Data3 != b.Data3 return false;
    for byte, index: a.Data4 if byte != b.Data4[index] return false;
    return true;
}
