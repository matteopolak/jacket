// A `#modify` block may assign a `$x := value` parameter as well as a type variable; what it
// leaves in the variable is the value the instance is baked with.
#import "Basic";

forced :: ($x := false) -> s64 #modify {
    x = true;
    return true;
} {
    #if x return 1;
    else return 0;
}

doubled :: ($n := 2) -> s64 #modify {
    n *= 2;
    return true;
} {
    return n;
}

main :: () {
    assert(forced() == 1);
    assert(forced(false) == 1);
    assert(doubled() == 4);
    assert(doubled(5) == 10);
    print("ok\n");
}
