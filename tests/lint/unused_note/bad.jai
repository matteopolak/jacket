#import "Basic";

// No compile-time code in this build looks at these notes.
Vertex :: struct {
    position: [3] float; @glsl
}

exported :: () -> int {
    return 42;
} @Exported

LIMIT :: 10; @Tunable

main :: () {
    vertex: Vertex;
    print("% % %\n", vertex.position[0], exported(), LIMIT);
}
