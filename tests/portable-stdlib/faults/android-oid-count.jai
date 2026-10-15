// Independent safety: a public count cannot index beyond twelve stored arcs.
A :: #import "Android/Toolchain";

main :: () {
    left: A.ASN1_Object_Identifier;
    right: A.ASN1_Object_Identifier;
    right.parts_count = 13;
    A.asn1_oid_equals(left, right);
}
