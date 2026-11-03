#import "Basic";

main :: () {
    items := int.[1, 2, 3, 4];
    total := 0;

    for items {
        total += it;
        continue;
    }

    print("%\n", total);
}
