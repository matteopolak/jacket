#import "Basic";
#import "String";

make_name :: (first: string) -> string {
    name := join(first, " Smith");
    defer free(name);
    return name;
}

make_buffer :: (size: int, zeroed: bool) -> *u8 {
    buffer := cast(*u8) alloc(size);

    defer free(buffer);

    if zeroed {
        memset(buffer, 0, size);
        return buffer;
    }

    return xx buffer;
}

main :: () {
    print("%\n", make_name("Ann"));
    print("%\n", make_buffer(8, true));
}
