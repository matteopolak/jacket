#import "Basic";
#import "Soa";
S :: #import "Soa";
Again :: #import "Soa";

Item :: struct {
    x: s32;
    y: float;
}

main :: () {
    values: Soa(Item, 2);
    values[0] = Item.{ 3, 5 };
    values[1] = Item.{ 7, 11 };
    first := values[0];
    second := values[1];

    assert(first.x == 3 && first.y == 5 && second.x == 7 && second.y == 11);
    assert(values.x[0] == 3 && values.y[1] == 11);
    assert(type_info(S.Soa(Item, 2)) == type_info(Again.Soa(Item, 2)));
    print("ok\n");
}
