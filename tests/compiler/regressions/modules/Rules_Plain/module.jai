// Module for tests/compiler/regressions/rules-modules-and-imports.jai.
answer :: 42;
other :: 7;

hits: int;

bump :: () -> int {
    hits += 1;
    return hits;
}

// Visible to importers through the context as soon as the module is imported.
#add_context rules_plain_marker := 5;

// `sub/part.jai` loads `deeper.jai` relative to itself.
#load "sub/part.jai";

#scope_file
secret :: 1; // jailint: allow(unused_private_declaration) the test checks that importers cannot see it
