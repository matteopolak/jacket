// Build_Options.cast_bounds_check is .FATAL by default: a value that does not fit the narrower
// target stops the program.
work :: () {
    w := 300;
    c := cast(u8) w;
}
#run work();
