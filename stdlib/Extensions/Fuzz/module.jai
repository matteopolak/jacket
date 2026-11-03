// Builds random arguments from a type's layout and runs fuzz cases over them. [gen] makes a value
// from a seeded [Rng], favouring the edge cases that tend to break code (zero, minimum and
// maximum integers, NaN, empty and malformed strings). [run_cases] drives a table of [Case]s from
// the command line, and [run_engine] hands them to libFuzzer.
// @experimental
//
// ```jai
// r: Rng;
//
// rng_seed(*r, 7);
// first := copy_string(gen(*r, string));
//
// case_end(*r);
//
// rng_seed(*r, 7);
// again := gen(*r, string);
//
// assert(first == again);
// case_end(*r);
// ```

#import "Basic";
#load "rng.jai";
#load "values.jai";
#load "runner.jai";
