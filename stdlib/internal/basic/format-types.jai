Shared_Print :: #import "internal/basic-context";

Formatter :: Shared_Print.Formatter;
FormatFloat :: Shared_Print.FormatFloat;
FormatStruct :: Shared_Print.FormatStruct;
FormatArray :: Shared_Print.FormatArray;
Print_Style :: Shared_Print.Print_Style;

formatFloat :: (value: Any, width := -1, trailing_width := -1, mode := FormatFloat.Mode.DECIMAL, zero_removal := FormatFloat.Zero_Removal.YES) -> FormatFloat {
    result: FormatFloat;
    result.value = value;
    result.width = width;
    result.trailing_width = trailing_width;
    result.mode = mode;
    result.zero_removal = zero_removal;

    return result;
}

formatStruct :: (value: Any, use_long_form_if_more_than_this_many_members := 5, use_newlines_if_long_form := false) -> FormatStruct {
    result: FormatStruct;
    result.value = value;
    result.use_long_form_if_more_than_this_many_members = use_long_form_if_more_than_this_many_members;
    result.use_newlines_if_long_form = use_newlines_if_long_form;

    return result;
}
