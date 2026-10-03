#import "Basic";
#load "../../internal/input/windows-devices/module.jai";

Packet :: struct {
    header: W.DEV_BROADCAST_HDR;
    class_guid: W.GUID;
    name: [8] u16;
}

calls: int;
expected: *u16;
expected_connect: bool;
expected_xbox: bool;

changed :: (path: *u16, connect: bool, is_xbox: bool) {
    assert(path == expected && connect == expected_connect && is_xbox == expected_xbox);
    assert(path[0] == 0x03bb && path[1] == 0);
    calls += 1;
}

main :: () {
    assert(size_of(W.GUID) == 16 && size_of(W.DEV_BROADCAST_HDR) == 12);
    packet: Packet;
    packet.header.dbch_size = size_of(Packet);
    packet.header.dbch_devicetype = W.DBT_DEVTYP_DEVICEINTERFACE;
    packet.name[0] = 0x03bb;
    expected = packet.name.data;
    assert(cast(*u8) expected - cast(*u8) *packet == 28);

    for xbox: bool.[false, true] {
        packet.class_guid = W.GUID_DEVINTERFACE_HID;
        if xbox packet.class_guid = W.XBOX_GUID;

        expected_xbox = xbox;

        for connect: bool.[false, true] {
            expected_connect = connect;
            kind := W.DBT_DEVICEREMOVECOMPLETE;
            if connect kind = W.DBT_DEVICEARRIVAL;
            assert(decode(cast(u64) kind, *packet.header, changed));
        }
    }

    assert(calls == 4);

    for bytes: 0..29 {
        packet.header.dbch_size = cast(u32) bytes;
        assert(!decode(W.DBT_DEVICEARRIVAL, *packet.header, changed));
    }

    packet.header.dbch_size = 30;
    assert(!decode(W.DBT_DEVICEARRIVAL, *packet.header, changed));
    packet.header.dbch_size = 31;
    assert(!decode(W.DBT_DEVICEARRIVAL, *packet.header, changed));
    packet.header.dbch_size = size_of(Packet);
    packet.header.dbch_devicetype = 0;
    assert(!decode(W.DBT_DEVICEARRIVAL, *packet.header, changed));
    packet.header.dbch_devicetype = W.DBT_DEVTYP_DEVICEINTERFACE;
    packet.class_guid = .{};
    assert(!decode(W.DBT_DEVICEARRIVAL, *packet.header, changed));
    packet.class_guid = W.GUID_DEVINTERFACE_HID;
    packet.class_guid.Data4[7] = 0;
    assert(!decode(W.DBT_DEVICEARRIVAL, *packet.header, changed));
    packet.class_guid = W.GUID_DEVINTERFACE_HID;
    assert(!decode(0, *packet.header, changed));
    assert(!decode(W.DBT_DEVICEARRIVAL, null, changed));
    assert(!decode(W.DBT_DEVICEARRIVAL, *packet.header, null));
    for *unit: packet.name unit.* = 0x03bb;

    assert(!decode(W.DBT_DEVICEARRIVAL, *packet.header, changed));
    assert(calls == 4);
    print("ok\n");
}
