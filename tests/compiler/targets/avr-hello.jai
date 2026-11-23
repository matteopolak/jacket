#import "Basic";
Board :: #import "Extensions/Embedded/Boards/Arduino_Uno";

// A string builder on the stack fits 2 KiB of RAM.
#assert STRING_BUILDER_BUFFER_SIZE + size_of(String_Builder.Buffer) == 256;

main :: () {
    print("hello\n");
}

#program_export "jai_start"
reset :: () #c_call #no_context {
    Board.start(main);
}
