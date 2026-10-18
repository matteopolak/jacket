#import "Basic";

// Names

/// A dashed aside - a - b is ordinary prose.
first :: () -> int {
    // -- two dashes are not a rule
    return 1;
}

/* Section: values */

// 1 - 2 = -1
second :: () -> int {
    return 2;
}

main :: () {
    print("%\n", first() + second());
}
