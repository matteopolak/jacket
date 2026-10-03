#assert OS == .MACOS || OS == .LINUX || OS == .ANDROID || OS == .WINDOWS;
Unmapping_Allocator :: Impl.Unmapping_Allocator;
get_unmapping_allocator :: Impl.get_unmapping_allocator;
unmapping_allocator_proc :: Impl.unmapping_allocator_proc;
deinit :: Impl.deinit;
deinit_unmapping_allocator :: Impl.deinit_unmapping_allocator;

#scope_file
Impl :: #import "internal/allocation/unmapping";
