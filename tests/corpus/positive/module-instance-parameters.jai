A :: #import,file "support/scaled.jai"(Factor=3);
B :: #import,file "support/scaled.jai"(Factor=5);
main :: () -> int { return A.scale(4) + B.scale(2); }
