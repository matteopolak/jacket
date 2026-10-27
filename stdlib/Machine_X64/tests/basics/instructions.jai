// test: only(.X64)
// Reviewed machine-x64.md macros; hardware prerequisites from actual CPUID.
#import "Basic";
M :: #import "Machine_X64";

// Keep optional instructions visible for disassembly without executing them on
// incapable CPUs. A capable CI machine exercises the same public wrappers.
#program_export "public_rdrand"
random_attempt :: () -> (success: bool, value: u64) #c_call {
    return M.rdrand();
}

#program_export "public_rdseed"
seed_attempt :: () -> (success: bool, value: u64) #c_call {
    return M.rdseed();
}

main :: () {
    cpu := M.get_cpu_info();

    assert(M.check_feature(cpu.feature_leaves, .TSC));
    memory: [64] u8;
    memory[0] = 73;
    M.prefetch(memory.data, .T0);
    M.prefetch(memory.data, .T1);
    M.prefetch(memory.data, .T2);
    M.prefetch(memory.data, .NTA);
    M.mfence();
    M.lfence();
    M.sfence();
    M.pause();
    assert(memory[0] == 73);
    // LFENCE brackets each sample: counter values are not wall-clock time.
    M.lfence();
    first := M.rdtsc();

    // Some hypervisors hide RDTSCP, so fall back to a second RDTSC sample there.
    second: u64;

    if M.check_feature(cpu.feature_leaves, .RDTSCP) {
        M.lfence();
        value, auxiliary := M.rdtscp();
        #assert type_of(auxiliary) == u32;
        second = value;
    } else {
        M.lfence();
        second = M.rdtsc();
    }

    M.lfence();
    assert(first > 0 && second >= first);

    if M.check_feature(cpu.feature_leaves, .RDRAND) {
        success, value := random_attempt();
        if !success assert(value == 0);
    }

    if M.check_feature(cpu.feature_leaves, .RDSEED) {
        success, value := seed_attempt();
        if !success assert(value == 0);
    }

    print("ok\n");
}
