// One top-level decode owns the blocks used by all its children. A header and its
// payload share one allocation; only actual field bytes are copied into payload chunks.
Storage_Block :: struct {
    next: *Storage_Block;
    count: s64;
    parent: Allocator;
    arena: [] u8;
    arena_limit: s64;
    arena_block: *Storage_Block;
    scrap: *Scrap;
}

// A dead region of the arena (an outgrown array buffer or a block tail) that later
// small allocations are served from. The link lives inside the dead memory itself.
Scrap :: struct {
    next: *Scrap;
    size: s64;
}

// Allocator scope used while replacing/freeing fields: interior payload pointers are
// retained until their owner releases the entire block. Other allocations free normally.
Storage_Allocator :: struct {
    parent: Allocator;
    storage: **Storage_Block;
    hot: *Storage_Block;
}

// Ignores individual frees into owned blocks, delegating every other operation.
storage_allocator :: (mode: Allocator_Mode, size: s64, old_size: s64, old: *void, data: *void) -> *void {
    scope := cast(*Storage_Allocator) data;

    if mode == .FREE && old && retains_pointer(scope, old) return null;

    return scope.parent.proc(mode, size, old_size, old, scope.parent.data);
}

// Adds one bounded payload allocation to the current top-level message's storage.
add_storage :: (r: *Reader, count: s64) -> [] u8 {
    block := cast(*Storage_Block) alloc(size_of(Storage_Block) + count);
    block.* = .{ count = count, parent = context.allocator, arena_limit = min(16384, r.bytes.count * 8) };

    if r.storage.* {
        block.next = r.storage.*.next;
        r.storage.*.next = block;
    } else r.storage.* = block;

    return .{ count, cast(*u8) (block + 1) };
}

// Places one field in a geometrically grown chunk bounded by unread input bytes.
payload_bytes :: (r: *Reader, count: s64) -> [] u8 {
    if !count return .{};

    if count > r.payload.count {
        r.chunk_size = min(r.bytes.count - r.pos, max(count, max(16384, r.chunk_size * 2)));
        r.payload = add_storage(r, r.chunk_size);
    }

    result: [] u8 = .{ count, r.payload.data };

    r.payload.data += count;
    r.payload.count -= count;

    return result;
}

// Shared generated-message free wrapper; nested messages with no independent blocks
// use the enclosing scope. Hand-built values still release their ordinary allocations.
free_message :: inline (value: *$T) {
    blocks := value.proto_storage;
    if !blocks {
        T.proto_release(value);
        return;
    }

    scope := Storage_Allocator.{ parent = context.allocator, storage = *blocks };
    ctx := context;
    ctx.allocator = .{ storage_allocator, *scope };
    push_context ctx T.proto_release(value);

    while blocks {
        next := blocks.next;
        Basic.free(blocks);
        blocks = next;
    }
}

// Releases a replaced scalar while retaining interior pointers in decode storage.
release_scalar :: (r: *Reader, value: *$T, $kind: Scalar) {
    #if kind == .STRING || kind == .RAW_STRING || kind == .BYTES {
        if !value.data return;

        if r.storage && owns_payload(r.storage.*, value.data) {
            value.* = .{};
            return;
        }

        free_scalar(value, kind);
    }
}

// Wrap only frees, never allocations: resizable arrays retain their original allocator.
release_field :: (r: *Reader, value: *$T, $kind: Scalar) {
    scope := Storage_Allocator.{ parent = context.allocator, storage = r.storage };
    ctx := context;
    if r.storage && r.storage.* ctx.allocator = .{ storage_allocator, *scope };
    push_context ctx free_field(value, kind);
}

// Releases a temporary decoded map entry under its top-level storage scope.
release_entry :: (r: *Reader, value: *$T) {
    scope := Storage_Allocator.{ parent = context.allocator, storage = r.storage };
    ctx := context;
    if r.storage && r.storage.* ctx.allocator = .{ storage_allocator, *scope };
    push_context ctx T.proto_free(value);
}

