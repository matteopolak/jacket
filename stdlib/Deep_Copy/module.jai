// Deep copies of values and the data they point to, with `Deep_Copy`.
//
// ```jai
// n: s64 = 5;
// copy := Deep_Copy(*n, .{});
// assert(copy != *n && copy.* == 5);
// ```

#module_parameters(
    // Tracks the depth of each copy, for debugging `Deep_Copy` itself.
    DC_DEBUG := false,
    // With `DC_DEBUG`, prints every value the copy visits.
    DEBUG_PRINT := false
);

#load "../internal/memory/deep-copy-types.jai";
#load "../internal/memory/deep-copy.jai";
