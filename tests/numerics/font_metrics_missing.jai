// Expected diagnostic: no Font ABI/metric provider is present in the reviewed input.
G :: #import "GetRect";

main :: () {
    G.em(1.0, null);
}
