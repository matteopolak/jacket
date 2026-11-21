#import "Basic";

main :: () {
    push_allocator(temp);
    names: [..] string;
    array_add(*names, "ada");

    print("%\n", names.count);
}
