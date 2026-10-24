//! Unit tests of the sandbox host, calling its libc procedures directly.
use super::guest::read_u32;
use super::*;

fn sig() -> ir::Sig {
    ir::Sig {
        params: vec![],
        returns: vec![],
        conv: ir::Conv::C,
        c_varargs: false,
        c_fixed: 0,
        c_abi: None,
    }
}

fn host() -> SandboxHost {
    let mut host = SandboxHost {
        cwd: "/tmp".to_string(),
        ..Default::default()
    };
    host.fs.dirs.insert("/tmp".to_string());
    host
}

/// A zero-terminated UTF-16 string.
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain([0]).collect()
}

fn call(host: &mut SandboxHost, symbol: &str, args: &[u64]) -> u64 {
    host.foreign(symbol, args, &sig())
        .unwrap_or_else(|| panic!("{symbol} is not provided"))
        .unwrap()[0]
}

fn errno(host: &SandboxHost) -> u32 {
    read_u32(host.errno_cell)
}

#[test]
fn process_creation_and_shared_memory_are_refused() {
    let mut h = host();
    for symbol in [
        "fork",
        "vfork",
        "pipe",
        "pipe2",
        "socketpair",
        "execv",
        "execve",
        "execvp",
        "shm_open",
        "shm_unlink",
    ] {
        assert_eq!(call(&mut h, symbol, &[0, 0, 0]), u64::MAX, "{symbol}");
        assert_eq!(errno(&h), EPERM as u32, "{symbol}");
    }
    // posix_spawn returns the error number itself.
    assert_eq!(call(&mut h, "posix_spawn", &[0, 0, 0, 0, 0, 0]), EPERM);
    assert_eq!(call(&mut h, "posix_spawnp", &[0, 0, 0, 0, 0, 0]), EPERM);
}

#[test]
fn darwin_error_and_realpath_names_are_served() {
    let mut h = host();
    let cell = call(&mut h, "__error", &[]);
    assert_ne!(cell, 0);
    assert_eq!(cell, call(&mut h, "__errno_location", &[]));
}

#[test]
fn c_numeric_strtof_rounds_the_decimal_directly_to_binary32() {
    let mut h = host();
    let text = b"1.0000000596046447753906250000000000000001!\0";
    let mut end = 0u64;
    let bits = call(
        &mut h,
        "strtof",
        &[text.as_ptr() as u64, (&raw mut end) as u64],
    );
    assert_eq!(bits, 0x3f80_0001);
    assert_eq!(end, text.as_ptr() as u64 + text.len() as u64 - 2);
}

#[test]
fn c_numeric_hexadecimal_and_nan_payloads_preserve_endptr() {
    let mut h = host();
    for (text, expected, consumed) in [
        (b"0x1.8p+2suffix\0".as_slice(), 6.0f64.to_bits(), 8),
        (b"nan(payload)tail\0".as_slice(), f64::NAN.to_bits(), 12),
    ] {
        let mut end = 0u64;
        assert_eq!(
            call(
                &mut h,
                "strtod",
                &[text.as_ptr() as u64, (&raw mut end) as u64]
            ),
            expected
        );
        assert_eq!(end, text.as_ptr() as u64 + consumed);
    }
}

#[test]
fn c_numeric_range_errors_set_errno_and_success_does_not_clear_it() {
    let mut h = host();
    call(&mut h, "__errno_location", &[]);
    let text = b"1e9999\0";
    assert_eq!(
        call(&mut h, "strtod", &[text.as_ptr() as u64, 0]),
        f64::INFINITY.to_bits()
    );
    assert_eq!(errno(&h), 34);
    let text = b"7\0";
    assert_eq!(
        call(&mut h, "strtod", &[text.as_ptr() as u64, 0]),
        7.0f64.to_bits()
    );
    assert_eq!(errno(&h), 34);
}

