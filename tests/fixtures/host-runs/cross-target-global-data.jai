// Global data and segments registered by compile-time code belong to the program being built,
// also when the build is a cross build, and the registering call may be several procedures deep.
#import "Basic";
C :: #import "Compiler";

register :: () -> [] u8 {
    handle, discrete := C.add_data_segment("fixture_data", .READ | .WRITE, alignment = 16);
    assert(!discrete);
    source: [3] u8 = .[13, 29, 47];
    return C.add_global_data(source, .USER_SEGMENT, handle);
}

registered :: #run register();
readonly :: #run C.add_global_data(u8.[5, 7], .READ_ONLY);

#run {
    temporary := registered;
    temporary[0] = 88;
}

main :: () {
    assert(registered.count == 3 && registered[0] == 13 && registered[2] == 47);
    assert(readonly.count == 2 && readonly[1] == 7);

    writable := registered;
    writable[0] = 23;

    found := false;
    info := C.get_runtime_info();

    for segment: info.global_data_info.segment_info {
        if segment.data.data == registered.data {
            assert(segment.segment_tag == .USER && segment.data.count == 3 && segment.data[0] == 23);
            found = true;
        }
    }

    assert(found);
    print("ok\n");
}
