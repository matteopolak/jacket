// Generated from docs/specs/stdlib/modules/xxhash.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file
binding_XXH_versionNumber :: () -> u32 #foreign xxh_vendor "XXH_versionNumber";
binding_XXH32 :: (input: *void, len: u64, seed: u32) -> u32 #foreign xxh_vendor "XXH32";
binding_XXH64 :: (input: *void, len: u64, seed: u64) -> u64 #foreign xxh_vendor "XXH64";
binding_XXH32_reset :: (state: *XXH32_state_t, seed: u32) -> s32 #foreign xxh_vendor "XXH32_reset";
binding_XXH64_reset :: (state: *XXH64_state_t, seed: u64) -> s32 #foreign xxh_vendor "XXH64_reset";
binding_XXH32_update :: (state: *XXH32_state_t, input: *void, len: u64) -> s32 #foreign xxh_vendor "XXH32_update";
binding_XXH64_update :: (state: *XXH64_state_t, input: *void, len: u64) -> s32 #foreign xxh_vendor "XXH64_update";
binding_XXH32_digest :: (state: *XXH32_state_t) -> u32 #foreign xxh_vendor "XXH32_digest";
binding_XXH64_digest :: (state: *XXH64_state_t) -> u64 #foreign xxh_vendor "XXH64_digest";
binding_abort :: () #foreign libc "abort"; @Exits
libc :: #library,system "c"; @native(external)
xxh_vendor :: #library,system "xxhash070";
#scope_export

// Types and layouts

// The state of a 32-bit hash fed in pieces. Start it with [XXH32_reset].
XXH32_state_t :: struct {
    total_len_32: u32;
    large_len: u32;
    v1: u32;
    v2: u32;
    v3: u32;
    v4: u32;
    mem32: [4] u32;
    memsize: u32;
    reserved: u32;
}

// The state of a 64-bit hash fed in pieces. Start it with [XXH64_reset].
XXH64_state_t :: struct {
    total_len: u64;
    v1: u64;
    v2: u64;
    v3: u64;
    v4: u64;
    mem64: [4] u64;
    memsize: u32;
    reserved: [2] u32;
}

// Aliases and constants

XXH_VERSION_MAJOR :: 0;

XXH_VERSION_MINOR :: 7;

XXH_VERSION_NUMBER :: (XXH_VERSION_MAJOR * 100 * 100 + XXH_VERSION_MINOR * 100 + XXH_VERSION_RELEASE);

XXH_VERSION_RELEASE :: 0;

// Convenience operations

// Returns the 32-bit hash of `len` bytes at `input`.
native_XXH32 :: (input: *void, len: u32, seed: u32) -> u32 {
    return binding_XXH32(input, cast(u64) len, seed);
}

// Returns the 64-bit hash of `len` bytes at `input`. The seed is widened to 64 bits.
//
// ```jai
// text := "hello";
// state: XXH64_state_t;
// XXH64_reset(*state, 7);
// XXH64_update(*state, text.data, 2);
// XXH64_update(*state, text.data + 2, 3);
// assert(XXH64_digest(*state) == native_XXH64(text.data, 5, 7));
// ```
native_XXH64 :: (input: *void, len: u32, seed: u32) -> u64 {
    return binding_XXH64(input, cast(u64) len, cast(u64) seed);
}

// Returns the 32-bit hash of everything added so far. The state can take more updates afterwards.
XXH32_digest :: (state: *XXH32_state_t) -> u32 {
    return binding_XXH32_digest(state);
}

// Starts a 32-bit hash with `seed`. Aborts the program if the library reports a failure.
XXH32_reset :: (state: *XXH32_state_t, seed: u32) {
    if binding_XXH32_reset(state, seed) != 0 binding_abort();
}

// Adds `len` bytes at `input` to the hash. Aborts the program if the library reports a failure.
XXH32_update :: (state: *XXH32_state_t, input: *void, len: u32) {
    if len == 0 return;
    if binding_XXH32_update(state, input, cast(u64) len) != 0 binding_abort();
}

// Returns the 64-bit hash of everything added so far. The state can take more updates afterwards.
XXH64_digest :: (state: *XXH64_state_t) -> u64 {
    return binding_XXH64_digest(state);
}

// Starts a 64-bit hash with `seed`. Aborts the program if the library reports a failure.
XXH64_reset :: (state: *XXH64_state_t, seed: u64) {
    if binding_XXH64_reset(state, seed) != 0 binding_abort();
}

// Adds `len` bytes at `input` to the hash. Aborts the program if the library reports a failure.
XXH64_update :: (state: *XXH64_state_t, input: *void, len: u32) {
    if len == 0 return;
    if binding_XXH64_update(state, input, cast(u64) len) != 0 binding_abort();
}

// Returns the version of the loaded library, encoded like [XXH_VERSION_NUMBER].
XXH_versionNumber :: () -> u32 {
    return binding_XXH_versionNumber();
}
