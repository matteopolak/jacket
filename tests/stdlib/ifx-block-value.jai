#import "Basic";

main :: () {
    n := 7;
    tabs := false;
    s := ifx !tabs then "spaces" else {
        n = 1;
        "tabs";
    }
    assert(s == "spaces" && n == 7);
    tabs = true;
    s = ifx !tabs then "spaces" else {
        n /= 2;
        tprint("tabs %", n);
    }
    assert(s == "tabs 3" && n == 3);
    print("ok\n");
}
