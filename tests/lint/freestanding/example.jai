#import "Basic";

main :: () {
    names: [..] string;
    array_add(*names, "ada");

    print("%\n", names.count);
}
