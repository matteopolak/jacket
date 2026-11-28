SIGNING_BLOCK_ID_V2_SIGNATURE: u32 : 0x7109871a;
SIGNING_BLOCK_ID_V3_SIGNATURE: u32 : 0xf05368c0;
SIGNING_BLOCK_ID_V31_SIGNATURE: u32 : 0x1b93ad61;
SIGNING_BLOCK_ID_VERITY_PADDING: u32 : 0x42726577;

// Lists the signature algorithm ids of the APK signing block; the signer in this module uses `ECDSA_SHA256`.
Algorithm_Id :: enum u32 {
    RSASSA_PSS_SHA256 :: 0x0101;
    RSASSA_PSS_SHA512 :: 0x0102;
    RSASSA_PKCS1_v1_5_SHA256 :: 0x0103;
    RSASSA_PKCS1_v1_5_SHA512 :: 0x0104;
    ECDSA_SHA256 :: 0x0201;
    ECDSA_SHA512 :: 0x0202;
    DSA_SHA256 :: 0x0301;
}

// Lists the hashes [apk_compute_digest_libressl] can be asked for; only `SHA256` is implemented.
Digest_Hash_Algorithm :: enum {
    SHA256;
    SHA512;
}

// Is a content digest in a signing block, with the algorithm it belongs to.
Digest :: struct {
    signature_algorithm_id: Algorithm_Id;
    data: string;
}

// Is a signature over a signer's signed data, with the algorithm used.
Signature :: struct {
    algorithm_id: Algorithm_Id;
    data: string;
}

// Is a DER-encoded X.509 certificate from a signing block.
Certificiate :: struct {
    data: string;
}

// Is a parsed APK signature scheme v2 block with its one signer.
//
// `signed_data` is the bytes the signature covers, and `subject_public_key_info` the signer's DER public key.
// The strings point into the data that was parsed.
Apk_Signing_Block_V2 :: struct {
    signers_count :: 1;
    digests: [..] Digest;
    signatures: [..] Signature;
    certificates: [..] Certificiate;
    signed_data: string;
    subject_public_key_info: string;
}
