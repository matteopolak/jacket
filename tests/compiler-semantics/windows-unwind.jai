// Pure calls expose LLVM's inferred nounwind; arithmetic after each call prevents tail calls.
#program_export "unwind_terminal"
terminal :: no_inline (value: s64) -> s64 #c_call {
    return value * 3 + 1;
}

#program_export "unwind_leaf"
leaf :: no_inline (value: s64) -> s64 #c_call {
    return terminal(value) + value;
}

#program_export "unwind_middle"
middle :: no_inline (value: s64) -> s64 #c_call {
    return leaf(value) + value;
}

#program_export "unwind_entry"
entry :: () -> s32 #c_call {
    return cast(s32) (middle(8) - 41);
}

// Exercise actual native stack walking through C callbacks, including ABI forwarding.
#program_export "unwind_capture_leaf"
capture_leaf :: no_inline (capture: () -> s32 #c_call) -> s32 #c_call {
    return capture() + 1;
}

#program_export "unwind_capture_middle"
capture_middle :: no_inline (capture: () -> s32 #c_call) -> s32 #c_call {
    return capture_leaf(capture) + 1;
}

main :: () {}
