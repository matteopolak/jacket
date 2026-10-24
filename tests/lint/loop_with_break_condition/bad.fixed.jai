#import "Basic";

main :: () {
    i := 0;
    done := false;

    while i < 10 {
        i += 1;
    }

    while !done {
        i += 1;
        if i > 20 done = true;
    }

    while i == 0 || i == 3 {
        i -= 1;
    }

    while i >= 100 {
        i -= 1;
    }

    while true {
        // wait for the flag
        if i > 40 break;
        i += 1;
    }

    print("%\n", i);
}
