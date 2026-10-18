#import "Basic";

// Seven levels: the seventh block is past the default limit of six.
deep :: (n: int) -> int {
    total := 0;

    for 0..n {
        if it > 0 {
            while total < 100 {
                for 0..it {
                    if it > 1 {
                        if it > 2 {
                            if it > 3 {
                                total += it;
                            }
                        }
                    }
                }

                break;
            }
        }
    }

    return total;
}

// A switch is a level of its own, and a procedure keeps the depth of the block it is written in.
carried :: (n: int) {
    if n > 0 {
        if n > 1 {
            if n > 2 {
                if n > 3 {
                    if n > 4 {
                        if n > 5 {
                            helper :: (x: int) {
                                if x > 8 {
                                    print("%\n", x);
                                }
                            }

                            helper(n);

                            if n == {
                                case 8; print("eight\n");
                            }
                        }
                    }
                }
            }
        }
    }
}

main :: () {
    print("%\n", deep(5));
    carried(9);
}
