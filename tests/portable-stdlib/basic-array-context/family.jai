B :: #import "Basic";

Parcel :: struct {
    value: int = 42;
}

operator == :: (a: Parcel, b: Parcel) -> bool {
    return a.value % 10 == b.value % 10;
}

#poke_name B operator==;

exact :: (a: Parcel, b: Parcel) -> bool {
    return a.value == b.value;
}

Kind :: enum {
    FIRST;
    SECOND;
}

Tracker :: struct {
    backing: Allocator;
    pointers: [32] *void;
    sizes: [32] s64;
    live: s64;
    bytes: s64;
    allocations: s64;
    frees: s64;
    calls: s64;
}

tracking :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    state := cast(*Tracker) allocator_data;
    state.calls += 1;
    slot: s64 = -1;

    if old_memory {
        for pointer, i: state.pointers if pointer == old_memory {
            slot = i;
            break;
        }

        if mode == .FREE || mode == .RESIZE B.assert(slot >= 0, "foreign allocation");
    }

    result := state.backing.proc(mode, size, old_size, old_memory, state.backing.data);
    if (mode == .ALLOCATE || mode == .RESIZE) && result {
        if slot < 0 {
            for pointer, i: state.pointers if pointer == null {
                slot = i;
                break;
            }

            B.assert(slot >= 0, "tracker capacity");
            state.live += 1;
            state.allocations += 1;
        }

        state.bytes += size - state.sizes[slot];
        state.pointers[slot] = result;
        state.sizes[slot] = size;
    } else if mode == .FREE && old_memory {
        state.bytes -= state.sizes[slot];
        state.pointers[slot] = null;
        state.sizes[slot] = 0;
        state.live -= 1;
        state.frees += 1;
    }

    return result;
}

