# Mimalloc

`#import "Extensions/Mimalloc"` binds [mimalloc](https://github.com/microsoft/mimalloc) (v2.2.4, MIT) as an `Allocator` for `context.allocator`. macOS and Linux only; other targets have an empty module.

```jai
#import "Extensions/Mimalloc";

context.allocator = mimalloc_allocator;
numbers := NewArray(1000, int);
array_free(numbers);

heap := make_heap();          // a private heap
text := copy_string("scratch",, heap);
release_heap(heap);           // frees `text` and everything else in it
```

## How it works

- `mimalloc_proc` forwards allocate, resize and free to `mi_malloc`, `mi_realloc` and `mi_free`. Blocks are 16-byte aligned; `allocate_aligned` takes any power of two.
- `mimalloc_allocator` uses the calling thread's heap and is safe from every thread. A block can be freed on a different thread from the one that allocated it.
- `make_heap` returns an `Allocator` whose data is a `mi_heap_t`. Allocate from it on the thread that made it; `release_heap` frees every block in one call, so there is no per-block free to forget.
- `IS_THIS_YOURS` answers by address range (`mi_is_in_heap_region`): true for any block mimalloc handed out, not only this heap's.
- `usable_size(block)` is at least the size requested; `collect()` returns unused pages to the OS now.
- Running `STARTUP`, `SHUTDOWN` or `THREAD_START` does nothing; `THREAD_STOP` calls `mi_thread_done`.

## How to change it

- Upgrade mimalloc: change `revision`, the URL and `sha256` of the `mimalloc` source in `tools/native/native-libs.json`, then `PYTHONPATH=tools python3 tools/native/build_native_libs.py mimalloc`.
- New binding: add the `#foreign` declaration under `#scope_file` in `module.jai` (C `size_t` is `u64`, `mi_heap_t*` is `*void`).
- Windows needs `bcrypt` and `advapi32` at link time and a `platforms` entry; not done yet.

## Configuration

Mimalloc reads `MIMALLOC_*` environment variables at startup (for example `MIMALLOC_VERBOSE=1`). The library is built into `target/native-libs/<os>-<arch>/` and `jaic` finds it there; release archives ship it.

## Dependencies

The native library from [`tools/native/native-libs.json`](../tools/native-libs.md), `Basic`. Tests are in `stdlib/Extensions/Mimalloc/tests`; `jacket bench --path stdlib/Extensions/Mimalloc` compares it with the default allocator. It is not faster for single-threaded small blocks; its advantage is the private heaps and cross-thread frees.
