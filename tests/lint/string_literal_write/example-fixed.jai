#import "Basic";

main :: () {
    word := copy_string("hello");
    word[0] = #char "j";
    print("%\n", word);
    free(word);
}
