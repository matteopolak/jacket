// rules: ptr.13
// Indexing is bounds checked by default: an out-of-range index stops the program (here a #run).
work :: () {
    values: [4] int;
    index := 7;
    values[index] = 1;
}
#run work();
