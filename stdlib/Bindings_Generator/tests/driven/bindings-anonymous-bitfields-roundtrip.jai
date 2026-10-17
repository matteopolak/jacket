#import "Basic";
G :: #import "developer-tools-anonymous-bits-generated";

main :: () {
    value: G.Container;

    assert(size_of(G.Container) == G.size() && align_of(G.Container) == G.alignment());
    G.initialize(*value);
    assert(G.get_inner_first(value) == 5 && G.get_inner_second(value) == 17 && value.tail == 99);
    G.set_inner_first(*value, 2);
    assert(G.get_inner_first(value) == 2 && G.get_inner_second(value) == 17 && value.tail == 99);
    G.set_inner_second(*value, 3);
    assert(G.read(*value) == 104 && value.tail == 99);
    print("ok\n");
}
