// Formats an integer as hexadecimal; the other `fh` variants add a minimum number of digits.
fh :: #bake_arguments formatInt(base = 16);
// Formats an integer as hexadecimal with at least two digits.
fh2 :: #bake_arguments formatInt(minimum_digits = 2, base = 16);
// Formats an integer as hexadecimal with at least four digits.
fh4 :: #bake_arguments formatInt(minimum_digits = 4, base = 16);
// Formats an integer as hexadecimal with at least eight digits.
fh8 :: #bake_arguments formatInt(minimum_digits = 8, base = 16);

#scope_file
using #import "Basic";
