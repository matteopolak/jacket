// Original compiler-worker reproducer: interpret on native Windows.
// Required result: start suspended, ResumeThread releases it, wait completes.
#import "Basic";
W :: #import "Windows";

callback :: (data: *void) -> u32 #c_call {
    return 17;
}

main :: () {
    id: u32;
    handle := cast(W.HANDLE) begin_thread(null, 0, cast(*void) callback, null, 4, *id);

    assert(handle != null);
    assert(W.WaitForSingleObject(handle, 0) == 258);
    assert(W.ResumeThread(handle) == 1);
    assert(W.WaitForSingleObject(handle, 5000) == 0);
    assert(W.CloseHandle(handle) != 0);
    print("ok\n");
}

#scope_file
libc :: #system_library "libc";
begin_thread :: (security: *void, stack_size: u32, entry: *void, argument: *void, flags: u32, id: *u32) -> u64 #foreign libc "_beginthreadex";
