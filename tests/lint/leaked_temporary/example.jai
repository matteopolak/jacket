#import "Basic";

is_seven :: (n: int) -> bool {
    return sprint("%", n) == "7";
}

main :: () {
    print("%\n", is_seven(7));
}
