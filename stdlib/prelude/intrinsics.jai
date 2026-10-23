// Canonical bootstrap intrinsics declarations.
memcpy :: (dest: *void, source: *void, count: s64) #intrinsic; @TakesByteCount

memset :: (dest: *void, value: u8, count: s64) #intrinsic; @TakesByteCount

memcmp :: (a: *void, b: *void, count: s64) -> s16 #intrinsic; @TakesByteCount

compare_and_swap :: (pointer: *$T, old: T, new: T) -> (success: bool, old_value: T) #intrinsic;

atomic_load :: (pointer: *$T) -> T #intrinsic;

atomic_store :: (pointer: *$T, new: T) #intrinsic;

atomic_exchange :: (pointer: *$T, new: T) -> old_value: T #intrinsic;
