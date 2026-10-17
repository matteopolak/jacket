// Loaded by main.jai. The debug-info tests set breakpoints by line in this file:
// keep line numbers stable (or update crates/jaic-cli/tests/debug_info.rs).
Vec2 :: struct {
    x: float;
    y: float;
}

Color :: enum u8 { RED; GREEN; BLUE; }

Node :: struct {
    value: int;
    next: *Node;
}

scale :: (v: Vec2, k: float) -> Vec2 {
    r: Vec2;
    r.x = v.x * k;
    r.y = v.y * k;
    return r;
}

total_of :: (xs: [] $T) -> T {
    total: T;
    for xs {
        total += it;  // line 25
    }
    return total;
}
