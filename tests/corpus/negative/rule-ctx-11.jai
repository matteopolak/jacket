// #add_context is only legal at file scope; inside a procedure body it is rejected.
// rules: ctx.11
main :: () {
    #add_context extra: int = 1;
}
