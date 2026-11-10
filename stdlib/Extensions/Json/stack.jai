// A stack that keeps its first frames inline and spills the rest to the heap, for the tree
// reader and the tree builder: documents nest a few levels, so most never allocate.

#scope_module

// Frames, innermost last. The first `INLINE` live in the stack itself; `spill` holds the rest and
// stays empty until the stack gets that deep. `count` is the depth, so a caller that saves it
// can restore it by assigning `count` (the reader's [json_rewind] does). Free `spill` with
// `array_free` when done.
Inline_Stack :: struct(Frame: Type, INLINE: int) {
    count: int;
    base: [INLINE] Frame;
    spill: [..] Frame;
}

// Adds `frame` as the innermost one.
stack_push :: (stack: *Inline_Stack($Frame, $INLINE), frame: Frame) {
    if stack.count < INLINE {
        stack.base[stack.count] = frame;
    } else {
        // Assigning `count` may have left frames above it; the spill holds exactly the live ones.
        stack.spill.count = stack.count - INLINE;
        array_add(*stack.spill, frame);
    }

    stack.count += 1;
}

// The innermost frame; the stack must not be empty.
stack_top :: (stack: *Inline_Stack($Frame, $INLINE)) -> *Frame {
    if stack.count <= INLINE return *stack.base[stack.count - 1];

    return *stack.spill[stack.count - INLINE - 1];
}

// Removes the innermost frame and returns it; the stack must not be empty.
stack_pop :: (stack: *Inline_Stack($Frame, $INLINE)) -> Frame {
    frame := stack_top(stack).*;

    stack.count -= 1;

    return frame;
}
