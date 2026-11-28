// Tells where an [Android_Signing_Key] gets its key: a JKS keystore or separate DER certificate and key files.
Android_Signing_Key_Kind :: enum {
    JKS;
    ASN1_CERTIFICATE_AND_PRIVATE_KEY;
}

// Lists the key algorithms of a generated signing key.
Android_Signing_Key_Signature_Algorithm :: enum {
    RSA;
    EC;
    DSA;
}

// Describes the key an APK is signed with.
//
// `kind` says which union member is in use: `jks` (whose fields are also reachable directly as
// `keystore_path`, `alias` and `password`) for a keystore, or `direct` for DER files. `static` holds
// certificate and private key bytes and has no matching kind. The password is plain text.
Android_Signing_Key :: struct {
    kind: Android_Signing_Key_Kind;
    union {
        using jks: struct {
            keystore_path: string;
            alias: string;
            password: string;
        }
        direct: struct {
            certificate_path: string;
            private_key_path: string;
        }
        static: struct {
            certificate: string;
            private_key: string;
        }
    }
}
