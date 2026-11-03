#import "Basic";

is_seven :: (n: int) -> bool {
    return tprint("%", n) == "7";
}

main :: () {
    print("%\n", is_seven(7));
}
