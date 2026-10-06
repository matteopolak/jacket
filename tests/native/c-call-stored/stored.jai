// #c_call procedures that C reads from memory (a struct field, a global), not from its
// arguments, called on the calling thread and on threads C starts (interpreter and native).
#import "Basic";
stored :: #library "libstored";

Unary :: #type (x: s32) -> s32 #c_call;

Holder :: struct {
    bias: s32;
    apply: Unary;
}

Sorter :: struct {
    compare: (a: *void, b: *void) -> s32 #c_call;
}

call_holder :: (h: *Holder, x: s32) -> s32 #foreign stored;
call_stored :: (slot: *Unary, x: s32) -> s32 #foreign stored;
sort_ints :: (xs: *s32, n: s32, s: *Sorter) #foreign stored;
call_holder_on_thread :: (h: *Holder, x: s32) -> s32 #foreign stored;
start_background :: (h: *Holder, x: s32) #foreign stored;
background_result :: () -> s32 #foreign stored;

calls: s32;

triple :: (x: s32) -> s32 #c_call {
    calls += 1;
    return x * 3;
}

negate :: (x: s32) -> s32 #c_call {
    calls += 1;
    return -x;
}

descending :: (a: *void, b: *void) -> s32 #c_call {
    x := <<cast(*s32) a;
    y := <<cast(*s32) b;
    return y - x;
}

global_unary: Unary = negate;
global_holder := Holder.{ 1000, triple };

// Made by compile-time code, where the field also holds a thunk, and baked into the program.
make_holder :: () -> Holder {
    return .{ 10, negate };
}
baked_holder :: #run make_holder();

main :: () {
    holder := Holder.{ 100, triple };
    // Read back from memory and called by C on this thread.
    print("% % %\n", call_holder(*holder, 5), call_holder(*global_holder, 2), call_stored(*global_unary, 9));
    // Called by interpreted code through the stored pointers.
    stored_proc := holder.apply;
    print("% % % %\n", stored_proc(7), global_unary(4), holder.apply == triple, global_holder.apply == stored_proc);
    sorter := Sorter.{ descending };
    xs: [5] s32 = .[3, 9, 1, 7, 5];
    sort_ints(xs.data, xs.count, *sorter);
    print("%\n", xs);
    // C's own threads, while this one waits in C and while it runs Jai code.
    print("%\n", call_holder_on_thread(*holder, 11));
    start_background(*global_holder, 6);
    result := background_result();
    while result < 0 result = background_result();
    baked := baked_holder;
    print("% % %\n", call_holder(*baked, 3), result, calls);
}
