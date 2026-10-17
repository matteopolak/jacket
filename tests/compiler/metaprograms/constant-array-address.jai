#import "Basic";

table :: string.["one", "two"];

main :: () {
    first := table.data;
    second := table.data;
    assert(first == second, "constant array data-address identity is unresolved");
}
