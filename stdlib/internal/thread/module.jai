#module_parameters(
    // Includes `Thread_Group`.
    LOAD_THREAD_GROUP := true
)(
    // The cache line size in bytes, which pads the records of workers.
    CACHE_LINE_SIZE: s32 = 64,
    // Reserved for a race detector; it must stay `false`.
    DO_RACE_DETECTOR := false,
    // Gives each thread's name to the operating system.
    SET_THREAD_NAMES := true
);

#assert !DO_RACE_DETECTOR;

#load "types.jai";
#if OS == .WINDOWS #load "native-windows.jai";
else #load "synchronization.jai";
#load "lifecycle.jai";
#if LOAD_THREAD_GROUP #load "group.jai";
#scope_module
#import "Basic";
#if OS == .WINDOWS W :: #import "internal/bindings/windows-api";
else P :: #import "internal/bindings/posix-api";
#if OS == .MACOS M :: #import "internal/thread/native-macos";

Crash :: #import "Runtime_Support_Crash_Handler";
#if OS == .WINDOWS Windows_Stack :: #import "internal/runtime/windows-stack-guarantee";

THREAD_DEBUG :: #exists(DEBUG) && DEBUG;
#assert !THREAD_DEBUG;
