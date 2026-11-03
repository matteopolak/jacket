#import "Basic";

parse :: (text: string) -> int {
    return text.count;
}

main :: () {
    print("%\n", parse("abc"));
}
