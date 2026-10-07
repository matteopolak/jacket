// The small memory modules at run time: Memory.copy with every kind of overlap, Relative_Pointers
// at each storage width (null, self, forwards, backwards, the range limits), Deep_Copy of a graph
// with cycles, shared nodes, strings and arrays, and Remap_Context from a differently shaped
// context. Tagged_Union is in tagged-union-module.jai.
#import "Basic";
Memory :: #import "Memory";
#import "Relative_Pointers";
#import "Deep_Copy";
#import "Hash_Table";
Remap :: #import "Remap_Context"(VERBOSE = false);

main :: () {
    check_memory_copy();
    check_relative_pointers();
    check_deep_copy();
    check_remap_context();
    print("ok\n");
}

check_memory_copy :: () {
    // Every overlap of a span of length n moved by every distance, against a reference copy.
    for count: s64.[0, 1, 2, 7, 16, 33] for distance : -40..40 {
        buffer: [128] u8;
        for *buffer it.* = cast(u8) it_index;
        source := 44;
        destination := source + distance;
        expected: [128] u8 = buffer;
        for i: 0..count - 1 expected[destination + i] = buffer[source + i];
        Memory.copy(buffer.data + destination, buffer.data + source, count);
        for buffer assert(it == expected[it_index], "copy of % bytes moved by %: byte % is %", count, distance, it_index, it);
    }
    // Disjoint buffers and a negative count.
    a := u8.[1, 2, 3];
    b: [3] u8;
    Memory.copy(b.data, a.data, 3);
    assert(b[0] == 1 && b[2] == 3);
    Memory.copy(b.data, a.data, -1);
    assert(b[1] == 2);
}

Node :: struct {
    value: s64;
    next: Relative_Pointer(s32, Node);
}

Record :: struct {
    name: Relative_String(s16);
    first: Relative_Pointer(s8, u8);
}

check_relative_pointers :: () {
    // Width by width: null, pointing at itself, forwards and backwards, and the farthest reach.
    check_width(s8);
    check_width(s16);
    check_width(s32);
    check_width(s64);

    // A list linked with relative pointers keeps working after the whole block is moved.
    nodes: [5] Node;
    for *nodes {
        it.value = it_index * 10;
        if it_index < 4 encode(*it.next, *nodes[it_index + 1]);
        else encode(*it.next, null);
    }
    moved: [5] Node;
    memcpy(moved.data, nodes.data, size_of(type_of(nodes)));
    sum := 0;
    cursor := *moved[0];
    while cursor {
        sum += cursor.value;
        assert(cast(*u8) cursor >= cast(*u8) moved.data && cast(*u8) cursor < cast(*u8) (moved.data + 5), "decoded into the moved block");
        cursor = decode(*cursor.next);
    }
    assert(sum == 100, "walked the moved list: %", sum);

    // A relative string next to its bytes.
    Packed :: struct {
        record: Record;
        bytes: [5] u8;
    }
    packed: Packed;
    memcpy(packed.bytes.data, "hello".data, 5);
    packed.record.name.count = 5;
    write_relative_pointer(s16, *packed.record.name.data, packed.bytes.data);
    encode(*packed.record.first, *packed.bytes[1]);
    copy := packed;
    assert(decode(*copy.record.name) == "hello", "relative string %", decode(*copy.record.name));
    assert(decode(*copy.record.first).* == #char "e");
    empty: Relative_String(s32);
    assert(decode(*empty) == "", "an empty relative string");

    // Explicit bases.
    block: [64] u8;
    slot := block.data + 8;
    target := block.data + 40;
    write_relative_pointer(s32, slot, block.data, target);
    pointer, offset := decode_relative_pointer(4, slot, block.data);
    assert(pointer == target && offset == 40, "relative to a base: % %", pointer, offset);
    pointer, offset = decode_relative_pointer(4, slot);
    assert(pointer == slot + 40 && offset == 40, "relative to itself, the stored displacement means something else");
    write_relative_pointer(s32, slot, null);
    pointer, offset = decode_relative_pointer(4, slot);
    assert(pointer == null && offset == 0);
}

check_width :: ($T: Type) {
    // How far the width reaches; s64 reaches everything.
    reach :: ifx size_of(T) == 8 then 0x7fff_ffff_ffff_ffff else (cast(s64) 1) << (size_of(T) * 8 - 1);
    storage: T;
    assert(decode_relative_pointer_static(T, *storage) == null, "%: zero is null", T);
    write_relative_pointer(T, *storage, null);
    assert(storage == 0 && decode_relative_pointer_static(T, *storage) == null);
    write_relative_pointer(T, *storage, *storage);
    assert(storage != 0, "%: a pointer to itself is not null", T);
    assert(decode_relative_pointer_static(T, *storage) == *storage, "%: self", T);
    base := cast(*u8) *storage;
    distances :: s64.[1, -1, 2, -2, 100, -100];
    for distances {
        if it >= reach || it <= -reach continue;
        target := base + it;
        write_relative_pointer(T, *storage, target);
        assert(decode_relative_pointer_static(T, *storage) == target, "%: distance %", T, it);
    }
    // The farthest each width reaches, both ways (never dereferenced).
    #if size_of(T) < 8 {
        far := base + (reach - 1);
        write_relative_pointer(T, *storage, far);
        assert(decode_relative_pointer_static(T, *storage) == far, "%: farthest forward", T);
        near := base - (reach - 1);
        write_relative_pointer(T, *storage, near);
        assert(decode_relative_pointer_static(T, *storage) == near, "%: farthest back", T);
    }
}

