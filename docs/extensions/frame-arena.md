# Frame_Arena

`#import "Extensions/Frame_Arena"` has two fixed-capacity bump allocators for data that lives one frame: `Frame_Arena` (allocate, then `reset`) and `Double_Frame_Arena` (two arenas swapped by `next_frame`, so the previous frame's data survives one more frame).

```jai
#import "Extensions/Frame_Arena";

frames: Double_Frame_Arena;
assert(init(*frames, 1 << 20));
defer fini(*frames);

context.allocator = as_allocator(*frames);
while running {
    next_frame(*frames);            // empties the arena from two frames ago
    build_scene();                  // allocates freely
    diff(previous(*frames));        // last frame's data is still valid here
}
```

## How it works

- `allocate` aligns the cursor (16 bytes by default, any power of two) and advances it. A full arena returns null; nothing grows.
- `reset` rewinds the cursor and counts a frame. With `poison_on_reset` the used bytes become `0xcc`, which makes a stale pointer obvious.
- `resize` grows the latest allocation in place; any other block is copied to the end and its old space stays used until `reset`. FREE does nothing.
- `Double_Frame_Arena.next_frame` makes the other arena current and resets it. Its allocator always allocates from the current arena, so it keeps working across frames. Resizing a block from the previous frame copies it into the current one.
- `statistics` reports `high_water`, the most the arena has held, to size the capacity.
- Not thread safe: give each thread its own arena.

`Pool` and `Flat_Pool` are also bump allocators. They grow, or assert when their address range runs out, and have no frame lifetime; the arenas have a hard capacity and a high-water mark and return null when full.

## How to change it

- Alignment and bump logic: `allocate` in `module.jai`. Keep the overflow-safe comparison (`size > capacity - start`).
- A deeper history (three or more frames): generalize `Double_Frame_Arena.arenas` and `current_index`; `previous` and `owns` assume two.

## Configuration

`Frame_Arena.poison_on_reset`. `init(arena, capacity, backing)` takes the block from `backing` (default `context.allocator`); `init_in_memory` takes memory you own.

## Dependencies

`Basic`. `jacket bench --path stdlib/Extensions/Frame_Arena`.
