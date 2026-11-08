#import "Basic";
#import "Extensions/Toml";

Stats :: struct {
    allocations: int;
}

count_float :: (mode: Allocator_Mode, size: s64, old_size: s64, memory: *void, data: *void) -> *void {
    stats := cast(*Stats) data;
    if mode == .ALLOCATE stats.allocations += 1;
    backing := context.default_allocator;
    return backing.proc(mode, size, old_size, memory, backing.data);
}

check_bits :: (bits: u64) {
    root: Toml_Value;
    value: Toml_Value;
    value.kind = .FLOAT;
    value.real = cast,force(float64) bits;
    array_add(*root.members, Toml_Member.{ key = "x", value = *value });
    defer array_free(root.members);
    text, error := write_toml(*root);

    assert(error.kind == .NONE, error.message);
    defer free(text);
    doc, read_error := parse_toml(text);

    assert(read_error.kind == .NONE, text);
    defer free_toml(*doc);
    assert((cast,force(u64) toml_get(doc.root, "x").real) == bits, text);
    exact, exact_error := write_toml(*root, .{ max_output_bytes = text.count });

    assert(exact_error.kind == .NONE && exact == text);
    free(exact);
    _, short_error := write_toml(*root, .{ max_output_bytes = text.count - 1 });

    assert(short_error.kind == .LIMIT);
    limited, inside_error := write_toml(*root, .{ max_output_bytes = text.count - 2 });

    assert(limited.count == 0 && inside_error.kind == .LIMIT);
    assert(toml_error_path(*inside_error) == "$[\"x\"]");
}

writer_allocations :: (floating: bool) -> int {
    root: Toml_Value;
    values: [12] Toml_Value;

    for *values {
        it.kind = ifx floating then .FLOAT else .INTEGER;
        if floating it.real = 1.25;
        else it.integer = 0;
        array_add(*root.members, Toml_Member.{ key = tprint("key%", it_index), value = it });
    }

    defer array_free(root.members);
    stats: Stats;

    {
        push_allocator(count_float, *stats);
        text, error := write_toml(*root);
        assert(error.kind == .NONE);
        free(text);
    }

    return stats.allocations;
}

intrinsic_allocations :: () -> int {
    stats: Stats;

    {
        push_allocator(count_float, *stats);
        scratch: String_Builder;

        defer free_buffers(*scratch);
        real: float64 = 1.25;
        format: FormatFloat;
        format.value = real;
        format.mode = .SHORTEST;

        for 1..12 {
            print_float(*scratch, format);
            assert(!scratch.failed);
            reset(*scratch);
        }
    }

    return stats.allocations;
}

main :: () {
    style := context.print_style;

    defer context.print_style = style;
    context.print_style.default_format_float.decimal_separator_character = 44;
    context.print_style.default_format_float.exponent_character = 69;
    context.print_style.default_format_float.thousand_separators = true;
    context.print_style.default_format_float.positive_number_prefix = .PLUS;
    for u64.[0, 0x8000000000000000, 1, 0x8000000000000001, 0x000fffffffffffff, 0x0010000000000000, 0x0010000000000001, 0x7fefffffffffffff, 0xffefffffffffffff, 0x3fefffffffffffff, 0x3ff0000000000000, 0x3ff0000000000001, 0x433fffffffffffff, 0x4340000000000000, 0x4340000000000001] check_bits(it);

    state: u64 = 0x9e3779b97f4a7c15;

    for 1..512 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        finite := state;
        if ((finite >> 52) & 0x7ff) == 0x7ff finite ^= 0x0010000000000000;

        check_bits(finite);
    }

    integers := writer_allocations(false);
    floats := writer_allocations(true);
    intrinsic := intrinsic_allocations();

    // Platform float conversion may allocate. The writer must not add a heap string
    // per value beyond those required by the same public conversion primitive.
    assert(floats <= integers + intrinsic, tprint("integer=% float=% intrinsic=%", integers, floats, intrinsic));
    doc, error := parse_toml("x=1.25");

    assert(error.kind == .NONE);
    text, write_error := write_toml(doc.root);

    assert(write_error.kind == .NONE && text == "x = 1.25\n", text);
    free(text);
    free_toml(*doc);

    // Shortest spelling: plain decimals while the exponent is small, always a float.
    spellings := string.["1.0", "-0.0", "100.0", "0.1", "0.05", "0.00012", "123456.789", "1.5e-7", "1e+21", "5e-324", "1.7976931348623157e+308", "123456789012345680.0", "0.0001"];

    for spellings {
        source := tprint("x = %\n", it);
        parsed, parse_error := parse_toml(source);

        assert(parse_error.kind == .NONE, it);
        written, written_error := write_toml(parsed.root);

        assert(written_error.kind == .NONE);
        assert(written == source || (it == "1.5e-7" && written == "x = 1.5e-07\n"), tprint("% -> %", it, written));
        free(written);
        free_toml(*parsed);
    }

    print("ok\n");
}
