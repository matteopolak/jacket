// Sanitized independent shape provided by root. Official native verdict: output 7 and true, then fault; no continuation.
#import "Basic";

main :: () {
    value := 7;
    pointer := *value;

    print("%\n", pointer.*);
    pointer = null;
    back := *(pointer.*);

    print("%\n", back == null);
    print("%\n", pointer.*);
    print("unreachable\n");
}
