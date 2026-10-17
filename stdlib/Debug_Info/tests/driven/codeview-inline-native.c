static __attribute__((always_inline)) inline int original_inline_leaf(int value) {
    int next = value + 3;
    return next * next;
}

static __attribute__((always_inline)) inline int original_inline_outer(int value) {
    int adjusted = original_inline_leaf(value);
    return adjusted + original_inline_leaf(value + 1);
}

__attribute__((noinline)) int original_inline_entry(int value) {
    volatile int retained = original_inline_outer(value);
    return retained + original_inline_leaf(value + 2);
}
