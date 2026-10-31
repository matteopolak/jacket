// Meow hash, a fast 128-bit non-cryptographic hash built on AES instructions. `MeowHash` hashes a
// buffer in one call; `MeowBegin`, `MeowAbsorb` and `MeowEnd` take it in pieces. The result lives in
// a vector register: read it with `MeowU64From` or `MeowU32From`. Only x64 CPUs with AES-NI have it.
#load "../internal/hash/meow/data.jai";
#load "../internal/hash/meow/registers.jai";
#load "../internal/hash/meow/finalize.jai";
#load "../internal/hash/meow/stream.jai";
