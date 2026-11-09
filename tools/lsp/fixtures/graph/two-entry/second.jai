#import "Basic";
#load "shared.jai";
#load "second_only.jai";

main :: () {
    counter: Counter;
    bump(*counter);
    bump(*counter);
    print("second: % %\n", counter.value, label());
}
