SIGNING_BLOCK_ID_V2_SIGNATURE: u32 : 0x7109871a;
SIGNING_BLOCK_ID_V3_SIGNATURE: u32 : 0xf05368c0;
SIGNING_BLOCK_ID_V31_SIGNATURE: u32 : 0x1b93ad61;
SIGNING_BLOCK_ID_VERITY_PADDING: u32 : 0x42726577;

Algorithm_Id :: enum u32 {
    RSASSA_PSS_SHA256 :: 0x0101;
    RSASSA_PSS_SHA512 :: 0x0102;
    RSASSA_PKCS1_v1_5_SHA256 :: 0x0103;
    RSASSA_PKCS1_v1_5_SHA512 :: 0x0104;
    ECDSA_SHA256 :: 0x0201;
    ECDSA_SHA512 :: 0x0202;
    DSA_SHA256 :: 0x0301;
}

Digest_Hash_Algorithm :: enum {
    SHA256;
    SHA512;
}

Digest :: struct {
    signature_algorithm_id: Algorithm_Id;
    data: string;
}

Signature :: struct {
    algorithm_id: Algorithm_Id;
    data: string;
}

Certificiate :: struct {
    data: string;
}

Apk_Signing_Block_V2 :: struct {
    signers_count :: 1;
    digests: [..] Digest;
    signatures: [..] Signature;
    certificates: [..] Certificiate;
    signed_data: string;
    subject_public_key_info: string;
}
