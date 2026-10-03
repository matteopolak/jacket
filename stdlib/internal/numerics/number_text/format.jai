append_number :: (builder: *String_Builder, value: float64, decimals: s32) {
    format: FormatFloat;
    format.value = value;
    format.trailing_width = max(decimals, 0);
    format.zero_removal = .NO;
    magnitude := abs(value);
    if magnitude != 0 && (magnitude < 0.000001 || magnitude > 9e15) format.mode = .SCIENTIFIC;

    print_to_builder(builder, "%", format);
}

#scope_file
#import "Basic";
#import "Math";
