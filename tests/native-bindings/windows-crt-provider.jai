#import "Basic";
W :: #import "Windows";

worker :: (data: *s32) -> u32 #c_call {
    data.* = 17;
    W._endthreadex(23);
    return 99;
}

main :: () {
    // The real Windows process environment contains SystemRoot. Both APIs
    // return CRT-owned storage, which the caller must not free.
    root := W.getenv("SystemRoot".data);
    assert(root != null);
    assert(root.* != 0);
    wide_name: [11] s16 = .[83, 121, 115, 116, 101, 109, 82, 111, 111, 116, 0];
    wide_root := W._wgetenv(*wide_name[0]);
    assert(wide_root != null);
    assert(wide_root.* != 0);

    value: s32;
    id: s32;
    handle := cast(W.HANDLE) W._beginthreadex(null, 0, cast(*void) worker, *value, cast(W.Creation_Flags) 0, *id);
    assert(handle != null);
    assert(id != 0);
    assert(W.WaitForSingleObject(handle, 5000) == W.WAIT_OBJECT_0);
    assert(value == 17);
    code: u32;
    assert(thread_exit_code(handle, *code) != 0);
    assert(code == 23);
    assert(W.CloseHandle(handle) != 0);
    print("windows-crt-provider: ok\n");
}

#scope_file
kernel :: #library,system "kernel32";
thread_exit_code :: (thread: W.HANDLE, code: *u32) -> s32 #foreign kernel "GetExitCodeThread";
