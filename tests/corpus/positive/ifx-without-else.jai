f :: (x: bool) -> string { return ifx x then "s"; }
g :: (x: bool) -> s64 { return ifx x then 5; }
main :: () -> int { return f(true).count + f(false).count + g(true) + g(false); }
