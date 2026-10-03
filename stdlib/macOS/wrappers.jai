// CF Create-rule results belong to the caller. Converted Jai bytes use the
// current context allocator and preserve embedded NULs.
// The SDK copies the context synchronously and manages info with the supplied
// C callbacks. Only storage order changes; ownership and callbacks stay native.
FSEventStreamCreate :: (allocator: CFAllocatorRef, callback: FSEventStreamCallback, _context: *FSEventStreamContext, pathsToWatch: CFArrayRef, sinceWhen: FSEventStreamEventId, latency: CFTimeInterval, flags: FSEventStreamCreateFlags) -> ConstFSEventStreamRef #c_call {
    native: NativeEvents.Context;
    pointer: *NativeEvents.Context;

    if _context {
        native.version = _context.version;
        native.info = _context.info;
        native.retain = cast(NativeEvents.Retain) _context.retain;
        native.release = cast(NativeEvents.Release) _context.release;
        native.copyDescription = cast(NativeEvents.CopyDescription) _context.copyDescription;
        pointer = *native;
    }

    return cast(ConstFSEventStreamRef) NativeEvents.create(allocator, cast(NativeEvents.Callback) callback,
                                                          pointer, pathsToWatch, sinceWhen, latency, flags);
}

cfstring_from_string :: (str: string) -> CFStringRef {
    return CFStringCreateWithBytes(null, str.data, str.count, .UTF8, 0);
}

to_string :: (cfstring: CFStringRef) -> string {
    length := CFStringGetLength(cfstring);
    if length == 0 return "";

    capacity := CFStringGetMaximumSizeForEncoding(length, .UTF8);
    if capacity < 0 binding_abort();

    memory := context.allocator.proc(.ALLOCATE, capacity, 0, null, context.allocator.data);
    if !memory binding_abort();

    used: CFIndex;
    converted := CFStringGetBytes(cfstring, CFRange.{ 0, length }, .UTF8, 0, 0,
                                  cast(*u8) memory, capacity, *used);

    if converted != length {
        context.allocator.proc(.FREE, 0, 0, memory, context.allocator.data);
        binding_abort();
    }

    return .{ data = cast(*u8) memory, count = used };
}

#scope_file
NativeEvents :: #import "internal/bindings/fsevents-context";

libc :: #library,system "c";
binding_abort :: () #foreign libc "abort";
