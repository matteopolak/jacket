#import "Basic";
#import "String";

main :: () {
    line := "  hello  ";

    line = trim(line);

    print("[%]\n", line);
}