#[test]
fn c_numeric_locale_queries_and_unsupported_locales_stay_in_the_sandbox() {
    let mut h = host();
    assert_ne!(call(&mut h, "setlocale", &[6, 0]), 0);
    let c = b"C\0";
    assert_ne!(call(&mut h, "setlocale", &[6, c.as_ptr() as u64]), 0);
    let other = b"de_DE.UTF-8\0";
    assert_eq!(call(&mut h, "setlocale", &[6, other.as_ptr() as u64]), 0);
    let text = b"1,5\0";
    let mut end = 0u64;
    assert_eq!(
        call(
            &mut h,
            "strtod",
            &[text.as_ptr() as u64, (&raw mut end) as u64]
        ),
        1.0f64.to_bits()
    );
    assert_eq!(end, text.as_ptr() as u64 + 1);
}

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
        h.last_error = 0;
        assert_eq!(call(&mut h, symbol, &[0; 10]), 0, "{symbol}");
        assert_eq!(call(&mut h, "GetLastError", &[]), 5, "{symbol}");
    }
}

/// A zero-terminated C string.
fn cstring(text: &str) -> Vec<u8> {
    text.bytes().chain([0]).collect()
}

fn open(h: &mut SandboxHost, path: &str, flags: u64) -> u64 {
    call(h, "open", &[cstring(path).as_ptr() as u64, flags])
}

fn write_to(h: &mut SandboxHost, fd: u64, text: &[u8]) -> u64 {
    call(h, "write", &[fd, text.as_ptr() as u64, text.len() as u64])
}

fn read_from(h: &mut SandboxHost, fd: u64, size: usize) -> Vec<u8> {
    let mut buf = vec![0u8; size];
    let n = call(h, "read", &[fd, buf.as_mut_ptr() as u64, size as u64]);
    if n == u64::MAX {
        return Vec::new();
    }
    buf.truncate(n as usize);
    buf
}

/// `(mode, size, links)` from `stat`, or the errno of a failed call.
fn stat_of(h: &mut SandboxHost, symbol: &str, subject: u64) -> Result<(u32, u64, u64), u32> {
    let mut st = [0u8; 144];
    if call(h, symbol, &[subject, st.as_mut_ptr() as u64]) == u64::MAX {
        return Err(errno(h));
    }
    let word = |at: usize| u64::from_le_bytes(st[at..at + 8].try_into().unwrap());
    Ok((word(24) as u32, word(48), word(16)))
}

fn size_of_path(h: &mut SandboxHost, path: &str) -> u64 {
    stat_of(h, "stat", cstring(path).as_ptr() as u64).unwrap().1
}

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn linux_stat_syscalls_reach_the_virtual_file_system() {
    let mut h = host();
    let missing = cstring("nope");
    let mut st = [0u8; 144];
    assert_eq!(
        call(
            &mut h,
            "syscall",
            &[4, missing.as_ptr() as u64, st.as_mut_ptr() as u64]
        ),
        u64::MAX
    );
    let root = cstring("/tmp");
    assert_eq!(
        call(
            &mut h,
            "syscall",
            &[4, root.as_ptr() as u64, st.as_mut_ptr() as u64]
        ),
        0
    );
    assert_eq!(
        call(
            &mut h,
            "syscall",
            &[6, root.as_ptr() as u64, st.as_mut_ptr() as u64]
        ),
        0
    );
}

