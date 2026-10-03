combine :: (left: int, right: int = 7) -> int { return left*2+right; }
main :: () -> int { return combine(right=5, left=4) + combine(3); }
