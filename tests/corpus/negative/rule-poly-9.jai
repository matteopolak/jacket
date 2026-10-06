// rules: poly.9
// #bake_arguments needs a single procedure or a polymorphic struct; a plain struct is rejected.
Plain :: struct { value: int; }
Baked :: #bake_arguments Plain(value = 1);

main :: () {
    b: Baked;
}
