// Cross-context ownership with a small initial scratch slab and one retained stack.
Io :: #import "Extensions/Io"(TEMPORARY_BYTES = 64, POOL_LIMIT = 1);

#load "support/allocator-contract.jai";
