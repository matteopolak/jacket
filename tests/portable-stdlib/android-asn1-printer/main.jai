// Refreshed reviewed supported diagnostic types/overloads; independent wire data.
A :: #import "Android/Toolchain";

main :: () {
    #assert !#exists(A.Diagnostic_Frame) && !#exists(A.diagnostic_hex_bytes);
    A.asn1_print("\x30\x03\x02\x01\x7f\x31\x02\x05\x00");
    A.asn1_print("\x06\x03\x2a\x03\x04\x13\x02hi\x0c\x02\xc3\xa9");
    A.asn1_print("\x03\x03\x00\xab\xcd\x04\x02\x00\xff");
    A.asn1_print("\x02\x09\x01\x02\x03\x04\x05\x06\x07\x08\x09");
    A.asn1_print("\x17\x0d250102030405Z\x17\x0d991231235959Z");
    A.asn1_print("\x17\x0d491231235959Z\x17\x0d500101000000Z");
    A.asn1_print("\xa2\x02\x05\x00\x82\x0a0123456789");
    A.asn1_print("\x05\x00", depth = 2);
    A.asn1_print("\x05\x00\x02\x01\x00", depth = 2, hack_skip_depth = true);
    A.asn1_print("named", "\x13\x02hi");
    value: A.ASN1_Value;
    value.tag.universal_type = .INTEGER;
    value.content = "\x05\x00";
    A.asn1_print("children", value);
}
