// Indexing is bounds-checked; `#no_abc` on a procedure, a `for` / `while` / `if`, or a block turns the
// check off, so a view may read past its count into memory it knows is there.
#import "Basic";

past_view :: (v: [] int) -> int #no_abc {
    return v[v.count];
}

main :: () {
    backing := int.[1, 2, 3, 4];
    view: [] int = backing;
    view.count = 2;
    if view[1] != 2 exit(1);
    if past_view(view) != 3 exit(2);
    sum := 0;
    for 0..2 #no_abc {
        sum += view[it];
    }
    if sum != 6 exit(3);
    #no_abc {
        if view[2] != 3 exit(7);
    }
    i := 2;
    while i < 4 #no_abc {
        sum += view[i];
        i += 1;
    }
    if sum != 13 exit(8);
    if view.count == 2 #no_abc then sum = view[3];
    if sum != 4 exit(9);

    a: [..] int;
    array_add(*a, 1, 3);
    array_insert_at(*a, 2, 1);
    if a.count != 3 || a[1] != 2 || a[2] != 3 exit(4);
    if pop(*a) != 3 exit(5);
    array_unordered_remove_by_index(*a, 1);
    if a.count != 1 || a[0] != 1 exit(6);
    print("ok\n");
}