#[test]
fn file_descriptors_write_read_seek_and_report_errors() {
    let mut h = host();
    assert_eq!(write_to(&mut h, 1, b"out"), 3);
    assert_eq!(write_to(&mut h, 2, b"err"), 3);
    assert_eq!(h.stdout, b"out");
    assert_eq!(h.stderr, b"err");
    assert_eq!(write_to(&mut h, 77, b"x"), u64::MAX);
    assert_eq!(errno(&h), EBADF as u32);

    let fd = open(&mut h, "a.txt", O_CREAT | O_RDWR);
    assert_eq!(fd, 3);
    assert_eq!(write_to(&mut h, fd, b"hello world"), 11);
    assert_eq!(call(&mut h, "lseek", &[fd, 6, 0]), 6);
    assert_eq!(read_from(&mut h, fd, 32), b"world");
    assert_eq!(call(&mut h, "lseek", &[fd, (-5i64) as u64, 1]), 6);
    assert_eq!(call(&mut h, "lseek64", &[fd, (-5i64) as u64, 2]), 6);
    assert_eq!(read_from(&mut h, fd, 2), b"wo");
    // Before the start, or with an unknown origin, the seek fails.
    assert_eq!(call(&mut h, "lseek", &[fd, (-1i64) as u64, 0]), u64::MAX);
    assert_eq!(errno(&h), EINVAL as u32);
    assert_eq!(call(&mut h, "lseek", &[fd, 0, 9]), u64::MAX);
    assert_eq!(call(&mut h, "lseek", &[500, 0, 0]), u64::MAX);

    assert_eq!(call(&mut h, "close", &[fd]), 0);
    assert_eq!(call(&mut h, "close", &[fd]), u64::MAX);
    assert_eq!(errno(&h), EBADF as u32);
    assert_eq!(read_from(&mut h, fd, 4), b"");
    assert_eq!(errno(&h), EBADF as u32);
}

#[test]
fn open_flags_decide_creation_truncation_and_access() {
    let mut h = host();
    assert_eq!(open(&mut h, "missing", 0), u64::MAX);
    assert_eq!(errno(&h), ENOENT as u32);
    assert_eq!(open(&mut h, "nodir/file", O_CREAT | O_WRONLY), u64::MAX);
    assert_eq!(errno(&h), ENOENT as u32);

    let fd = open(&mut h, "f", O_CREAT | O_WRONLY);
    write_to(&mut h, fd, b"abcdef");
    call(&mut h, "close", &[fd]);
    assert_eq!(open(&mut h, "f", O_CREAT | O_EXCL | O_WRONLY), u64::MAX);
    assert_eq!(errno(&h), EEXIST as u32);
    assert_eq!(open(&mut h, "/tmp", O_WRONLY), u64::MAX);
    assert_eq!(errno(&h), EISDIR as u32);
    // A directory can be opened for reading.
    assert_ne!(open(&mut h, "/tmp", 0), u64::MAX);

    let append = open(&mut h, "f", O_WRONLY | O_APPEND);
    write_to(&mut h, append, b"!");
    call(&mut h, "close", &[append]);
    let read = open(&mut h, "f", 0);
    assert_eq!(read_from(&mut h, read, 32), b"abcdef!");
    // A read-only descriptor refuses writes.
    assert_eq!(write_to(&mut h, read, b"x"), u64::MAX);
    assert_eq!(errno(&h), EBADF as u32);
    call(&mut h, "close", &[read]);

    // Truncation needs write access.
    open(&mut h, "f", O_TRUNC);
    assert_eq!(size_of_path(&mut h, "f"), 7);
    let truncated = open(&mut h, "f", O_TRUNC | O_WRONLY);
    assert_eq!(size_of_path(&mut h, "f"), 0);
    // A write-only descriptor refuses reads.
    assert_eq!(read_from(&mut h, truncated, 4), b"");
    assert_eq!(errno(&h), EBADF as u32);
}

