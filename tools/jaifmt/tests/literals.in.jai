main :: () {
    ünïcödé := 1;
    big := 1_000_000;
    hex := 0xFF_FF;
    flt := 1.5e-3;
    half := .5;
    member := v.x;
    range := 0..10;
    uninit: int = ---;
    poly :: ($T: Type, x: $$U) {}
    `outer := 1;
    note: int; @Note
    code :: #code { a := 1; };
    #insert code;
    v := #run compute();
    words :: string.["a","b"];
    proc :: () #expand { `x := 1; }
    p :: (a: int) #modify { return true; } {}
    args :: (rest: ..int) { other(..rest); }
    group\_fraction := 3;
}
operator == :: (a: V, b: V) -> bool { return a.x==b.x; }
operator []= :: (a: *V, i: int, x: float) {}
operator * :: (a: V, s: float) -> V #symmetric { return .{a.x*s}; }
