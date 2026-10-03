# Hash_Table

## Public declaration inventory

Module parameter: `COUNT_COLLISIONS := false`.

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `Table :: struct (Key_Type: Type, Value_Type: Type, given_hash_function: (Key_Type) -> u32 = null, given_compare_function: (Key_Type, Key_Type) -> bool = null, LOAD_FACTOR_PERCENT: u32 = 70, REFILL_REMOVED := true)`
- `for_expansion :: (table: *$T/Table, body: Code, flags: For_Flags) #expand`
- `init :: (using table: *Table, slots_to_allocate: s64 = 0) #deprecated "Not needed: a zero-initialized Table is ready to use."`
- `deinit :: (table: *Table)`
- `table_reset_keeping_memory :: (using table: *Table)`
- `table_reset :: (using table: *Table)`
- `table_resize :: (using table: *Table, expected_count: s64)`
- `Walk_Table :: (code: Code) #expand`
- `table_ensure_space :: (using table: *Table, items: s64)`
- `table_add :: (table: *Table, key: table.Key_Type, value: table.Value_Type) -> *table.Value_Type`
- `table_set :: (table: *Table, key: table.Key_Type, value: table.Value_Type) -> *table.Value_Type`
- `table_contains :: (table: *Table, key: table.Key_Type) -> bool`
- `table_find_pointer :: (table: *Table, key: table.Key_Type) -> *table.Value_Type`
- `table_find :: (table: *Table, key: table.Key_Type) -> (success: bool, table.Value_Type)`
- `table_find_multiple :: (table: *Table, key: table.Key_Type) -> [] table.Value_Type`
- `table_remove :: (table: *Table, key: table.Key_Type) -> (success: bool, value: table.Value_Type)`
- `table_remove_multiple :: (table: *Table, key: table.Key_Type) -> (num_removed: s64)`
- `find_or_add :: (table: *Table, key: table.Key_Type) -> (entry: *table.Value_Type, newly_added: bool)`
- `NEVER_OCCUPIED_HASH :: 0`
- `REMOVED_HASH :: 1`
- `FIRST_VALID_HASH :: 2`
- `expand :: (table: *Table, new_allocated := -1)`
- `next_power_of_two :: inline (x : int) -> int`

## Public type members

#### `Table` (struct)

- `hash_function :: given_hash_function` when `given_hash_function != null`; otherwise `hash_function :: (key: Key_Type) -> u32`
- `compare_function :: given_compare_function` when `given_compare_function != null`; otherwise `compare_function :: (a: Key_Type, b: Key_Type) -> bool`
- `count:        int`
- `allocated:    int`
- `slots_filled: int`
- `allocator:      Allocator`
- `Entry :: struct { hash: u32; key: Key_Type; value: Value_Type; }`
- `entries: [] Entry`
- When module parameter `COUNT_COLLISIONS := true`: `add_collisions: s64`, `find_collisions: s64`
- `SIZE_MIN :: 32`

`COUNT_COLLISIONS` is a module parameter and defaults to `false`; the two counters are absent when disabled. `REFILL_REMOVED` defaults to `true` and controls whether insertion may reclaim tombstone slots. `LOAD_FACTOR_PERCENT` defaults to 70 and is a generic parameter; values must be less than 100 for probing to terminate.

## Behavioral contract and constraints

### `Walk_Table` expansion contract

`Walk_Table(code)` is exported and expands at the call site. The caller must make a `table` value visible, with the table shape and capacity invariants maintained by this module, and a `key` value of that table's `Key_Type`. The expansion introduces caller-scope `hash: u32` and `index: u32`, and a loop-control binding named `table_while_loop`; the latter is the current nonzero slot hash and may be used by inserted code as the loop condition/`continue` target. The `code` argument is inserted once per occupied slot encountered along the lookup probe sequence. It is not a general dense-table iterator: it stops at the first never-occupied slot, while tombstones are visited. It is intended for composing the module's lookup/update operations; user-facing whole-table iteration is provided by `for_expansion`.

The expansion requires a nonempty initialized table with power-of-two capacity when invoked. Its public signature does not encode those preconditions, and behavior for a caller violating them is unspecified. Avoid depending on the numeric probe order or on collision-counting side effects as public iteration order.

`Table(Key_Type, Value_Type, ...)` maps keys using its configured hash and equality procedures (defaults: `Hash.get_hash` and `==`). A stored key must keep a stable hash/equality identity until removed. The default load threshold is 70%; `LOAD_FACTOR_PERCENT` must be below 100. The default `REFILL_REMOVED` behavior reuses tombstones. Growth/rehash can invalidate all pointers into `entries`; removal invalidates pointers to the removed value. No thread-safety guarantee is documented.

| Operation | Observable behavior |
| --- | --- |
| `table_add` | Always inserts a new pair, even if an equal key already exists. Returns a pointer to the inserted value. Duplicate keys remain distinct entries; ordinary lookup selects the first matching entry encountered by the table probe. |
| `table_set` | Replaces the value of the first matching key and returns its pointer. If absent, inserts a new pair. It does not collapse any other duplicates. |
| `table_find_pointer` / `table_contains` | Search the first match; return null / false on an empty table or miss. Returned pointer refers into table storage and can be invalidated by a later growth, reset, or removal of that entry. |
| `table_find` | Returns `(true, value-copy)` on hit. On miss returns `false` plus an uninitialized value; use only the boolean to decide whether the value is valid. |
| `table_find_multiple` | Returns copies of all matching values in a slice allocated from `Basic.temporary_allocator`; the slice lasts only as long as that temporary-storage scope/region remains valid. Empty table or no matches returns an empty slice. The module marks this operation as not yet tested. |
| `table_remove` | Removes only the first match and returns `(true, value-copy)`. A miss (including an unallocated table) returns `false` and an uninitialized value. |
| `table_remove_multiple` | Removes every matching entry and returns the number removed; empty table returns zero. |
| `find_or_add` | Existing key returns its value pointer and `false`; absent key inserts a default/uninitialized value slot and returns its pointer with `true`. The caller must initialize the value before reading it. |
| `table_reset_keeping_memory` / `table_reset` | Both clear entries; the former retains allocated backing storage, while the latter releases it. Existing pointers and iterators must be discarded after either operation. |
| `table_resize` / `table_ensure_space` | Rebuild storage to support the requested live-entry count / additional items. Rebuild invalidates entry pointers. `table_resize` does not reduce capacity below the current count. |

The table uses tombstones after deletion so remaining probe chains continue to work; callers must use the table API rather than assuming a dense or stable entry order. `table_find_multiple` is the appropriate lookup when duplicates are intentionally stored. Allocation failure behavior beyond allocator behavior/assertions is not separately specified.
