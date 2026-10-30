// test: skip_on(.WASM, reason = "needs Extensions/Io fibers, which wasm cannot run (no stack switching)")
// The same ownership assertions with no initial slab and immediate stack release.
Io :: #import "Extensions/Io"(TEMPORARY_BYTES = 0, POOL_LIMIT = 0);

#load "support/allocator-contract.jai";
