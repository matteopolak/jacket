Buffer :: struct {
    other: u8 = 99;
}

construct :: ($T: Type, memory: *T) -> *T {
    memory.* = T.{};
    return memory;
}
