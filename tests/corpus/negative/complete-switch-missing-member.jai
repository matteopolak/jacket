// An `if #complete` over an enum must name every member.
#import "Basic";
E :: enum { A; B; C; }
main :: () {
    e := E.A;
    if #complete e == {
        case .A; print("a");
        case .B; print("b");
    }
}
