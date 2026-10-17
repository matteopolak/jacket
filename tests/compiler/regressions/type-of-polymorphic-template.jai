// A polymorphic struct template is a `Type` value that prints as its name, and
// `type_of` of a polymorphic procedure keeps its signature, with `$` for the parts that
// depend on a type variable.
#import "Basic";
#import "String";

Entity :: struct(Payload: Type) {
    payload: Payload;
}

other_poly :: (x: $T, y: int) -> T, string {
    return x, "";
}

with_pointer :: (x: *$T, y: [$N] int, s: string) {}

baked :: ($N: int, f: float) -> int {
    return N;
}

with_rest :: (x: int, rest: ..Any) -> $R {
    return 0;
}

main :: () -> s64 {
    kind := type_of(Entity);
    if kind != Type return 1;
    if tprint("%", Entity) != "Entity" return 2;

    template := Entity;
    if template != Entity return 3;
    if template == Entity(int) return 4;
    if !contains(tprint("%", Entity(int)), type_info(Entity(int)).name) return 5;

    info := cast(*Type_Info_Procedure) type_of(other_poly);
    if info.argument_types.count != 2 || info.return_types.count != 2 return 10;
    if info.argument_types[1] != type_info(int) || info.return_types[1] != type_info(string) return 6;

    pointer_info := cast(*Type_Info_Procedure) type_of(with_pointer);
    if pointer_info.argument_types.count != 3 || pointer_info.argument_types[2] != type_info(string) return 7;

    baked_info := cast(*Type_Info_Procedure) type_of(baked);
    if baked_info.argument_types.count != 2 || baked_info.argument_types[0] != type_info(int) || baked_info.return_types[0] != type_info(int) return 8;

    rest_info := cast(*Type_Info_Procedure) type_of(with_rest);
    if rest_info.argument_types.count != 2 || rest_info.argument_types[0] != type_info(int) return 9;

    return 0;
}
