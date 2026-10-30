// `#code return` with no value, as an argument, is the code of a bare `return`.
// rules: macro.13
#import "Basic";

or_return :: (ok: bool, handler: Code) #expand {
    if !ok {
        #insert handler;
    }
}

run :: (ok: bool) {
    or_return(ok, #code return);
    print("reached\n");
}

main :: () {
    run(true);
    run(false);
    print("ok\n");
}
