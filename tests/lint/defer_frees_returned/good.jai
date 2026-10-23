#import "Basic";
#import "String";

name_length :: (first: string) -> int {
    name := join(first, " Smith");
    defer free(name);
    return name.count;
}

make_buffer :: (size: int, ok: bool) -> *u8 {
    buffer := cast(*u8) alloc(size);
    defer if !ok free(buffer);
    return buffer;
}

handed_on :: (size: int) -> *u8 {
    buffer := cast(*u8) alloc(size);

    defer free(buffer);
    result := buffer;

    buffer = null;

    return result;
}

copy_of :: (first: string) -> string {
    name := join(first, " Smith");
    defer free(name);
    return copy_string(name);
}

main :: () {
    print("%\n", name_length("Ann"));
    print("%\n", make_buffer(8, true));
    print("%\n", handed_on(8));
    print("%\n", copy_of("Ann"));
}
