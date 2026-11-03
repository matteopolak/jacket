#import "Basic";

greeting :: (name: string) -> string {
    return tprint("%", name);
}

main :: () {
    print("%\n", greeting("Ada"));
}
