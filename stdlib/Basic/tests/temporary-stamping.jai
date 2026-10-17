<<<<<<< HEAD:tests/portable-stdlib/temporary-stamping.jai
B :: #import "Basic"()(TEMP_ALLOCATOR_OVERWRITE_MEMORY = true);

Stamped :: B;
=======
B :: #import "Basic";
Stamped :: #import "Basic"()(TEMP_ALLOCATOR_OVERWRITE_MEMORY = true);
>>>>>>> main:stdlib/Basic/tests/temporary-stamping.jai

main :: () {
    initial: [256] u8;
    storage: Stamped.Temporary_Storage_Paged_Alloc;
    storage.data = initial.data;
    storage.size = initial.count;
    tracker := Tracker.{ underlying = context.allocator };

    storage.overflow_allocator = .{ capture_free, *tracker };

    push_context {
        context.temporary_storage = *storage;
        first := cast(*u8) Stamped.talloc(16);
        mark := Stamped.get_temporary_storage_mark();
        second := cast(*u8) Stamped.talloc(32);

        for i: 0..31 second[i] = 1;

        Stamped.set_temporary_storage_mark(mark);
        for i: 0..31 B.assert(second[i] == Stamped.SET_STORAGE_MARK_STAMP);

        B.assert(storage.current_page_bytes_occupied == mark.current_page_bytes_occupied);
        overflow := cast(*u8) Stamped.talloc(512);
        overflow[0] = 17;
        Stamped.set_temporary_storage_mark(mark);
        B.assert(tracker.freed == 1 && storage.overflow_pages == null);
        for i: 0..15 first[i] = 2;

        Stamped.reset_temporary_storage();
        for i: 0..15 B.assert(first[i] == Stamped.RESET_STORAGE_STAMP);

        B.assert(storage.current_page_bytes_occupied == 0);
    }

    B.print("ok\n");
}

Tracker :: struct {
    underlying: Allocator;
    freed: int;
}

capture_free :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    tracker := cast(*Tracker) allocator_data;

    if mode == .FREE {
        header := B.align_forward(size_of(Stamped.Overflow_Page), 32);
        data := cast(*u8) old_memory + header;
        for i: 0..old_size - header - 1 B.assert(data[i] == Stamped.FREE_PAGE_STAMP);
        tracker.freed += 1;
    }

    return tracker.underlying.proc(mode, requested_size, old_size, old_memory, tracker.underlying.data);
}
