#import "Basic";

random_string :: () -> string {
    return copy_string("hello",, temp);
}

fill :: (destination: *void) {
    (cast(*string) destination).* = random_string();
}

main :: () {
    mark := get_temporary_storage_mark();
    value: string;

    fill(*value);
    assert(value == "hello");
    set_temporary_storage_mark(mark);
}

saved: string;

keep :: () {
    fill(*saved);
}

read_after_restore :: () {
    mark := get_temporary_storage_mark();
    value: string;

    fill(*value);
    set_temporary_storage_mark(mark);
    assert(value == "hello");
}
