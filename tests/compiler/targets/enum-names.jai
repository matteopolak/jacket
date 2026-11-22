#import "Basic";

E :: enum {
    ALPHA;
    BETA;
    GAMMA;
}

names :: () -> string {
    info := cast(*Type_Info_Enum) type_info(E);
    out: String_Builder;
    for n: info.names print(*out, "[%:%] ", it_index, n);
    return builder_to_string(*out);
}

main :: () {
    text := names();
    defer free(text);
    print("%\n", text);
}
