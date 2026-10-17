__array_bounds_check_fail :: (index: s64, limit: s64, line_number: s64, filename: *u8) #no_context {
    report_location(filename, line_number);
    write_string("array index out of bounds: ", true);
    write_number(index, 10, true);
    write_string(" >= ", true);
    write_number(limit, 10, true);
    write_string("\n", true);
    c_abort();
}

__cast_bounds_check_fail :: (pre_value: s64, pre_flags: u32, post_value: s64, post_flags: u32, fatal: u8, line_number: s64, filename: *u8) #no_context {
    report_location(filename, line_number);
    write_string("cast out of bounds\n", true);
    if fatal != 0 c_abort();
}

__null_pointer_check_fail :: (index: s64, line_number: s64, filename: *u8) #no_context {
    report_location(filename, line_number);
    write_string("null pointer access\n", true);
    c_abort();
}

__arithmetic_overflow :: (left: s64, right: s64, type_code: u16, line_number: s64, filename: *u8) #no_context #no_aoc {
    report_location(filename, line_number);
    write_string("arithmetic overflow computing ", true);
    if (type_code & 0x4000) != 0 write_number(left, 10, true);
    else write_nonnegative_number(cast(u64) left, 10, true);

    operation := (type_code >> 7) & 3;

    if operation == 1 write_string(" + ", true);
    else if operation == 2 write_string(" - ", true);
    else if operation == 3 write_string(" * ", true);
    else write_string(" ? ", true);
    if (type_code & 0x4000) != 0 write_number(right, 10, true);
    else write_nonnegative_number(cast(u64) right, 10, true);

    write_string(" as ", true);
    if (type_code & 0x4000) != 0 write_string("s", true);
    else write_string("u", true);

    write_number(cast(s64) (type_code & 0x7f) * 8, 10, true);
    write_string("\n", true);
    if (type_code & 0x8000) != 0 c_abort();
}

one_time_init :: (synch_value: *s32, to_insert: Code) #expand {
    won, previous := compare_and_swap(synch_value, cast(s32) 0, cast(s32) 1);
    if won {
        defer {
            finished, old := compare_and_swap(synch_value, cast(s32) 1, cast(s32) 2);
        }

        #insert to_insert;
    } else {
        finished, old := compare_and_swap(synch_value, cast(s32) 2, cast(s32) 2);
        while !finished finished, old = compare_and_swap(synch_value, cast(s32) 2, cast(s32) 2);
    }
}

#scope_file
report_location :: (filename: *u8, line: s64) #no_context {
    write_string(to_string(filename), true);
    write_string(":", true);
    write_number(line, 10, true);
    write_string(": ", true);
}
