#import "Basic";

Holder :: struct { views: [] [] int; }

local_views :: () {
    values: [2] int;
    view := [] int.[values];
    holder: Holder;
    holder.views = view;
    print("%", holder.views[0][0]);
}

bad_view :: () -> [] int {
    values: [2] int;
    return values;
}
