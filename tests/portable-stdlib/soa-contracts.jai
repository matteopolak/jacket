#import "Basic";
#import "Soa";
S :: #import "Soa";
Again :: #import "Soa";

Nested :: struct {
    value: s64;
}

Item :: struct {
    x: s64;
    weight: float;
    nested: Nested;
    constant :: 17;
}

main :: () {
    values: S.Soa(Item, 3);

    #assert type_of(values.x) == [3] s64;
    #assert type_of(values.weight) == [3] float;
    #assert type_of(values.nested) == [3] Nested;
    #assert !#exists(S.Soa(Item, 3).constant);
    #assert !#exists(S.make_fields);
    #assert !#exists(S.make_access);
    reader: (S.Soa(Item, 3), int) -> Item = S.operator [];
    repeated: (Again.Soa(Item, 3), int) -> Item = Again.operator [];

    if reader != repeated exit(10);
    if type_info(S.Soa(Item, 3)).name != "Soa" exit(2);
    for index: 0..2 values[index] = Item.{ index + 10, cast(float) index + 0.5, .{ index + 100 } };

    for index: 0..2 {
        value := values[index];
        if value.x != index + 10 || value.weight != cast(float) index + 0.5 || value.nested.value != index + 100 exit(3);
        if values.x[index] != value.x || values.weight[index] != value.weight || values.nested[index].value != value.nested.value exit(4);
    }

    visited: s64;

    for value, index: values {
        if index != visited || value.x != index + 10 exit(5);
        visited += 1;
    }

    if visited != 3 exit(6);

    for *value, index: values {
        value.x += 20;
        value.nested.value += 30;
        if index == 0 continue;
        if index == 1 break;
    }

    if values.x[0] != 30 || values.x[1] != 31 || values.x[2] != 12 exit(7);
    if values.nested[0].value != 130 || values.nested[1].value != 131 || values.nested[2].value != 102 exit(8);

    empty: S.Soa(Item, 0);

    visited = 0;
    for empty visited += 1;
    if visited != 0 exit(9);

    S.example();
    write_string("ok\n");
}
