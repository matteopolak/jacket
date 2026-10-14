C :: #import "Curl";

libc :: #library,system "c";
abort :: () #foreign libc;
getcwd :: (buffer: *u8, size: u64) -> *u8 #foreign libc;
strlen :: (text: *u8) -> u64 #foreign libc;

check :: (value: bool) {
    if !value abort();
}

Sink :: struct {
    data: [1024] u8;
    count: u64;
}

receive :: (data: *u8, size: u64, nitems: u64, userdata: *void) -> u64 #c_call {
    sink := cast(*Sink) userdata;
    n := size * nitems;

    if n == 0 return 0;
    if sink.count + n > 1024 return 0;
    for i: 0..n - 1 sink.data[sink.count + i] = data[i];

    sink.count += n;

    return n;
}

main :: () {
    check(C.curl_global_init(C.CURL_GLOBAL_DEFAULT) == .OK);
    defer C.curl_global_cleanup();
    check(C.curl_version() != null);
    handle := C.curl_easy_init();

    check(handle != null);
    defer C.curl_easy_cleanup(handle);
    cwd: [4096] u8;

    check(getcwd(cwd.data, 4096) != null);
    url: [8192] u8;
    prefix := "file://";
    n := cast(s64) strlen(cwd.data);
    suffix := "/tests/native-bindings/curl-data.txt";

    for i: 0..prefix.count - 1 url[i] = prefix[i];
    for i: 0..n - 1 url[prefix.count + i] = cwd[i];
    for i: 0..suffix.count - 1 url[prefix.count + n + i] = suffix[i];

    sink: Sink;

    check(C.curl_easy_setopt(handle, .URL, url.data) == .OK);
    check(C.curl_easy_setopt(handle, .WRITEFUNCTION, receive) == .OK);
    check(C.curl_easy_setopt(handle, .WRITEDATA, *sink) == .OK);
    check(C.curl_easy_perform(handle) == .OK);
    expected := "clean-room native transfer\n";

    check(sink.count == cast(u64) expected.count);
    for i: 0..expected.count - 1 check(sink.data[i] == expected[i]);

    escaped := C.curl_easy_escape(handle, "a b", 3);

    check(escaped != null);
    defer C.curl_free(escaped);
    check(escaped[0] == #char "a" && escaped[1] == #char "%" && escaped[2] == #char "2" && escaped[3] == #char "0" && escaped[4] == #char "b");
    output: [64] u8;
    length := C.curl_msnprintf(output.data, 64, "%d-%s", 17, cast(*u8) "hello");

    check(length == 8 && output[0] == #char "1" && output[7] == #char "o");
    length = C.curl_msnprintf(output.data, 64, "%d:%u:%.2f:%s", cast(s8) -17, cast(u16) 65535, cast(float32) 1.25, "ok");
    promoted := "-17:65535:1.25:ok";

    check(length == promoted.count);
    for i: 0..promoted.count - 1 check(output[i] == promoted[i]);

    length = C.curl_msnprintf(output.data, 64, "literal");
    check(length == 7);
    allocated := C.curl_maprintf("%s", "allocated");

    check(allocated != null && strlen(allocated) == 9);
    C.curl_free(allocated);
}
