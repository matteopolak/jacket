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

fn named(
    machine: u16,
    symbol: &str,
    dll: &str,
    kind: u16,
    ordinal: u16,
    export: Option<&str>,
) -> Vec<u8> {
    let mut data = format!("{symbol}\0{dll}\0").into_bytes();
    if let Some(export) = export {
        data.extend(format!("{export}\0").bytes());
    }
    let mut out = vec![0; 20];
    out[2..4].copy_from_slice(&0xffffu16.to_le_bytes());
    out[6..8].copy_from_slice(&machine.to_le_bytes());
    out[12..16].copy_from_slice(&(data.len() as u32).to_le_bytes());
    out[16..18].copy_from_slice(&ordinal.to_le_bytes());
    out[18..20].copy_from_slice(&(kind << 2).to_le_bytes());
    out.extend(data);
    out
}

#[test]
fn per_symbol_selection_keeps_disjoint_providers_and_rejects_real_ambiguity() {
    let data = archive(&[
        named(0xaa64, "one", "one.dll", 1, 0, None),
        named(0xaa64, "two", "two.dll", 1, 0, None),
        named(0xaa64, "one", "ONE.DLL", 1, 0, None),
    ]);
    let parsed = imports::archive(&data, 0xaa64).unwrap();
    assert_eq!(parsed.providers, ["one.dll", "two.dll"]);
    assert_eq!(parsed.select("one").unwrap().unwrap().provider, "one.dll");
    assert_eq!(parsed.select("two").unwrap().unwrap().provider, "two.dll");
    assert!(parsed.select("unknown").unwrap().is_none());
    assert!(parsed.select("One").unwrap().is_none());
    assert!(parsed.select("__imp_one").unwrap().is_none());
    let ambiguous = archive(&[
        named(0xaa64, "one", "one.dll", 1, 0, None),
        named(0xaa64, "one", "two.dll", 1, 0, None),
    ]);
    assert!(
        imports::archive(&ambiguous, 0xaa64)
            .unwrap()
            .select("one")
            .is_err()
    );
}

#[test]
fn import_name_types_preserve_public_symbol_and_actual_export() {
    use imports::ImportTarget::{Name, Ordinal};
    for (symbol, kind, export, expected) in [
        ("_plain@8", 1, None, Name("_plain@8".into())),
        ("_prefix@8", 2, None, Name("prefix@8".into())),
        ("@undecorated@8", 3, None, Name("undecorated".into())),
        ("?question@8", 3, None, Name("question".into())),
        (
            "local_alias",
            4,
            Some("actual_export"),
            Name("actual_export".into()),
        ),
        ("ordinal_alias", 0, None, Ordinal(42)),
    ] {
        let data = archive(&[named(0xaa64, symbol, "real.dll", kind, 42, export)]);
        let parsed = imports::archive(&data, 0xaa64).unwrap();
        let selected = parsed.select(symbol).unwrap().unwrap();
        assert_eq!(selected.target, expected);
        assert_eq!(selected.symbol, symbol);
        assert_eq!(selected.provider, "real.dll");
        if let Some(export) = export {
            assert!(parsed.select(export).unwrap().is_none());
        }
    }
}

