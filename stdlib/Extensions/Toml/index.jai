#scope_module
// Stack-only insertion cursors borrow immutable keys and retain indices, not array pointers.
// The bound limits an optimization only; deeper paths retain ordinary checked insertion.
Toml_Insert_Hint :: struct {
    table: *Toml_Value;
    index: *Toml_Index;
    root: int = -1;
    index_count: int;
    member_count: int;
    key: string;
    absent: bool;
    complete: bool;
    count: int;
    path: [64] int = ---;
    left: [64] bool = ---;
}

toml_hint_key_matches :: inline (a: string, b: string) -> bool {
    if a.count != b.count return false;
    return a.data == b.data || toml_key_compare(a, b) == 0;
}

// Captures both a complete index and a writer's prefix index without conflating their counts.
toml_probe_index :: (table: *Toml_Value, index: *Toml_Index, key: string, hint: *Toml_Insert_Hint) -> int {
    hint.table = table;
    hint.index = index;
    hint.key = key;
    hint.member_count = table.members.count;
    hint.index_count = ifx index then index.nodes.count else 0;
    hint.root = ifx index then index.root else -1;
    hint.absent = false;
    hint.complete = index != null;
    hint.count = 0;

    if !index {
        for table.members if it.key == key return it_index;
        hint.absent = true;
        return -1;
    }

    current := index.root;
    while current >= 0 {
        comparison := toml_key_compare(key, table.members[current].key);
        if comparison == 0 {
            hint.complete = false;
            return current;
        }

        if hint.count < hint.path.count {
            hint.path[hint.count] = current;
            hint.left[hint.count] = comparison < 0;
            hint.count += 1;
        } else hint.complete = false;

        current = ifx comparison < 0 then index.nodes[current].left else index.nodes[current].right;
    }

    hint.absent = true;

    return -1;
}

toml_hint_before_matches :: inline (table: *Toml_Value, index: *Toml_Index, key: string, hint: *Toml_Insert_Hint) -> bool {
    if !hint.absent || hint.table != table || hint.index != index || hint.member_count != table.members.count || !toml_hint_key_matches(hint.key, key) return false;
    return !index || (hint.index_count == index.nodes.count && hint.root == index.root);
}

// Run before reservation/member mutation. A stale absence result must be searched again.
toml_prepare_insertion :: inline (table: *Toml_Value, index: *Toml_Index, key: string, hint: *Toml_Insert_Hint) -> int {
    if toml_hint_before_matches(table, index, key, hint) return -1;
    return toml_probe_index(table, index, key, hint);
}

toml_hint_after_matches :: inline (table: *Toml_Value, index: *Toml_Index, slot: int, hint: *Toml_Insert_Hint, added_member: bool) -> bool {
    if !hint.absent || hint.table != table || table.members.count != hint.member_count + (ifx added_member then 1 else 0) || !toml_hint_key_matches(hint.key, table.members[slot].key) return false;
    if hint.index == index return hint.index_count == index.nodes.count && hint.root == index.root;
    // The only representation change allowed after preparation is building the first index.
    return added_member && !hint.index && !hint.complete && index.nodes.count == hint.member_count;
}

toml_hint_path_matches :: (index: *Toml_Index, hint: *Toml_Insert_Hint) -> bool {
    if !hint.complete || hint.index != index || hint.count < 0 || hint.count > hint.path.count return false;

    current := index.root;

    for i: 0..hint.count - 1 {
        node := hint.path[i];
        if node != current || node < 0 || node >= hint.index_count return false;
        current = ifx hint.left[i] then index.nodes[node].left else index.nodes[node].right;
    }

    return current == -1;
}

// Compare raw decoded bytes, including NUL. For example, "a" precedes "aa".
toml_key_compare :: (a: string, b: string) -> int {
    count := min(a.count, b.count);
    if count > 0 {
        order := memcmp(a.data, b.data, count);
        if order < 0 return -1;
        if order > 0 return 1;
    }

    if a.count == b.count return 0;

    return ifx a.count < b.count then -1 else 1;
}

