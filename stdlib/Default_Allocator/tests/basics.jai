#import "Basic";
D :: #import "Default_Allocator";
Unchecked :: #import "Default_Allocator"(ENABLE_ASSERTS = false, ENABLE_VALIDATE_ARGS = false);

// Only the x64 implementation (heaps, as on the reference platforms) names a ready-made allocator;
// the others have the procedure alone.
#if CPU == .X64 && OS != .IOS {
    #assert #exists(D.allocator);
} else {
    #assert !#exists(D.allocator);
}

main :: () {
    memory := cast(*u8) D.allocator_proc(.ALLOCATE, 8, 0, null, null);

    assert(memory != null);
    memset(memory, 42, 8);
    grown := cast(*u8) D.allocator_proc(.RESIZE, 128, 8, memory, null);

    assert(grown != null);
    for i: 0..7 assert(grown[i] == 42);

    failed := D.allocator_proc(.RESIZE, 0x7fffffffffffffff, 128, grown, null);

    assert(failed == null && grown[0] == 42 && grown[7] == 42);
    caps := cast(Allocator_Caps) cast(u64) D.allocator_proc(.CAPS, 0, 0, null, null);

    assert((caps & (.FREE | .ACTUALLY_RESIZE | .MULTIPLE_THREADS)) == (.FREE | .ACTUALLY_RESIZE | .MULTIPLE_THREADS));
    #if CPU == .X64 && OS != .IOS {
        assert((caps & (.CREATE_HEAP | .IS_THIS_YOURS)) == (.CREATE_HEAP | .IS_THIS_YOURS));
    } else {
        assert((caps & (.CREATE_HEAP | .IS_THIS_YOURS)) == 0);
    }
    D.allocator_proc(.FREE, 0, 128, grown, null);
    D.unsafe_reset();
    small := Unchecked.allocator_proc(.ALLOCATE, 4, 0, null, null);

    assert(small != null);
    Unchecked.allocator_proc(.FREE, 0, 4, small, null);
    print("ok\n");
}
