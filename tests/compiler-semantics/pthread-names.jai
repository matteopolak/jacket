// Independent public pthread ABI control. The test driver defines DARWIN.
pthread_self :: () -> u64 #foreign;
pthread_getname_np :: (thread: u64, name: *u8, size: u64) -> s32 #foreign;
pthread_create :: (thread: *u64, attributes: *void, entry: *void, data: *void) -> s32 #foreign;
pthread_join :: (thread: u64, result: **void) -> s32 #foreign;

#if DARWIN {
    pthread_setname_np :: (name: *u8) -> s32 #foreign;
} else {
    pthread_setname_np :: (thread: u64, name: *u8) -> s32 #foreign;
}

set_name :: (name: *u8) -> s32 #no_context {
    #if DARWIN {
        return pthread_setname_np(name);
    } else {
        return pthread_setname_np(pthread_self(), name);
    }
}

same_handle :: (thread: u64, expected: *u8) -> bool #no_context {
    actual: [64] u8;
    if pthread_getname_np(thread, actual.data, 64) != 0 return false;

    for 0..63 {
        if actual[it] != expected[it] return false;
        if actual[it] == 0 return true;
    }

    return false;
}

same_name :: (expected: *u8) -> bool #no_context {
    return same_handle(pthread_self(), expected);
}

worker :: (data: *void) -> *void #c_call {
    name: [7] u8 = .[119, 111, 114, 107, 101, 114, 0];
    if set_name(name.data) != 0 || !same_name(name.data) return null;
    return cast(*void) 17;
}

check :: () -> s64 #no_context {
    previous: [64] u8;
    if pthread_getname_np(pthread_self(), previous.data, 64) != 0 return 1;

    name: [6] u8 = .[112, 114, 111, 98, 101, 0];
    named := set_name(name.data) == 0 && same_name(name.data);
    too_long: [65] u8;

    for 0..63 too_long[it] = 65;

    rejected := set_name(too_long.data) != 0 && same_name(name.data);
    small: [1] u8;
    short_status := pthread_getname_np(pthread_self(), small.data, 1);
    short_buffer: bool;

    #if DARWIN {
        short_buffer = short_status == 0 && small[0] == 0;
    } else {
        short_buffer = short_status != 0;
    }

    restored := set_name(previous.data) == 0;

    if !named return 2;
    if !rejected return 3;
    if !short_buffer return 4;
    if !restored return 5;

    thread: u64;
    if pthread_create(*thread, null, cast(*void) worker, null) != 0 return 6;

    observed: [64] u8;
    if pthread_getname_np(thread, observed.data, 64) != 0 return 7;

    #if !DARWIN {
        if pthread_setname_np(thread, name.data) != 0 || !same_handle(thread, name.data) return 8;
    }

    result: *void;
    if pthread_join(thread, *result) != 0 || result != cast(*void) 17 return 9;

    return 0;
}
