#import "Basic";
#import "Extensions/Toml";

// Compile the actual private index helpers against the module's concrete field types.
// No testing hook or additional exported API is required.
Toml_Index :: type_of((Toml_Value.{}).internal_lookup.*);
Toml_Index_Node :: type_of((Toml_Index.{}).nodes.data.*);

#load "../index.jai";

add_raw :: (table: *Toml_Value, index: *Toml_Index, key: string) {
    hint: Toml_Insert_Hint;

    assert(toml_probe_index(table, index, key, *hint) < 0);
    assert(toml_prepare_insertion(table, index, key, *hint) < 0);
    array_add(*table.members, Toml_Member.{ key = key });
    assert(toml_tree_add(table, index, table.members.count - 1, *hint, true) < 0);
}

check_rotation :: (keys: [] string) {
    table: Toml_Value;
    index: Toml_Index;

    table.internal_lookup = *index;
    defer array_free(table.members);
    defer array_free(index.nodes);
    for keys add_raw(*table, *index, it);

    assert(table.members[index.root].key == "b");
    assert(index.nodes[index.root].height == 2);
    for keys assert(toml_tree_find(*table, *index, it) >= 0);
}

check_guards :: () {
    table: Toml_Value;
    index: Toml_Index;

    table.internal_lookup = *index;
    defer array_free(table.members);
    defer array_free(index.nodes);
    for 0..31 add_raw(*table, *index, tprint("key%", formatInt(it, minimum_digits = 3)));

    hint: Toml_Insert_Hint;

    assert(toml_probe_index(*table, *index, "z", *hint) < 0 && hint.complete);
    hint.path[0] = -2;
    array_add(*table.members, Toml_Member.{ key = "z" });
    assert(toml_tree_add(*table, *index, 32, *hint, true) < 0);
    assert(toml_tree_find(*table, *index, "z") == 32);

    assert(toml_probe_index(*table, *index, "y", *hint) < 0);
    add_raw(*table, *index, "y");
    assert(toml_prepare_insertion(*table, *index, "y", *hint) == 33);
    assert(table.members.count == 34 && index.nodes.count == 34);
    assert(toml_probe_index(*table, *index, "later", *hint) < 0);
    assert(toml_prepare_insertion(*table, *index, "y", *hint) == 33);

    assert(toml_probe_index(*table, *index, "relocated", *hint) < 0);
    array_reserve(*index.nodes, 2048);
    array_reserve(*table.members, 2048);
    assert(toml_prepare_insertion(*table, *index, "relocated", *hint) < 0);
    array_add(*table.members, Toml_Member.{ key = "relocated" });
    assert(toml_tree_add(*table, *index, 34, *hint, true) < 0);
    assert(toml_tree_find(*table, *index, "relocated") == 34);

    // A changed snapshot must re-probe, not feed a stale absence into unchecked insertion.
    assert(toml_probe_index(*table, *index, "late", *hint) < 0);
    add_raw(*table, *index, "late");
    array_add(*table.members, Toml_Member.{ key = "late" });
    assert(toml_tree_add(*table, *index, 36, *hint, true) == 35);
    table.members.count -= 1;
    assert(table.members.count == index.nodes.count);
}

check_prefix_and_overflow :: () {
    table: Toml_Value;
    index: Toml_Index;

    table.internal_lookup = *index;
    defer array_free(table.members);
    defer array_free(index.nodes);
    // A writer indexes a prefix of a fixed complete member array.
    for string.["d", "b", "a", "c"] array_add(*table.members, Toml_Member.{ key = it });

    for table.members {
        hint: Toml_Insert_Hint;
        assert(toml_probe_index(*table, *index, it.key, *hint) < 0);
        assert(toml_tree_add(*table, *index, it_index, *hint, false) < 0);
    }

    for table.members assert(toml_tree_find(*table, *index, it.key) == it_index);

    array_reset(*table.members);
    array_reset(*index.nodes);
    index.root = 0;

    // A deep ordered tree proves the path cap is not an input rejection. This artificial
    // prefix need not be AVL-balanced; ordinary insertion must still preserve its keys.
    for 0..69 {
        array_add(*table.members, Toml_Member.{ key = tprint("key%", formatInt(it, minimum_digits = 3)) });
        array_add(*index.nodes, Toml_Index_Node.{ right = ifx it < 69 then it + 1 else -1, height = 70 - it });
    }

    array_add(*table.members, Toml_Member.{ key = "key070" });
    hint: Toml_Insert_Hint;

    assert(toml_probe_index(*table, *index, "key070", *hint) < 0 && !hint.complete && hint.absent);
    assert(toml_tree_add(*table, *index, 70, *hint, false) < 0);
    for table.members assert(toml_tree_find(*table, *index, it.key) == it_index);
}

main :: () {
    check_rotation(string.["c", "b", "a"]);
    check_rotation(string.["a", "b", "c"]);
    check_rotation(string.["c", "a", "b"]);
    check_rotation(string.["a", "c", "b"]);
    check_guards();
    check_prefix_and_overflow();
    check_quota_reuse();
    print("ok\n");
}

check_quota_reuse :: () {
    doc, error := new_toml_document(.{ max_retained_bytes = 32768 });

    assert(error.kind == .NONE);

    for 0..31 {
        value, value_error := toml_new_value(*doc, .INTEGER);
        assert(value_error.kind == .NONE);
        assert(toml_insert(*doc, doc.root, tprint("key%", it), value).kind == .NONE);
    }

    next, next_error := toml_new_value(*doc, .INTEGER);

    assert(next_error.kind == .NONE);
    bytes: [65536] u8;
    for *bytes it.* = 113;

    retained := doc.retained_bytes;
    failure := toml_insert(*doc, doc.root, string.{ bytes.count, bytes.data }, next);

    assert(failure.kind == .LIMIT && doc.retained_bytes == retained && next.internal_parent == null);
    assert(toml_insert(*doc, doc.root, "after-limit", next).kind == .NONE);
    assert(toml_get(doc.root, "after-limit") == next);
    free_toml(*doc);
}
