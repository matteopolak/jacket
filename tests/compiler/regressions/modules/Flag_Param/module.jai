#module_parameters(MODES := Mode.WRITE) {
    Mode :: enum_flags u32 {
        READ;
        WRITE;
        APPEND;
    }
}

can_read :: () -> bool {
    return (MODES & .READ) != 0;
}

can_write :: () -> bool {
    return (MODES & .WRITE) != 0;
}

can_append :: () -> bool {
    return (MODES & .APPEND) != 0;
}
