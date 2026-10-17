// Real Java-produced JKS through reviewed public read/decrypt/sign interfaces.
#import "Basic";
A :: #import "Android/Toolchain";
F :: #import "File";

Verifier :: #library "../../../../../target/portable-crypto-integration/sdk-verify";
verify_signature :: (public_key: *u8, public_count: u64, message: *u8, message_count: u64, signature: *u8, signature_count: u64) -> s32 #foreign Verifier "integration_verify_signature";

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 4);
    algorithm := A.Android_Signing_Key_Signature_Algorithm.EC;
    if args[3] == "RSA" algorithm = .RSA;
    else if args[3] == "DSA" algorithm = .DSA;
    else assert(args[3] == "EC");

    ok, key := A.android_generate_jks_keystore(args[1], args[2], algorithm = algorithm);

    assert(ok && key.kind == .JKS && key.jks.keystore_path == args[2]);
    defer free(key.jks.keystore_path);
    defer free(key.jks.alias);
    defer free(key.jks.password);
    data, loaded := F.read_entire_file(args[2]);

    assert(loaded);
    defer free(data);
    read_ok, certificate, protected_key := A.jks_read(data, key.jks.password);

    assert(read_ok && certificate.count > 0 && protected_key.count > 0);
    assert(certificate.data >= data.data && certificate.data + certificate.count <= data.data + data.count);
    assert(protected_key.data >= data.data && protected_key.data + protected_key.count <= data.data + data.count);
    decrypted, private_key := A.decrypt_private_key(protected_key, key.jks.password);

    assert(decrypted && private_key.count > 0);
    defer free(private_key);
    public_key := A.x509_get_subject_public_key_info(certificate).value_data;
    message := "actual Java JKS integration\x00binary";
    signed, signature := A.sign_data(private_key, message);

    assert(signed && signature.count > 0);
    defer free(signature);
    valid := verify_signature(public_key.data, cast(u64) public_key.count, message.data, cast(u64) message.count,
                              signature.data, cast(u64) signature.count);

    assert(valid == 1);
    print("ok\n");
}
