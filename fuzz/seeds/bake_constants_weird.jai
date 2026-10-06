f :: ($N: int) -> int { return N; }
g :: #bake_constants f(N=1<<62);
h :: #bake_constants g(N=1);
main :: () { x := g(); }