#[test]
fn per_symbol_filtering_cannot_hide_wrong_cpu_or_malformed_aliases() {
    let data = archive(&[
        named(0xa641, "only_ec", "ec.dll", 1, 0, None),
        named(0xa641, "both", "ec.dll", 1, 0, None),
        named(0xaa64, "both", "arm.dll", 1, 0, None),
    ]);
    let parsed = imports::archive(&data, 0xaa64).unwrap();
    assert_eq!(parsed.select("both").unwrap().unwrap().provider, "arm.dll");
    assert!(parsed.select("only_ec").unwrap_err().contains("0xa641"));
    assert!(imports::archive(&data, 0x8664).is_err());
    for bad in [
        named(0xa641, "bad", "ec.dll", 4, 0, None),
        named(0xa641, "bad", "ec.dll", 4, 0, Some("")),
        named(0xaa64, "bad", "arm.dll", 1, 0, Some("unexpected")),
    ] {
        let data = archive(&[named(0xaa64, "good", "arm.dll", 1, 0, None), bad]);
        assert!(imports::archive(&data, 0xaa64).is_err());
    }
    let conflict = archive(&[
        named(0xaa64, "alias", "real.dll", 4, 0, Some("first")),
        named(0xaa64, "alias", "real.dll", 4, 0, Some("second")),
    ]);
    assert!(
        imports::archive(&conflict, 0xaa64)
            .unwrap()
            .select("alias")
            .is_err()
    );
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
fn mixed_machines_select_exact_compatible_providers_in_either_order() {
    for objects in [
        vec![short(0xa641, "ec-only.dll"), short(0xaa64, "arm.dll")],
        vec![short(0xaa64, "arm.dll"), short(0xa641, "ec-only.dll")],
    ] {
        let data = archive(&objects);
        assert_eq!(imports::providers(&data, 0xaa64).unwrap(), ["arm.dll"]);
        assert_eq!(imports::providers(&data, 0xa641).unwrap(), ["ec-only.dll"]);
        assert!(imports::providers(&data, 0x8664).is_err());
    }
    assert!(imports::providers(&archive(&[short(0xa641, "ec.dll")]), 0xaa64).is_err());
    let data = archive(&[
        short(0xa641, "ignored.dll"),
        short(0xaa64, "arm.dll"),
        short(0xaa64, "ARM.DLL"),
        short(0xaa64, "second.dll"),
    ]);
    assert_eq!(
        imports::providers(&data, 0xaa64).unwrap(),
        ["arm.dll", "second.dll"]
    );
}

#[test]
fn incompatible_members_are_validated_not_ignored() {
    let mut flags = short(0xa641, "ec.dll");
    flags[18..20].copy_from_slice(&0xffffu16.to_le_bytes());
    for bad in [
        flags,
        short(0xa641, "../bad.dll"),
        short(0xa641, "ec.dll")[..19].to_vec(),
    ] {
        for objects in [
            vec![short(0xaa64, "arm.dll"), bad.clone()],
            vec![bad.clone(), short(0xaa64, "arm.dll")],
        ] {
            assert!(imports::providers(&archive(&objects), 0xaa64).is_err());
        }
    }
}

#[test]
fn gnu_mixed_machines_preserve_exact_provider_identity() {
    let data = archive(&[gnu(0x8664, b"x64.dll\0", 0), gnu(0xaa64, b"arm.dll\0", 0)]);
    assert_eq!(imports::providers(&data, 0xaa64).unwrap(), ["arm.dll"]);
    assert_eq!(imports::providers(&data, 0x8664).unwrap(), ["x64.dll"]);
}

#[test]
fn archive_alignment_accepts_sdk_nul_and_standard_newline_only() {
    // An odd-sized independent short import, followed by a second member.
    let first = short(0xaa64, "pad0.dll");
    assert_eq!(first.len() % 2, 1);
    let mut data = archive(&[first.clone(), short(0xaa64, "second.dll")]);
    let pad = 8 + 60 + first.len();
    for byte in 0..=u8::MAX {
        data[pad] = byte;
        let result = imports::providers(&data, 0xaa64);
        if matches!(byte, 0 | b'\n') {
            assert_eq!(result.unwrap(), ["pad0.dll", "second.dll"]);
        } else {
            assert!(result.is_err(), "unexpected padding byte {byte}");
        }
    }
    let mut single = archive(&[first]);
    *single.last_mut().unwrap() = 0;
    assert_eq!(imports::providers(&single, 0xaa64).unwrap(), ["pad0.dll"]);
    single.pop();
    assert!(imports::providers(&single, 0xaa64).is_err());
}

#[test]
fn padding_never_repairs_missing_or_corrupt_archive_boundaries() {
    let first = short(0xaa64, "pad0.dll");
    let pad = 8 + 60 + first.len();
    let mut data = archive(&[first, short(0xaa64, "second.dll")]);
    data[pad] = 0;
    let mut missing = data.clone();
    missing.remove(pad);
    assert!(imports::providers(&missing, 0xaa64).is_err());
    let mut extra = data.clone();
    extra.insert(pad, 0);
    assert!(imports::providers(&extra, 0xaa64).is_err());
    let mut bad_header = data.clone();
    bad_header[pad + 1 + 58] = 0;
    assert!(imports::providers(&bad_header, 0xaa64).is_err());
    for end in pad + 1..data.len() {
        // Ending exactly after a complete first member is itself valid.
        if end == pad + 1 {
            continue;
        }
        assert!(
            imports::providers(&data[..end], 0xaa64).is_err(),
            "prefix {end}"
        );
    }
}

#[test]
fn sdk_padding_applies_to_longnames_without_consuming_even_payload_bytes() {
    let mut data = archive(&[b"ab\0".to_vec(), short(0xaa64, "odd.dll")]);
    data[8..24].fill(b' ');
    data[8..10].copy_from_slice(b"//");
    data[8 + 60 + 3] = 0;
    assert_eq!(imports::providers(&data, 0xaa64).unwrap(), ["odd.dll"]);
    // The last member's payload is even and ends in the provider terminator.
    // No additional alignment byte is required or consumed.
    assert_eq!(data.last(), Some(&0));
    data.push(0);
    assert!(imports::providers(&data, 0xaa64).is_err());
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

#[test]
fn llvm_generated_short_imports_retain_actual_symbol_exports() {
    for (data, machine) in [
        (
            include_bytes!("fixtures/windows-imports/ms-x64.lib").as_slice(),
            0x8664,
        ),
        (
            include_bytes!("fixtures/windows-imports/ms-arm64.lib").as_slice(),
            0xaa64,
        ),
    ] {
        let parsed = imports::archive(data, machine).unwrap();
        let import = parsed.select("ordinary_entry").unwrap().unwrap();
        assert_eq!(import.provider, "independently_named_42.dll");
        assert_eq!(
            import.target,
            imports::ImportTarget::Name("ordinary_entry".into())
        );
        assert!(parsed.select("not_exported").unwrap().is_none());
    }
    let parsed =
        imports::archive(include_bytes!("fixtures/windows-imports/gnu-x64.a"), 0x8664).unwrap();
    assert!(parsed.imports.is_empty());
    assert_eq!(parsed.providers, ["independently_named_42.dll"]);
}
