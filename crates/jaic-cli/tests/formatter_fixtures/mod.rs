//! Narrow transport-metadata recognition for repository formatter acceptance.
use std::path::Path;

pub fn require_readonly_capability(retained: bool, ci: bool) {
    assert!(
        retained || !ci,
        "CI formatter acceptance requires a filesystem that retains readonly permissions"
    );
}

pub fn is_appledouble_metadata(path: &Path) -> bool {
    if !path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("._"))
    {
        return false;
    }
    let bytes = std::fs::read(path).unwrap();
    is_appledouble_bytes(&bytes)
}

fn is_appledouble_bytes(bytes: &[u8]) -> bool {
    if bytes.len() < 26 || bytes[..8] != [0, 5, 0x16, 7, 0, 2, 0, 0] {
        return false;
    }
    let count = usize::from(u16::from_be_bytes([bytes[24], bytes[25]]));
    let table_end = 26 + count * 12;
    if table_end > bytes.len() {
        return false;
    }
    for entry in bytes[26..table_end].as_chunks::<12>().0 {
        let offset = u64::from(u32::from_be_bytes(entry[4..8].try_into().unwrap()));
        let length = u64::from(u32::from_be_bytes(entry[8..12].try_into().unwrap()));
        if length > 0 && (offset < table_end as u64 || offset + length > bytes.len() as u64) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_unavailable_readonly_capability_is_explicitly_supported() {
        require_readonly_capability(false, false);
        require_readonly_capability(true, false);
        require_readonly_capability(true, true);
    }

    #[test]
    #[should_panic(expected = "CI formatter acceptance requires a filesystem")]
    fn ci_cannot_waive_readonly_acceptance() {
        require_readonly_capability(false, true);
    }

    fn metadata() -> Vec<u8> {
        let mut bytes = vec![0; 39];
        bytes[..8].copy_from_slice(&[0, 5, 0x16, 7, 0, 2, 0, 0]);
        bytes[24..26].copy_from_slice(&1_u16.to_be_bytes());
        bytes[26..30].copy_from_slice(&2_u32.to_be_bytes());
        bytes[30..34].copy_from_slice(&38_u32.to_be_bytes());
        bytes[34..38].copy_from_slice(&1_u32.to_be_bytes());
        bytes[38] = 0xff;
        bytes
    }

    #[test]
    fn metadata_requires_complete_header_table_and_in_bounds_payload() {
        let valid = metadata();
        assert!(is_appledouble_bytes(&valid));
        for end in 0..valid.len() {
            assert!(!is_appledouble_bytes(&valid[..end]));
        }
        for (position, value) in [
            (0, 1_u32),
            (4, 1),
            (30, 0),
            (30, 40),
            (34, 2),
            (34, u32::MAX),
        ] {
            let mut bad = valid.clone();
            bad[position..position + 4].copy_from_slice(&value.to_be_bytes());
            assert!(!is_appledouble_bytes(&bad));
        }
        let mut bad = valid;
        bad[24..26].copy_from_slice(&2_u16.to_be_bytes());
        assert!(!is_appledouble_bytes(&bad));
        assert!(!is_appledouble_bytes(b"main :: () {}"));
    }

    #[test]
    fn ordinary_and_invalid_sidecar_named_source_stays_visible() {
        let root = std::env::temp_dir().join(format!("formatter-discovery-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let ordinary = root.join("ordinary.jai");
        let metadata_path = root.join("._ordinary.jai");
        let named_source = root.join("._legitimate.jai");
        let invalid = root.join("._invalid.jai");
        std::fs::write(&ordinary, metadata()).unwrap();
        std::fs::write(&metadata_path, metadata()).unwrap();
        std::fs::write(&named_source, "value :: 1;").unwrap();
        std::fs::write(&invalid, [0xff]).unwrap();
        assert!(!is_appledouble_metadata(&ordinary));
        assert!(is_appledouble_metadata(&metadata_path));
        assert!(!is_appledouble_metadata(&named_source));
        assert!(!is_appledouble_metadata(&invalid));
        assert!(std::fs::read_to_string(invalid).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
