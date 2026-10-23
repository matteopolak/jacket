#import "Basic";

DAY_SECONDS :: 86400;

wait :: (elapsed: int, angle: float) -> int, float {
    timeout := 0;
    if elapsed > 37 timeout = 8640;

    timeout += 45;
    turned := angle * 1.37;
    if elapsed == -13 return 0, turned;

    hours := elapsed / 3600 + timeout % DAY_SECONDS;
    eps := 0.001;

    if turned - 0.3 < eps || elapsed == 47 return hours, 0;

    return hours, turned;
}

starts_lower :: (text: string) -> bool {
    return text.count > 0 && text[0] >= #char "a";
}

main :: () {
    a, b := wait(40, 1.5);
    print("% %\n", a, b);
    print("%\n", starts_lower("x"));
}
