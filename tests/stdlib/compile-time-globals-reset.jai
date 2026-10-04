#import "Basic";

// Global state changed by compile-time code does not reach the running program,
// unless the variable is declared #no_reset.

counter := 0;
#no_reset kept := 0;
table: [4] int;
#no_reset kept_table: [4] int;

runs := 0;

once :: () -> int {
    runs += 1;
    return 42;
}

#run {
    counter = 5;
    kept = 7;
    for 0..3 { table[it] = it + 1; kept_table[it] = it + 10; }
}

value := #run once();

main :: () -> s64 {
    if counter != 0 return 1;
    if kept != 7 return 2;
    if table[2] != 0 return 3;
    if kept_table[3] != 13 return 4;
    // `value := #run f()` evaluates f once, at compile time.
    if value != 42 return 5;
    if runs != 0 return 6;
    return 0;
}
