#import "Basic";

main :: () {
    i := 0;

    while true {
        if i >= 10 break;
        i += 1;
    }

    print("%\n", i);
}
