#import "Basic";

Choice :: enum {
    FIRST;
}

main :: () {
    value := to_s128(Choice.FIRST);
    print("%", value);
}
