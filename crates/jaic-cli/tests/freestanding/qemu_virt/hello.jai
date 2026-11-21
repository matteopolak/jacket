#import "Basic";
Board :: #import "Extensions/Embedded/Boards/Qemu_Virt_Aarch64";

main :: () {
    array: [..] int;
    for 1..50 array_add(*array, it * it);
    print("hello from % cells, last %\n", array.count, array[49]);
}

#program_export "jai_start"
reset :: () #c_call #no_context {
    Board.start(main);
}
