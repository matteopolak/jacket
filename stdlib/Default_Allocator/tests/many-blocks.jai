#import "Basic";
D :: #import "Default_Allocator";

// Freeing and resizing blocks must not cost time proportional to the number of live blocks:
// a tree of a million nodes is freed oldest first.
COUNT :: 200000;

main :: () {
    start := current_time_monotonic();
    blocks := NewArray(COUNT, *u8);

    defer array_free(blocks);

    for index: 0..COUNT - 1 {
        block := cast(*u8) D.allocator_proc(.ALLOCATE, 16 + index % 48, 0, null, null);

        assert(block != null);
        block[0] = cast(u8) (index % 251);
        blocks[index] = block;
    }

    // Grow some blocks (they may move), then release the oldest first, then the rest.
    for index: 0..COUNT - 1 {
        if index % 7 != 0 continue;

        size := 16 + index % 48;
        grown := cast(*u8) D.allocator_proc(.RESIZE, size + 200, size, blocks[index], null);

        assert(grown != null && grown[0] == cast(u8) (index % 251));
        blocks[index] = grown;
    }

    #if CPU == .X64 && OS != .IOS {
        for index: 0..COUNT - 1 assert(D.is_this_yours(D.allocator_default_heap, blocks[index]));
    }

    for index: 0..COUNT - 1 {
        assert(blocks[index][0] == cast(u8) (index % 251));

        size := 16 + index % 48;
        if index % 7 == 0 size += 200;

        D.allocator_proc(.FREE, 0, size, blocks[index], null);
    }

    #if CPU == .X64 && OS != .IOS {
        for index: 0..COUNT - 1 assert(!D.is_this_yours(D.allocator_default_heap, blocks[index]));
    }

    elapsed, _ := to_float64_seconds(current_time_monotonic() - start);

    assert(elapsed < 20, "allocating and freeing % blocks took % seconds", COUNT, elapsed);
    print("ok\n");
}
