// Basic's allocation entry points and array helpers, checked through a counting allocator: alloc,
// free and realloc edge cases, New/NewArray (aligned, empty, overflowing), heaps, push_allocator,
// temporary storage marks and overflow pages, and the [..]/[] helpers including removal while
// iterating.
#import "Basic";

Counting :: struct {
    allocations: s64;
    frees: s64;
    resizes: s64;
    live: s64;
    last_size: s64;
}

counting_proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    counts := cast(*Counting) data;
    base := Context_Base.default_allocator;
    if mode == {
        case .ALLOCATE;
            counts.allocations += 1;
            counts.live += 1;
            counts.last_size = size;
        case .RESIZE;
            counts.resizes += 1;
            if !old_memory counts.live += 1;
            counts.last_size = size;
        case .FREE;
            counts.frees += 1;
            counts.live -= 1;
        case .CAPS;
            if old_memory {
                name := cast(*string) old_memory;
                name.* = "counting";
            }
            return cast(*void) cast(u64) (Allocator_Caps.FREE | .ACTUALLY_RESIZE);
    }
    return base.proc(mode, size, old_size, old_memory, base.data);
}

Defaults :: struct {
    a: s32 = 7;
    b: string = "seven";
    c: [3] u8 = .[1, 2, 3];
}

Wide :: struct {
    bytes: [64] u8;
} #align 64

main :: () {
    counts: Counting;
    counting := Allocator.{ counting_proc, *counts };

    // alloc, free and realloc.
    {
        assert(alloc(-1, counting) == null, "a negative size");
        assert(counts.allocations == 0, "a negative size never reaches the allocator");
        empty := alloc(0, counting);
        free(empty, counting);
        free(null, counting);
        assert(counts.frees == 1, "free(null) does not reach the allocator: % frees", counts.frees);
        none: Allocator;
        assert(alloc(16, none) == null, "an allocator without a proc");
        assert(realloc(null, 16, 0, none) == null);

        memory := cast(*u8) alloc(10, counting);
        for 0..9 memory[it] = cast(u8) (it + 1);
        grown := cast(*u8) realloc(memory, 100000, 10, counting);
        assert(grown != null);
        for 0..9 assert(grown[it] == it + 1, "realloc kept byte %: %", it, grown[it]);
        grown[99999] = 5;
        shrunk := cast(*u8) realloc(grown, 4, 100000, counting);
        for 0..3 assert(shrunk[it] == it + 1, "a shrinking realloc kept byte %", it);
        assert(realloc(shrunk, -1, 4, counting) == null && realloc(shrunk, 4, -1, counting) == null, "negative sizes");
        free(shrunk, counting);
        s := alloc_string(5, counting);
        assert(s.count == 5 && s.data != null);
        free(s, counting);
        assert(alloc_string(0, counting) == "");
        assert(counts.live == 0, "everything was freed: % live", counts.live);
    }

    // get_capabilities.
    {
        caps, name := get_capabilities(counting);
        assert(caps == .FREE | .ACTUALLY_RESIZE && name == "counting", "caps % name %", caps, name);
        caps, name = get_capabilities(Allocator.{});
        assert(caps == 0 && name == "");
        caps, name = get_capabilities(temp);
        assert((caps & .HINT_I_AM_PER_FRAME_TEMPORARY_STORAGE) != 0, "temp caps %", caps);
    }

    // New and NewArray.
    {
        push_allocator(counting);
        d := New(Defaults);
        assert(d.a == 7 && d.b == "seven" && d.c[2] == 3, "New initializes: %", d.*);
        free(d);
        raw := New(Defaults, initialized = false);
        assert(raw != null);
        free(raw);
        explicit := New(s64, allocator = counting);
        assert(explicit.* == 0);
        free(explicit);

        values := NewArray(1000, Defaults);
        for values assert(it.a == 7 && it.b == "seven", "element % not initialized", it_index);
        free(values.data);
        zeros := NewArray(4096, s32);
        for zeros assert(it == 0);
        free(zeros.data);
        nothing, base := NewArray(0, s64);
        assert(nothing.count == 0 && nothing.data == null && base == null);
        negative := NewArray(-5, s64);
        assert(negative.count == 0);
        huge := NewArray(0x7fff_ffff_ffff_ffff / 4, s64);
        assert(huge.count == 0 && huge.data == null, "an overflowing NewArray is refused");
        for alignment: s32.[16, 64, 256, 4096] {
            aligned, aligned_base := NewArray(3, s64, alignment = alignment);
            assert(cast(u64) aligned.data % cast(u64) alignment == 0, "NewArray alignment %: %", alignment, aligned.data);
            assert(aligned_base != null && cast(*u8) aligned.data >= cast(*u8) aligned_base);
            for aligned assert(it == 0);
            free(aligned_base);
        }
        wide := New(Wide);
        assert(cast(u64) wide % 64 == 0, "New of a #align 64 type: %", wide);
        free(wide);
        assert(counts.live == 0, "% live after New/NewArray", counts.live);
    }
    assert(context.allocator.proc != counting_proc, "push_allocator restores the allocator");

    // The (proc, data) overload of push_allocator, nested.
    {
        inner: Counting;
        push_allocator(counting_proc, *inner);
        assert(context.allocator.data == *inner);
        {
            push_allocator(counting);
            assert(context.allocator.data == *counts);
        }
        assert(context.allocator.data == *inner, "the inner push_allocator was undone");
        p := alloc(8);
        free(p);
        assert(inner.allocations == 1 && inner.frees == 1);
    }

    // create_heap / destroy_heap on the default allocator.
    {
        default_allocator := context.allocator;
        caps := get_capabilities(default_allocator);
        if caps & .CREATE_HEAP {
            heap := create_heap(default_allocator);
            assert(heap.data != null && heap.data != default_allocator.data);
            p := alloc(100, heap);
            assert(p != null);
            assert(heap.proc(.IS_THIS_YOURS, 0, 0, p, heap.data) != null, "the heap owns its block");
            destroy_heap(heap);
        }
    }

    check_temporary_storage();
    check_resizable_arrays();
    check_array_views();
    check_small_helpers();
    print("ok\n");
}

