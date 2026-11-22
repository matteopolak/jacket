#import "Basic";
Board :: #import "Extensions/Embedded/Boards/Arduino_Uno";

main :: () {
    print("hello\n");
}

#program_export "jai_start"
reset :: () #c_call #no_context {
    Board.start(main);
}
