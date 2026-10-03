# Unmapping_Allocator


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `Unmapping_Allocator :: struct`
- `get_unmapping_allocator :: () -> Allocator`
- `deinit_unmapping_allocator :: (a: Allocator)`
- `deinit :: (ua: *Unmapping_Allocator)`
- `unmapping_allocator_proc :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Unmapping_Allocator`

```jai
Unmapping_Allocator :: struct {

    table: Table(*void, u64);
    mutex: Mutex;
}
```
## Contracts and constraints

`get_unmapping_allocator()` creates an allocator backed by OS virtual-memory reservation and page protection. Allocation rounds the writable region to pages and positions the returned address so the requested range ends immediately before a protected page; zero/small Windows requests receive at least an 8-byte writable region. Allocation failure logs the OS error and returns null. Free changes the old region to read-only and retains its mapping so stale reads/writes fault; addresses are not recycled. Resize always allocates a new region, copies `min(old_size, requested_size)` when allocation succeeds, then protects the old region. If new allocation fails, the old region is left unprotected/live. This allocator is expensive and memory consumption grows with freed allocations. It supports thread calls through a mutex but rejects heap creation/destruction and ownership queries by assertion. `deinit_unmapping_allocator` destroys tracking metadata/mutex; it is not a substitute for freeing/protecting outstanding allocations. OS page-protection support controls availability.


Platform coverage note: PS5 branches exist, but their declarations are not specified here; exports for those targets are unspecified.
