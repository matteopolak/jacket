#import "Basic";

main :: () {
    parts: [..] string;

    defer array_free(parts);

    array_add(*parts, "a");
    array_add(*parts, "b");

    for parts print("%\n", it);

    numbers: [..] int;

    array_add(*numbers, 4);
    array_add(*numbers, 5);
    array_add(*numbers, 6);

    total := 0;

    for i: 0..numbers.count - 1 {
        total += numbers[i];
    }

    print("%\n", total);
}
