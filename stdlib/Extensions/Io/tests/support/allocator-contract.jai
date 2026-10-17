// Objects can be destroyed by a fiber whose active allocator is not their owner.
#import "Basic";

Owner :: struct {
    backing: Allocator;
    live: int;
    allocated: int;
    freed: int;
}

Allocation :: struct {
    memory: *void;
    owner: *Owner;
}

records: [4096] Allocation;
first: Owner;
second: Owner;
group: Io.Wait_Group;

tracked :: (mode: Allocator_Mode, size: s64, old_size: s64, memory: *void, data: *void) -> *void {
    owner := cast(*Owner) data;
    slot := -1;

    if memory && (mode == .FREE || mode == .RESIZE) {
        for records {
            if it.memory == memory {
                assert(it.owner == owner, "allocation released through a different owner");
                slot = it_index;
                break;
            }
        }

        assert(slot >= 0, "release of an unknown allocation");
    }

    result := owner.backing.proc(mode, size, old_size, memory, owner.backing.data);

    if mode == .FREE && memory {
        records[slot] = .{};
        owner.live -= 1;
        owner.freed += 1;
    }

    if mode == .RESIZE && result && memory records[slot].memory = result;

    if result && (mode == .ALLOCATE || (mode == .RESIZE && !memory)) {
        for *record: records {
            if !record.memory {
                record.* = .{ result, owner };
                owner.live += 1;
                owner.allocated += 1;

                return result;
            }
        }

        assert(false, "allocation tracking capacity exhausted");
    }

    return result;
}

worker :: () {
    assert(context.allocator.data == *second, "spawner allocator must be inherited");
    local := New(int);

    free(local);

    // Overflow pages belong to the scheduler, not this fiber's active allocator.
    text := cast(*u8) talloc(12000);
    text[11999] = 73;
    Io.yield();
    assert(text[11999] == 73);
    Io.done(*group);
}

root :: () {
    channel := Io.make_channel(int, 4);
    left, right := Io.make_pipe();

    {
        push_allocator(Allocator.{ tracked, *second });
        Io.free_channel(channel);
        assert(context.allocator.data == *second, "free must restore the caller allocator");

        payload: [3] u8 = .[11, 22, 33];
        count, error := Io.write(left, payload);

        assert(count == 3 && error == .NONE);
        buffer: [3] u8;

        count, error = Io.read(right, buffer);
        assert(count == 3 && error == .NONE && buffer[2] == 33);
        assert(Io.close(left) == .NONE);
        assert(Io.close(right) == .NONE);
        assert(context.allocator.data == *second);

        // Engine operation records outlive the calling fiber's allocator scope.
        file, open_error := Io.open_file(ifx OS == .WINDOWS then "NUL" else "/dev/null");

        assert(open_error == .NONE);
        bytes: [32] u8;
        read_count, read_error := Io.read(*file, bytes);

        assert(read_count == 0 && read_error == .NONE);
        assert(Io.close(*file) == .NONE);

        address, resolve_error := Io.resolve("localhost", 4242);

        assert(resolve_error == .NONE && address.port == 4242);

        // Exceed the retained stack limit, then exercise a retained fiber again.
        for 1..3 {
            Io.add(*group, 4);
            for 1..4 Io.spawn(worker);
            assert(Io.wait(*group) == .NONE);
        }
    }

    assert(context.allocator.data == *first);
}

main :: () {
    first.backing = context.allocator;
    second.backing = context.allocator;

    {
        push_allocator(Allocator.{ tracked, *first });
        Io.io_run(root);
    }

    assert(first.allocated > 0 && second.allocated > 0);
    assert(first.live == 0 && second.live == 0, "all tracked allocations must be freed: %, %", first.live, second.live);
    assert(first.allocated == first.freed && second.allocated == second.freed);
    print("ok\n");
}
