#import "Basic";
Board :: #import "Extensions/Embedded/Boards/Arduino_Uno";

// Code pointers on AVR are word addresses in program memory. Calls through them, from a table in
// RAM and from a variable, reach the right procedure, for `#c_call` and for Jai procedures.
Step :: #type () #c_call;
Twice :: #type (s32) -> s32;

step_a :: () #c_call {
    write_string("a");
}

step_b :: () #c_call {
    write_string("b");
}

step_c :: () #c_call {
    write_string("c");
}

twice :: (value: s32) -> s32 {
    return value + value;
}

steps: [3] Step = .[step_a, step_b, step_c];

main :: () {
    for step: steps step();

    function: Twice = twice;
    if function(21) == 42 write_string("\nok\n");
}

#program_export "jai_start"
reset :: () #c_call #no_context {
    Board.start(main);
}
