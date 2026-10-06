// `#load`ed by tests/stdlib/rules-modules-and-imports.jai: shares that file's namespace, so it
// sees the loader's declarations and the loader sees its own.
loaded_value :: () -> int {
    return LOADER_BASE + 1;
}
