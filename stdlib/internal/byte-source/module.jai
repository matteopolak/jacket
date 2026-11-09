// Bytes that arrive in pieces, for the readers that take input a piece at a time (Json, Csv).
// Each of them names this type, so one source works with all of them.

// Puts the next piece in `buffer` and returns how many bytes it wrote, 0 at the end. A non-empty
// string says the source failed and why. `data` is passed back as it was given.
Byte_Fill :: #type (data: *void, buffer: [] u8) -> s64, string;

// Where the pieces come from: `fill` is called with `data` until it returns 0 or fails.
Byte_Source :: struct {
    data: *void;
    fill: Byte_Fill;
}
