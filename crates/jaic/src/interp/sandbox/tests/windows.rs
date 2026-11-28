//! The sandbox's Win32 files, directory listings, text conversion and process refusal.
use super::*;

#[test]
fn windows_file_round_trip() {
    let mut h = host();
    let name = wide("data.txt");
    let (create_always, open_existing) = (2, 3);
    let handle = call(
        &mut h,
        "CreateFileW",
        &[
            name.as_ptr() as u64,
            0x4000_0000,
            0,
            0,
            create_always,
            0x80,
            0,
        ],
    );

    assert_ne!(handle, u64::MAX);
    let mut done = 0u32;
    let payload = b"hello windows";
    let ok = call(
        &mut h,
        "WriteFile",
        &[
            handle,
            payload.as_ptr() as u64,
            payload.len() as u64,
            &raw mut done as u64,
            0,
        ],
    );

    assert_eq!((ok, done as usize), (1, payload.len()));
    assert_eq!(call(&mut h, "CloseHandle", &[handle]), 1);

    let handle = call(
        &mut h,
        "CreateFileW",
        &[
            name.as_ptr() as u64,
            0x8000_0000,
            0,
            0,
            open_existing,
            0x80,
            0,
        ],
    );

    assert_ne!(handle, u64::MAX);
    let mut buf = [0u8; 32];
    let ok = call(
        &mut h,
        "ReadFile",
        &[
            handle,
            buf.as_mut_ptr() as u64,
            buf.len() as u64,
            &raw mut done as u64,
            0,
        ],
    );

    assert_eq!((ok, done as usize), (1, payload.len()));
    assert_eq!(&buf[..payload.len()], payload);
    let mut size = 0u64;

    assert_eq!(
        call(&mut h, "GetFileSizeEx", &[handle, &raw mut size as u64]),
        1
    );
    assert_eq!(size as usize, payload.len());
    assert_eq!(call(&mut h, "CloseHandle", &[handle]), 1);

    // A missing file: INVALID_HANDLE_VALUE and ERROR_FILE_NOT_FOUND.
    let missing = wide("nope.txt");
    let bad = call(
        &mut h,
        "CreateFileW",
        &[
            missing.as_ptr() as u64,
            0x8000_0000,
            0,
            0,
            open_existing,
            0x80,
            0,
        ],
    );

    assert_eq!(bad, u64::MAX);
    assert_eq!(call(&mut h, "GetLastError", &[]), 2);
}

#[test]
#[allow(
    clippy::disallowed_methods,
    reason = "the sandbox's records are read back from host memory"
)]
fn windows_find_files_lists_a_directory() {
    let mut h = host();

    h.fs.dirs.insert("/tmp/d".to_string());
    h.fs.files
        .insert("/tmp/d/a.txt".to_string(), b"abc".to_vec());
    h.fs.files.insert("/tmp/d/b.txt".to_string(), Vec::new());
    let pattern = wide("d\\*");
    let mut data = vec![0u8; 592];
    let find = call(
        &mut h,
        "FindFirstFileW",
        &[pattern.as_ptr() as u64, data.as_mut_ptr() as u64],
    );

    assert_ne!(find, u64::MAX);
    let name_of = |data: &[u8]| {
        let units: Vec<u16> = data[44..44 + 520]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .take_while(|&u| u != 0)
            .collect();
        String::from_utf16(&units).unwrap()
    };
    let mut names = vec![name_of(&data)];

    loop {
        let ok = call(&mut h, "FindNextFileW", &[find, data.as_mut_ptr() as u64]);

        if ok == 0 {
            assert_eq!(call(&mut h, "GetLastError", &[]), 18);
            break;
        }

        names.push(name_of(&data));
    }

    names.sort();
    assert_eq!(names, [".", "..", "a.txt", "b.txt"]);
    assert_eq!(call(&mut h, "FindClose", &[find]), 1);
    // A directory with nothing matching.
    let none = wide("d\\*.zzz");
    let bad = call(
        &mut h,
        "FindFirstFileW",
        &[none.as_ptr() as u64, data.as_mut_ptr() as u64],
    );

    assert_eq!(bad, u64::MAX);
}

#[test]
fn windows_text_conversion_and_current_directory() {
    let mut h = host();
    let text = "h\u{e9}llo \u{4e16}";
    let mut utf16 = [0u16; 16];
    let n = call(
        &mut h,
        "MultiByteToWideChar",
        &[
            65001,
            0,
            text.as_ptr() as u64,
            text.len() as u64,
            utf16.as_mut_ptr() as u64,
            16,
        ],
    );
    let expected: Vec<u16> = text.encode_utf16().collect();

    assert_eq!(n as usize, expected.len());
    assert_eq!(&utf16[..expected.len()], &expected[..]);
    // A room of 0 asks for the count only.
    assert_eq!(
        call(
            &mut h,
            "MultiByteToWideChar",
            &[65001, 0, text.as_ptr() as u64, text.len() as u64, 0, 0],
        ),
        n
    );
    let mut back = [0u8; 32];
    let m = call(
        &mut h,
        "WideCharToMultiByte",
        &[
            65001,
            0,
            utf16.as_ptr() as u64,
            n,
            back.as_mut_ptr() as u64,
            32,
            0,
            0,
        ],
    );

    assert_eq!(&back[..m as usize], text.as_bytes());

    let mut cwd = [0u16; 16];
    let len = call(
        &mut h,
        "GetCurrentDirectoryW",
        &[16, cwd.as_mut_ptr() as u64],
    );

    assert_eq!(String::from_utf16(&cwd[..len as usize]).unwrap(), "/tmp");
    // Too small: the size needed, terminator included.
    assert_eq!(
        call(
            &mut h,
            "GetCurrentDirectoryW",
            &[2, cwd.as_mut_ptr() as u64]
        ),
        5
    );
}

#[test]
fn windows_process_creation_is_access_denied() {
    let mut h = host();

    for symbol in [
        "CreateProcessW",
        "CreateProcessA",
        "CreatePipe",
        "CreateFileMappingW",
    ] {
        h.windows.last_error = 0;
        assert_eq!(call(&mut h, symbol, &[0; 10]), 0, "{symbol}");
        assert_eq!(call(&mut h, "GetLastError", &[]), 5, "{symbol}");
    }
}
