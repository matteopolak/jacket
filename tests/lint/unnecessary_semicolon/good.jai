#import "Basic";

Point :: struct {
    x: int;
    y: int;
}

helper :: () {
    print("helper\n");
}

LIMIT :: 10;

main :: () {
    total := 0;
    total += 1;

    for 0..LIMIT;

    if total > 100;

    for 0..3 {
        total += it;
    }

    if total == {
        case 1;
            print("one\n");
        case;
            print("many\n");
    }

    print("%\n", total);
    helper();
}
