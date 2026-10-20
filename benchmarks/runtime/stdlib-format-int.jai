// Integer formatting through print_to_builder, formatInt and tprint. Prints the length and a checksum.
#import "Basic";

main :: () {
    builder: String_Builder;
    state: u64 = 11;
    total := 0;

    for 0..600_000 - 1 {
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
    }

    text := builder_to_string(*builder);
    checksum: u64 = 1469598103934665603;

    for text checksum = (checksum ^ it) * 1099511628211;

    print("% % %\n", text.count, total, checksum);
}
