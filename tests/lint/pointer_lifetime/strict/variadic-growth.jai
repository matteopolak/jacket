#import "Basic";

Owner :: struct {
    values: [..] int;
}

storage: Owner;

append_view :: (owner: *Owner, extra: [] int) -> [] int {
    assert(owner.values.allocated >= owner.values.count + extra.count);
    view: [] int;
    view.data = owner.values.data + owner.values.count;
    view.count = extra.count;
    old := owner.values.data;

    array_add(*owner.values, ..extra);
    current := owner.values.data;

    assert(current == old);

    return view;
}

main :: () {
    extra := int.[1, 2];
    view := append_view(*storage, extra);
    print("%", view);
}