// Releases the previous oneof selection without freeing block-backed payloads.
release_oneof :: (r: *Reader, value: *$T) {
    scope := Storage_Allocator.{ parent = context.allocator, storage = r.storage };
    ctx := context;
    if r.storage && r.storage.* ctx.allocator = .{ storage_allocator, *scope };
    push_context ctx free_oneof(value);
}

// Tests whether a field aliases any payload block in this owner.
owns_payload :: (block: *Storage_Block, pointer: *void) -> bool {
    address := cast(u64) pointer;

    while block {
        first := cast(u64) (block + 1);
        if address >= first && address < first + cast(u64) block.count return true;
        block = block.next;
    }

    return false;
}

// Larger blocks would strand too much unused tail when a decode ends.
MAX_ARENA_BLOCK :: 524288;

// Allocates arrays and message objects from a stable per-decode arena. Array
// allocators retain this heap header, never a Reader or another stack variable.
// Growing the newest allocation extends it in place; any other outgrown buffer
// becomes scrap that later allocations are carved from, so growth leaves no dead space.
structure_allocator :: (mode: Allocator_Mode, size: s64, old_size: s64, old: *void, data: *void) -> *void {
    root := cast(*Storage_Block) data;

    if mode == .FREE return null;
    if mode != .ALLOCATE && mode != .RESIZE return root.parent.proc(mode, size, old_size, old, root.parent.data);

    needed := (size + 15) & ~15;

    if old && old_size {
        held := (old_size + 15) & ~15;

        if cast(u64) old + cast(u64) held == cast(u64) root.arena.data {
            extra := needed - held;

            if extra <= root.arena.count {
                root.arena.data += extra;
                root.arena.count -= extra;
                return old;
            }

            // The only occupant of a block can be grown by resizing the block itself.
            if root.arena_block && old == cast(*void) (root.arena_block + 1) return grow_block(root, needed);
        }

        result := arena_take(root, needed);

        memcpy(result, old, min(size, old_size));
        arena_scrap(root, old, held);

        return result;
    }

    return arena_take(root, needed);
}

// Resizes the current block, which holds exactly one array, and relinks it.
grow_block :: (root: *Storage_Block, needed: s64) -> *void {
    block := root.arena_block;
    previous := root;

    while previous.next != block previous = previous.next;

    grown := cast(*Storage_Block) root.parent.proc(.RESIZE, size_of(Storage_Block) + needed, size_of(Storage_Block) + block.count, block, root.parent.data);

    previous.next = grown;
    grown.count = needed;
    root.arena_block = grown;
    root.arena = .{ 0, cast(*u8) (grown + 1) + needed };

    return grown + 1;
}

// Records a dead region; regions too small to hold the link are simply lost.
arena_scrap :: inline (root: *Storage_Block, memory: *void, size: s64) {
    if size < size_of(Scrap) return;

    node := cast(*Scrap) memory;

    node.* = .{ next = root.scrap, size = size };
    root.scrap = node;
}

// Bump-allocates from the current region, moving to a scrap region or a new block when full.
arena_take :: (root: *Storage_Block, needed: s64) -> *void {
    if root.arena.count < needed {
        arena_scrap(root, root.arena.data, root.arena.count);

        found: *Scrap;
        link := *root.scrap;
        scanned := 0;

        while link.* && scanned < 8 {
            if link.*.size >= needed {
                found = link.*;
                link.* = found.next;
                break;
            }

            link = *link.*.next;
            scanned += 1;
        }

        if found {
            root.arena = .{ found.size, cast(*u8) found };
            root.arena_block = null;
        } else {
            capacity := max(needed, root.arena_limit);

            root.arena_limit = min(root.arena_limit * 2, MAX_ARENA_BLOCK);
            block := cast(*Storage_Block) root.parent.proc(.ALLOCATE, size_of(Storage_Block) + capacity, 0, null, root.parent.data);
            block.* = .{ next = root.next, count = capacity };
            root.next = block;
            root.arena = .{ capacity, cast(*u8) (block + 1) };
            root.arena_block = block;
        }
    }

    result := root.arena.data;

    root.arena.data += needed;
    root.arena.count -= needed;

    return result;
}

