#import "Basic";

identity :: (code: Code) -> Code {
    return code;
}

main :: () {
    #insert #run identity(#code {
        first := 1;
        first := 7;
    });
}
