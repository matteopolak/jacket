// rules: code.9
#import "Basic";

twice :: (call: Code) -> s64 #expand {
    r := #insert call;
    return r * 2;
}

fail :: (msg: string) -> s64 #expand {
    return twice(msg.count);
}

main :: () {
    if true {
        local :: (n: int) -> s64 #expand {
            return twice(n + base);
        }
        base := 10;
        assert(local(1) == 22);
    }
    assert(fail("abc") == 6);
    print("ok\n");
}
