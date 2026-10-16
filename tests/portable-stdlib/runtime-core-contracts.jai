// Reviewed Runtime_Support records/storage and actual primitive behavior.
#import "Basic";
R :: #import "Runtime_Support";
Again :: #import "Runtime_Support";

main :: () {
    #assert R.OS_IS_UNIX == (OS == .MACOS || OS == .LINUX || OS == .PS5 || OS == .IOS || OS == .ANDROID);
    assert(*R.synch_initted == *Again.synch_initted);
    assert(R.synch_initted == 0);
    R.synch_initted = 17;
    assert(Again.synch_initted == 17);
    R.synch_initted = 0;

    bytes: [32] u8;
    for *byte: bytes byte.* = 0xa5;

    bytes[4] = 7;
    bytes[5] = 0;
    bytes[6] = 255;
    R.__element_duplicate(bytes.data + 4, 5, 3);
    for i: 0..3 assert(bytes[i] == 0xa5);

    for i: 0..4 {
        assert(bytes[4 + i * 3] == 7);
        assert(bytes[5 + i * 3] == 0);
        assert(bytes[6 + i * 3] == 255);
    }

    for i: 19..31 assert(bytes[i] == 0xa5);

    R.__element_duplicate(bytes.data + 4, 1, 3);
    assert(bytes[4] == 7 && bytes[6] == 255);

    text := u8.[65, 0, 66, 0];

    assert(R.c_style_strlen(text.data) == 1);
    view := R.to_string(text.data);

    assert(view.count == 1 && view.data == text.data && view == "A");
    text[0] = 67;
    assert(view == "C");

    base: R.Context_Base;

    assert(base.allocator.proc == R.Context_Base.default_allocator.proc);
    assert(base.logger == R.runtime_support_default_logger && base.assertion_failed == R.runtime_support_assertion_failed);
    assert(!base.context_info && !base.temporary_storage && !base.stack_trace && !base.handling_assertion_failure);
    info := cast(*Type_Info_Struct) type_info(R.Context_Base);

    assert(info.name == "Context_Base");
    storage: R.Temporary_Storage;

    assert(type_of(storage.implementation_data) == [256] u8);
    assert(!storage.data && storage.size == 0 && storage.total_bytes_occupied == 0 && storage.high_water_mark == 0);
    assert((cast(*Type_Info_Struct) type_info(R.Temporary_Storage)).name == "Temporary_Storage");

    sync: s32;
    calls: s64;
    value: s64;

    for 0..31 {
        R.one_time_init(*sync, #code {
            calls += 1;
            value = 37;
        });
        assert(calls == 1 && value == 37);
    }

    write_string("ok\n");
}
