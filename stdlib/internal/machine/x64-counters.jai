rdtscp :: () -> (timestamp: u64, msr: u32) #expand {
    high: u64;
    low: u64;
    auxiliary: u32;

    #asm { rdtscp high, low, auxiliary; }

    return (high << 32) | low, auxiliary;
}

rdtsc :: () -> u64 #expand {
    high: u64;
    low: u64;
    #asm { rdtsc high, low; }
    return (high << 32) | low;
}

// Each wrapper makes exactly one hardware attempt and returns its carry flag.
// The caller owns feature checks and any application-specific retry policy.
rdrand :: () -> (success: bool, value: u64) #expand {
    sample: u64;
    carry: u8;
    #asm { rdrand.q sample; setc carry; }
    return carry != 0, sample;
}

rdseed :: () -> (success: bool, value: u64) #expand {
    sample: u64;
    carry: u8;
    #asm { rdseed.q sample; setc carry; }
    return carry != 0, sample;
}
