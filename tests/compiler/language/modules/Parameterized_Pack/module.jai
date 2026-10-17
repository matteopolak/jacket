#module_parameters (BASE: s64 = 5);

sequence := 0;

configured_value :: () -> s64 {
    return BASE;
}

next_sequence :: () -> s64 {
    sequence += 1;
    return sequence;
}
