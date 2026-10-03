pause :: () #expand {
    #asm { pause; }
}

prefetch :: (pointer: *void, $hint: Prefetch_Hint) #expand {
    #if hint == .NTA #asm { prefetchnta [pointer]; } else #if hint == .T2 #asm { prefetcht2 [pointer]; } else #if hint == .T1 #asm { prefetcht1 [pointer]; } else #if hint == .T0 #asm { prefetcht0 [pointer]; } else #assert false, "invalid prefetch hint";
}

// These are real architectural fences, with compiler memory effects supplied
// by the general assembler lowering. No private C shim or timed delay is used.
sfence :: () #expand {
    #asm { sfence; }
}

lfence :: () #expand {
    #asm { lfence; }
}

mfence :: () #expand {
    #asm { mfence; }
}
