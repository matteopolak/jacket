Shared_Print :: #import "internal/basic-context";

Formatter :: Shared_Print.Formatter;
FormatFloat :: Shared_Print.FormatFloat;
FormatStruct :: Shared_Print.FormatStruct;
FormatArray :: Shared_Print.FormatArray;
Print_Style :: Shared_Print.Print_Style;

// Wraps a float for `print` so it comes out with a chosen width, number of decimals or notation.
//
// ```jai
// assert(tprint("%", formatFloat(3.14159, trailing_width = 2)) == "3.14");
// ```
//
// - width: the least number of characters; shorter text is padded with spaces on the left. -1 for no
//   padding.
// - trailing_width: digits after the decimal point; 6 when negative.
// - mode: `.DECIMAL`, `.SCIENTIFIC` or `.SHORTEST`.
// - zero_removal: whether zeros at the end of the fraction are dropped: `.YES` drops them all,
//   `.ONE_ZERO_AFTER_DECIMAL` keeps one, `.NO` keeps them. With `trailing_width` set, dropped zeros
//   leave spaces so columns still line up.
formatFloat :: (value: Any, width := -1, trailing_width := -1, mode := FormatFloat.Mode.DECIMAL, zero_removal := FormatFloat.Zero_Removal.YES) -> FormatFloat {
    result: FormatFloat;
    result.value = value;
    result.width = width;
    result.trailing_width = trailing_width;
    result.mode = mode;
    result.zero_removal = zero_removal;

    return result;
}

// Wraps a struct for `print` with its own layout instead of the default one.
//
// - use_long_form_if_more_than_this_many_members: structs with more members than this print as `name =
//   value; ...`, shorter ones as `{a, b}`.
// - use_newlines_if_long_form: put each member of a long form on its own line.
formatStruct :: (value: Any, use_long_form_if_more_than_this_many_members := 5, use_newlines_if_long_form := false) -> FormatStruct {
    result: FormatStruct;
    result.value = value;
    result.use_long_form_if_more_than_this_many_members = use_long_form_if_more_than_this_many_members;
    result.use_newlines_if_long_form = use_newlines_if_long_form;

    return result;
}
