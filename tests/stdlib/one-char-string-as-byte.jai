// A one-character string constant converts to u8 (comparisons and switch cases on bytes).
#import "Basic";
main :: () {
    c: u8 = #char "'";
    assert(c == "'");
    hit := false;
    if c == {
        case "'"; hit = true;
        case "x"; hit = false;
    }
    assert(hit);
}
