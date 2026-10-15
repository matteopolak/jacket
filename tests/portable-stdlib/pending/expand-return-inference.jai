// Original general language reproducer, without module/symbol special casing.
value :: (input: s64) #expand {
    return input * 2;
}

main :: () {
    result := value(21);
    if result != 42 assert(false);
}

#import "Basic";
