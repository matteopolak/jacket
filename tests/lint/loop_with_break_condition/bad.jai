#import "Basic";

main :: () {
    i := 0;
    done := false;

    while true {
        if i >= 10 break;
        i += 1;
    }

    while true {
        if done {
            break;
        }

        i += 1;
        if i > 20 done = true;
    }

    while true {
        if !(i == 0 || i == 3) break;
        i -= 1;
    }

    while true {
        if i < 100 {
            break;
        }

        i -= 1;
    }

    while true {
        // wait for the flag
        if i > 40 break;
        i += 1;
    }

    print("%\n", i);
}
