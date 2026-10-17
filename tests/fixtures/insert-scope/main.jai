#import "Basic";

main :: () {
    a :: #code b := 2;
    #insert a;
    print("b: %\n", b);
}
