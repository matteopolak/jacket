// Actual x64 interpreter test, without linter-sandbox native file operations.
M :: #import "Machine_X64";

#assert #run verify();

verify :: () -> bool {
    cpu := M.get_cpu_info();
    a: u32 = 1;
    b: u32;
    c: u32 = 0;
    d: u32;

    #asm { cpuid a, b, c, d; }
    if cpu.feature_leaves[0] != d || cpu.feature_leaves[1] != c return false;

    masks: [11] u32;

    for position: 0..351 {
        flag := cast(M.x86_Feature_Flag) position;
        if M.check_feature(masks, flag) return false;

        M.enable_feature(*masks, flag);
        if !M.check_feature(masks, flag) return false;

        M.disable_feature(*masks, flag);
        if M.check_feature(masks, flag) return false;
    }

    return true;
}

main :: () {}
