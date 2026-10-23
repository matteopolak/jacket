# Crypto helpers

Private stdlib modules under `stdlib/internal/crypto/`: SHA-1/256/384/512 (`sha.jai`), HMAC-SHA-256 and PBKDF2 (`hmac.jai`), and OS random bytes (`random.jai`). They exist for the Postgres client's SCRAM authentication; public modules re-export what they need (internal sources can only be imported from inside `stdlib/`).

## How it works

- `sha.jai`: `sha1`, `sha256`, `sha384`, `sha512` take a string and return the digest array. SHA-256 also has `sha256_block_state` and `sha256_continue`, which let a caller hash from a saved state after a fixed 64-byte prefix.
- `hmac.jai`: `hmac_sha256(key, message)` and `pbkdf2_hmac_sha256(password, salt, iterations)` (32-byte key only). Both compute the inner and outer padded-key states once; PBKDF2 then costs two compressions per iteration.
- `random.jai`: `random_bytes(out: [] u8) -> bool` calls `getentropy` (macOS, Linux; at most 256 bytes per call) or `BCryptGenRandom` with the system-preferred RNG (Windows). It returns false on failure; never fall back to a weaker source.

## How to change it

- The SHA-256 round loop writes its rotations out by hand: a call per rotation made `-interp` about three times slower.
- `hash_blocks` has a `prefix` parameter (bytes already absorbed) used by `sha256_continue`; the default keeps the plain digests unchanged.
- A longer PBKDF2 output needs a block counter in `alloc_salt` and a loop over blocks.

## Configuration

None. Tests: `stdlib/Hash/tests/crypto-*.jai` (RFC 4231, RFC 7914 and FIPS 180 vectors), run natively and with `-interp`.

## Dependencies

`Basic`; libc (`getentropy`) or `bcrypt` on Windows.
