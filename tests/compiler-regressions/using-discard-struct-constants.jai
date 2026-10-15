// Several top-level `using _ :: struct { ... }` declarations each bring their constants into scope.
#import "Basic";

using _ :: struct {
    A :: 1;
} @Tag
using _ :: struct {
    B :: 2;
} @Tag

main :: () {
    if A + B != 3 exit(1);
    print("ok\n");
}
