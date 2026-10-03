Android_Signing_Key_Kind :: enum {
    JKS;
    ASN1_CERTIFICATE_AND_PRIVATE_KEY;
}

Android_Signing_Key_Signature_Algorithm :: enum {
    RSA;
    EC;
    DSA;
}

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
