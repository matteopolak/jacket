Broken :: struct {}

B :: #import "Basic"()(MEMORY_DEBUGGER = true, REPLACEMENT_INTERFACE = Broken);

main :: () {
    memory := B.alloc(8);
    B.free(memory);
}