toml_tree_find :: (table: *Toml_Value, index: *Toml_Index, key: string) -> int {
    current := index.root;
    while current >= 0 {
        comparison := toml_key_compare(key, table.members[current].key);
        if comparison == 0 return current;
        current = ifx comparison < 0 then index.nodes[current].left else index.nodes[current].right;
    }

    return -1;
}

toml_tree_height :: (index: *Toml_Index, node: int) -> int {
    if node < 0 return 0;
    return index.nodes[node].height;
}

toml_tree_update :: (index: *Toml_Index, node: int) {
    index.nodes[node].height = max(toml_tree_height(index, index.nodes[node].left), toml_tree_height(index, index.nodes[node].right)) + 1;
}

toml_tree_left :: (index: *Toml_Index, root: int) -> int {
    pivot := index.nodes[root].right;
    middle := index.nodes[pivot].left;

    index.nodes[pivot].left = root;
    index.nodes[root].right = middle;
    toml_tree_update(index, root);
    toml_tree_update(index, pivot);

    return pivot;
}

toml_tree_right :: (index: *Toml_Index, root: int) -> int {
    pivot := index.nodes[root].left;
    middle := index.nodes[pivot].right;

    index.nodes[pivot].right = root;
    index.nodes[root].left = middle;
    toml_tree_update(index, root);
    toml_tree_update(index, pivot);

    return pivot;
}

toml_tree_balance :: inline (index: *Toml_Index, root: int) -> int {
    toml_tree_update(index, root);
    left := index.nodes[root].left;
    right := index.nodes[root].right;
    balance := toml_tree_height(index, left) - toml_tree_height(index, right);

    if balance > 1 {
        if toml_tree_height(index, index.nodes[left].left) < toml_tree_height(index, index.nodes[left].right) index.nodes[root].left = toml_tree_left(index, left);
        return toml_tree_right(index, root);
    }

    if balance < -1 {
        if toml_tree_height(index, index.nodes[right].right) < toml_tree_height(index, index.nodes[right].left) index.nodes[root].right = toml_tree_right(index, right);
        return toml_tree_left(index, root);
    }

    return root;
}

toml_tree_insert :: (table: *Toml_Value, index: *Toml_Index, root: int, slot: int) -> int {
    if root < 0 return slot;

    comparison := toml_key_compare(table.members[slot].key, table.members[root].key);

    assert(comparison != 0, "duplicate checked before index insertion");
    if comparison < 0 index.nodes[root].left = toml_tree_insert(table, index, index.nodes[root].left, slot);
    else index.nodes[root].right = toml_tree_insert(table, index, index.nodes[root].right, slot);

    return toml_tree_balance(index, root);
}

// A prepared absent-key hint may survive capacity relocation, never semantic mutation.
// Unexpected snapshot changes re-probe before touching index nodes; duplicates are returned.
toml_tree_add :: (table: *Toml_Value, index: *Toml_Index, slot: int, hint: *Toml_Insert_Hint = null, added_member: bool = false) -> int {
    assert(slot == index.nodes.count);

    if hint && !toml_hint_after_matches(table, index, slot, hint, added_member) {
        fresh: Toml_Insert_Hint;
        duplicate := toml_probe_index(table, index, table.members[slot].key, *fresh);
        if duplicate >= 0 return duplicate;
        return toml_tree_add(table, index, slot, *fresh, false);
    }

    reuse := hint && toml_hint_path_matches(index, hint);

    array_add(*index.nodes, Toml_Index_Node.{});

    if reuse {
        child := slot;
        cursor := hint.count;

        while cursor > 0 {
            cursor -= 1;
            parent := hint.path[cursor];
            if hint.left[cursor] index.nodes[parent].left = child;
            else index.nodes[parent].right = child;

            child = toml_tree_balance(index, parent);
        }

        index.root = child;
    } else index.root = toml_tree_insert(table, index, index.root, slot);

    return -1;
}
