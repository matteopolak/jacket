// Original general reproducer for the reviewed Socket field-element type idiom.
#import "Basic";

Samples :: struct {
    words: [4] u64;
}

Word :: type_of(Samples.words[0]);

main :: () {
    #assert Word == u64;
    #assert size_of(Word) == 8;
    print("ok\n");
}
