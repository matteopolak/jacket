#import "Basic";

main :: () {
    rows := int.[1, 2];

    for row: rows {
        for 0..row print("%", it);
        print(" row %\n", row);
    }
}
