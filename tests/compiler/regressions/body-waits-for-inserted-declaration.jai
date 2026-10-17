// A procedure body may use a name another body declares into the file through
// `#insert,scope(...)`, whichever is lowered first.
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
    declares();
    if uses() != 5 exit(1);
    print("ok\n");
}
