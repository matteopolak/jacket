#import "Basic";
P :: #import "Pool";

main :: () {
    allocator := Allocator.{ P.pool_allocator_proc, null };
    flags, name := get_capabilities(allocator);

    assert(name.count > 0 && name.data != null);
    repeated_flags, repeated_name := get_capabilities(allocator);

    assert(flags == repeated_flags && name == repeated_name && name.data == repeated_name.data);
    print("ok\n");
}