Tree :: struct {
    name: string;
    children: [..] *Tree;
    parent: *Tree;
    shared: *Leaf;
    scores: [] s32;
    fixed: [2] Leaf;
    cache: *Leaf; @NoDeepCopy
    callback: () -> s64;
    opaque: *void;
}

Leaf :: struct {
    label: string;
    weight: float64;
}

forty_two :: () -> s64 {
    return 42;
}

upper :: (input: string, data: *void) -> string {
    calls := cast(*s64) data;
    calls.* += 1;
    result := copy_string(input);
    for 0..result.count - 1 if result[it] >= #char "a" && result[it] <= #char "z" result[it] -= 32;
    return result;
}

check_deep_copy :: () {
    shared_leaf := Leaf.{ "shared", 0.5 };
    root: Tree;
    root.name = copy_string("root");
    root.shared = *shared_leaf;
    root.cache = *shared_leaf;
    root.callback = forty_two;
    untouched: s64 = 5;
    root.opaque = *untouched;
    root.fixed[1] = .{ copy_string("fixed"), 2 };
    root.scores = NewArray(3, s32);
    root.scores[2] = 7;
    for 0..2 {
        child := New(Tree);
        child.name = tprint("child %", it);
        child.parent = *root;
        child.shared = *shared_leaf;
        array_add(*root.children, child);
    }
    // A cycle that does not go through the root.
    array_add(*root.children[0].children, root.children[1]);
    array_add(*root.children[1].children, root.children[0]);

    copy := Deep_Copy(*root, .{});
    assert(copy != null && copy != *root);
    assert(copy.name == "root" && copy.name.data != root.name.data, "strings are copied");
    assert(copy.children.count == 3 && copy.children.data != root.children.data, "arrays are copied");
    assert(copy.children.allocated == 3, "a copied [..] holds exactly its items");
    for copy.children {
        assert(it != root.children[it_index], "child % is a new node", it_index);
        assert(it.name == root.children[it_index].name);
        assert(it.parent == copy, "child % points back at the copied root", it_index);
        assert(it.shared == copy.shared, "a shared leaf stays shared");
    }
    assert(copy.children[0].children[0] == copy.children[1] && copy.children[1].children[0] == copy.children[0], "the cycle is rebuilt among the copies");
    assert(copy.shared != *shared_leaf && copy.shared.label == "shared" && copy.shared.weight == 0.5);
    assert(copy.cache == *shared_leaf, "a @NoDeepCopy member keeps the original pointer");
    assert(copy.callback == forty_two && copy.callback() == 42, "procedure pointers are kept");
    assert(copy.opaque == *untouched, "void pointers are not followed");
    assert(copy.scores.count == 3 && copy.scores.data != root.scores.data && copy.scores[2] == 7);
    assert(copy.fixed[1].label == "fixed" && copy.fixed[1].label.data != root.fixed[1].label.data && copy.fixed[1].weight == 2);
    // The copy is independent of the original.
    root.children[2].name = "changed";
    root.scores[2] = 0;
    assert(copy.children[2].name == "child 2" && copy.scores[2] == 7);

    // A string mapper sees every string.
    calls: s64;
    config: Deep_Copy_Config;
    config.string_mapper_proc = upper;
    config.string_mapper_data = *calls;
    mapped := Deep_Copy(*root, config);
    assert(mapped.name == "ROOT" && mapped.children[0].name == "CHILD 0" && mapped.shared.label == "SHARED", "mapped %", mapped.name);
    assert(calls >= 6, "the mapper ran % times", calls);

    // One pointer map across calls keeps shared objects shared between copies.
    map: Pointer_Map;
    first := Deep_Copy(root.children[0], .{}, pointer_map = *map);
    second := Deep_Copy(root.children[1], .{}, pointer_map = *map);
    assert(first.shared == second.shared, "both copies share one leaf copy");
    assert(first.children[0] == second, "the earlier copy links to the later one through the map");
    again := Deep_Copy(root.children[0], .{}, pointer_map = *map);
    assert(again == first, "copying the same object again returns the earlier copy");
    deinit(*map.table);

    // Copying into storage the caller provides.
    into: Leaf;
    result := Deep_Copy(*shared_leaf, .{}, preallocated_root = *into);
    assert(result == *into && into.label == "shared" && into.label.data != shared_leaf.label.data);
    assert(Deep_Copy(cast(*Leaf) null, .{}) == null);
}

// A context from some other program: its base has a few fields that match ours by name (one with
// a different size), the rest do not.
Foreign_Base :: struct {
    unrelated: [3] u64;
    thread_index: u32;
    allocator: Allocator;
    logger_data: *void;
    log_level: u64;
}

Foreign_Context :: struct {
    using base: Foreign_Base;
    print_style: s32;
}

check_remap_context :: () {
    same := Remap.remap_context();
    assert(same.thread_index == context.thread_index && same.allocator.proc == context.allocator.proc, "an own context remaps to itself");
    {
        Remap.Remap_Context();
        assert(context.context_info == type_info(#Context));
    }
    foreign: Foreign_Context;
    foreign.thread_index = 9;
    foreign.allocator = context.allocator;
    marker: s64;
    foreign.logger_data = *marker;
    foreign.print_style = 77;
    foreign.log_level = 2;
    remapped := Remap.remap_context(*foreign, type_info(Foreign_Context));
    assert(remapped.thread_index == 9, "thread_index carried over: %", remapped.thread_index);
    assert(remapped.allocator.proc == context.allocator.proc, "allocator carried over");
    assert(remapped.logger_data == *marker, "logger_data carried over");
    assert(remapped.context_info == type_info(#Context));
    fresh: #Context;
    assert(remapped.logger == fresh.logger, "fields the foreign context lacks keep their defaults");
    assert(remapped.log_level == .NORMAL, "a field whose size differs is not copied");
}
