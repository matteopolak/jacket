# Overwriting_Allocator


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `Overwriting_Allocator :: struct`
- `get_overwriting_allocator :: (base_allocator: Allocator = .{}, overwrite_byte: u8 = 0xcd) -> Allocator`
- `deinit_overwriting_allocator :: (a: Allocator)`
- `deinit :: (ua: *Overwriting_Allocator)`
- `overwriting_allocator_proc :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Overwriting_Allocator`

```jai
Overwriting_Allocator :: struct {
    base_allocator: Allocator;

    table: Table(*void, u64);

    overwrite_byte: u8;

    mutex: Mutex;
}
```
## Behavioral contract

`get_overwriting_allocator(base_allocator := .{}, overwrite_byte := 0xcd)` uses the provided allocator when it has a procedure, otherwise captures `context.allocator` at construction. Each successful allocation records the requested size. Fresh allocation bytes are filled with `overwrite_byte`; free fills the recorded requested extent before forwarding the free. Resize always allocates separately, returns null if that allocation fails while leaving the old allocation live, copies `min(old_size, requested_size)` bytes on success, poisons/frees the old allocation, then fills any newly grown suffix with the selected byte. It does not zero memory. Invalid or previously freed pointers are logged and asserted. This is a debug aid, not a guard-page guarantee.

Invalid/double frees are logged and asserted. Heap create/destroy and ownership-query capability modes are unsupported and assert; allocation lifecycle/thread modes pass through or no-op as declared. `deinit_overwriting_allocator` must be called when the wrapper is no longer needed. The wrapper uses a mutex and advertises multithread/free capabilities; its base allocator must not recursively be this wrapper. Depends on the wrapped allocator, table, and thread mutex APIs.