#[test]
fn stat_access_and_chmod_describe_files_and_directories() {
    let mut h = host();
    let fd = open(&mut h, "s.txt", O_CREAT | O_RDWR);
    write_to(&mut h, fd, b"12345");
    let file = cstring("s.txt");
    let (mode, size, links) = stat_of(&mut h, "stat", file.as_ptr() as u64).unwrap();
    assert_eq!((mode, size, links), (S_IFREG | 0o644, 5, 1));
    assert_eq!(stat_of(&mut h, "lstat", file.as_ptr() as u64).unwrap().1, 5);
    assert_eq!(stat_of(&mut h, "fstat", fd).unwrap().1, 5);
    let (mode, size, _) = stat_of(&mut h, "stat", cstring("/tmp").as_ptr() as u64).unwrap();
    assert_eq!((mode, size), (S_IFDIR | 0o755, 4096));
    assert_eq!(
        stat_of(&mut h, "stat", cstring("nope").as_ptr() as u64),
        Err(ENOENT as u32)
    );
    assert_eq!(stat_of(&mut h, "fstat", 99), Err(EBADF as u32));

    for symbol in ["access", "chmod"] {
        assert_eq!(call(&mut h, symbol, &[file.as_ptr() as u64, 0o600]), 0);
        let dir = cstring("/tmp");
        assert_eq!(call(&mut h, symbol, &[dir.as_ptr() as u64, 0]), 0);
        let missing = cstring("nope");
        assert_eq!(
            call(&mut h, symbol, &[missing.as_ptr() as u64, 0]),
            u64::MAX
        );
        assert_eq!(errno(&h), ENOENT as u32, "{symbol}");
    }
    assert_eq!(call(&mut h, "fchmod", &[fd, 0o600]), 0);
    assert_eq!(call(&mut h, "fchmod", &[fd + 40, 0o600]), u64::MAX);
    assert_eq!(errno(&h), EBADF as u32);
}

#[test]
fn unlink_remove_and_rename_move_files_and_keep_their_identity() {
    let mut h = host();
    let fd = open(&mut h, "old", O_CREAT | O_WRONLY);
    write_to(&mut h, fd, b"data");
    call(&mut h, "close", &[fd]);
    let (old, new) = (cstring("old"), cstring("new"));
    let before = h.fs.inode("/tmp/old");
    let stamp = h.fs.mtimes["/tmp/old"];

    let pair = [old.as_ptr() as u64, new.as_ptr() as u64];
    assert_eq!(call(&mut h, "rename", &pair), 0);
    assert_eq!(open(&mut h, "old", 0), u64::MAX);
    assert_eq!(errno(&h), ENOENT as u32);
    assert_eq!(h.fs.inode("/tmp/new"), before);
    assert_eq!(h.fs.mtimes["/tmp/new"], stamp);
    let moved = open(&mut h, "new", 0);
    assert_eq!(read_from(&mut h, moved, 8), b"data");
    // Renaming what is gone, or a directory, fails.
    assert_eq!(call(&mut h, "rename", &pair), u64::MAX);
    assert_eq!(errno(&h), ENOENT as u32);
    let tmp = cstring("/tmp");
    assert_eq!(
        call(
            &mut h,
            "rename",
            &[tmp.as_ptr() as u64, new.as_ptr() as u64]
        ),
        u64::MAX
    );
    assert_eq!(errno(&h), EINVAL as u32);

    assert_eq!(call(&mut h, "unlink", &[new.as_ptr() as u64]), 0);
    assert_eq!(call(&mut h, "unlink", &[new.as_ptr() as u64]), u64::MAX);
    assert_eq!(errno(&h), ENOENT as u32);

    // `remove` also takes empty directories; `unlink` leaves directories alone.
    h.fs.dirs.insert("/tmp/empty".to_string());
    let empty = cstring("/tmp/empty");
    assert_eq!(call(&mut h, "remove", &[empty.as_ptr() as u64]), 0);
    assert!(!h.fs.is_dir("/tmp/empty"));
    h.fs.dirs.insert("/tmp/again".to_string());
    let again = cstring("/tmp/again");
    assert_eq!(call(&mut h, "unlink", &[again.as_ptr() as u64]), u64::MAX);
    let nothing = cstring("nothing");
    assert_eq!(call(&mut h, "remove", &[nothing.as_ptr() as u64]), u64::MAX);
    assert_eq!(errno(&h), ENOENT as u32);
}

