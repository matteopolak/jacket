// `#code` inside an array literal followed by a trailing comma.
#import "Basic";

count := 0;

f :: (c: [] Code) {
    count = c.count;
}

main :: () {
    f(.[#code 2 + 4,]);
    assert(count == 1);
}
