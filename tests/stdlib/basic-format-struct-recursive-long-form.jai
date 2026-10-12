// FormatStruct.recursive_long_form_depth: nested structs reached through a long-form struct
// take the same format, forced to long form, for the requested number of levels.
#import "Basic";

Leaf :: struct {
    a: s64 = 1;
    b: s64 = 2;
}

Middle :: struct {
    leaf: Leaf;
    n: s64 = 3;
}

Top :: struct {
    middle: Middle;
    m: s64 = 4;
}

main :: () -> s64 {
    value: Top;

    plain := formatStruct(value, use_long_form_if_more_than_this_many_members = -1);

    if tprint("%", plain) != "{middle = {{1, 2}, 3}; m = 4; }" return 1;

    one := plain;
    one.recursive_long_form_depth = 1;
    if tprint("%", one) != "{middle = {leaf = {1, 2}; n = 3; }; m = 4; }" return 2;

    all := plain;
    all.recursive_long_form_depth = -1;
    if tprint("%", all) != "{middle = {leaf = {a = 1; b = 2; }; n = 3; }; m = 4; }" return 3;

    // The setting does not leak into later prints, nor into a short-form parent.
    if tprint("%", value) != "{{{1, 2}, 3}, 4}" return 4;
    short := formatStruct(value, use_long_form_if_more_than_this_many_members = 5);
    short.recursive_long_form_depth = -1;
    if tprint("%", short) != "{{{1, 2}, 3}, 4}" return 5;

    named := all;
    named.draw_type_name = true;
    if tprint("%", named) != "Top{middle = Middle{leaf = Leaf{a = 1; b = 2; }; n = 3; }; m = 4; }" return 6;
    return 0;
}
