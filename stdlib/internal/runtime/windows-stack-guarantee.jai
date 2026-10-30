// Windows reports a stack overflow by running the exception dispatcher on the overflowing stack, and
// the room it leaves for that is not always enough: on processors with large extended register state
// (AVX-512, AMX) the dispatcher needs more than the default reserve, and the program ends with an
// access violation (0xC0000005) instead of a stack overflow (0xC00000FD). Reserving room for the
// calling thread makes the overflow report itself, with or without a crash handler.

// Bytes of stack kept for the dispatcher and whatever handles the exception on it. The crash handler
// (`internal/runtime/crash-native`) asks for the same amount.
STACK_GUARANTEE :: 128 * 1024;

// Reserves the room on the calling thread. Call it once at the start of every thread.
reserve :: () #c_call {
    bytes: u32 = STACK_GUARANTEE;

    SetThreadStackGuarantee(*bytes);
}

#scope_file
kernel32 :: #system_library "kernel32"; @native(external)
SetThreadStackGuarantee :: (bytes: *u32) -> s32 #foreign kernel32;
