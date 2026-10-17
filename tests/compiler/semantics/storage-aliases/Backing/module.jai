provider :: #library "alias-provider";
Handle :: #type,distinct u64;

Pair :: struct {
    left: u32;
    right: u32;
}

storage_alias_counter: Handle #elsewhere provider;
storage_alias_pair: Pair #elsewhere provider;
storage_alias_read :: () -> u64 #foreign provider;
storage_alias_write :: (value: u64) #foreign provider;
