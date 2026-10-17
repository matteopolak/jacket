// Original SDK contract: real kernel aliases retain the interpreted entry result.
kernel :: #system_library "kernel32";
crt :: #system_library "ucrtbase";
CreateThread :: (security: *void, stack: u64, entry: *void, data: *void, flags: u32, id: *u32) -> *void #foreign kernel;
begin_thread :: (security: *void, stack: u32, entry: *void, data: *void, flags: u32, id: *u32) -> *void #foreign crt "_beginthreadex";
GetCurrentProcess :: () -> *void #foreign kernel;
DuplicateHandle :: (source_process: *void, source: *void, target_process: *void, target: **void, access: u32, inherit: s32, flags: u32) -> s32 #foreign kernel;
GetExitCodeThread :: (thread: *void, code: *u32) -> s32 #foreign kernel;
WaitForSingleObject :: (handle: *void, milliseconds: u32) -> u32 #foreign kernel;
CloseHandle :: (handle: *void) -> s32 #foreign kernel;
ResumeThread :: (thread: *void) -> u32 #foreign kernel;
Sleep :: (milliseconds: u32) #foreign kernel;
ExitThread :: (code: u32) #foreign kernel;
end_thread :: (code: u32) #foreign crt "_endthreadex";

worker :: (value: *u32) -> u32 #c_call {
    Sleep(20);
    return value.*;
}

explicit_exit :: (value: *u32) -> u32 #c_call {
    Sleep(20);
    ExitThread(value.*);
    return 0;
}

explicit_crt_exit :: (value: *u32) -> u32 #c_call {
    Sleep(20);
    end_thread(value.*);
    return 0;
}

one :: (entry: *void, use_crt: bool, expected: u32) -> s32 #no_context {
    id: u32;
    handle: *void;

    if use_crt handle = begin_thread(null, 0, entry, *expected, 4, *id);
    else handle = CreateThread(null, 0, entry, *expected, 4, *id);
    if handle == null || id == 0 return 1;

    process := GetCurrentProcess();
    first: *void;
    second: *void;
    denied: *void;

    if DuplicateHandle(process, handle, process, *first, 0, 0, 2) == 0 return 2;
    if DuplicateHandle(process, first, process, *second, 0, 0, 2) == 0 return 3;
    if DuplicateHandle(process, first, process, *denied, 0, 0, 0) == 0 return 4;

    code: u32;
    if GetExitCodeThread(first, *code) == 0 || code != 259 return 5;
    if GetExitCodeThread(denied, *code) != 0 return 6;
    if ResumeThread(handle) != 1 return 7;
    if CloseHandle(handle) == 0 return 8;
    if WaitForSingleObject(second, 5000) != 0 return 9;
    if GetExitCodeThread(second, *code) == 0 || code != expected return 10;
    if CloseHandle(first) == 0 return 11;
    if GetExitCodeThread(second, *code) == 0 || code != expected return 12;
    if CloseHandle(second) == 0 || CloseHandle(denied) == 0 return 13;
    if GetExitCodeThread(second, *code) != 0 return 14;

    return 0;
}

check :: () -> s32 #no_context {
    result := one(cast(*void) worker, false, 0x8123_4567);
    if result != 0 return result;

    result = one(cast(*void) worker, true, 259);
    if result != 0 return 20 + result;

    result = one(cast(*void) explicit_exit, false, 0x8765_4321);
    if result != 0 return 40 + result;

    result = one(cast(*void) explicit_crt_exit, true, 0xf123_4567);
    if result != 0 return 60 + result;

    return 0;
}

main :: () -> s32 #no_context {
    return check();
}
