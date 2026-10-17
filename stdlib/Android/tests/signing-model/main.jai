// Reviewed shared model: exact canonical records/enums and union storage.
using #import "Basic";
A :: #import "Android/Toolchain";

main :: () {
    key: A.Android_Signing_Key;

    assert(key.kind == .JKS);
    assert(key.keystore_path.count == 0 && key.alias.count == 0 && key.password.count == 0);
    assert(type_info(A.Android_Signing_Key).name == "Android_Signing_Key");
    assert(type_info(A.Android_Signing_Key_Kind).name == "Android_Signing_Key_Kind");
    assert(type_info(A.Android_Signing_Key_Signature_Algorithm).name == "Android_Signing_Key_Signature_Algorithm");
    assert(cast(s64) A.Android_Signing_Key_Kind.ASN1_CERTIFICATE_AND_PRIVATE_KEY == 1);
    assert(cast(s64) A.Android_Signing_Key_Signature_Algorithm.RSA == 0);
    assert(cast(s64) A.Android_Signing_Key_Signature_Algorithm.EC == 1);
    assert(cast(s64) A.Android_Signing_Key_Signature_Algorithm.DSA == 2);
    key.jks.keystore_path = "fixture.jks";
    key.alias = "fixture-alias";
    key.password = "fixture-only";
    assert(key.jks.alias == "fixture-alias");
    assert(cast(*void) *key.jks.keystore_path == cast(*void) *key.direct.certificate_path);
    assert(cast(*void) *key.direct.certificate_path == cast(*void) *key.static.certificate);
    assert(cast(*void) *key.jks.alias == cast(*void) *key.direct.private_key_path);
    assert(cast(*void) *key.direct.private_key_path == cast(*void) *key.static.private_key);
    key.kind = .ASN1_CERTIFICATE_AND_PRIVATE_KEY;
    key.static.certificate = "fixture certificate bytes";
    key.static.private_key = "fixture private bytes";
    assert(key.direct.certificate_path == "fixture certificate bytes");
    assert(key.direct.private_key_path == "fixture private bytes");
    assert(key.password == "fixture-only");
    print("ok\n");
}
