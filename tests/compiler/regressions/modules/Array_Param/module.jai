// A parameter `[$N] T` with a `.[...]` default has the default's count when the import gives no value.
#module_parameters(NAMES: [$N] string = .[], FLAGS: [$M] int = .[1, 2, 3]);

name_count :: () -> int { return NAMES.count; }

flag_sum :: () -> int {
    total := 0;
    for FLAGS total += it;
    return total;
}