#[test]
fn hard_links_share_contents_and_count_in_stat() {
    let mut h = host();
    let fd = open(&mut h, "one", O_CREAT | O_RDWR);
    write_to(&mut h, fd, b"shared");
    let (one, two, gone) = (cstring("one"), cstring("two"), cstring("gone"));
    let link = [one.as_ptr() as u64, two.as_ptr() as u64];
    assert_eq!(call(&mut h, "link", &link), 0);
    assert_eq!(stat_of(&mut h, "stat", two.as_ptr() as u64).unwrap().2, 2);
    assert_eq!(h.fs.inode("/tmp/one"), h.fs.inode("/tmp/two"));
    // A write through one name shows in the other.
    write_to(&mut h, fd, b"+more");
    call(&mut h, "close", &[fd]);
    let via = open(&mut h, "two", 0);
    assert_eq!(read_from(&mut h, via, 32), b"shared+more");
    assert_eq!(call(&mut h, "link", &link), u64::MAX);
    assert_eq!(errno(&h), EEXIST as u32);
    let from_gone = [gone.as_ptr() as u64, two.as_ptr() as u64];
    assert_eq!(call(&mut h, "link", &from_gone), u64::MAX);
    assert_eq!(errno(&h), ENOENT as u32);
    // Dropping one name leaves the other with the contents and a single link.
    assert_eq!(call(&mut h, "unlink", &[one.as_ptr() as u64]), 0);
    assert_eq!(
        stat_of(&mut h, "stat", two.as_ptr() as u64).unwrap(),
        (S_IFREG | 0o644, 11, 1)
    );
}

#[test]
fn working_directory_follows_chdir_and_resolves_relative_paths() {
    let mut h = host();
    let mut buf = [0u8; 16];
    let at = buf.as_mut_ptr() as u64;
    assert_eq!(call(&mut h, "getcwd", &[at, 16]), at);
    assert_eq!(&buf[..5], b"/tmp\0");
    // The buffer must hold the path and its terminator.
    assert_eq!(call(&mut h, "getcwd", &[at, 4]), 0);
    assert_eq!(errno(&h), 34);

    let sub = cstring("sub");
    assert_eq!(call(&mut h, "mkdir", &[sub.as_ptr() as u64, 0o755]), 0);
    assert_eq!(call(&mut h, "chdir", &[sub.as_ptr() as u64]), 0);
    call(&mut h, "getcwd", &[at, 16]);
    assert_eq!(&buf[..9], b"/tmp/sub\0");
    assert_ne!(open(&mut h, "inside", O_CREAT | O_WRONLY), u64::MAX);
    assert!(h.fs.exists_file("/tmp/sub/inside"));
    // A file is no directory.
    let inside = cstring("inside");
    assert_eq!(call(&mut h, "chdir", &[inside.as_ptr() as u64]), u64::MAX);
    assert_eq!(errno(&h), ENOENT as u32);
    let up = cstring("..");
    assert_eq!(call(&mut h, "chdir", &[up.as_ptr() as u64]), 0);
    assert_eq!(h.cwd, "/tmp");

    h.cwd.clear();
    call(&mut h, "getcwd", &[at, 16]);
    assert_eq!(&buf[..2], b"/\0");
}

#[test]
fn realpath_fills_the_given_buffer_or_allocates_one() {
    let mut h = host();
    open(&mut h, "real.txt", O_CREAT | O_WRONLY);
    let name = cstring("./real.txt");
    let mut out = [0u8; 32];
    for symbol in ["realpath", "realpath$DARWIN_EXTSN"] {
        let args = [name.as_ptr() as u64, out.as_mut_ptr() as u64];
        assert_eq!(call(&mut h, symbol, &args), out.as_ptr() as u64, "{symbol}");
        assert_eq!(&out[..14], b"/tmp/real.txt\0", "{symbol}");
    }
    let fresh = call(&mut h, "realpath", &[name.as_ptr() as u64, 0]);
    assert_ne!(fresh, 0);
    assert_eq!(read_bytes(fresh, 14), b"/tmp/real.txt\0");
    let dir = cstring("/tmp");
    assert_ne!(call(&mut h, "realpath", &[dir.as_ptr() as u64, 0]), 0);
    let nothing = cstring("nothing");
    let missing = call(&mut h, "realpath", &[nothing.as_ptr() as u64, 0]);
    assert_eq!((missing, errno(&h)), (0, ENOENT as u32));
}

