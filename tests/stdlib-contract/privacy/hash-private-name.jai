// Expected check failure: HASH_INIT is explicitly file-local in Hash's public spec.
Hash :: #import "Hash";

main :: () {
    private_value := Hash.HASH_INIT;
}
