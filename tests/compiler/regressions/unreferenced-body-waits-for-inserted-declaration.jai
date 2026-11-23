// Bodies nothing calls are checked together, so one may use a name another declares into the
// file through `#insert,scope(...)`.
#import "Basic";

Top :: #code();

uses :: () -> int {
    g: Generated;
    g.value = 5;
    return g.value;
}

declares :: () {
    #insert,scope(Top) "Generated :: struct { value: int; }";
}

main :: () {
    print("ok\n");
}