#[test]
fn mkdir_and_rmdir_enforce_the_directory_rules() {
    let mut h = host();
    let (a, ab, file) = (cstring("a"), cstring("a/b"), cstring("file"));
    assert_eq!(
        call(&mut h, "mkdir", &[ab.as_ptr() as u64, 0o755]),
        u64::MAX
    );
    assert_eq!(errno(&h), ENOENT as u32);
    assert_eq!(call(&mut h, "mkdir", &[a.as_ptr() as u64, 0o755]), 0);
    assert_eq!(call(&mut h, "mkdir", &[a.as_ptr() as u64, 0o755]), u64::MAX);
    assert_eq!(errno(&h), EEXIST as u32);
    open(&mut h, "file", O_CREAT | O_WRONLY);
    assert_eq!(call(&mut h, "mkdir", &[file.as_ptr() as u64, 0]), u64::MAX);
    assert_eq!(errno(&h), EEXIST as u32);

    assert_eq!(call(&mut h, "mkdir", &[ab.as_ptr() as u64, 0o755]), 0);
    assert_eq!(call(&mut h, "rmdir", &[a.as_ptr() as u64]), u64::MAX);
    assert_eq!(errno(&h), ENOTEMPTY as u32);
    assert_eq!(call(&mut h, "rmdir", &[file.as_ptr() as u64]), u64::MAX);
    assert_eq!(errno(&h), ENOTDIR as u32);
    let absent = cstring("zzz");
    assert_eq!(call(&mut h, "rmdir", &[absent.as_ptr() as u64]), u64::MAX);
    assert_eq!(errno(&h), ENOENT as u32);
    assert_eq!(call(&mut h, "rmdir", &[ab.as_ptr() as u64]), 0);
    assert_eq!(call(&mut h, "rmdir", &[a.as_ptr() as u64]), 0);
    assert!(!h.fs.is_dir("/tmp/a"));
    // The name can be made again after it was removed.
    assert_eq!(call(&mut h, "mkdir", &[a.as_ptr() as u64, 0o755]), 0);
}

#[test]
fn directory_streams_list_dot_entries_then_the_contents() {
    let mut h = host();
    let (d, inner) = (cstring("d"), cstring("d/inner"));
    call(&mut h, "mkdir", &[d.as_ptr() as u64, 0o755]);
    call(&mut h, "mkdir", &[inner.as_ptr() as u64, 0o755]);
    open(&mut h, "d/file.txt", O_CREAT | O_WRONLY);

    let stream = call(&mut h, "opendir", &[d.as_ptr() as u64]);
    assert_ne!(stream, 0);
    let mut seen = Vec::new();
    for symbol in ["readdir", "readdir64"].into_iter().cycle() {
        let entry = call(&mut h, symbol, &[stream]);
        if entry == 0 {
            break;
        }
        seen.push((cstr(entry + 19), read_u8(entry + 18)));
    }
    let expected = [(".", 4), ("..", 4), ("file.txt", 8), ("inner", 4)];
    assert_eq!(seen, expected.map(|(name, kind)| (name.to_string(), kind)));
    // The stream stays at its end until it is closed.
    assert_eq!(call(&mut h, "readdir", &[stream]), 0);
    assert_eq!(call(&mut h, "closedir", &[stream]), 0);
    assert_eq!(call(&mut h, "closedir", &[stream]), u64::MAX);
    assert_eq!(errno(&h), EBADF as u32);
    assert_eq!(call(&mut h, "readdir", &[stream]), 0);

    let file = cstring("d/file.txt");
    let opened = call(&mut h, "opendir", &[file.as_ptr() as u64]);
    assert_eq!((opened, errno(&h)), (0, ENOTDIR as u32));
    let nowhere = cstring("nowhere");
    let opened = call(&mut h, "opendir", &[nowhere.as_ptr() as u64]);
    assert_eq!((opened, errno(&h)), (0, ENOENT as u32));
}

