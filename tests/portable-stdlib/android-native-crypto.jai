#import "Basic";
F :: #import "File";
A :: #import "Android/Toolchain";

load :: (name: string) -> string {
    data, ok := F.read_entire_file(tprint("%/%", FIXTURE_ROOT, name));
    assert(ok);
    return data;
}

verify :: (public_key: *A.test_evp_pkey_st, message: string, signature: string) -> s32 {
    ctx := A.test_EVP_MD_CTX_new();

    assert(ctx != null);
    defer A.test_EVP_MD_CTX_free(ctx);
    A.CheckSSL(A.test_EVP_DigestVerifyInit(ctx, null, A.test_EVP_sha256(), null, public_key));
    A.CheckSSL(A.test_EVP_DigestVerifyUpdate(ctx, message.data, cast(u64) message.count));

    return A.test_EVP_DigestVerifyFinal(ctx, signature.data, cast(u64) signature.count);
}

main :: () {
    message := load("message.bin");

    defer free(message);

    for kind: string.["ec", "rsa"] {
        private_bytes := load(tprint("%.der", kind));
        public_bytes := load(tprint("%.spki", kind));
        sdk_signature := load(tprint("%.sdk-signature", kind));

        defer free(private_bytes);
        defer free(public_bytes);
        defer free(sdk_signature);
        key := A.libressl_get_public_key(public_bytes);

        assert(key != null);
        defer A.test_EVP_PKEY_free(key);
        ec_name := "EC\x00";
        rsa_name := "RSA\x00";

        assert(A.test_EVP_PKEY_is_a(key, ec_name.data) == (ifx kind == "ec" then 1 else 0));
        assert(A.test_EVP_PKEY_is_a(key, rsa_name.data) == (ifx kind == "rsa" then 1 else 0));
        assert(verify(key, message, sdk_signature) == 1);
        assert(verify(key, "changed", sdk_signature) == 0);

        // Repeated context/key construction exercises releases and variable EC sizes.
        for 0..15 {
            ok, signature := A.sign_data(private_bytes, message);

            assert(ok && signature.count > 0);
            assert(verify(key, message, signature) == 1);
            assert(verify(key, "changed", signature) == 0);
            assert(F.write_entire_file(tprint("%/%.jai-signature", FIXTURE_ROOT, kind), signature));
            free(signature);
        }

        traditional := load(tprint("%.traditional.der", kind));

        defer free(traditional);
        ok, signature := A.sign_data(traditional, message);

        assert(ok && verify(key, message, signature) == 1);
        free(signature);
        ok, signature = A.sign_data(private_bytes, "");
        assert(ok && verify(key, "", signature) == 1);
        free(signature);
    }

    sdk_store := load("sdk.jks");
    sdk_certificate := load("sdk.certificate.der");
    sdk_public := load("sdk.spki");

    defer free(sdk_store);
    defer free(sdk_certificate);
    defer free(sdk_public);
    ok, certificate, encrypted := A.jks_read(sdk_store, "password");

    assert(ok && certificate == sdk_certificate);
    decrypted_ok, decrypted := A.decrypt_private_key(encrypted, "password");

    assert(decrypted_ok);
    defer free(decrypted);
    key := A.libressl_get_public_key(sdk_public);

    assert(key != null);
    defer A.test_EVP_PKEY_free(key);
    signed_ok, signature := A.sign_data(decrypted, message);

    assert(signed_ok && verify(key, message, signature) == 1);
    assert(F.write_entire_file(tprint("%/sdk.jai-signature", FIXTURE_ROOT), signature));
    free(signature);
    print("ok\n");
}
