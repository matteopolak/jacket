// The pause and prefetch hints carry no meaning for results, so targets without them skip them.
pause :: () #expand {
    #if CPU == .X64 {
        #asm { pause; }
    }
}

prefetch :: (pointer: *void, $hint: Prefetch_Hint) #expand {
    #if CPU == .X64 {
        #if hint == .NTA #asm { prefetchnta [pointer]; } else #if hint == .T2 #asm { prefetcht2 [pointer]; } else #if hint == .T1 #asm { prefetcht1 [pointer]; } else #if hint == .T0 #asm { prefetcht0 [pointer]; } else #assert false, "invalid prefetch hint";
    }
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
