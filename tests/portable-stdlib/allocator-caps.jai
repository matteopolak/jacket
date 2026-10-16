#import "Basic";
P :: #import "Pool";

main :: () {
    name: string;
    flags := cast(Allocator_Caps) cast(u64) P.pool_allocator_proc(.CAPS, 0, 0, *name, null);

    assert(name == "Pool");
    assert(flags == (.IS_THIS_YOURS | .HINT_I_AM_A_FAST_BUMP_ALLOCATOR));
    saved := name;
    pool: P.Pool;

    P.set_allocators(*pool);
    assert(P.get(*pool, 16) != null);
    P.release(*pool);
    assert(saved == "Pool" && saved.data == name.data);
    assert(P.pool_allocator_proc(.CAPS, 0, 0, null, null) == cast(*void) cast(u64) flags);

    name = "";
    heap := context.default_allocator;

    flags = cast(Allocator_Caps) cast(u64) heap.proc(.CAPS, 0, 0, *name, null);
    assert(name.count > 0);
    expected: Allocator_Caps = .FREE | .ACTUALLY_RESIZE | .MULTIPLE_THREADS | .HINT_I_AM_A_GENERAL_HEAP_ALLOCATOR;

    #if (CPU == .X64 && OS != .IOS) || OS == .NN_SWITCH2 {
        expected |= .CREATE_HEAP | .IS_THIS_YOURS;
    }

    assert(flags == expected);
    name = "";
    flags = cast(Allocator_Caps) cast(u64) temporary_allocator_proc(.CAPS, 0, 0, *name, null);
    assert(name.count > 0 && flags == .HINT_I_AM_PER_FRAME_TEMPORARY_STORAGE);

    #if OS == .MACOS || OS == .LINUX {
        name = "";
        flags = cast(Allocator_Caps) cast(u64) O.overwriting_allocator_proc(.CAPS, 0, 0, *name, null);
        assert(name.count > 0 && flags == (.MULTIPLE_THREADS | .FREE | .HINT_I_AM_A_DEBUG_ALLOCATOR));
        saved = name;
        name = "";
        flags = cast(Allocator_Caps) cast(u64) U.unmapping_allocator_proc(.CAPS, 0, 0, *name, null);
        assert(name.count > 0 && flags == (.MULTIPLE_THREADS | .FREE | .HINT_I_AM_A_DEBUG_ALLOCATOR));
        assert(saved == "Overwriting_Allocator");
    }

    #if CPU != .X64 && OS != .NN_SWITCH2 {
        name = "";
        flags = cast(Allocator_Caps) cast(u64) D.allocator_proc(.CAPS, 0, 0, *name, null);
        assert(name.count > 0 && flags == (.MULTIPLE_THREADS | .FREE | .ACTUALLY_RESIZE | .HINT_I_AM_A_GENERAL_HEAP_ALLOCATOR));
    }

    print("ok\n");
}

#scope_file
#if OS == .MACOS || OS == .LINUX {
    O :: #import "Overwriting_Allocator";
    U :: #import "Unmapping_Allocator";
}

#if CPU != .X64 && OS != .NN_SWITCH2 D :: #import "Default_Allocator";
