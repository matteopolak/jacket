#import "Basic";

// Counts the bytes in `text`.
parse :: (text: string) -> int {
    return text.count;
}

main :: () {
    print("%\n", parse("abc"));
}
