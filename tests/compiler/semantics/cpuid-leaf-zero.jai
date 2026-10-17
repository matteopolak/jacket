// Original CPUID result oracle. Leaf zero vendor fields have no APIC/topology ID.
#program_export "cpuid_register"
cpuid_register :: (leaf: u32, subleaf: u32, index: u32) -> u32 #c_call {
    a := leaf;
    b: u32;
    c := subleaf;
    d: u32;

    #asm { cpuid a, b, c, d; }
    if index == 0 return a;
    if index == 1 return b;
    if index == 2 return c;

    return d;
}
