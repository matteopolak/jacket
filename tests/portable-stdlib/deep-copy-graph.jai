// Real cyclic/shared graph regression against production reflection metadata.
#import "Basic";
D :: #import "Deep_Copy";
H :: #import "Hash_Table";

Node :: struct {
    name: string;
    value: s64;
    next: *Node;
    shared: *Node;
    references: [] *Node;
    bytes: [..] u8;
    scalar: *s64;
}

mapped :: (input: string, data: *void) -> string {
    count := cast(*s64) data;
    count.* += 1;
    return "mapped";
}

release_graph :: (root: *Node) {
    child := root.next;

    free(root.name);
    free(child.name);
    array_free(root.references);
    array_free(root.bytes);
    free(child);
    free(root);
}

main :: () {
    #assert !#exists(D.copy_value);
    scalar: s64 = 91;
    child := Node.{ name = "child", value = 2, scalar = *scalar };
    root := Node.{ name = "root", value = 1, next = *child, shared = *child };

    child.next = *root;
    references := (*Node).[*child, *child];

    root.references = references;
    array_add(*root.bytes, cast(u8) 9);
    array_add(*root.bytes, cast(u8) 0xff);
    defer array_free(root.bytes);
    copy := D.Deep_Copy(*root, .{});

    assert(copy != *root && copy.name == root.name && copy.name.data != root.name.data);
    assert(copy.next != *child && copy.next == copy.shared && copy.next.next == copy);
    assert(copy.references.data != root.references.data && copy.references[0] == copy.next && copy.references[1] == copy.next);
    assert(copy.bytes.count == 2 && copy.bytes.allocated == root.bytes.allocated && copy.bytes.data != root.bytes.data);
    assert(copy.bytes.allocator.proc == context.allocator.proc && copy.bytes[1] == 0xff);
    assert(copy.next.scalar == *scalar);
    copy.next.value = 7;
    copy.bytes[0] = 88;
    assert(child.value == 2 && root.bytes[0] == 9);
    release_graph(copy);

    map: D.Pointer_Map;
    first := D.Deep_Copy(*child, .{}, pointer_map = *map);
    second := D.Deep_Copy(*root, .{}, pointer_map = *map);

    assert(first.next == second && second.next == first);
    assert(D.Deep_Copy(*root, .{}, pointer_map = *map) == second);
    H.deinit(*map.table);
    release_graph(second);

    simple := Node.{ name = "source", next = *child };
    preallocated: Node;
    count: s64;
    result := D.Deep_Copy(*simple, .{ string_mapper_proc = mapped, string_mapper_data = *count, follow_struct_pointers = false }, *preallocated);

    assert(result == *preallocated && result.next == *child && result.name == "mapped" && count == 1);
    assert(D.Deep_Copy(cast(*Node) null, .{}) == null);
    assert(D.helper_count > 0 && D.time_total >= 0);
    print("ok\n");
}
