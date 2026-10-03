//! Check emitted object format and architecture without loading the object.
pub fn check(bytes: &[u8], triple: &str) {
    if triple.starts_with("wasm") {
        assert_eq!(&bytes[..8], b"\0asm\x01\0\0\0", "{triple}");
    } else if triple.contains("windows") {
        let expected = if triple.starts_with("x86_64") {
            0x8664
        } else {
            0xaa64
        };
        assert_eq!(
            u16::from_le_bytes(bytes[..2].try_into().unwrap()),
            expected,
            "{triple}"
        );
    } else if triple.contains("apple") {
        assert_eq!(&bytes[..4], b"\xcf\xfa\xed\xfe", "{triple}");
        let expected = if triple.starts_with("x86_64") {
            0x01000007
        } else {
            0x0100000c
        };
        assert_eq!(
            u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            expected,
            "{triple}"
        );
    } else {
        assert_eq!(&bytes[..4], b"\x7fELF", "{triple}");
        assert_eq!(bytes[4], 2, "{triple}");
        assert_eq!(bytes[5], 1, "{triple}");
        let expected = if triple.starts_with("x86_64") {
            62
        } else {
            183
        };
        assert_eq!(
            u16::from_le_bytes(bytes[18..20].try_into().unwrap()),
            expected,
            "{triple}"
        );
    }
}
