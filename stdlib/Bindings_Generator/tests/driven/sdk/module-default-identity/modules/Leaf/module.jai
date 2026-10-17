#module_parameters (FLAG := true, SECOND := true)();

Record :: struct {
    value: s64;
}

// Type-only identity control; no SDK or runtime behavior is simulated.
pointer :: () -> *Record {
    return null;
}
