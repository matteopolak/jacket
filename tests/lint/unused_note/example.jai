#import "Basic";

// Nothing in the build reads `@Exported`: the metaprogram that collected these was removed.
answer :: () -> int {
    return 42;
} @Exported

main :: () {
    print("%\n", answer());
}
