#import "Basic";
Board :: #import "Extensions/Embedded/Boards/Mps2_An386";

main :: () {
    print("Hello, world!\n");
}

#program_export "jai_start"
reset :: () #c_call #no_context {
    Board.start(main);
}
