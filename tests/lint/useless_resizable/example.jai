#import "Basic";

main :: () {
    parts: [..] string;

    defer array_free(parts);

    array_add(*parts, "a");
    array_add(*parts, "b");

    for parts print("%\n", it);
}