main :: () {
    B.assert(!#exists(B.copy_array_elements));
    B.assert(!#exists(B.Impl));
    B.assert(!#exists(B.Interface_Definition));
    original := context.allocator;
    owner: Tracker;
    owner.backing = original;
    other: Tracker;
    other.backing = original;
    allocator: Allocator = .{ tracking, *owner };
    alternate: Allocator = .{ tracking, *other };

    push_context {
        context.allocator = allocator;
        values: [..] Parcel;

        B.array_reserve(*values, 4);
        B.assert(values.allocated >= 4 && values.allocator.proc == tracking && values.allocator.data == *owner);
        capacity := values.allocated;

        B.array_reserve(cast(*Resizable_Array) *values, capacity, size_of(Parcel));
        first := B.array_add(*values);

        B.assert(first.value == 42);
        B.array_add(*values, Parcel.{ 12 });
        B.array_add(*values, Parcel.{ 23 }, Parcel.{ 34 });
        B.assert(values.count == 4);
        B.maybe_grow(cast(*Resizable_Array) *values, size_of(Parcel));
        B.assert(values.allocated > capacity && values[0].value == 42 && values[3].value == 34);
        added, index := B.array_add_if_unique(*values, Parcel.{ 2 });

        B.assert(!added && index == 0 && values.count == 4);
        added, index = B.array_add_if_unique(*values, Parcel.{ 5 });
        B.assert(added && index == 4 && values.count == 5);
        found, at := B.array_find(values, Parcel.{ 13 });

        B.assert(found && at == 2);
        found, at = B.array_find(values, Parcel.{ 3 }, compare = exact);
        B.assert(!found && at == -1);
        found, at = B.array_find(values, Parcel.{ 23 }, compare = exact);
        B.assert(found && at == 2);

        view: [] Parcel = .{ values.count, values.data };

        B.assert(B.array_ordered_remove_by_value(*view, Parcel.{ 3 }));
        B.assert(view.count == 4 && view[2].value == 34 && view[3].value == 5);
        B.assert(B.array_unordered_remove_by_value(*view, Parcel.{ 2 }, stop_after_first = true) == 1);
        B.assert(view.count == 3);
        B.assert(B.array_unordered_remove_by_value(*view, Parcel.{ 2 }) == 1 && view.count == 2);
        B.array_ordered_remove_by_index(*view, 0);
        B.assert(view.count == 1 && view[0].value == 34);
        B.array_unordered_remove_by_index(*view, 0);
        B.assert(view.count == 0);
        values.count = view.count;

        B.array_add(*values);
        data := values.data;

        capacity = values.allocated;
        B.array_reset_keeping_memory(*values);
        B.assert(values.count == 0 && values.data == data && values.allocated == capacity);
        B.array_resize(*values, 3);
        B.assert(values.count == 3 && values[0].value == 42 && values[2].value == 42);
        B.array_insert_at(*values, Parcel.{ 77 }, 1);
        B.assert(values.count == 4 && values[1].value == 77 && values[2].value == 42);
        B.assert(B.peek_pointer(values) == values.data + 3 && B.peek(values).value == 42);
        view = .{ values.count, values.data };
        B.assert(B.pop(*view).value == 42 && view.count == 3);
        values.count = view.count;
        tail := B.array_view(values, 1);
        bounded := B.array_view(values, 1, 99);

        B.assert(tail.count == 2 && bounded.count == 2 && tail.data == values.data + 1);
        B.assert(B.array_view(values, 99).count == 0);

        copied := B.array_copy(tail);

        B.assert(copied.count == 2 && copied[0].value == 77 && copied[1].value == 42 && copied.data != tail.data);
        B.array_resize(*copied, 4);
        B.assert(copied.count == 4 && copied[0].value == 77 && copied[2].value == 42 && copied[3].value == 42);
        B.array_resize(*copied, 2, initialize = false);
        B.assert(copied.count == 2 && copied[1].value == 42);
        B.array_reset(*copied);
        B.assert(copied.count == 0 && copied.data == null);
        B.array_resize(*copied, 2);
        B.assert(copied[0].value == 42 && copied[1].value == 42);
        B.array_resize(*copied, 0);
        B.assert(copied.count == 0 && copied.data == null);

        destination: [..] Parcel;

        B.array_copy(*destination, values);
        B.assert(destination.count == 3 && destination[1].value == 77);
        B.array_copy(*destination, B.array_view(destination, 1));
        B.assert(destination.count == 2 && destination[0].value == 77 && destination[1].value == 42);
        B.array_add(*destination, ..B.array_view(destination, 0));
        B.assert(destination.count == 4 && destination[2].value == 77 && destination[3].value == 42);
        B.array_free(destination);
        destination = .{};

        owned_view := B.array_copy(values);
        owned_dynamic := B.resizable(owned_view);

        B.assert(owned_dynamic.data == owned_view.data && owned_dynamic.allocator.data == *owner);
        B.array_free(owned_dynamic);
        owned_view = .{};
        owned_dynamic = .{};
        detached := B.array_copy(values);
        pointer := detached.data;
        count := detached.count;

        B.array_free(detached);
        B.assert(detached.data == pointer && detached.count == count);
        detached = .{};

        Holder :: struct {
            allocator: Allocator;
        }

        holder: Holder;

        B.remember_allocators(*holder);
        B.assert(holder.allocator.proc == tracking && holder.allocator.data == *owner);

        push_context {
            context.allocator = .{};
            B.remember_allocators(*holder);
            B.assert(holder.allocator.proc == context.default_allocator.proc && holder.allocator.data == context.default_allocator.data);
        }

        empty: [] Parcel;
        unset := B.resizable(empty, set_allocators = false);

        B.assert(unset.data == null && unset.count == 0 && unset.allocated == 0);
        unset.allocator = allocator;
        B.array_free(unset);
        supplied_view := B.array_copy(values);
        supplied := B.resizable(supplied_view, set_allocators = false);
        supplied.allocator = allocator;
        live_before := owner.live;
        other_before := other.calls;

        push_context {
            context.allocator = alternate;
            B.array_free(supplied);
        }

        B.assert(owner.live == live_before - 1 && other.calls == other_before);
        supplied_view = .{};
        supplied = .{};
        calls := other.calls;

        push_context {
            context.allocator = alternate;
            B.array_reset(*values);
        }

        B.assert(values.data == null && values.count == 0 && values.allocated == 0);
        B.assert(other.calls == calls && owner.live == 0 && owner.bytes == 0 && owner.allocations == owner.frees);
    }

    B.assert(context.allocator.proc == original.proc && context.allocator.data == original.data);
    ints := int.[1, 2, 3];
    enums := Kind.[.FIRST, .SECOND];
    found, index := B.array_find(ints, 2);

    B.assert(found && index == 1);
    found, index = B.array_find(enums, Kind.SECOND);
    B.assert(found && index == 1);
    wide := B.S128.[B.to_s128(1), B.to_s128(2)];

    found, index = B.array_find(wide, B.to_s128(2));
    B.assert(found && index == 1);
    times := B.Apollo_Time.[B.seconds_to_apollo(1), B.seconds_to_apollo(2)];

    found, index = B.array_find(times, B.seconds_to_apollo(2));
    B.assert(found && index == 1);
}
