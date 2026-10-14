#import "Basic";
R :: #import "Runtime_Support";

main :: () {
    assert(context.temporary_storage != null);
    assert(context.context_info == type_info(#Context));
    mark := get_temporary_storage_mark();
    pointer := talloc(65536);

    assert(pointer != null);
    set_temporary_storage_mark(mark);
    other := R.__jai_runtime_init(0, null);

    assert(other != null && other.temporary_storage != null && other.context_info == type_info(#Context));
    R.__jai_runtime_fini(other);
    print("ok\n");
}