fn stdin_stream(h: &mut SandboxHost) -> u64 {
    read_u64(h.foreign_data("stdin").unwrap())
}

#[test]
fn stdin_reads_lines_and_characters_until_the_end() {
    let mut h = host();
    h.set_stdin(b"first\nsecond\nthird line\nxyz");
    let stream = stdin_stream(&mut h);
    // `__stdinp` is the same stream, and other variables are not served.
    assert_eq!(read_u64(h.foreign_data("__stdinp").unwrap()), stream);
    assert_eq!(h.foreign_data("stdout"), None);

    let mut line = [0u8; 32];
    let at = line.as_mut_ptr() as u64;
    assert_eq!(call(&mut h, "fgets", &[at, 32, stream]), at);
    assert_eq!(&line[..7], b"first\n\0");
    // A short buffer takes part of a line.
    call(&mut h, "fgets", &[at, 4, stream]);
    assert_eq!(&line[..4], b"sec\0");
    assert_eq!(call(&mut h, "fgets", &[at, 1, stream]), 0);

    let (mut text, mut capacity) = (0u64, 0u64);
    let slots = [&raw mut text as u64, &raw mut capacity as u64];
    assert_eq!(call(&mut h, "getline", &[slots[0], slots[1], stream]), 4);
    assert_eq!(read_bytes(text, 5), b"ond\n\0");
    assert!(capacity >= 120);
    // The buffer is reused when it is big enough.
    let first_buffer = text;
    let n = call(
        &mut h,
        "getdelim",
        &[slots[0], slots[1], b' ' as u64, stream],
    );
    assert_eq!((n, text), (6, first_buffer));
    assert_eq!(read_bytes(text, 7), b"third \0");

    assert_eq!(call(&mut h, "getc", &[stream]), b'l' as u64);
    assert_eq!(call(&mut h, "ungetc", &[b'L' as u64, stream]), b'L' as u64);
    assert_eq!(call(&mut h, "fgetc", &[stream]), b'L' as u64);
    assert_eq!(call(&mut h, "getc_unlocked", &[stream]), b'i' as u64);
    assert_eq!(call(&mut h, "fgetc_unlocked", &[stream]), b'n' as u64);
    assert_eq!(call(&mut h, "getchar", &[]), b'e' as u64);
    assert_eq!(call(&mut h, "getchar_unlocked", &[]), b'\n' as u64);
    assert_eq!(call(&mut h, "ungetc", &[(-1i64) as u64, stream]), u64::MAX);
    assert_eq!(call(&mut h, "feof", &[stream]), 0);

    let mut rest = [0u8; 8];
    let into = rest.as_mut_ptr() as u64;
    assert_eq!(call(&mut h, "fread", &[into, 1, 8, stream]), 3);
    assert_eq!(&rest[..3], b"xyz");
    assert_eq!(call(&mut h, "feof", &[stream]), 1);
    assert_eq!(call(&mut h, "fread", &[into, 0, 8, stream]), 0);
    assert_eq!(call(&mut h, "fgetc", &[stream]), u64::MAX);
    assert_eq!(call(&mut h, "getchar", &[]), u64::MAX);
    assert_eq!(call(&mut h, "fgets", &[at, 32, stream]), 0);
    assert_eq!(
        call(&mut h, "getline", &[slots[0], slots[1], stream]),
        u64::MAX
    );
}