check_temporary_storage :: () {
    storage := context.temporary_storage;
    assert(storage != null);
    reset_temporary_storage();
    assert(storage.total_bytes_occupied == 0 && storage.high_water_mark == 0);

    first := talloc(10);
    assert(cast(u64) first % 8 == 0);
    second := talloc(1);
    assert(cast(*u8) second == cast(*u8) first + 16, "allocations are 8-aligned and adjacent");
    assert(temporary_allocator_proc(.IS_THIS_YOURS, 0, 0, first, null) != null);
    local: s64;
    assert(temporary_allocator_proc(.IS_THIS_YOURS, 0, 0, *local, null) == null);

    // A RESIZE that fits the old block's rounding stays in place.
    grown := realloc(second, 8, 1, temp);
    assert(grown == second, "resize within the 8-byte rounding stays put");
    moved := cast(*u8) realloc(first, 64, 10, temp);
    assert(moved != first);

    mark := get_temporary_storage_mark();
    for 1..100 talloc(1000);
    assert(storage.total_bytes_occupied > 100000, "overflow pages hold %", storage.total_bytes_occupied);
    big := cast(*u8) talloc(1_000_000);
    big[999_999] = 1;
    assert(temporary_allocator_proc(.IS_THIS_YOURS, 0, 0, big + 999_999, null) != null, "overflow pages belong to temp");
    high := storage.high_water_mark;
    set_temporary_storage_mark(mark);
    assert(storage.total_bytes_occupied == mark.total_bytes_occupied, "the mark restores the occupancy");
    assert(storage.high_water_mark == high, "the high water mark survives set_temporary_storage_mark");
    assert(talloc(8) == cast(*u8) storage.data + mark.current_page_bytes_occupied, "allocation resumes at the mark");

    {
        auto_release_temp();
        for 1..50 talloc(4000);
    }
    assert(storage.total_bytes_occupied == mark.total_bytes_occupied + 8, "auto_release_temp released its scope: %", storage.total_bytes_occupied);

    text := talloc_string(3);
    assert(text.count == 3);
    s := tprint("% and %", 1, "two");
    assert(s == "1 and two");
    reset_temporary_storage();
    assert(storage.total_bytes_occupied == 0 && storage.high_water_mark == 0);

    // A storage of our own, with its overflow allocator.
    counts: Counting;
    mine: Temporary_Storage;
    assert(init_temporary_storage(*mine, 256, allocator = .{ counting_proc, *counts }));
    assert(counts.allocations == 1);
    {
        scoped := context;
        push_context, defer_pop scoped;
        context.temporary_storage = *mine;
        a := talloc(200);
        assert(cast(*u8) a == mine.data);
        b := talloc(200);
        assert(counts.allocations == 2, "the second block overflowed into a new page");
        assert(temporary_allocator_proc(.IS_THIS_YOURS, 0, 0, b, null) != null);
        reset_temporary_storage();
        assert(counts.frees == 1, "reset released the overflow page");
        assert(talloc(16) == mine.data);
    }
    deinit_temporary_storage(*mine, .{ counting_proc, *counts }, true);
    assert(counts.live == 0, "% live after deinit", counts.live);
    assert(!init_temporary_storage(*mine, -1));
    // Storage handed in by the caller is used as is.
    buffer: [128] u8;
    assert(init_temporary_storage(*mine, buffer.count, buffer.data));
    assert(mine.data == buffer.data);
    deinit_temporary_storage(*mine);

    // Without temporary storage, temp falls back to the context allocator.
    {
        scoped := context;
        push_context, defer_pop scoped;
        context.temporary_storage = null;
        p := talloc(32);
        assert(p != null);
        free(p, Context_Base.default_allocator);
        mark := get_temporary_storage_mark();
        assert(mark.total_bytes_occupied == 0);
        set_temporary_storage_mark(mark);
        reset_temporary_storage();
    }
}

