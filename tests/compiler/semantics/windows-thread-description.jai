// Original SDK API regression: naming must change the real kernel thread object.
kernel32 :: #system_library "kernel32";
CreateThread :: (security: *void, stack_size: u64, entry: *void, data: *void, flags: u32, id: *u32) -> *void #foreign kernel32;
WaitForSingleObject :: (handle: *void, timeout: u32) -> u32 #foreign kernel32;
ResumeThread :: (handle: *void) -> u32 #foreign kernel32;
CloseHandle :: (handle: *void) -> s32 #foreign kernel32;
GetThreadId :: (handle: *void) -> u32 #foreign kernel32;
GetCurrentThread :: () -> *void #foreign kernel32;
GetCurrentThreadId :: () -> u32 #foreign kernel32;
SetThreadDescription :: (handle: *void, description: *u16) -> s32 #foreign kernel32;
GetThreadDescription :: (handle: *void, description: **u16) -> s32 #foreign kernel32;
LocalFree :: (memory: *void) -> *void #foreign kernel32;
DuplicateHandle :: (process: *void, source: *void, target: *void, output: **void, access: u32, inherit: s32, flags: u32) -> s32 #foreign kernel32;
GetCurrentProcess :: () -> *void #foreign kernel32;
SetHandleInformation :: (handle: *void, mask: u32, flags: u32) -> s32 #foreign kernel32;
GetLastError :: () -> u32 #foreign kernel32;

same :: (handle: *void, expected: *u16) -> bool #no_context {
    output: *u16;
    if GetThreadDescription(handle, *output) < 0 || output == null return false;

    equal := true;
    i := 0;

    while output[i] != 0 || expected[i] != 0 {
        if output[i] != expected[i] {
            equal = false;
            break;
        }

        i += 1;
    }

    return LocalFree(output) == null && equal;
}

worker :: (data: *s32) -> u32 #c_call {
    self := GetCurrentThread();
    name: [4] u16 = .[0x96ea, 0xd83d, 0xde80, 0];

    if SetThreadDescription(self, name.data) < 0 || !same(self, name.data) return 1;

    data.* = 1;

    return 17;
}

check :: () -> s32 #no_context {
    first: [3] u16 = .[65, 0x96ea, 0];
    second: [2] u16 = .[66, 0];
    empty: [1] u16;
    current := GetCurrentThread();
    previous: *u16;

    if GetThreadDescription(current, *previous) < 0 || previous == null return 19;
    if SetThreadDescription(current, first.data) < 0 || !same(current, first.data) return 1;

    CloseHandle(current);
    if !same(current, first.data) return 20;
    if GetThreadId(current) != GetCurrentThreadId() return 2;

    value: s32;
    id: u32;
    thread := CreateThread(null, 0, cast(*void) worker, *value, 4, *id);

    if thread == null || id == 0 || GetThreadId(thread) != id return 3;
    if WaitForSingleObject(thread, 0) != 258 || value != 0 return 4;
    if !same(thread, empty.data) return 5;
    if SetThreadDescription(thread, first.data) < 0 || !same(thread, first.data) return 6;

    duplicate: *void;
    process := GetCurrentProcess();

    if DuplicateHandle(process, thread, process, *duplicate, 0, 0, 2) == 0 return 7;
    if !same(duplicate, first.data) return 8;

    query_only: *void;
    if DuplicateHandle(process, thread, process, *query_only, 0x800, 0, 0) == 0 return 25;
    if !same(query_only, first.data) || SetThreadDescription(query_only, second.data) >= 0 return 26;
    if CloseHandle(query_only) == 0 return 27;
    if SetThreadDescription(duplicate, second.data) < 0 || !same(thread, second.data) return 9;
    if SetThreadDescription(thread, empty.data) < 0 || !same(duplicate, empty.data) return 10;
    if SetHandleInformation(thread, 2, 2) == 0 return 22;
    if SetHandleInformation(duplicate, 2, 2) == 0 || CloseHandle(duplicate) != 0 return 28;

    expected_error := GetLastError();
    if CloseHandle(thread) != 0 || GetLastError() != expected_error return 23;
    if !same(thread, empty.data) || GetThreadId(thread) != id return 29;
    if SetHandleInformation(duplicate, 2, 0) == 0 return 30;
    if SetHandleInformation(thread, 2, 0) == 0 return 24;
    if ResumeThread(thread) != 1 || WaitForSingleObject(thread, 5000) != 0 || value != 1 return 11;

    worker_name: [4] u16 = .[0x96ea, 0xd83d, 0xde80, 0];
    if !same(thread, worker_name.data) || !same(duplicate, worker_name.data) return 12;
    if !same(current, first.data) return 13;
    if CloseHandle(thread) == 0 || !same(duplicate, worker_name.data) return 14;

    output: *u16;
    if SetThreadDescription(thread, first.data) >= 0 || GetThreadDescription(thread, *output) >= 0 return 15;
    if CloseHandle(duplicate) == 0 return 16;
    if SetThreadDescription(null, first.data) >= 0 || GetThreadDescription(null, *output) >= 0 return 17;
    if SetThreadDescription(cast(*void) -1, first.data) >= 0 return 18;
    if SetThreadDescription(current, previous) < 0 || LocalFree(previous) != null return 21;

    return 0;
}

#program_export "thread_description_check"
main :: () -> s32 #c_call {
    return check();
}
