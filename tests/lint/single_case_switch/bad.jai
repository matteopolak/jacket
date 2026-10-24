#import "Basic";

Mode :: enum {
    QUIET;
    VERBOSE;
    DEBUG;
}

announce :: (mode: Mode) {
    if mode == {
        case .VERBOSE;
            print("loud\n");
    }
}

choose :: (n: int) -> string {
    if n % 2 == {
        case 0;
            return "even";
        case;
            return "odd";
    }

    return "";
}

fallback :: (name: string) {
    if name == {
        case;
            print("anonymous\n");
        case "root";
            print("admin\n");
    }
}

main :: () {
    announce(.VERBOSE);
    print("%\n", choose(3));
    fallback("root");
}
