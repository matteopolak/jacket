#import "Basic";

main :: () {
    // Exactly -1 femtosecond in the reviewed signed128 representation.
    input := Apollo_Time.{ 0xffffffffffffffff, -1 };
    seconds, valid := to_float64_seconds(input);

    if !valid exit(2);

    if seconds == 0 {
        write_string("negative femtosecond converted to zero\n");
        exit(3);
    }

    if seconds != -1e-15 exit(4);
}
