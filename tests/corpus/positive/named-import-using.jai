using,only(scale) S :: #import,file "support/scaled.jai"(Factor = 3);
main :: () -> int { return scale(4) + S.scale(1); }
