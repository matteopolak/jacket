//! Permission-metadata controls; no process memory is accessed.
#[path = "../src/interp/probe_maps.rs"]
mod maps;

#[test]
fn mapping_permissions_are_read_only_metadata_with_exact_boundaries() {
    let text = b"1000-3000 rw-p 00000000 00:00 0 [heap]\n3000-4000 r--p 00000000 00:00 0\n4000-5000 --xp 00000000 00:00 0\n5000-6000 ---p 00000000 00:00 0\n6000-7000 rw-s 00000000 00:00 0 pathname with spaces\n";
    assert_eq!(maps::region(text, 0x1000, true), Some((0x1000, 0x3000)));
    assert_eq!(maps::region(text, 0x2fff, false), Some((0x1000, 0x3000)));
    assert_eq!(maps::region(text, 0x3000, false), Some((0x3000, 0x4000)));
    assert_eq!(maps::region(text, 0x3000, true), None);
    assert_eq!(maps::region(text, 0x4000, false), None);
    assert_eq!(maps::region(text, 0x5000, true), None);
    assert_eq!(maps::region(text, 0x6000, true), Some((0x6000, 0x7000)));
    assert_eq!(maps::region(text, 0x7000, false), None);
    assert_eq!(maps::region(text, 0xfff, false), None);
}

#[test]
fn malformed_or_unavailable_mapping_metadata_fails_closed() {
    for text in [
        "",
        "1000-2000",
        "invalid rw-p",
        "2000-1000 rw-p",
        "1000-1000 rw-p",
        "1000-2000 rwx",
        "1000-2000 abcdef",
        "1000-2000 rw-z",
        "1000-2000 -w-p",
        "1000-10000000000000000 rw-p",
    ] {
        assert_eq!(maps::region(text.as_bytes(), 0x1000, true), None, "{text}");
    }
}

#[test]
fn non_utf8_names_and_unusual_path_bytes_do_not_change_permissions() {
    let text = b"1000-2000 rw-p 00000000 00:00 0 /a path\xff [ignored]\n";
    assert_eq!(maps::region(text, 0x1800, true), Some((0x1000, 0x2000)));
}
