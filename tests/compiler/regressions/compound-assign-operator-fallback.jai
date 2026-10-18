// `a op= b` is `a = a op b` when no `operator op=` accepts the operands, even though one exists for others.
#import "Basic";

V :: struct { x, y: float; }

operator / :: (a: V, k: float) -> V { return .{ a.x / k, a.y / k }; }
operator /= :: (a: *V, b: V) { a.x /= b.x; a.y /= b.y; }

calls: int;
divisor :: () -> float { calls += 1; return 2.0; }

main :: () {
    v := V.{4, 2};
    v /= divisor();
    assert(v.x == 2 && v.y == 1);
    assert(calls == 1);
    v /= V.{2, 1};
    assert(v.x == 1 && v.y == 1);
}
