// Used by tests/stdlib/dead-code-elimination.jai. Nothing calls `stale` or `loud`: with the
// default dead-code elimination their bodies are never checked, so neither the type error nor
// the failing #run surfaces.

twice :: (x: int) -> int {
    return 2 * x;
}

stale :: () {
    x: int = "a module body nothing calls";
}

loud :: () {
    #run assert(false, "the #run of a module body nothing calls ran");
}

// A file a module loads is module code too.
#load "loaded.jai";