#[test]
fn stdin_getline_grows_a_small_buffer_and_read_takes_descriptor_zero() {
    let mut h = host();
    h.set_stdin(&[b'a'; 200]);
    h.feed_stdin(b"\nrest");
    let stream = stdin_stream(&mut h);
    let small = h.alloc(4);
    let (mut text, mut capacity) = (small, 4u64);
    let slots = [&raw mut text as u64, &raw mut capacity as u64];
    assert_eq!(call(&mut h, "getline", &[slots[0], slots[1], stream]), 201);
    assert_ne!(text, small);
    assert!(capacity >= 202);
    assert!(!h.allocations.contains_key(&small));
    assert_eq!(read_bytes(text + 200, 2), b"\n\0");

    let mut buf = [0u8; 16];
    let at = buf.as_mut_ptr() as u64;
    assert_eq!(call(&mut h, "read", &[0, at, 3]), 3);
    assert_eq!(&buf[..3], b"res");
    assert_eq!(call(&mut h, "read", &[0, at, 16]), 1);
    assert_eq!(call(&mut h, "read", &[0, at, 16]), 0);
    // A line call on a stream that is not standard input is left to others.
    assert!(h.stdin_foreign(LibcFn::Getline, &[0, 0, 0]).is_none());
}

#[test]
fn stdin_wants_data_only_while_a_read_would_find_too_little() {
    let mut h = host();
    let stream = stdin_stream(&mut h);
    let line = [0, 32, stream];
    assert!(h.stdin_wants_data("read", &[0, 0, 8]));
    assert!(!h.stdin_wants_data("read", &[0, 0, 0]));
    assert!(!h.stdin_wants_data("read", &[5, 0, 8]));
    assert!(h.stdin_wants_data("fread", &[0, 2, 4, stream]));
    assert!(!h.stdin_wants_data("fread", &[0, 0, 4, stream]));
    assert!(!h.stdin_wants_data("fread", &[0, 1, 4, stream + 8]));
    assert!(h.stdin_wants_data("fgets", &line));
    assert!(!h.stdin_wants_data("fgets", &[0, 1, stream]));
    assert!(h.stdin_wants_data("getline", &[0, 0, stream]));
    assert!(h.stdin_wants_data("getdelim", &[0, 0, b';' as u64, stream]));
    assert!(h.stdin_wants_data("fgetc", &[stream]));
    assert!(h.stdin_wants_data("getc_unlocked", &[stream]));
    assert!(h.stdin_wants_data("getchar", &[]));
    assert!(!h.stdin_wants_data("malloc", &[8]));
    assert!(!h.stdin_wants_data("no_such_function", &[]));

    // Enough buffered input, or a delimiter in it, satisfies the call.
    h.feed_stdin(b"ab;");
    assert!(!h.stdin_wants_data("read", &[0, 0, 8]));
    assert!(h.stdin_wants_data("fread", &[0, 1, 4, stream]));
    assert!(!h.stdin_wants_data("fread", &[0, 1, 3, stream]));
    assert!(h.stdin_wants_data("fgets", &line));
    assert!(!h.stdin_wants_data("getdelim", &[0, 0, b';' as u64, stream]));
    assert!(h.stdin_wants_data("getline", &[0, 0, stream]));
    h.feed_stdin(b"\n");
    assert!(!h.stdin_wants_data("fgets", &line));
    assert!(!h.stdin_wants_data("getline", &[0, 0, stream]));

    // Once the input is over, nothing waits.
    let mut ended = host();
    ended.end_stdin();
    assert!(!ended.stdin_wants_data("getchar", &[]));
    let mut closed = host();
    closed.close_stdin();
    assert!(!closed.stdin_wants_data("getchar", &[]));
}

#[test]
fn stdin_end_marker_is_consumed_by_the_read_that_finds_nothing() {
    let mut h = host();
    let stream = stdin_stream(&mut h);
    h.feed_stdin(b"x");
    h.end_stdin();
    assert_eq!(call(&mut h, "getc", &[stream]), b'x' as u64);
    assert_eq!(call(&mut h, "getc", &[stream]), u64::MAX);
    assert_eq!(call(&mut h, "feof", &[stream]), 1);
    // The end was reported once; later input can still arrive.
    assert!(h.stdin_wants_data("getchar", &[]));
    h.feed_stdin(b"y");
    assert_eq!(call(&mut h, "getchar", &[]), b'y' as u64);
    assert_eq!(call(&mut h, "feof", &[stream]), 0);
}