check_resizable_arrays :: () {
    counts: Counting;
    a: [..] s64;
    a.allocator = .{ counting_proc, *counts };

    // Growth across many elements keeps every value.
    for 0..9999 array_add(*a, it * 3);
    assert(a.count == 10000 && a.allocated >= 10000);
    for a assert(it == it_index * 3, "a[%] = %", it_index, it);
    assert(counts.allocations < 40, "growth is geometric: % allocations", counts.allocations);
    reserved := a.allocated;
    array_reserve(*a, 10);
    assert(a.allocated == reserved, "reserving less never shrinks");
    array_reset_keeping_memory(*a);
    assert(a.count == 0 && a.allocated == reserved && a.data != null);
    array_reset(*a);
    assert(a.count == 0 && a.allocated == 0 && a.data == null);
    assert(counts.live == 0, "array_reset freed the block: % live", counts.live);

    // Insertion and removal keep the expected order.
    array_add(*a, 1, 2, 3);
    array_insert_at(*a, 0, 0);
    array_insert_at(*a, 99, 2);
    array_insert_at(*a, 4, a.count);
    assert(a.count == 6 && a[0] == 0 && a[1] == 1 && a[2] == 99 && a[3] == 2 && a[4] == 3 && a[5] == 4, "after inserts %", a);
    array_ordered_remove_by_index(*a, 2);
    assert(a.count == 5 && a[2] == 2 && a[4] == 4);
    view: [] s64 = a;
    assert(array_ordered_remove_by_value(*view, 3) && view.count == 4 && view[3] == 4);
    assert(!array_ordered_remove_by_value(*view, 42));
    a.count = view.count;
    array_unordered_remove_by_index(*a, 0);
    assert(a.count == 3 && a[0] == 4, "the last element fills the hole: %", a);
    assert(peek(a) == 2 && peek_pointer(a).* == 2);
    assert(pop(*a) == 2 && a.count == 2);

    added, index := array_add_if_unique(*a, 1);
    assert(!added && index == 1, "1 is already at index 1: % %", added, index);
    added, index = array_add_if_unique(*a, 5);
    assert(added && index == 2 && a[2] == 5);
    found, at := array_find(a, 5);
    assert(found && at == 2);
    found, at = array_find(a, 77);
    assert(!found && at == -1);

    // Removing duplicates by value, all or only the first.
    array_reset(*a);
    array_add(*a, 7, 1, 7, 7, 2, 7);
    dup_view: [] s64 = a;
    assert(array_unordered_remove_by_value(*dup_view, 7, stop_after_first = true) == 1 && dup_view.count == 5);
    assert(array_unordered_remove_by_value(*dup_view, 7) == 3 && dup_view.count == 2, "left %", dup_view);
    for dup_view assert(it != 7);
    assert(array_unordered_remove_by_value(*dup_view, 9) == 0);

    // `remove` while iterating removes exactly the matching elements, visiting each once.
    array_reset(*a);
    for 0..99 array_add(*a, it);
    visited := 0;
    for a {
        visited += 1;
        if it % 3 == 0 remove it;
    }
    assert(visited == 100, "visited %", visited);
    assert(a.count == 66, "% left after removing multiples of 3", a.count);
    sum := 0;
    for a {
        assert(it % 3 != 0);
        sum += it;
    }
    assert(sum == 4950 - 1683, "sum %", sum);
    // Removing every element.
    for a remove it;
    assert(a.count == 0);

    // Resizing, copying, swapping.
    array_resize(*a, 5);
    for a assert(it == 0);
    a[4] = 44;
    array_resize(*a, 2);
    array_resize(*a, 6);
    assert(a.count == 6 && a[5] == 0, "growing again zero-initializes: %", a);
    copy := array_copy(a);
    assert(copy.count == 6 && copy.data != a.data);
    copy[0] = 10;
    assert(a[0] == 0, "array_copy is a deep copy");
    other: [..] s64;
    array_add(*other, 9);
    array_swap(*a, *other);
    assert(a.count == 1 && a[0] == 9 && other.count == 6);
    array_copy(*other, a);
    assert(other.count == 1 && other[0] == 9);
    array_free(copy);
    array_free(other);
    array_reset(*a);
    assert(counts.live == 0, "% live", counts.live);

    // maybe_grow and init_range on raw storage.
    raw: [..] Defaults;
    maybe_grow(cast(*Resizable_Array) *raw, size_of(Defaults));
    assert(raw.allocated >= 1 && raw.count == 0);
    raw.count = raw.allocated;
    memset(raw.data, 0, raw.count * size_of(Defaults));
    init_range(raw.data, 0, raw.count);
    for raw assert(it.a == 7 && it.b == "seven");
    array_reset(*raw);

    // Structs with initializers grow initialized.
    structs: [..] Defaults;
    array_resize(*structs, 3);
    for structs assert(it.a == 7, "resized element % not initialized", it_index);
    pointer := array_add(*structs);
    assert(pointer.a == 7 && structs.count == 4);
    array_free(structs);
}

