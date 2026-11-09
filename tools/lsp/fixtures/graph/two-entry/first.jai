#import "Basic";
#load "shared.jai";

main :: () {
    counter: Counter;
    bump(*counter);
    print("first: %\n", counter.value);
}
