#import "Basic";
G :: #import "developer-tools-canonical-generated";

main :: () {
    record: G.OriginalRecord;

    G.original_initialize(*record, 11, 22);
    assert(G.original_sum(*record) == 55);
    G.original_global = 31;
    assert(G.original_sum(*record, 9) == 73);
    assert(G.OriginalEnum.ORIGINAL_FAILED == cast(G.OriginalEnum) -7);
    assert(G.OriginalAlias == G.OriginalRecord && G.Constants.Answer == 5);
    assert(size_of(G.OriginalRecord) == 16 && align_of(G.OriginalRecord) == 8);
    container: G.OriginalContainer;

    G.original_container_initialize(*container, 12, 18);
    assert(container.inner.payload == 12 && container.tail == 18);
    container.inner.payload = 24;
    assert(G.original_container_read(*container) == 42);
    print("ok\n");
}
