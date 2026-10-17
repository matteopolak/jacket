M :: #import "macOS";
#load "../../../../internal/bindings/fsevents-context/module.jai";

probe :: #library,system "fsevents_context_probe";
lifetime :: (create: #type (allocator: M.CFAllocatorRef, callback: M.FSEventStreamCallback, _context: *M.FSEventStreamContext, pathsToWatch: M.CFArrayRef, sinceWhen: M.FSEventStreamEventId, latency: M.CFTimeInterval, flags: M.FSEventStreamCreateFlags) -> M.ConstFSEventStreamRef #c_call) #foreign probe "binding_fsevents_lifetime";
delivery :: (create: #type (allocator: M.CFAllocatorRef, callback: M.FSEventStreamCallback, _context: *M.FSEventStreamContext, pathsToWatch: M.CFArrayRef, sinceWhen: M.FSEventStreamEventId, latency: M.CFTimeInterval, flags: M.FSEventStreamCreateFlags) -> M.ConstFSEventStreamRef #c_call) #foreign probe "binding_fsevents_delivery";

#assert size_of(M.FSEventStreamContext) == 40;
#assert offset_of(M.FSEventStreamContext, "copyDescription") == 0;
#assert offset_of(M.FSEventStreamContext, "info") == 8;
#assert offset_of(M.FSEventStreamContext, "retain") == 16;
#assert offset_of(M.FSEventStreamContext, "release") == 24;
#assert offset_of(M.FSEventStreamContext, "version") == 32;
#assert size_of(Context) == 40;
#assert offset_of(Context, "version") == 0;
#assert offset_of(Context, "info") == 8;
#assert offset_of(Context, "retain") == 16;
#assert offset_of(Context, "release") == 24;
#assert offset_of(Context, "copyDescription") == 32;

main :: () {
    lifetime(M.FSEventStreamCreate);
    delivery(M.FSEventStreamCreate);
}
