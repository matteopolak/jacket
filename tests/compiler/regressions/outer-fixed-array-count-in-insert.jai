// A block nested in a procedure, such as the `#insert -> string` below, can ask for the `.count`
// of an enclosing procedure's fixed array: the count is part of the type, so no value is read.
// rules: proc.16
#import "Basic";
#import "String";

Mask :: struct {
    words: [4] u64;
}

clear_words :: (mask: Mask) -> Mask {
    result := mask;

    #insert -> string {
        lines: [..] string;

        for index: 0..result.words.count - 1 {
            array_add(*lines, sprint("result.words[%] = 0;", index));
        }

        return join(..lines, "\n");
    };

    return result;
}

main :: () {
    mask: Mask;
    mask.words[1] = 7;
    cleared := clear_words(mask);

    assert(cleared.words[1] == 0);
    assert(mask.words[1] == 7);
    print("ok\n");
}
