Metrics :: struct {
    if_points: s32 #align 1;
    maximum_if_depth: s32 #align 1;
    maximum_loop_depth: s32 #align 1;
    total_calls: s32 #align 1;
    same_file_calls: s32 #align 1;
    same_module_calls: s32 #align 1;
    nonconstant_calls: s32 #align 1;
    allocations: s32 #align 1;
    frees: s32 #align 1;
    reassignments: s32 #align 1;
    global_reads: s32 #align 1;
    global_writes: s32 #align 1;
}
