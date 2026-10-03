#module_parameters(LOAD_THREAD_GROUP := true)(CACHE_LINE_SIZE: s32 = 64, DO_RACE_DETECTOR := false, SET_THREAD_NAMES := true);
#assert !DO_RACE_DETECTOR;

#load "types.jai";
#if OS == .WINDOWS #load "native-windows.jai";
else #load "synchronization.jai";
#load "lifecycle.jai";
#if LOAD_THREAD_GROUP #load "group.jai";
#scope_module
#import "Basic";
#if OS == .WINDOWS W :: #import "Windows";
else P :: #import "POSIX";
#if OS == .MACOS M :: #import "internal/thread/native-macos";

THREAD_DEBUG :: #exists(DEBUG) && DEBUG;
#assert !THREAD_DEBUG;
