// A module header example.
//
// ```jai
// total := sum(1, 2);
// assert(total == 3);
// ```

// A declaration example.
//
// ```jai
// r := add(1, 2);
// if r > 2 {
//     print("big\n");
// }
// ```
add :: (a: int, b: int) -> int {
    return a + b;
}

// A file-level example with its own main.
//
// ```jai
// helper :: (x: int) -> int {
//     return x * 2;
// }
//
// main :: () {
//     print("%\n", helper(2));
// }
// ```
helper :: () {}

// Options stay on the fence.
//
// ```jai,no_run
// x := 1;
// ```
//
// ```jai,only(linux, macos) should_fail
// assert(1 == 2);
// ```
//
// ```jai skip_on(wasm)
// y := 2;
// ```
options :: () {}

// Marked ignore or compile_fail, so left alone.
//
// ```jai,ignore
// x:=1;
// ```
//
// ```jai,compile_fail
// y:=oops(;
// ```
//
// Not Jai, so left alone.
//
// ```
// z:=1;
// ```
//
// ```c
// int z=1;
// ```
skipped :: () {}

// Does not parse, so left alone.
//
// ```jai
// x:=(1;
// ```
//
// ```jai
// if x {
// ```
//
// ```jai
// x:=1; }
// foo::()  {
// ```
unparseable :: () {}

// Turned off.
//
// ```jai
// // jaifmt: off
// m:=int.[1,0,
//          0,1];
// ```
//
// ```jai
// a:=1;
// // jaifmt: off
// b:=   2;
// ```
off :: () {}

// jaifmt: off
// ```jai
// x:=1;
// ```
// jaifmt: on

// ```jai
// x := 1;
// ```
after_off :: () {}

Outer :: struct {
    // A doc comment inside a struct, indented.
    //
    // ```jai
    // o := Outer.{ x = 1, y = 2 };
    // for 0..3 {
    //     print("%\n", it);
    // }
    // ```
    //
    // Text between examples is kept as is   .
    //
    //   ```jai
    //   nested := 1;
    //   ```
    x: int;
    y: int;

    Inner :: struct {
        // ```jai
        // i: Outer.Inner;
        // if i.z {
        //     print("z");
        // }
        // ```
        z: bool;
    }
}

// Statements with blocks and strings.
//
// ```jai
// text:=#string END
//   keep   this
// END;
// s:="a   b";
// ```
strings :: () {}

// A block that is already formatted stays put.
//
// ```jai
// x := 1;
// ```
//
// Blank lines are kept.
//
// ```jai
// a := 1;
//
// b := 2;
// ```
tidy :: () {}

main :: () {
    // Comments inside code are found too.
    //
    // ```jai
    // v := 1 + 2;
    // ```
    x := 1; // ```jai
}
