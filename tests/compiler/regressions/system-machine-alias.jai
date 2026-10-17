#import "Basic";
S :: #import "System";

#if CPU == .X64 {
    M :: #import "Machine_X64";

    #assert S.M64.Cpu_X86 == M.Cpu_X86;
    #assert type_info(S.M64.Cpu_X86) == type_info(M.Cpu_X86);
    #assert S.M64.x86_Vendor == M.x86_Vendor;
} else {
    #assert !#exists(S.M64);
}

main :: () {
    print("ok\n");
}
