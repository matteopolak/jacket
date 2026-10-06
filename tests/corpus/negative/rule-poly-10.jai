// rules: poly.10
// Baking more arguments than the procedure has parameters is rejected.
pair :: (a: int, b: int) -> int { return a + b; }
over :: #bake_arguments pair(1, 2, 3);

main :: () {
    x := over();
}
