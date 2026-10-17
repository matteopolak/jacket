// Default public Flat_Pool aggregate values, without reserving OS memory.
#import "Basic";
FP :: #import "Flat_Pool";

main :: () {
    pool: FP.Flat_Pool;

    assert(pool.alignment == 8);
    assert(pool.current_point == null && pool.memory_base == null);
    assert(pool.first_uncommitted_page == null);
    assert(pool.address_limit == null);
    print("ok\n");
}
