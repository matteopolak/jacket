#import "Basic";

main :: () {
    ts: Temporary_Storage_Paged_Alloc;
    bytes: [64] u8;

    ts.data = bytes.data;
    ts.size = bytes.count;
    context.temporary_storage = *ts.base;
    mark := get_temporary_storage_mark();
    allocator := context.allocator;

    {
        push_allocator(temp);
        a := cast(*u8) alloc(16);

        assert(a != null && cast(u64) a % 16 == 0);
        memset(a, 99, 16);
        big := cast(*u8) alloc(8192);

        assert(big != null);
        big[8191] = 55;
        assert(a[0] == 99);
    }

    assert(context.allocator.proc == allocator.proc && context.allocator.data == allocator.data);
    assert(ts.total_bytes_occupied >= 8208);
    assert(ts.overflow_pages != null);
    set_temporary_storage_mark(mark);
    assert(ts.total_bytes_occupied == 0 && ts.overflow_pages == null);

    {
        auto_release_temp();
        talloc(8192);
        assert(ts.total_bytes_occupied >= 8192);
    }

    assert(ts.total_bytes_occupied == 0 && ts.overflow_pages == null);
    reset_temporary_storage();
    print("ok\n");
}
