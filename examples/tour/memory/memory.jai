// Manual memory, made pleasant: defer, the context, and allocators.

tour_memory :: () {
    // `defer` runs code when the scope ends, in reverse order.
    {
        print("Entering a scope...\n");
        defer print("  ...and deferred 1 runs last.\n");
        defer print("  ...deferred 2 runs before it,\n");
        print("  doing some work,\n");
    }

    // New and free: a heap object you own.
    Node :: struct {
        value: int;
        next: *Node;
    }
    head: *Node;
    for 1..5 {
        node := New(Node);
        node.value = it * it;
        node.next = head;
        head = node;
    }
    print("A hand-made linked list: ");
    walker := head;
    while walker {
        print("% ", walker.value);
        walker = walker.next;
    }
    print("\n");
    while head {
        next := head.next;
        free(head);
        head = next;
    }

    // The temporary allocator is a scratch pad: allocate freely, then reset it all at once.
    // (main calls reset_temporary_storage() after every stop of the tour.)
    scratch := tprint("% + % = %", 2, 2, 4);
    print("Made \"%\" in temporary storage, nothing to free.\n", scratch);

    // Every procedure receives an implicit `context`. Change the allocator in it,
    // and everything called below uses your allocator, even library code.
    counter: Counting_Allocator;
    {
        push_allocator(counting_allocator_proc, *counter);
        numbers: [..] int;
        for 1..100 array_add(*numbers, it);
        greeting := sprint("hello from %", "the context");
        free(greeting);
        array_free(numbers);
    }
    print("The counting allocator saw % allocations, % frees and % resizes (% bytes).\n",
        counter.allocations, counter.frees, counter.resizes, counter.bytes);

    // push_context swaps the whole context for a block: here, a custom logger.
    new_context := context;
    new_context.logger = shouting_logger;
    push_context new_context {
        log("the logger is just a procedure in the context");
    }
    log("and it is back to normal now");
}

Counting_Allocator :: struct {
    allocations, frees, resizes, bytes: int;
}

// An allocator is a procedure plus a data pointer. This one keeps a tally,
// then hands the real work to the default heap allocator.
counting_allocator_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void,
    allocator_data: *void) -> *void {
    tally := cast(*Counting_Allocator) allocator_data;
    if mode == {
        case .ALLOCATE;
            tally.allocations += 1;
            tally.bytes += size;
        case .RESIZE;
            tally.resizes += 1;
            tally.bytes += size;
        case .FREE;
            tally.frees += 1;
    }
    heap := context.default_allocator;
    return heap.proc(mode, size, old_size, old_memory, heap.data);
}

shouting_logger :: (message: string, data: *void, info: Log_Info) {
    push_allocator(temp);
    print("  LOG: %!!!\n", to_upper_copy(message));
}
