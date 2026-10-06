// Used by tests/corpus/negative/rule-dce-3.jai and rule-dce-8.jai. Nothing in those programs'
// reachable code calls `broken`, so only checking unreferenced code reaches its error.
broken :: () {
    missing_in_called_module_body();
}

fine :: () -> int { return 1; }
