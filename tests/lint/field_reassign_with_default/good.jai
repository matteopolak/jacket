#import "Basic";

Config :: struct {
    width: int = 80;
    height: int = 24;
    title: string;
}

Pair :: union {
    a: int;
    b: float;
}

make :: (title: string, flag: bool) -> Config {
    // Already a literal.
    first := Config.{ width = 100 };

    // Something happens between the declaration and the assignments.
    second: Config;

    print("making\n");
    second.width = 5;

    // The value reads the variable.
    third: Config;
    third.width = third.height * 2;

    // Nested members and compound assignments.
    fifth: Config;
    fifth.width += 1;

    // A union takes one member.
    pair: Pair;
    pair.a = 1;

    // Assigned only conditionally.
    sixth: Config;
    if flag sixth.width = 9;

    return first;
}

main :: () {
    c := make("x", true);
    print("%\n", c);
}
