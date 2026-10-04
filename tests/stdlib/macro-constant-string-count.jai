#import "Basic";

takes_u64 :: (n: u64) -> u64 { return n; }

report :: (message: string) -> u64 #expand {
    return takes_u64(message.count);
}

shrink :: (message: string) -> int #expand {
    message.count -= 1;
    return message.count;
}

main :: () {
    assert(report("hello") == 5);
    assert(shrink("hello") == 4);
    print("ok\n");
}
