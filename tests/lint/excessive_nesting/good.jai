#import "Basic";

// Six levels is within the default limit.
at_the_limit :: (n: int) -> int {
    total := 0;

    for 0..n {
        if it > 0 {
            while total < 100 {
                for 0..it {
                    if it > 1 {
                        total += it;
                    }
                }

                break;
            }
        }
    }

    return total;
}

// `else if` chains stay on the level of the first `if`, and `#if` is not a level.
chain :: (n: int) {
    if n > 0 {
        if n > 1 {
            if n > 2 {
                if n > 3 {
                    if n == 6 print("six\n");
                    else if n == 7 print("seven\n");
                    else if n == 8 {
                        #if true {
                            print("eight\n");
                        }
                    } else {
                        print("more\n");
                    }
                }
            }
        }
    }
}

// A procedure written in a shallow block starts counting from there.
shallow :: (n: int) {
    if n > 0 {
        helper :: (x: int) {
            if x > 0 {
                if x > 1 {
                    if x > 2 {
                        print("%\n", x);
                    }
                }
            }
        }

        helper(n);
    }
}

main :: () {
    print("%\n", at_the_limit(5));
    chain(8);
    shallow(3);
}
