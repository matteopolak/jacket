// Compiler-private diagnostic transport. The public facade exports no hook.
report_native_check :: (reason: s64, a: s64, b: s64, fatal: bool, line: s64, filename: *u8) #no_context #no_abc #no_aoc {
    write_string(to_string(filename), true);
    write_string(":", true);
    write_number(line, 10, true);
    if fatal write_string(": error: ", true);
    else write_string(": warning: ", true);

    if reason == CHECK_MISSING_RETURN write_string("reached the end of a procedure that must return a value", true);
    else if reason == CHECK_ASM_DIVISION write_string("#asm division fault: the divisor is zero or the quotient does not fit", true);
    else if reason == CHECK_ARRAY_BOUNDS {
        write_string("array bounds check failed: index ", true);
        write_number(a, 10, true);
        write_string(" is outside an array of ", true);
        write_number(b, 10, true);
        write_string(" element", true);
        if b != 1 write_string("s", true);
    } else if reason == CHECK_CAST_OVERFLOW {
        write_string("cast of ", true);
        if (b & 0x200) != 0 write_number(a, 10, true);
        else write_nonnegative_number(cast(u64) a, 10, true);

        write_string(" to `", true);
        if (b & 0x100) != 0 write_string("s", true);
        else write_string("u", true);

        write_number((b & 0xff) * 8, 10, true);
        write_string("` overflows", true);
    } else if reason == CHECK_INCOMPLETE_SWITCH {
        write_string("no case of the `#complete` switch matches its value, ", true);
        write_number(a, 10, true);
    } else if reason == CHECK_DIVISION_BY_ZERO write_string("integer division by zero", true);
    else if reason == CHECK_NULL_DEREFERENCE {
        write_string("null pointer dereference: ", true);
        if a == 1 write_string("write", true);
        else if a == 2 write_string("memory copy", true);
        else if a == 3 write_string("memory fill", true);
        else write_string("read", true);

        if b == 0 write_string(" through a null pointer", true);
        else {
            write_string(" at address 0x", true);
            write_nonnegative_number(cast(u64) b, 16, true);
            write_string(", just past null (a member of a null struct pointer?)", true);
        }
    } else if reason == CHECK_ATOMIC_ALIGNMENT {
        write_string("atomic address 0x", true);
        write_nonnegative_number(cast(u64) a, 16, true);
        write_string(" must be aligned to ", true);
        write_nonnegative_number(cast(u64) b, 10, true);
        write_string(" bytes", true);
    } else if reason == CHECK_VARIADIC_CAPACITY {
        write_string("variadic segment count ", true);
        write_nonnegative_number(cast(u64) a, 10, true);
        write_string(" exceeds the remaining pack capacity ", true);
        write_nonnegative_number(cast(u64) b, 10, true);
    } else write_string("runtime check failed", true);

    write_string("\n", true);
    if fatal check_exit(1);
}

#scope_file

CHECK_MISSING_RETURN :: 1;
CHECK_ASM_DIVISION :: 2;
CHECK_ARRAY_BOUNDS :: 3;
CHECK_CAST_OVERFLOW :: 4;
CHECK_INCOMPLETE_SWITCH :: 5;
CHECK_DIVISION_BY_ZERO :: 6;
CHECK_NULL_DEREFERENCE :: 7;
CHECK_ATOMIC_ALIGNMENT :: 8;
CHECK_VARIADIC_CAPACITY :: 9;

check_libc :: #system_library "libc"; @native(external)
check_exit :: (status: s32) #foreign check_libc "exit";
