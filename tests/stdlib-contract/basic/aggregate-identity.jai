// Public defaults and canonical type identity across repeated identical imports.
#import "Basic";
Basic_Alias :: #import "Basic";

Pair :: struct {
    left: s64;
    right: s64;
}

#assert(Basic_Alias.String_Builder == String_Builder);
#assert(Basic_Alias.FormatInt == FormatInt);

main :: () {
    builder: String_Builder;

    assert(!builder.initialized && !builder.failed);
    assert(builder.subsequent_buffer_size == -1);
    assert(STRING_BUILDER_BUFFER_SIZE == 4096 - size_of(String_Builder.Buffer));

    integer := formatInt(7);

    assert(integer.base == 10 && integer.minimum_digits == 1);
    assert(integer.padding == #char "0");
    assert(integer.digits_per_comma == 0 && integer.comma_string == "");

    floating := formatFloat(1.25);

    assert(floating.width == -1 && floating.trailing_width == -1);
    assert(floating.mode == FormatFloat.Mode.DECIMAL);
    assert(floating.zero_removal == FormatFloat.Zero_Removal.YES);
    assert(sprint("%", floating) == "1.25");

    array_format: FormatArray;

    assert(array_format.separator == ", ");
    assert(array_format.begin_string == "[" && array_format.end_string == "]");
    assert(array_format.stop_printing_after_this_many_elements == 100);
    assert(!array_format.draw_separator_after_last_element);

    pair_format := formatStruct(Pair.{ left = 3, right = 9 });

    assert(pair_format.use_long_form_if_more_than_this_many_members == 5);
    assert(sprint("%", pair_format) == "{3, 9}");

    print("ok\n");
}
