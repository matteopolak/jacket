# Flat_Pool

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

Module parameters: `DEFAULT_VIRTUAL_MEMORY_RESERVE := 256 * 1024 * 1024` bytes and `OVERWRITE_ALL_POOLS_ON_RESET := false`.

- `Flat_Pool :: struct`
- `get :: (using pool: *Flat_Pool, nbytes: s64) -> *void`
- `init :: (using pool: *Flat_Pool, reserve := DEFAULT_VIRTUAL_MEMORY_RESERVE)`
- `reset :: (using pool: *Flat_Pool, overwrite_memory := false)`
- `flat_pool_allocator_proc :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void`

- `CAPS_VERSION_STRING :: "modules/Flat_Pool"`
- module parameter `DEFAULT_VIRTUAL_MEMORY_RESERVE := 256 * 1024 * 1024`
- module parameter `OVERWRITE_ALL_POOLS_ON_RESET := false`
- `example :: ()`
- `fini :: (using pool: *Flat_Pool)`

## Public type members

#### `Flat_Pool` (struct)

- `alignment : s32 = 8`
- `current_point: *u8`
- `memory_base:            *u8`
- `first_uncommitted_page: *u8`
- `address_limit:          *u8`

## Behavioral contract and constraints

`init(pool, reserve := DEFAULT_VIRTUAL_MEMORY_RESERVE)` has an unannotated integer `reserve` parameter, defaulted from the integer module parameter (the module does not explicitly declare a fixed-width type). It rounds the requested byte count up to the OS page size, reserves that address range, and initializes the bump cursor and commit boundary. `get(nbytes)` lazily calls `init` if `memory_base` is null. It aligns the current bump position to `alignment` (default 8), returns that address, and advances the cursor by `nbytes`; it does not initialize the bytes. Alignment must be a power of two, though it is not validated. A request beyond the reserved range asserts; a page-commit failure returns null without advancing the cursor for that request. Negative sizes are invalid. Successful allocations remain at stable addresses until reset; there is no per-allocation free.

`reset(pool, overwrite_memory := false)` fills the previously allocated prefix with byte `0xcc` when either the call argument is true or the module parameter `OVERWRITE_ALL_POOLS_ON_RESET` is true, then rewinds the cursor; it retains the reservation and committed pages. The options combine with OR: a per-call false cannot disable a true module parameter. Reset invalidates every prior allocation. `flat_pool_allocator_proc` treats FREE as a no-op, implements resize by allocating a new region and copying `min(old_size, requested_size)` bytes, and advertises fast bump allocation plus ownership query. It rejects thread and heap lifecycle requests by assertion. `fini` releases the reserved address range; callers must not retain or use its allocations afterward. Allocation failure can return null on commit failure; reserve exhaustion asserts.

Platform coverage note: exports for PS5 and Nintendo targets are unspecified.
