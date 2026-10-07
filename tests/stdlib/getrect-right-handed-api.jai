// Right-handed (y-up) GetRect public API: rect math, ordering of cuts, and the standard signatures.
// GetRect (through Simp) needs native C libraries and a window system, which the WASM targets lack
// (tests/stdlib-runtime-skips.txt lists those runs); getrect-right-handed-geometry.jai tests the
// geometry it shares on every target.
G :: #import "GetRect";
WT :: #import "Window_Type";

near :: (a: float, b: float) -> bool {
    d := a - b;
    if d < 0 d = -d;
    return d < 0.0001;
}

layout_cases :: () -> bool {
    r := G.get_rect(10, 20, 100, 60);

    // cut_top takes the highest-y slice; the remainder keeps the original bottom.
    top, rest := G.cut_top(r, 15, 5);
    if !near(top.y, 65) || !near(top.h, 15) || !near(top.x, 10) || !near(top.w, 100) return false;
    if !near(rest.y, 20) || !near(rest.h, 40) return false;

    // cut_bottom takes the lowest-y slice; the remainder starts above it.
    bottom, above := G.cut_bottom(r, 15, 5);
    if !near(bottom.y, 20) || !near(bottom.h, 15) || !near(above.y, 40) || !near(above.h, 40) return false;

    // Stacking widgets downward means walking y down from the top of the rect.
    row := G.get_rect(0, 100, 50, 10);
    row.y -= row.h + 2;
    if !near(row.y, 88) return false;

    left, right_rest := G.cut_left(r, 30, 0);
    right, left_rest := G.cut_right(r, 30, 0);
    if !near(left.x, 10) || !near(right_rest.x, 40) || !near(right.x, 80) || !near(left_rest.w, 70) return false;

    // y-up containment: the top edge (y+h) is exclusive, the bottom edge (y) inclusive.
    if !G.is_inside(10, 20, r) || G.is_inside(10, 80, r) || !G.is_inside(10, 79.9, r) return false;

    shrunk := G.shrink(r, 5);
    if !near(shrunk.x, 15) || !near(shrunk.y, 25) || !near(shrunk.w, 90) || !near(shrunk.h, 50) return false;

    both := G.intersect(r, G.get_rect(50, 0, 100, 30));
    if !near(both.x, 50) || !near(both.y, 20) || !near(both.w, 60) || !near(both.h, 10) return false;

    p0, p1, p2, p3 := G.get_quad(G.get_rect(0, 0, 2, 3));
    if p0.y != 0 || p2.y != 3 || p1.x != 2 || p3.x != 0 return false;
    return true;
}

// Compile-time proof that the entry points use the standard signatures.
frame_signature :: (window: WT.Window_Type) {
    background_occluded, is_active := G.ui_per_frame_update(window, 640, 480, 0.0);
    if background_occluded && is_active return;
}

main :: () -> s64 {
    if !layout_cases() return 1;
    return 0;
}
