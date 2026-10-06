// A #no_context procedure takes no hidden context, so naming `context` in it is an error.
// rules: ctx.8 ctx.9
#import "Basic";

lonely :: () -> u32 #no_context {
    return context.thread_index;
}

main :: () {
    lonely();
}
