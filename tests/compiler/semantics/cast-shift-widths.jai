// Independent explicit-conversion/SDK encoding checks; no stdlib imports.
// Prefix casts include shifts; narrow shift result promotion remains unspecified.
widen_then_shift :: (value: u8) -> u32 #no_context {
    return (cast(u32) value) << 8;
}

comma_then_shift :: (value: u8) -> u32 #no_context {
    return cast(u32, value) <<8;
}

encode_control :: (group: u8, command: u8) -> u32 #no_context {
    return 0x80040000 | ((cast(u32) group) << 8) | (cast(u32) command);
}

// Observation only: do not turn the current numeric width into a reference oracle.
prefix_shift_observation :: (value: u8) -> u32 #no_context {
    return cast(u32) value << 8;
}

#program_export "prefix_control_observation"
prefix_control_observation :: (group: u8, command: u8) -> u32 #c_call {
    return 0x80040000 | (cast(u32) group << 8) | (cast(u32) command);
}

#assert(encode_control(0x66, 126) == 0x8004667e);
#assert(widen_then_shift(0xa5) == 0xa500);
#assert(comma_then_shift(0xa5) == 0xa500);

#program_export "cast_shift_check"
check :: () -> s32 #c_call {
    checked: u32;

    for index: 0..255 {
        value := cast(u8) index;
        expected := (cast(u32) index) * 256;

        if widen_then_shift(value) != expected return 1;
        if comma_then_shift(value) != expected return 2;
        if (widen_then_shift(value) >> 8) != cast(u32) value return 3;
        if encode_control(value, 126) != (0x8004007e | expected) return 4;

        checked += 1;
    }

    if checked != 256 return 5;

    return 0;
}

main :: () -> s32 {
    return check();
}
