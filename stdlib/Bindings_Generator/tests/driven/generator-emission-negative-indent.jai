#import "Basic";
B :: #import "Bindings_Generator";

main :: () {
    builder: String_Builder;

    defer free_buffers(*builder);
    generator: B.Generator;
    generator.tab_depth = -1;
    context.generator = *generator;
    B.append_indentation(*builder);
}
