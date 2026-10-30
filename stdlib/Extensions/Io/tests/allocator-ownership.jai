// test: skip_on(.WASM, reason = "needs Extensions/Io fibers, which wasm cannot run (no stack switching)")
// Cross-context ownership with a small initial scratch slab and one retained stack.
Io :: #import "Extensions/Io"(TEMPORARY_BYTES = 64, POOL_LIMIT = 1);

#load "support/allocator-contract.jai";
