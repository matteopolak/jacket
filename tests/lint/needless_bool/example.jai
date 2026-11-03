#import "Basic";

is_positive :: (n: int) -> bool {
    if n > 0 return true;
    else return false;
}

main :: () {
    print("%\n", is_positive(3));
}
