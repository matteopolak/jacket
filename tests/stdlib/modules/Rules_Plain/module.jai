// Module for tests/stdlib/rules-modules-and-imports.jai.
answer :: 42;
other :: 7;

hits: int;

bump :: () -> int {
    hits += 1;
    return hits;
}

// Visible to importers through the context as soon as the module is imported.
#add_context rules_plain_marker := 5;

// `sub/part.jai` loads `deeper.jai` relative to itself; loading it twice is a no-op.
#load "sub/part.jai";
#load "sub/part.jai";

#scope_file
secret :: 1;
