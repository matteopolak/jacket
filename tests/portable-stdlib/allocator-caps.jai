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
    assert(flags == (.FREE | .ACTUALLY_RESIZE | .MULTIPLE_THREADS | .HINT_I_AM_A_GENERAL_HEAP_ALLOCATOR));
    print("ok\n");
}
