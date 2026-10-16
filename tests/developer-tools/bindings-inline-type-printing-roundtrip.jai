#import "Basic";
G :: #import "developer-tools-inline-generated";

main :: () {
    container: G.Container;

    assert(size_of(G.Container) == 8 && align_of(G.Container) == 4);
    container.inner.payload = 12;
    container.tail = 18;
    assert(container.inner.payload + container.tail == 30);
    container.inner.payload = 24;
    assert(container.inner.payload + container.tail == 42);
    print("ok\n");
}
