#import "Basic";

text :: () -> string #foreign;

main :: () {
    for 0..3 {
        result := text();
        assert(result == "hello");
        free(result);
    }
}

stale :: () {
    p := New(int);

    for 0..3 {
        assert(p.* == 0);
        free(p);
    }
}
