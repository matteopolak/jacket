#import "Basic";

main :: () {
    rows := int.[1, 2];
    // The inner iterator has a name.
    for rows {
        for cell: 0..it print("%", cell);
        print(" row %\n", it);
    }
    // The outer `it` is copied before the inner loop.
    for rows {
        row := it;
        for 0..row print("%", it);
        print(" row %\n", row);
    }
    // The outer `it` is not needed after the inner loop.
    for rows {
        print("row %:", it);
        for 0..it print(" %", it);
        print("\n");
    }
}
