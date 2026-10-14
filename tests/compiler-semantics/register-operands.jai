reg :: __reg;

extract :: (operand: reg, $lane: int) -> u64 #expand {
    result: u64;
    #asm { pextrq result, operand, lane; }
    return result;
}

extract32 :: (operand: reg, $lane: int) -> u32 #expand {
    result: u32;
    #asm { pextrd result, operand, lane; }
    return result;
}

forward :: (operand: $T, $lane: int) -> u64 #expand {
    return extract(operand, lane);
}

mutate :: (operand: reg, value: u64) #expand {
    #asm { pinsrq operand, value, 1; }
}

#program_export "register_check"
check :: () -> s32 #c_call {
    push_context {
        words: [2] u64 = .[0x1122334455667788, 0x1020304050607080];
        p := words.data;

        #asm { movdqu.x v:, [p]; }
        if extract(v, 0) != words[0] || forward(v, 1) != words[1] return 1;
        if extract32(v, 1) != 0x11223344 || extract32(v, 3) != 0x10203040 return 2;

        mutate(v, 0xaabbccddeeff0011);
        if extract(v, 1) != 0xaabbccddeeff0011 return 3;
    }

    return 0;
}