check_array_views :: () {
    numbers: [10] s64;
    for *numbers it.* = it_index;
    tail := array_view(numbers, 6);
    assert(tail.count == 4 && tail[0] == 6);
    middle := array_view(numbers, 2, 3);
    assert(middle.count == 3 && middle[0] == 2 && middle[2] == 4);
    empty := array_view(numbers, 10);
    assert(empty.count == 0);
    assert(peek(middle) == 4 && peek_pointer(middle) == *numbers[4]);
    m := middle;
    assert(pop(*m) == 4 && m.count == 2);

    heap := NewArray(4, s64);
    for *heap it.* = it_index * 10;
    array_resize(*heap, 6);
    assert(heap.count == 6 && heap[3] == 30 && heap[5] == 0, "%", heap);
    array_resize(*heap, 2, initialize = false);
    assert(heap.count == 2 && heap[1] == 10);
    copied := array_copy(heap);
    assert(copied.count == 2 && copied[1] == 10 && copied.data != heap.data);
    array_reset(*heap);
    assert(heap.count == 0 && heap.data == null);
    array_free(copied);
    unordered := NewArray(3, s64);
    unordered[0] = 5;
    unordered[2] = 8;
    array_unordered_remove_by_index(*unordered, 0);
    assert(unordered.count == 2 && unordered[0] == 8);
    free(unordered.data);

    as_resizable := resizable(numbers, set_allocators = false);
    assert(as_resizable.count == 10 && as_resizable.data == numbers.data && as_resizable.allocated == 10 && as_resizable.allocator.proc == null);
    with_allocator := resizable(numbers);
    assert(with_allocator.allocator.proc == context.allocator.proc);
}

