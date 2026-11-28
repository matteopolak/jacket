// Copies `count` bytes from `source` to `dest`. The ranges may overlap.
memcpy :: (dest: *void, source: *void, count: s64) #intrinsic; @TakesByteCount

// Sets `count` bytes at `dest` to `value`.
memset :: (dest: *void, value: u8, count: s64) #intrinsic; @TakesByteCount

// Compares `count` bytes of `a` and `b`: negative, 0 or positive as `a` sorts before, equal to or
// after `b`.
memcmp :: (a: *void, b: *void, count: s64) -> s16 #intrinsic; @TakesByteCount

// Atomically replaces `pointer.*` with `new` if it equals `old`. Returns whether it did and the
// value that was there.
compare_and_swap :: (pointer: *$T, old: T, new: T) -> (success: bool, old_value: T) #intrinsic;

// Atomically reads `pointer.*`.
atomic_load :: (pointer: *$T) -> T #intrinsic;

// Atomically writes `new` to `pointer.*`.
atomic_store :: (pointer: *$T, new: T) #intrinsic;

// Atomically writes `new` to `pointer.*` and returns the value it replaced.
atomic_exchange :: (pointer: *$T, new: T) -> old_value: T #intrinsic;
