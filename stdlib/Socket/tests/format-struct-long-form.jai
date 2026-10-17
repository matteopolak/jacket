// The long form of a struct writes the name/value separator once per member.
#import "Basic";

P :: struct {
    a: int;
    b: float;
}

main :: () {
    text := tprint("%", formatStruct(P.{ 1, 2 }, use_long_form_if_more_than_this_many_members = -1));
    if text != "{a = 1; b = 2; }" {
        print("unexpected long form: %\n", text);
        exit(1);
    }
}
