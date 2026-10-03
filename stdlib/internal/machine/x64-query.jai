Cpu_X86 :: struct {
    vendor: x86_Vendor;
    feature_leaves: [Leaf.NUMBER] u32;
}

Machine_Options_X86 :: struct {
    features: struct {
        leaves: [Leaf.NUMBER] u32;
    };
}

get_machine_options_x86 :: (build_options: *$T) -> *Machine_Options_X86 #modify {
    info := cast(*Type_Info) T;
    return info.type == .STRUCT, "machine options require a build-options struct";
} {
    #assert type_info(type_of(build_options.machine_options)).type == .ARRAY;
    #assert type_of(build_options.machine_options.data) == *u8;
    #assert size_of(type_of(build_options.machine_options)) >= size_of(Machine_Options_X86);
    #assert size_of(Machine_Options_X86) <= 128;
    assert(build_options != null, "machine options require a live build-options pointer");
    memory := build_options.machine_options.data;

    assert(cast(u64) memory % cast(u64) align_of(Machine_Options_X86) == 0, "machine-options storage is not aligned for x64 feature words");

    return cast(*Machine_Options_X86) memory;
}

check_feature :: (leaves: [Leaf.NUMBER] u32, flag: x86_Feature_Flag) -> bool {
    value := cast(u16) flag;
    assert(value < 32 * cast(u16) Leaf.NUMBER, "invalid x64 feature index");
    return (leaves[value / 32] & (cast(u32) 1 << (value % 32))) != 0;
}

enable_feature :: (leaves: *[Leaf.NUMBER] u32, flag: x86_Feature_Flag) {
    value := cast(u16) flag;
    assert(value < 32 * cast(u16) Leaf.NUMBER, "invalid x64 feature index");
    leaves.*[value / 32] |= cast(u32) 1 << (value % 32);
}

disable_feature :: (leaves: *[Leaf.NUMBER] u32, flag: x86_Feature_Flag) {
    value := cast(u16) flag;
    assert(value < 32 * cast(u16) Leaf.NUMBER, "invalid x64 feature index");
    leaves.*[value / 32] &= ~(cast(u32) 1 << (value % 32));
}

get_cpu_info :: () -> Cpu_X86 {
    result: Cpu_X86;
    maximum, b, c, d := query(0, 0);
    vendor: [12] u8;

    memcpy(vendor.data, *b, 4);
    memcpy(vendor.data + 4, *d, 4);
    memcpy(vendor.data + 8, *c, 4);
    name := string.{ 12, vendor.data };
    if name == "GenuineIntel" result.vendor = .INTEL;
    else if name == "AuthenticAMD" result.vendor = .AMD;

    if maximum >= 1 {
        _a, _b, ecx, edx := query(1, 0);
        result.feature_leaves[Leaf._01H_EDX] = edx;
        result.feature_leaves[Leaf._01H_ECX] = ecx;
    }

    if maximum >= 7 {
        subleaf_max, ebx, ecx, edx := query(7, 0);

        result.feature_leaves[Leaf._07H_00H_EBX] = ebx;
        result.feature_leaves[Leaf._07H_00H_ECX] = ecx;
        result.feature_leaves[Leaf._07H_00H_EDX] = edx;

        if subleaf_max >= 1 {
            eax, _b, _c, _d := query(7, 1);
            result.feature_leaves[Leaf._07H_01H_EAX] = eax;
        }
    }

    if maximum >= 13 {
        eax, _b, _c, _d := query(13, 1);
        result.feature_leaves[Leaf._0DH_01H_EAX] = eax;
    }

    if maximum >= 25 {
        _a, ebx, _c, _d := query(25, 0);
        result.feature_leaves[Leaf._19H_EBX] = ebx;
    }

    extended_max, _b, _c, _d := query(0x80000000, 0);
    if extended_max >= 0x80000001 {
        _a, _b, ecx, edx := query(0x80000001, 0);
        result.feature_leaves[Leaf._01H_EXT_EDX] = edx;
        result.feature_leaves[Leaf._01H_EXT_ECX] = ecx;
    }

    if extended_max >= 0x80000008 {
        _a, ebx, _c, _d := query(0x80000008, 0);
        result.feature_leaves[Leaf._08H_EXT_EBX] = ebx;
    }

    return result;
}

#scope_file
#import "Basic";

Leaf :: x86_Feature_Leaf;

query :: (leaf: u32, subleaf: u32) -> (a: u32, b: u32, c: u32, d: u32) {
    a = leaf;
    c = subleaf;
    #asm { cpuid a, b, c, d; }
    return a, b, c, d;
}
