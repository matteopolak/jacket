// print shows a null Type value as "(null)". Found through The_Way_to_Jai
// examples/26/26.31_tagged_union.jai, which prints a zeroed Type tag.
#import "Basic";

Tagged :: struct {
    tag: Type;
    value: s64;
}

main :: () {
    t: Type;
    print("%\n", t);
    x: Tagged;
    print("%\n", x);
}
