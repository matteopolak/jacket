Bit_Array :: Impl.Bit_Array;
init_bit_array :: Impl.init_bit_array;
make_bit_array :: Impl.make_bit_array;
deinit :: Impl.deinit;
set_bit :: Impl.set_bit;
clear_bit :: Impl.clear_bit;
toggle_bit :: Impl.toggle_bit;
set_bit_to :: Impl.set_bit_to;
clear_all_bits :: Impl.clear_all_bits;
set_all_bits :: Impl.set_all_bits;
toggle_all_bits :: Impl.toggle_all_bits;
for_expansion :: Impl.for_expansion;
only_set :: Impl.only_set;
only_unset :: Impl.only_unset;

using,only(.["[]", "[]="]) #import "internal/collections/bit-array";

#scope_file
Impl :: #import "internal/collections/bit-array";
