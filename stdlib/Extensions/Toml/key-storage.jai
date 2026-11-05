#scope_module
// Only library-produced, nonempty keys can be views into the retained source allocation.
// Bounds use numeric offsets without forming pointer ends or subtracting unrelated pointers.
// Example: a parsed bare key at source byte zero borrows that source slice until free_toml.
toml_key_uses_source :: inline (source: string, key: string) -> bool {
    if source.count <= 0 || key.count <= 0 || !source.data || !key.data return false;

    base := cast(u64) source.data;
    address := cast(u64) key.data;

    if address < base return false;

    offset := address - base;

    return offset < cast(u64) source.count && cast(u64) key.count <= cast(u64) source.count - offset;
}
