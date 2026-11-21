#import "Basic";
Board :: #import "Extensions/Embedded/Boards/Qemu_Virt_Aarch64";

main :: () {
    print("before\n");
    assert(false, "boom");
}

#program_export "jai_start"
reset :: () #c_call #no_context {
    Board.start(main);
}
