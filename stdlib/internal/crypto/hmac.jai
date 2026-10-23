// HMAC-SHA-256 (RFC 2104) and PBKDF2 (RFC 8018) with a 32-byte derived key.
S :: #import "internal/crypto/sha";

hmac_sha256 :: (key: string, message: string) -> [32] u8 {
    inner, outer := pad_states(key);
    return finish(inner, outer, message);
}

pbkdf2_hmac_sha256 :: (password: string, salt: string, iterations: int) -> [32] u8 {
    result: [32] u8;
    if iterations < 1 return result;

    // The padded-key states are computed once and reused for every round.
    inner, outer := pad_states(password);

    seed := alloc_salt(salt);

    defer free(seed.data);

    previous := finish(inner, outer, seed);

    result = previous;

    for 2..iterations {
        previous = finish(inner, outer, string.{ 32, previous.data });
        for byte: 0..31 result[byte] ^= previous[byte];
    }

    return result;
}

#scope_file
#import "Basic";

// Salt followed by the big-endian block index 1.
alloc_salt :: (salt: string) -> string {
    data: *u8 = alloc(salt.count + 4);
    if salt.count > 0 memcpy(data, salt.data, salt.count);

    data[salt.count] = 0;
    data[salt.count + 1] = 0;
    data[salt.count + 2] = 0;
    data[salt.count + 3] = 1;

    return string.{ salt.count + 4, data };
}

pad_states :: (key: string) -> [8] u32, [8] u32 {
    block: [64] u8;

    if key.count > 64 {
        digest := S.sha256(key);
        memcpy(block.data, digest.data, 32);
    } else if key.count > 0 {
        memcpy(block.data, key.data, key.count);
    }

    inner_block: [64] u8;
    outer_block: [64] u8;

    for index: 0..63 {
        inner_block[index] = block[index] ^ 0x36;
        outer_block[index] = block[index] ^ 0x5c;
    }

    return S.sha256_block_state(inner_block.data), S.sha256_block_state(outer_block.data);
}

finish :: (inner: [8] u32, outer: [8] u32, message: string) -> [32] u8 {
    first := S.sha256_continue(inner, 64, message);
    return S.sha256_continue(outer, 64, string.{ 32, first.data });
}
