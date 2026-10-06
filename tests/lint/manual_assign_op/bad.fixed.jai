#import "Basic";

State :: struct {
    score: int;
}

main :: () {
    total := 0;
    total += 5;
    s: State;
    s.score *= 2;
    mask: u32 = 0xF0;
    mask |= 1;
    f := 1.5;
    f -= 0.5;
    print("% % % %\n", total, s, mask, f);
}
