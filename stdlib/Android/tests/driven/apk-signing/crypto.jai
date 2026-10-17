// Public orchestration checks against independently generated native EC vectors.
using #import "Basic";
A :: #import "Android/Toolchain";
F :: #import "File";

crypto_messages: s64;
crypto_assertions: s64;

crypto_capture :: (message: string, user_data: *void, info: Log_Info) {
    crypto_messages += 1;
}

crypto_continue_assertion :: (location: Source_Code_Location, message: string) -> bool {
    crypto_assertions += 1;
    return false;
}

crypto_fixture :: (directory: string, name: string) -> string {
    path := sprint("%/%", directory, name);

    defer free(path);
    data, loaded := F.read_entire_file(path);

    assert(loaded);

    return data;
}

crypto_export :: (directory: string, prefix: string, model: A.Apk_Signing_Block_V2) {
    for name, index: string.["signed-data.bin", "signature.der", "spki.der", "digest.bin"] {
        output := sprint("%/%-%", directory, prefix, name);
        bytes := string.[model.signed_data, model.signatures[0].data,
                        model.subject_public_key_info, model.digests[0].data];

        assert(F.write_entire_file(output, bytes[index]));
        free(output);
    }
}

main :: () {
    args := get_command_line_arguments();

    defer array_free(args);
    assert(args.count == 2);
    unsigned_path := sprint("%/unsigned.zip", args[1]);
    native_path := sprint("%/native-signed.zip", args[1]);
    key_path := sprint("%/native-private-key.der", args[1]);

    defer free(unsigned_path);
    defer free(native_path);
    defer free(key_path);
    key := crypto_fixture(args[1], "native-private-key.der");
    certificate := crypto_fixture(args[1], "native-certificate.der");

    defer free(key);
    defer free(certificate);
    assert(!A.apk_verify(unsigned_path));
    assert(A.apk_verify(native_path));
    signed_ok, signed_bytes := A.apk_sign2(unsigned_path, key, certificate);

    assert(signed_ok && signed_bytes.count > 0);
    defer free(signed_bytes);
    output_path := sprint("%/jai-signed.zip", args[1]);

    defer free(output_path);
    certificate_tail := certificate[certificate.count - 1];

    certificate.data[certificate.count - 1] ^= 1;
    assert(F.write_entire_file(output_path, signed_bytes));
    certificate.data[certificate.count - 1] = certificate_tail;
    assert(A.apk_verify(output_path));
    generated := A.apk_load(output_path);

    defer A.apk_deinit(*generated);
    generated_cursor := string.{ generated.signing_block.count - 44, generated.signing_block.data + 20 };
    generated_ok, generated_model := A.apk_read_signing_block_v2(*generated_cursor);

    assert(generated_ok && generated_cursor.count == 0);
    defer array_free(generated_model.digests);
    defer array_free(generated_model.signatures);
    defer array_free(generated_model.certificates);
    assert(generated_model.certificates[0].data == certificate);

    crypto_export(args[1], "jai", generated_model);
    jks_path := sprint("%/native/keystore.jks", args[1]);

    defer free(jks_path);
    jks_ok, jks_bytes := A.apk_sign(unsigned_path, jks_path);

    assert(jks_ok && jks_bytes.count > 0);
    defer free(jks_bytes);
    jks_output := sprint("%/jks-signed.zip", args[1]);

    defer free(jks_output);
    assert(F.write_entire_file(jks_output, jks_bytes));
    assert(A.apk_verify(jks_output));
    jks_apk := A.apk_load(jks_output);

    defer A.apk_deinit(*jks_apk);
    jks_cursor := string.{ jks_apk.signing_block.count - 44, jks_apk.signing_block.data + 20 };
    jks_parsed_ok, jks_model := A.apk_read_signing_block_v2(*jks_cursor);

    assert(jks_parsed_ok && jks_cursor.count == 0);
    defer array_free(jks_model.digests);
    defer array_free(jks_model.signatures);
    defer array_free(jks_model.certificates);
    expected_jks_certificate := crypto_fixture(args[1], "native/jks-certificate.der");

    defer free(expected_jks_certificate);
    assert(jks_model.certificates[0].data == expected_jks_certificate);
    crypto_export(args[1], "jks", jks_model);

    refused, empty := A.apk_sign2(native_path, key, certificate);

    assert(!refused && empty.count == 0);
    unknown_keystore, unknown_bytes := A.apk_sign(unsigned_path, key_path);

    assert(!unknown_keystore && unknown_bytes.count == 0);

    // The reviewed verifier logs mismatches and still returns true.
    apk := A.apk_load(native_path);

    defer A.apk_deinit(*apk);
    assert(apk.is_signed);
    cursor := string.{ apk.signing_block.count - 44, apk.signing_block.data + 20 };
    parsed_ok, parsed := A.apk_read_signing_block_v2(*cursor);

    assert(parsed_ok && cursor.count == 0);
    defer array_free(parsed.digests);
    defer array_free(parsed.signatures);
    defer array_free(parsed.certificates);
    logger := context.logger;

    context.logger = crypto_capture;
    signature := parsed.signatures[0].data;
    last := signature.count - 1;
    original := signature[last];

    signature.data[last] ^= 1;
    signature_result := A.apk_verify(apk);

    signature.data[last] = original;
    signature_messages := crypto_messages;
    content_offset: s64;

    for apk {
        content_offset = it.offset;
        break;
    }

    content_original := apk.content[content_offset];

    apk.content.data[content_offset] ^= 1;
    digest_result := A.apk_verify(apk);

    apk.content.data[content_offset] = content_original;
    digest_messages := crypto_messages - signature_messages;
    cert_spki := A.x509_get_subject_public_key_info(parsed.certificates[0].data);
    certificate_last := cert_spki.value_data.count - 1;
    certificate_original := cert_spki.value_data[certificate_last];

    cert_spki.value_data.data[certificate_last] ^= 1;
    certificate_result := A.apk_verify(apk);

    cert_spki.value_data.data[certificate_last] = certificate_original;
    certificate_messages := crypto_messages - signature_messages - digest_messages;

    context.logger = logger;
    assert(signature_result && signature_messages >= 1);
    assert(digest_result && digest_messages >= 1);
    assert(certificate_result && certificate_messages >= 1);
    assert(A.apk_verify(apk));

    // Unsupported key kinds and structural preconditions assert, not success.
    rsa := crypto_fixture(args[1], "native/rsa-private.der");

    defer free(rsa);
    original_assertion := context.assertion_failed;

    context.logger = crypto_capture;
    context.assertion_failed = crypto_continue_assertion;
    rsa_ok, rsa_output := A.apk_sign2(unsigned_path, rsa, certificate);

    context.assertion_failed = original_assertion;
    context.logger = logger;
    assert(!rsa_ok && rsa_output.count == 0 && crypto_assertions == 1);
    policy_bytes := crypto_fixture(args[1], "verification-structure.bin");

    defer free(policy_bytes);
    policies := policy_bytes;
    good_count := A.read(u32, *policies);

    for 0..cast(s64) good_count - 1 A.read_length_prefixed_block(*policies);

    bad_count := A.read(u32, *policies);
    for 0..cast(s64) bad_count - 1 {
        input := A.read_length_prefixed_block(*policies);
        variant := apk;
        variant.signing_block = input;
        before := crypto_assertions;

        context.logger = crypto_capture;
        context.assertion_failed = crypto_continue_assertion;
        rejected := A.apk_verify(variant);

        context.assertion_failed = original_assertion;
        context.logger = logger;
        assert(!rejected && crypto_assertions == before + 1);
    }

    assert(policies.count == 0);
    algorithm_byte := parsed.signatures[0].data.data - 8;
    algorithm_original := algorithm_byte[0];

    algorithm_byte[0] ^= 1;
    before := crypto_assertions;

    context.logger = crypto_capture;
    context.assertion_failed = crypto_continue_assertion;
    mismatched_algorithms := A.apk_verify(apk);

    context.assertion_failed = original_assertion;
    context.logger = logger;
    algorithm_byte[0] = algorithm_original;
    assert(!mismatched_algorithms && crypto_assertions == before + 1);
    print("ok\n");
}
