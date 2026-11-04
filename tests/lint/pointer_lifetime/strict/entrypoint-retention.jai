#import "Basic";

outer_saved: *int;
inner_saved: *int;
helper_saved: *int;

helper :: () {
    local := 7;
    helper_saved = *local;
}

main :: () {
    outer := 42;

    outer_saved = *outer;

    {
        inner := 8;
        inner_saved = *inner;
    }

    print("%", outer_saved.*);
}
