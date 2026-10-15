#import "Basic";
W :: #import "Windows";
I :: #import "Input";
WC :: #import "Window_Creation";

user32 :: #library,system "user32";
send_message :: (window: W.HWND, message: u32, wparam: W.WPARAM, lparam: W.LPARAM) -> s64 #foreign user32 "SendMessageW";

Packet :: struct {
    header: W.DEV_BROADCAST_HDR;
    class_guid: W.GUID;
    name: [128] u16;
}

calls: int;
expected_path: *u16;
expected_connect: bool;
expected_xbox: bool;

changed :: (path: *u16, connect: bool, is_xbox: bool) {
    assert(path == expected_path && connect == expected_connect && is_xbox == expected_xbox);
    assert(path[0] == #char "C" && path[1] == 0x03bb && path[2] == 0);
    calls += 1;
}

main :: () {
    window := WC.create_window(100, 80, "Native device notification regression");

    assert(window != null);
    defer assert(W.DestroyWindow(window) != 0);
    I.the_gamepad_handler = changed;
    defer I.the_gamepad_handler = null;
    packet: Packet;
    packet.header.dbch_size = size_of(Packet);
    packet.header.dbch_devicetype = W.DBT_DEVTYP_DEVICEINTERFACE;
    packet.name[0] = #char "C";
    packet.name[1] = 0x03bb;
    expected_path = packet.name.data;

    for xbox: bool.[false, true] {
        packet.class_guid = W.GUID_DEVINTERFACE_HID;
        if xbox packet.class_guid = W.XBOX_GUID;

        expected_xbox = xbox;

        for connect: bool.[true, false] {
            expected_connect = connect;
            kind := W.DBT_DEVICEREMOVECOMPLETE;
            if connect kind = W.DBT_DEVICEARRIVAL;

            before := calls;

            send_message(window, W.WM_DEVICECHANGE, cast(W.WPARAM) kind, cast(W.LPARAM) *packet);
            assert(calls == before + 1);
        }
    }

    assert(calls == 4);
    // Header/type/class/path validation must precede callback invocation.
    packet.header.dbch_size = size_of(W.DEV_BROADCAST_HDR);
    send_message(window, W.WM_DEVICECHANGE, W.DBT_DEVICEARRIVAL, cast(W.LPARAM) *packet);
    packet.header.dbch_size = size_of(Packet);
    packet.header.dbch_devicetype = 0;
    send_message(window, W.WM_DEVICECHANGE, W.DBT_DEVICEARRIVAL, cast(W.LPARAM) *packet);
    packet.header.dbch_devicetype = W.DBT_DEVTYP_DEVICEINTERFACE;
    packet.class_guid = .{};
    send_message(window, W.WM_DEVICECHANGE, W.DBT_DEVICEARRIVAL, cast(W.LPARAM) *packet);
    packet.class_guid = W.GUID_DEVINTERFACE_HID;
    for *unit: packet.name unit.* = #char "X";

    send_message(window, W.WM_DEVICECHANGE, W.DBT_DEVICEARRIVAL, cast(W.LPARAM) *packet);
    send_message(window, W.WM_DEVICECHANGE, W.DBT_DEVICEARRIVAL, 0);
    assert(calls == 4);
    print("ok\n");
}
