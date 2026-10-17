// Reviewed Machine_X64 records/feature masks, plus official native C CPUID oracle.
#import "Basic";
M :: #import "Machine_X64";
#if OS_IS_UNIX F :: #import "File";

main :: () {
    #assert CPU == .X64;
    assert(type_info(M.Cpu_X86).name == "Cpu_X86");
    assert(type_info(M.x86_Feature_Flag).name == "x86_Feature_Flag");
    assert(type_info(M.Machine_Options_X86).name == "Machine_Options_X86");
    #assert !#exists(M.query);
    #assert !#exists(M.Leaf);
    #assert size_of(M.x86_Vendor) == 1;
    #assert size_of(M.x86_Feature_Leaf) == 2;
    #assert size_of(M.x86_Feature_Flag) == 2;
    #assert cast(u16) M.x86_Feature_Flag.SSE2 == 26;
    #assert cast(u16) M.x86_Feature_Flag.AVX2 == 197;
    #assert cast(u16) M.x86_Feature_Flag.XFD == 324;

    masks: [11] u32;

    for position: 0..351 {
        flag := cast(M.x86_Feature_Flag) position;

        assert(!M.check_feature(masks, flag));
        M.enable_feature(*masks, flag);
        assert(M.check_feature(masks, flag));
        assert(masks[position / 32] == (cast(u32) 1 << (position % 32)));
        M.enable_feature(*masks, flag);
        M.disable_feature(*masks, flag);
        assert(!M.check_feature(masks, flag));
        assert(masks[position / 32] == 0);
    }

    M.enable_feature(*masks, .FPU);
    M.enable_feature(*masks, .HYPERVISOR);
    M.enable_feature(*masks, .XFD);
    M.disable_feature(*masks, .HYPERVISOR);
    assert(M.check_feature(masks, .FPU) && M.check_feature(masks, .XFD));
    options: M.Machine_Options_X86;
    options.features.leaves = masks;
    assert(options.features.leaves[10] == 16);

    cpu := M.get_cpu_info();

    #if OS_IS_UNIX {
        bytes, loaded := F.read_entire_file("target/portable-machine/cpuid.bin");

        assert(loaded && bytes.count == 48);
        defer free(bytes);
        assert(cast(u32) cpu.vendor == read_word(bytes, 0));
        for i: 0..10 assert(cpu.feature_leaves[i] == read_word(bytes, i + 1));
    } else {
        assert(false, "native C oracle fixture requires its Unix-hosted runner");
    }

    print("ok\n");
}

#scope_file
read_word :: (bytes: string, index: s64) -> u32 {
    result: u32;

    for i: 0..3 {
        part := cast(u32) bytes.data[index * 4 + i];
        result |= part << (i * 8);
    }

    return result;
}
