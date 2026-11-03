#import "Basic";

main :: () {
    n := 3;
    label := ifx n > 0 then "positive" else "not positive";
    print("%\n", label);
}
