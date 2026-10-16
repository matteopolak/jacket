//! Portable metadata controls; these tests never load or execute a Windows DLL.
#[path = "../src/interp/native/windows_imports.rs"]
mod imports;

fn short(machine: u16, provider: &str) -> Vec<u8> {
    let data = format!("entry\0{provider}\0");
    let mut out = vec![0; 20];
    out[2..4].copy_from_slice(&0xffffu16.to_le_bytes());
    out[6..8].copy_from_slice(&machine.to_le_bytes());
    out[12..16].copy_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend(data.bytes());
    out
}

fn archive(objects: &[Vec<u8>]) -> Vec<u8> {
    let mut out = b"!<arch>\n".to_vec();
    for object in objects {
        out.extend(
            format!(
                "{:<16}{:<12}{:<6}{:<6}{:<8}{:<10}`\n",
                "unit.obj/",
                0,
                0,
                0,
                0,
                object.len()
            )
            .bytes(),
        );
        out.extend(object);
        if object.len() % 2 != 0 {
            out.push(b'\n');
        }
    }
    out
}

fn gnu(machine: u16, payload: &[u8], relocations: u16) -> Vec<u8> {
    let mut object = vec![0; 60];
    object[..2].copy_from_slice(&machine.to_le_bytes());
    object[2..4].copy_from_slice(&1u16.to_le_bytes());
    object[20..28].copy_from_slice(b".idata$7");
    object[36..40].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    object[40..44].copy_from_slice(&60u32.to_le_bytes());
    object[52..54].copy_from_slice(&relocations.to_le_bytes());
    object.extend(payload);
    object
}

#[test]
fn short_imports_preserve_real_provider_and_deduplicate_case() {
    let bytes = archive(&[
        short(0x8664, "Actual_42.dll"),
        short(0x8664, "ACTUAL_42.DLL"),
    ]);
    assert_eq!(
        imports::providers(&bytes, 0x8664).unwrap(),
        ["Actual_42.dll"]
    );
    assert_eq!(
        imports::providers(&archive(&[short(0xaa64, "printer.drv")]), 0xaa64).unwrap(),
        ["printer.drv"]
    );
}

#[test]
fn gnu_descriptor_name_is_not_an_arbitrary_dll_string() {
    let bytes = archive(&[
        gnu(0x8664, b"real-provider.dll\0", 0),
        gnu(0x8664, b"wrong.dll\0", 1),
    ]);
    assert_eq!(
        imports::providers(&bytes, 0x8664).unwrap(),
        ["real-provider.dll"]
    );
    assert!(
        imports::providers(
            &archive(&[b"not a COFF object: decoy.dll\0".to_vec()]),
            0x8664
        )
        .unwrap()
        .is_empty()
    );
}

#[test]
fn incompatible_and_malformed_metadata_reject() {
    for bytes in [
        archive(&[short(0xaa64, "real.dll")]),
        archive(&[short(0x8664, "../escape.dll")]),
        archive(&[gnu(0x8664, b"unterminated.dll", 0)]),
    ] {
        assert!(imports::providers(&bytes, 0x8664).is_err());
    }
    let bytes = archive(&[short(0x8664, "real.dll")]);
    for end in 0..bytes.len() {
        if end == 8 {
            continue;
        } // An empty archive is valid.
        assert!(
            imports::providers(&bytes[..end], 0x8664).is_err(),
            "prefix {end}"
        );
    }
}

#[test]
fn multiple_providers_are_reported_without_synthesized_handle() {
    let bytes = archive(&[short(0x8664, "one.dll"), short(0x8664, "two.dll")]);
    assert_eq!(
        imports::providers(&bytes, 0x8664).unwrap(),
        ["one.dll", "two.dll"]
    );
}

#[test]
fn long_microsoft_descriptor_uses_the_name_relocation_not_hint_strings() {
    let mut data = vec![0u8; 100];
    data[..2].copy_from_slice(&0x8664u16.to_le_bytes());
    data[2..4].copy_from_slice(&2u16.to_le_bytes());
    data[8..12].copy_from_slice(&130u32.to_le_bytes());
    data[12..16].copy_from_slice(&1u32.to_le_bytes());
    data[20..28].copy_from_slice(b".idata$2");
    data[36..40].copy_from_slice(&20u32.to_le_bytes());
    data[40..44].copy_from_slice(&100u32.to_le_bytes());
    data[44..48].copy_from_slice(&120u32.to_le_bytes());
    data[52..54].copy_from_slice(&1u16.to_le_bytes());
    data[60..68].copy_from_slice(b".idata$6");
    data[76..80].copy_from_slice(&14u32.to_le_bytes());
    data[80..84].copy_from_slice(&148u32.to_le_bytes());
    data.resize(148, 0);
    data[120..124].copy_from_slice(&12u32.to_le_bytes());
    data[128..130].copy_from_slice(&3u16.to_le_bytes());
    data[142..144].copy_from_slice(&2u16.to_le_bytes());
    data.extend(b"real-name.dll\0");
    assert_eq!(
        imports::providers(&archive(&[data]), 0x8664).unwrap(),
        ["real-name.dll"]
    );
}

#[test]
fn short_header_flags_and_provider_paths_are_validated() {
    let mut bad = short(0x8664, "real.dll");
    bad[18..20].copy_from_slice(&0xffffu16.to_le_bytes());
    assert!(imports::providers(&archive(&[bad]), 0x8664).is_err());
    for name in ["", "C:bad.dll", "a\\bad.dll", "bad\n.dll"] {
        assert!(imports::providers(&archive(&[short(0x8664, name)]), 0x8664).is_err());
    }
}

#[test]
fn source_configured_and_sdk_candidate_order_and_explicit_names() {
    use imports::{Candidate, candidates};
    use std::path::{Path, PathBuf};
    let dirs = vec![PathBuf::from("configured"), PathBuf::from("sdk")];
    let plan = candidates("widget", false, "source", &dirs, false).unwrap();
    assert_eq!(
        plan[0],
        Candidate::Archive(Path::new("source").join("widget.lib"))
    );
    let source_dll = plan
        .iter()
        .position(|p| *p == Candidate::Dll(Path::new("source").join("widget.dll")))
        .unwrap();
    let configured = plan
        .iter()
        .position(|p| *p == Candidate::Archive(Path::new("configured").join("widget.lib")))
        .unwrap();
    assert!(source_dll < configured);
    assert_eq!(
        plan.last(),
        Some(&Candidate::Dll(PathBuf::from("widget.dll")))
    );
    let dll = candidates("actual.dll", true, "", &[], false).unwrap();
    assert_eq!(dll, [Candidate::Dll(PathBuf::from("actual.dll"))]);
    let explicit = candidates("actual.lib", true, "", &[], false).unwrap();
    assert_eq!(explicit, [Candidate::Archive(PathBuf::from("actual.lib"))]);
}

#[test]
fn real_official_tool_generated_archives_have_the_same_provider() {
    for (data, machine) in [
        (
            include_bytes!("fixtures/windows-imports/ms-x64.lib").as_slice(),
            0x8664,
        ),
        (
            include_bytes!("fixtures/windows-imports/ms-arm64.lib").as_slice(),
            0xaa64,
        ),
        (
            include_bytes!("fixtures/windows-imports/gnu-x64.a").as_slice(),
            0x8664,
        ),
    ] {
        assert_eq!(
            imports::providers(data, machine).unwrap(),
            ["independently_named_42.dll"]
        );
    }
}