Holder :: struct {
    name: string;
    items: [..] s64;
    nested: struct {
        allocator: Allocator;
        more: [..] u8;
    };
    allocator: Allocator;
}

Color :: enum s16 {
    RED :: -2;
    GREEN :: 5;
    BLUE :: 9;
}

check_small_helpers :: () {
    x, y := swap(1, 2);
    assert(x == 2 && y == 1);
    p, q := "p", "q";
    swap(*p, *q);
    assert(p == "q" && q == "p");
    Swap(*p, *q);
    assert(p == "p");

    assert(IsScalar(s8) && IsScalar(float32) && IsScalar(bool) && IsScalar(Color));
    assert(!IsScalar(string) && !IsScalar(*u8) && !IsScalar(Holder));

    values := enum_values_as_s64(Color);
    assert(values.count == 3 && values[0] == -2 && values[2] == 9);
    as_enum := enum_values_as_enum(Color);
    assert(as_enum.count == 3 && as_enum[0] == .RED && as_enum[1] == .GREEN && as_enum[2] == .BLUE);
    free(as_enum.data);
    low, high := enum_range(Color);
    assert(low == -2 && high == 9 && enum_highest_value(Color) == 9);
    assert(enum_names(Color)[1] == "GREEN");

    holder: Holder;
    other: Allocator = .{ counting_proc, null };
    holder.nested.allocator = other;
    remember_allocators(*holder);
    assert(holder.allocator.proc == context.allocator.proc, "a missing allocator is filled in");
    assert(holder.items.allocator.proc == context.allocator.proc, "an array's allocator is filled in");
    assert(holder.nested.more.allocator.proc == context.allocator.proc);
    assert(holder.nested.allocator.proc == counting_proc, "a set allocator is kept");

    integer, signed, bits := is_integer_type(s16);
    assert(integer && signed && bits == 16);
    integer, signed, bits = is_integer_type(u64);
    assert(integer && !signed && bits == 64);
    integer = is_integer_type(float32);
    assert(!integer);
    {
        lo, hi := get_integer_range(s8);
        assert(lo == -128 && hi == 127);
    }
    {
        lo, hi := get_integer_range(u16);
        assert(lo == 0 && hi == 65535);
    }
    {
        lo, hi := get_integer_range(s64);
        assert(lo == -9223372036854775807 - 1 && hi == 9223372036854775807);
    }
    {
        lo, hi := get_integer_range(u64);
        assert(lo == 0 && hi == 0xffff_ffff_ffff_ffff);
    }
    assert(clamp_to_another_integer_type(300, u8) == 255);
    assert(clamp_to_another_integer_type(-5, u8) == 0);
    assert(clamp_to_another_integer_type(-500, s8) == -128);
    assert(clamp_to_another_integer_type(-100, s8) == -100);
    assert(clamp_to_another_integer_type(cast(u64) 0xffff_ffff_ffff_ffff, s64) == 9223372036854775807);
    assert(clamp_to_another_integer_type(cast(s8) -1, s64) == -1);
    assert(clamp_to_another_integer_type(cast(u8) 200, s8) == 127);
    assert(clamp_to_another_integer_type(cast(s32) 70000, u16) == 65535);

    assert(min(3, 1, 2) == 1 && max(3, 9, 2) == 9 && min(5) == 5);
    assert(Min(1.5, 0.5) == 0.5 && Max(-1, -2) == -1);
    assert(clamp(5, 0, 3) == 3 && clamp(-5, 0, 3) == 0 && clamp(2, 0, 3) == 2);
    assert(align_forward(0, 8) == 0 && align_forward(1, 8) == 8 && align_forward(16, 8) == 16 && align_forward(17, 16) == 32);
}
