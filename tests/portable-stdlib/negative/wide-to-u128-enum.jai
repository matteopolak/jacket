#import "Basic";

Choice :: enum {
    FIRST;
}

main :: () {
    value := to_u128(Choice.FIRST);
    print("%", value);
}
