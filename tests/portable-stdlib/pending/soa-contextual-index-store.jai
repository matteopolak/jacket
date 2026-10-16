#import "Soa";
S :: #import "Soa";

Item :: struct {
    value: s64;
}

main :: () {
    values: S.Soa(Item, 1);
    values[0] = .{ 17 };
}
