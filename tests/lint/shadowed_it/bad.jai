#import "Basic";

main :: () {
    rows := int.[1, 2];
    for rows {
        for 0..it print("%", it);
        print(" row %\n", it);
    }
}
