append_number :: (builder: *String_Builder, value: float64, decimals: s32) {
    format: FormatFloat;
    format.value = value;
    format.trailing_width = max(decimals, 0);
    format.zero_removal = .NO;
    // Outside this range plain digits get too long, or lose precision.
    SMALLEST_PLAIN :: 0.000001;
    LARGEST_PLAIN :: 9e15;

    magnitude := abs(value);

    if magnitude != 0 && (magnitude < SMALLEST_PLAIN || magnitude > LARGEST_PLAIN) format.mode = .SCIENTIFIC;

    print_to_builder(builder, "%", format);
}

#scope_file
#import "Basic";
#import "Math";
