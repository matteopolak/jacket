#import "Basic";

Point :: struct {
    x: int;
    y: int;
};

helper :: () {
    print("helper\n");
};

main :: () {
    total := 0;
    total += 1;

    if total > 0 {
        print("positive\n");
    }

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
