// What every program has without importing anything: allocators, source locations, logging,
// type information and the compiler's intrinsics.

// Single bootstrap entry; subsystem files share this module scope.
#load "platform.jai";
#load "allocation.jai";
#load "storage.jai";
#load "reflection.jai";
#load "diagnostics.jai";
#load "intrinsics.jai";
#load "context.jai";
