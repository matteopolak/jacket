// Signs an unsigned APK with the single key in a JKS keystore, and returns `true` and the signed file's bytes.
//
// Reads `keystore_filepath`, checks the JKS magic and the store's integrity digest with `keystore_password`,
// decrypts the private key with the same password, and signs as [apk_sign2] does. The file is not written; the
// caller writes and frees the returned bytes. The store must hold exactly one private-key entry, whose alias
// is ignored, and the last certificate of its chain is used. Returns `false` and an empty string when the
// store cannot be read or the password is wrong; JKS parse errors also fail an assertion.
apk_sign :: (unsigned_apk_filepath: string, keystore_filepath: string, keystore_password := "password") -> bool, string {
    data, loaded := Apk_Signing_File.read_entire_file(keystore_filepath);
    if !loaded return false, "";

    defer free(data);
    if data.count < 4 return false, "";
    if data[0] != 0xfe || data[1] != 0xed || data[2] != 0xfe || data[3] != 0xed return false, "";

    store_ok, certificate, encrypted := jks_read(data, keystore_password);
    if !store_ok return false, "";

    key_ok, private_key := decrypt_private_key(encrypted, keystore_password);
    if !key_ok return false, "";

    defer {
        memset(private_key.data, 0, private_key.count);
        free(private_key);
    }

    return apk_sign2(unsigned_apk_filepath, private_key, certificate);
}

// Signs the unsigned APK `unsigned_apk_filepath` with an APK signature scheme v2 block.
//
// `private_key_info` is the PKCS#8 private key and `certificate` the X.509 certificate, both DER. Only ECDSA
// keys are accepted, signed with SHA-256. Before the block is built, the key is checked against the
// certificate's public key by verifying a signature made with the private key. Returns `true` and the signed
// file's bytes, which the caller frees, or `false` and an empty string for an already signed APK, a key that
// is not ECDSA, or a key that does not match the certificate (the last two also log an error that fails an
// assertion). It does not check the certificate's validity dates or chain, and writes no file.
apk_sign2 :: (unsigned_apk_filepath: string, private_key_info: string, certificate: string) -> bool, string {
    apk := apk_load(unsigned_apk_filepath);

    defer apk_deinit(*apk);
    if apk.is_signed return false, "";

    key := libressl_get_private_key(private_key_info);
    if !key return false, "";

    defer EVP_PKEY_free(key);
    if !apk_v2_require_ec_key(key) return false, "";

    spki := x509_get_subject_public_key_info(certificate);
    public_key := libressl_get_public_key(spki.value_data);

    if !public_key return false, "";

    defer EVP_PKEY_free(public_key);
    if !apk_v2_require_ec_key(public_key) return false, "";

    digest := apk_compute_digest_libressl(apk, .SHA256);
    digest_bytes := string.{ SHA256_DIGEST_LENGTH, digest.data };
    signed_data := apk_v2_encode_signed_data(digest_bytes, certificate);

    defer free(signed_data);
    signed_ok, signature := sign_data(private_key_info, signed_data);
    if !signed_ok return false, "";

    defer free(signature);

    if !apk_v2_verify_signature(public_key, signed_data, signature) {
        apk_v2_failure("APK private key does not match certificate public key");
        return false, "";
    }

    block := apk_v2_serialize_block(digest_bytes, certificate, signature, spki.value_data);

    defer free(block);

    return true, apk_v2_assemble(apk, block);
}

// Verifies the APK v2 signature of the file `filepath`; see the overload taking a loaded [Android_Apk].
apk_verify :: (filepath: string, verbose := false) -> bool {
    apk := apk_load(filepath);
    defer apk_deinit(*apk);
    return apk_verify(apk, verbose);
}

// Checks the APK v2 signing block of `apk`.
//
// Returns `false` when the APK is unsigned, only partly loaded, or its signing block is malformed or its
// public key is not an ECDSA key. The signature over the signed data, the content digest and the match of the
// certificate's public key are compared, but a mismatch of those only logs an error: the call still returns
// `true`. Only the first signer is looked at, and the certificate itself is not validated. `verbose` prints a
// line when the checks have run.
apk_verify :: (apk: Android_Apk, verbose := false) -> bool {
    if !apk.is_signed return false;

    if !apk.entire_apk_is_loaded {
        apk_v2_failure("APK verification requires the complete file");
        return false;
    }

    valid, block := apk_v2_read_container(apk.signing_block, verify_structure = true);
    if !valid {
        apk_v2_failure("Unsupported or malformed APK signing block structure");
        return false;
    }

    defer apk_v2_free_arrays(*block);
    key := libressl_get_public_key(block.subject_public_key_info);
    if !key return false;

    defer EVP_PKEY_free(key);
    if !apk_v2_require_ec_key(key) return false;

    if !apk_v2_verify_signature(key, block.signed_data, block.signatures[0].data) {
        apk_v2_log_mismatch("APK signature check failed\n");
    }

    digest := apk_compute_digest_libressl(apk, .SHA256);
    if string.{ SHA256_DIGEST_LENGTH, digest.data } != block.digests[0].data {
        apk_v2_log_mismatch("APK content digest mismatch\n");
    }

    certificate_spki := x509_get_subject_public_key_info(block.certificates[0].data);
    if certificate_spki.value_data != block.subject_public_key_info {
        apk_v2_log_mismatch("APK certificate public key mismatch\n");
    }

    if verbose print("APK v2 ECDSA/SHA256 signer checked\n");

    // Reviewed compatibility: cryptographic mismatches log without making false.
    return true;
}

#scope_file
Apk_Signing_File :: #import "File";

apk_v2_require_ec_key :: (key: *evp_pkey_st) -> bool {
    name: [3] u8 = .[69, 67, 0];
    if EVP_PKEY_is_a(key, name.data) == 1 return true;
    apk_v2_failure("APK signer supports ECDSA keys only");
    return false;
}

apk_v2_verify_signature :: (key: *evp_pkey_st, data: string, signature: string) -> bool {
    state := EVP_MD_CTX_new();
    if !state {
        apk_v2_failure("APK verification digest context allocation failed");
        return false;
    }

    defer EVP_MD_CTX_free(state);
    initialized := EVP_DigestVerifyInit(state, null, EVP_sha256(), null, key);

    CheckSSL(initialized);
    if initialized != 1 return false;

    updated := EVP_DigestVerifyUpdate(state, data.data, cast(u64) data.count);

    CheckSSL(updated);
    if updated != 1 return false;

    return EVP_DigestVerifyFinal(state, signature.data, cast(u64) signature.count) == 1;
}

apk_v2_log_mismatch :: (message: string) {
    context.logger(message, context.logger_data, Log_Info.{ common_flags = .ERROR });
}
