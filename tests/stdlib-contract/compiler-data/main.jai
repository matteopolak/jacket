#import "Basic";
C :: #import "Compiler";

register :: () -> [] u8 {
    handle, discrete := C.add_data_segment("orchid_data", .READ | .WRITE, alignment=32);
    assert(!discrete);
    source: [3] u8 = .[13, 29, 47];
    result := C.add_global_data(source, .USER_SEGMENT, handle);
    source[0] = 99;
    snapshot := C.get_runtime_info();
    assert(snapshot.global_data_info != null);
    found := false;
    for entry: snapshot.global_data_info.segment_info {
        if entry.data.data == result.data {
            assert(entry.segment_tag == .USER && entry.data[0] == 13);
            found = true;
        }
    }
    assert(found);
    return result;
}

registered :: #run register();
alias :: registered;
readonly :: #run C.add_global_data(u8.[5, 7], .READ_ONLY);
zeroed :: #run C.add_global_data(u8.[0, 0], .BSS);
segment_handle, discrete_segment :: #run C.add_data_segment("later_registration", alignment=64);
handle_alias :: segment_handle;
later_data :: #run C.add_global_data(u8.[61], .USER_SEGMENT, handle_alias);
#run {
    temporary := registered;
    temporary[0] = 88;
}

main :: () {
    assert(registered.count == 3 && registered[0] == 13 && registered[2] == 47);
    assert(alias.data == registered.data);
    assert(((cast(u64) registered.data) & 31) == 0);
    writable := registered;
    writable[0] = 23;
    assert(alias[0] == 23);
    assert(readonly.count == 2 && readonly[1] == 7);
    assert(zeroed.count == 2 && zeroed[0] == 0);
    assert(!discrete_segment && later_data.count == 1 && later_data[0] == 61);
    assert(((cast(u64) later_data.data) & 63) == 0);
    info := C.get_runtime_info();
    assert(info.global_data_info != null);
    assert(info.global_data_info.segment_info.count > 0);
    assert(C.get_type_table().count == info.type_table.count);
    found := false;
    for segment: info.global_data_info.segment_info {
        if segment.data.data == registered.data {
            assert(segment.segment_tag == .USER && segment.data.count == 3 && segment.data[0] == 23);
            found = true;
        }
    }
    assert(found);
    print("ok\n");
}
