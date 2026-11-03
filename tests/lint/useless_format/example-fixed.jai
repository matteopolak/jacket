#import "Basic";

greeting :: (name: string) -> string {
    return name;
}

main :: () {
    print("%\n", greeting("Ada"));
}
