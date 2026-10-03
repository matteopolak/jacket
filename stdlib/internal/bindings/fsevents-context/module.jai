// SDK 27.0 FSEvents.h. Opaque CF/stream objects are represented only by pointer;
// all callback parameters, returns and conventions have the native C ABI.
#assert OS == .MACOS;

Retain :: #type (info: *void) -> *void #c_call;
Release :: #type (info: *void) #c_call;
CopyDescription :: #type (info: *void) -> *void #c_call;
Callback :: #type (stream: *void, info: *void, count: u64, paths: *void, flags: *u32, ids: *u64) #c_call;

Context :: struct {
    version: s64;
    info: *void;
    retain: Retain;
    release: Release;
    copyDescription: CopyDescription;
}

create :: (allocator: *void, callback: Callback, context: *Context, paths: *void, since: u64, latency: float64, flags: u32) -> *void #foreign library "FSEventStreamCreate";

#scope_file
library :: #library,system "CoreServices";
