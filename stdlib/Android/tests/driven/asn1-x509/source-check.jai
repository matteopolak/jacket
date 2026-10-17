// Explicit source/target gate; behavior uses genuine native-generated DER vectors.
A :: #import "Android/Toolchain";

main :: () {
    A.x509_get_subject_public_key_info("");
}
