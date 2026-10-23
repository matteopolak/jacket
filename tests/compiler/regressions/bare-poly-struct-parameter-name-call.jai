// A bare polymorphic struct parameter (`pairs: Table`) leaves the struct's name usable as a
// call in the body: `Table(string, s64)` is still an instantiation, not a call of the instance.
#import "Basic";
#import "Hash_Table";

copy_of :: (pairs: Table) -> int {
    copy: Table(string, s64);

    for pairs table_add(*copy, it_index, it);

    return copy.count;
}

main :: () {
    pairs: Table(string, s64);

    table_add(*pairs, "a", 1);
    table_add(*pairs, "b", 2);

    assert(copy_of(pairs) == 2);

    print("ok\n");
}
