#import "Basic";

is_live :: (code: int) -> bool {
    return code != 404 || code != 410;
}

main :: () {
    print("%\n", is_live(200));
}
