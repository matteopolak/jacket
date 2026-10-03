Pair :: struct { left: int = 4; right: int; }
main :: () -> int { p: Pair = .{right=8}; p.left += 3; return p.left + p.right; }
