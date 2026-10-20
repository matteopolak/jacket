// A `#compile_time` procedure may assign to its aggregate arguments (n00bmind's reflector
// fills in a defaulted `Source_Code_Location` that way).
// rules: proc.12
#import "Basic";

fill :: (st: int, loc: Source_Code_Location = .{}) -> bool #compile_time {
    if !loc.fully_pathed_filename loc = #location();
    return loc.line_number > 0 && st == 1;
}

main :: () {
    assert(#run fill(1));
    print("ok\n");
}
