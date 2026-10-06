// rules: ctexec.7
// A trap inside #run (here an out-of-range array index) is reported as a compile-time
// execution error.
values: [3] int;

read_past_end :: () -> int {
    index := 7;
    return values[index];
}

BROKEN :: #run read_past_end();

main :: () {
    x := BROKEN;
}
