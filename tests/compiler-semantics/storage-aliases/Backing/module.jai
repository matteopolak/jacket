provider :: #library "cleanroom-alias-provider";
Handle :: #type,distinct u64;

Pair :: struct {
    left: u32;
    right: u32;
}

cleanroom_alias_counter: Handle #elsewhere provider;
cleanroom_alias_pair: Pair #elsewhere provider;
cleanroom_alias_read :: () -> u64 #foreign provider;
cleanroom_alias_write :: (value: u64) #foreign provider;
