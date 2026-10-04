#import "Basic";

main :: () {
    s := "abcdef";
    t := s.data;
    multi := true;
    t += ifx multi then 3 else 1;
    assert(t.* == #char "d");
    p := t - (ifx multi then 2 else 1);
    assert(p.* == #char "b");
    q: *u8 = s.data;
    assert(p - q == 1);
    print("ok\n");
}
