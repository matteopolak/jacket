// String formatting and building from the compiler's own modules. Prints the length and a checksum.
#import "Basic";

main :: () {
    builder: String_Builder;

    for 0..400_000 - 1 print_to_builder(*builder, "item % is % (%)\n", it, it * 3, formatFloat(cast(float64) it / 7.0, trailing_width = 3));

    text := builder_to_string(*builder);
    checksum: u64 = 1469598103934665603;

    for text checksum = (checksum ^ it) * 1099511628211;

    print("% %\n", text.count, checksum);
}
