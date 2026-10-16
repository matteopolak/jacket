B :: #import "Basic";

Parcel :: struct {
    value: int;
}

operator == :: (a: Parcel, b: Parcel) -> bool {
    return a.value % 10 == b.value % 10;
}

#poke_name B operator==;

exact :: (a: Parcel, b: Parcel) -> bool {
    return a.value == b.value;
}

Other :: struct {
    value: int;
}

other_compare :: (a: Other, b: Other) -> bool {
    return a.value == b.value;
}

check :: () -> bool {
    items := Parcel.[.{ 12 }, .{ 23 }, .{ 32 }];
    found, index := B.array_find(items, Parcel.{ 2 });

    if !found || index != 0 return false;

    found, index = B.array_find(items, Parcel.{ 2 }, compare = exact);
    if found || index != -1 return false;

    view: [] Parcel = .{ items.count, items.data };
    if !B.array_ordered_remove_by_value(*view, Parcel.{ 2 }) return false;
    if view.count != 2 || view[0].value != 23 || view[1].value != 32 return false;
    if B.array_unordered_remove_by_value(*view, Parcel.{ 2 }, stop_after_first = true) != 1 return false;
    if view.count != 1 || view[0].value != 23 return false;

    unrelated := Other.[.{ 1 }, .{ 2 }];

    found, index = B.array_find(unrelated, Other.{ 2 }, compare = other_compare);

    return found && index == 1 && !#exists(B.copy_array_elements) && !#exists(B.allocator_call) && !#exists(B.Impl);
}

#assert check();

main :: () {
    B.assert(check());
}
