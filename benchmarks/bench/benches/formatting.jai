// Number formatting through print_to_builder, formatInt, formatFloat and tprint.
#import "Basic";
#import "Extensions/Testing";

// One line of text with an integer and two floats, `item 5 is 15 (0.714)`.
format_item :: (b: *Bench) {
    builder: String_Builder;

    reset_timer(b);

    for 0..b.iterations - 1 {
        print_to_builder(*builder, "item % is % (%)\n", it, it * 3, formatFloat(cast(float64) it / 7.0, trailing_width = 3));

        if builder_string_length(*builder) > 1 << 20 {
            reset(*builder);
        }
    }

    black_box(builder_string_length(*builder));
    free_buffers(*builder);
} @bench @samples(20)

// Three integers in decimal, one in hexadecimal and one padded to eight digits per iteration, and a
// tprint every 64th.
format_integers :: (b: *Bench) {
    builder: String_Builder;
    state: u64 = 11;
    total := 0;

    reset_timer(b);

    for 0..b.iterations - 1 {
        state = state * 6364136223846793005 + 1442695040888963407;
        signed := cast(s64) (state >> 20) - (1 << 42);
        small := cast(u32) (state >> 50);

        print_to_builder(*builder, "% % %,", signed, small, it);
        print_to_builder(*builder, "%;", formatInt(state >> 32, base = 16));
        print_to_builder(*builder, "%", formatInt(cast(int) small, minimum_digits = 8));

        if it % 64 == 0 {
            text := tprint("% %", it, signed);
            total += text.count;
            reset_temporary_storage();
        }

        if builder_string_length(*builder) > 1 << 20 {
            reset(*builder);
        }
    }

    black_box(total);
    black_box(builder_string_length(*builder));
    free_buffers(*builder);
} @bench @samples(20)
