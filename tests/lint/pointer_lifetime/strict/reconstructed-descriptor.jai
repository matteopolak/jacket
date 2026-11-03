#import "Basic";

modified_view :: (flag: bool) -> string {
    text := tprint("%", "192.0.2.1:0");
    if flag return string.{ text.count - 4, text.data + 1 };
    return string.{ text.count - 2, text.data };
}

mixed_view :: (flag: bool) -> string {
    if flag return "static";
    text := tprint("%", "temporary");
    return string.{ text.count, text.data };
}

durable_view :: () -> string {
    return copy_string(modified_view(true));
}

saved: string;

main :: () {
    saved = modified_view(true);
    saved = mixed_view(true);
    saved = durable_view();
}

view_of :: (text: string) -> string {
    return string.{ text.count, text.data };
}

dangling_view :: () {
    value := copy_string("abc");
    old_view := view_of(value);

    free(value);
    print("%", old_view[0]);

    temporary := tprint("%", "abc");
    old_temporary := view_of(temporary);

    reset_temporary_storage();
    print("%", old_temporary[0]);
}
