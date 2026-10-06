// `push_context,defer_pop;` names no context to push. It used to compile, and changes the block
// made to `context` leaked out of it.
main :: () {
    {
        push_context,defer_pop;
        context.thread_index = 77;
    }
}
