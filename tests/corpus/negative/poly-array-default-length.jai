// The default `.[]` is checked against the count inferred from the argument.
#import "Basic";

print_tags :: (prefix: string, tags: [$N] string = .[]) { print("% %\n", prefix, N); }

main :: () {
    print_tags("mtag", .["m1", "m2"]);
}
