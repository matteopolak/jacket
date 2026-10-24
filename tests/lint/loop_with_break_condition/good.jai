#import "Basic";

main :: () {
    i := 0;
    done := false;

    while i < 10 {
        i += 1;
    }

    while true {
        i += 1;
        if i >= 10 break; // not the first statement
    }

    while true {
        if i >= 10 {
            print("done\n");
            break;
        }

        i += 1;
    }

    while true {
        if i >= 20 break;
        else i += 2;
    }

    while: outer true {
        while true {
            if i >= 30 break outer;
            i += 1;
        }
    }

    while true {
        if done return;
        done = true;
    }

    print("%\n", i);
}
