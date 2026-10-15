#import "Basic";

identity :: (code: Code) -> Code {
    return code;
}

total: int;

main :: () {
    #insert #run identity(#code {
        first := 7;
        second := 9;
        assert(first == 7 && second == 9);
        total += first + second;
    });
    assert(total == 16);
}
