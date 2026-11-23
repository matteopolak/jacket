#import "Basic";

// A string builder holds 4 KiB, or 256 bytes where pointers are 2 bytes wide.
BUILDER_BYTES :: STRING_BUILDER_BUFFER_SIZE + size_of(String_Builder.Buffer);
#assert BUILDER_BYTES == ifx size_of(*void) == 2 then 256 else 4096;

main :: () {}