// Produces the stable allocator only when a structural allocation is needed.
decode_allocator :: inline (r: *Reader) -> Allocator {
    if r.options.borrow return context.allocator;
    if !r.storage.* add_storage(r, 0);
    return .{ structure_allocator, r.storage.* };
}

// Constructs a singular message in decode storage (or the ordinary borrow allocator).
new_message :: inline (r: *Reader, $T: Type) -> *T {
    return New(T,, decode_allocator(r));
}

// Grows a repeated array with a stable arena allocator captured by its container.
append_decoded :: inline (r: *Reader, values: *[..] $T, value: T) {
    if values.allocator.proc {
        array_add(values, value);
        return;
    }

    ctx := context;
    ctx.allocator = decode_allocator(r);
    push_context ctx array_add(values, value);
}

// Moves an independently decoded child's storage into the enclosing message before
// merging it. Stable arena headers remain alive until the enclosing owner is freed.
absorb_storage :: (r: *Reader, child: **Storage_Block) {
    if !child.* || child == r.storage return;

    blocks := child.*;

    child.* = null;

    if !r.storage.* {
        r.storage.* = blocks;
        return;
    }

    tail := blocks;
    while tail.next tail = tail.next;

    tail.next = r.storage.*.next;
    r.storage.*.next = blocks;
}

// Unknown wire copies use the same stable arena as repeated containers.
append_unknown :: (r: *Reader, bytes: [] u8) {
    ctx := context;
    ctx.allocator = decode_allocator(r);
    push_context ctx array_add(r.unknown, ..bytes);
}

// Appends one re-encoded closed-enum tag byte to unknown storage.
append_unknown_byte :: (r: *Reader, byte: u8) {
    append_decoded(r, r.unknown, byte);
}

// Replaces a string with an owned copy. This accepts a slice from another decoded
// message without sharing its lifetime. A directly assigned non-block string must
// be an individually owned allocation from the message's allocator.
set_string :: (owner: *$T, field: *string, replacement: string) {
    owned := copy_string(replacement);
    r := Reader.{ storage = *owner.proto_storage };
    release_scalar(*r, field, .STRING);
    field.* = owned;
}

// Replaces bytes with an owned copy, including bytes from another decoded message.
set_bytes :: (owner: *$T, field: *[] u8, replacement: [] u8) {
    owned := allocate_bytes(replacement.count);
    if replacement.count memcpy(owned.data, replacement.data, replacement.count);

    r := Reader.{ storage = *owner.proto_storage };

    release_scalar(*r, field, .BYTES);
    field.* = owned;
}

// Avoids routing block-backed pointers through the general heap free machinery.
free_owned_pointer :: inline (pointer: *void) {
    if !pointer return;

    allocator := context.allocator;
    if allocator.proc == storage_allocator {
        scope := cast(*Storage_Allocator) allocator.data;
        if retains_pointer(scope, pointer) return;
        allocator = scope.parent;
    }

    allocator.proc(.FREE, 0, 0, pointer, allocator.data);
}

// Consecutive field releases usually hit the same chunk. Cache that interval so
// payload frees do not walk the structural-block list for every string.
retains_pointer :: inline (scope: *Storage_Allocator, pointer: *void) -> bool {
    address := cast(u64) pointer;
    hot := scope.hot;

    if hot {
        first := cast(u64) (hot + 1);
        if address >= first && address < first + cast(u64) hot.count return true;
    }

    block := scope.storage.*;
    while block {
        first := cast(u64) (block + 1);
        if address >= first && address < first + cast(u64) block.count {
            scope.hot = block;
            return true;
        }

        block = block.next;
    }

    return false;
}
