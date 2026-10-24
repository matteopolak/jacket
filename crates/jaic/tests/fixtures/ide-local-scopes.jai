#import "Basic";

Settings :: struct {
    level: int;
}

tune :: (settings: *Settings) {
    settings.level += 1;
}

#run {
    options: Settings;

    tune(*options);

    for 0..1 {
        inner := it + options.level;
        print("%\n", inner);
    }
}

Twice :: struct(N: int) #modify {
    doubled := N * 2;
    N = doubled;
} {
    value: [N] int;
}

main :: () {
    t: Twice(3);
    print("%\n", t.value.count);
}
