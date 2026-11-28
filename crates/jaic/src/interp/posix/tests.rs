//! The ABI tables against the stdlib's generated POSIX declarations, which the compiler
//! evaluates for each target.
use std::fmt::Write;

use super::{ERRORS, Open, PosixAbi, StatSyscall};
use crate::sema::{Compiler, NativeFs, Options, ProgramSource};
use crate::target::{TargetCpu, TargetOs};

/// Every POSIX target the stdlib declares, with the ABI the tables give it.
fn targets() -> Vec<(TargetOs, TargetCpu)> {
    let mut targets = vec![
        (TargetOs::MacOS, TargetCpu::X64),
        (TargetOs::MacOS, TargetCpu::Arm64),
    ];

    for cpu in [
        TargetCpu::X64,
        TargetCpu::Arm64,
        TargetCpu::X86,
        TargetCpu::Arm,
        TargetCpu::S390x,
        TargetCpu::PowerPc64,
    ] {
        targets.push((TargetOs::Linux, cpu));
    }

    targets
}

/// `#assert`s that the declarations of `os` on `cpu` agree with the tables.
fn assertions(abi: PosixAbi) -> String {
    let mut text = String::from("#import \"POSIX\";\n");
    let mut check = |name: &str, value: i64| {
        writeln!(text, "#assert {name} == {value} \"{name}\";").unwrap();
    };

    for &(name, linux, _) in ERRORS {
        check(name, abi.errno(linux) as i64);
    }

    // Read access is no bit (`O_RDONLY` is 0), so each flag is checked with it.
    let read = Open {
        read: true,
        ..Open::default()
    };
    let flags = [
        (
            "O_WRONLY",
            Open {
                write: true,
                ..Open::default()
            },
        ),
        (
            "O_RDWR",
            Open {
                write: true,
                ..read
            },
        ),
        (
            "O_CREAT",
            Open {
                create: true,
                ..read
            },
        ),
        (
            "O_EXCL",
            Open {
                exclusive: true,
                ..read
            },
        ),
        (
            "O_TRUNC",
            Open {
                truncate: true,
                ..read
            },
        ),
        (
            "O_APPEND",
            Open {
                append: true,
                ..read
            },
        ),
        (
            "O_NONBLOCK",
            Open {
                nonblock: true,
                ..read
            },
        ),
        (
            "O_DIRECTORY",
            Open {
                directory: true,
                ..read
            },
        ),
        (
            "O_NOFOLLOW",
            Open {
                no_follow: true,
                ..read
            },
        ),
        (
            "O_CLOEXEC",
            Open {
                close_on_exec: true,
                ..read
            },
        ),
        (
            "O_NOCTTY",
            Open {
                no_tty: true,
                ..read
            },
        ),
        (
            "O_SYNC",
            Open {
                sync: true,
                ..read
            },
        ),
    ];

    for (name, open) in flags {
        check(name, abi.encode_open(open) as i64);
        assert_eq!(
            abi.decode_open(abi.encode_open(open)),
            (open, 0),
            "{abi:?} {name}"
        );
    }

    check("AT_FDCWD", abi.at_working_directory());
    check("AT_SYMLINK_NOFOLLOW", abi.at_no_follow() as i64);

    if !abi.darwin && matches!(abi.cpu, TargetCpu::X64 | TargetCpu::Arm64) {
        for number in 0..500 {
            let name = match abi.stat_syscall(number) {
                Some(StatSyscall::Stat) => "SYS_stat",
                Some(StatSyscall::Lstat) => "SYS_lstat",
                Some(StatSyscall::Fstat) => "SYS_fstat",
                Some(StatSyscall::Fstatat) => "SYS_newfstatat",
                None => continue,
            };

            check(name, number as i64);
        }
    }

    text.push_str("main :: () {}\n");
    text
}

#[test]
fn the_tables_agree_with_the_stdlib_declarations() {
    let stdlib = test_support::repo::stdlib();

    for (os, cpu) in targets() {
        let abi = PosixAbi::new(os == TargetOs::MacOS, Some(cpu));
        let mut options = Options::host();

        crate::target::select_os_cpu(&mut options, Some(os), Some(cpu));
        options.import_paths = vec![stdlib.clone()];
        options.set_stdlib(&NativeFs, &stdlib);
        let mut compiler = Compiler::new(options, std::rc::Rc::new(NativeFs));
        let source = assertions(abi);

        if let Err(error) = compiler.compile_sources(&[ProgramSource::String(source)]) {
            panic!("{os:?} {cpu:?}: {}", error.render(&compiler.src.sources));
        }
    }
}

#[test]
fn errors_one_os_lacks_become_eio_and_the_rest_round_trip() {
    let darwin = PosixAbi::new(true, None);

    for &(name, linux, number) in ERRORS {
        assert_eq!(darwin.errno(linux), number, "{name}");
        assert_eq!(darwin.linux_errno(number), linux, "{name}");
    }

    // Linux's EHWPOISON and Darwin's EAUTH have no counterpart.
    assert_eq!(darwin.errno(133), 5);
    assert_eq!(darwin.linux_errno(80), 5);
}

#[test]
fn open_flags_no_open_names_are_left_over() {
    let linux = PosixAbi::new(false, Some(TargetCpu::X64));
    // O_CREAT | O_WRONLY | O_LARGEFILE (none on x86-64) | O_PATH.
    let (open, rest) = linux.decode_open(0x40 | 1 | 0x0020_0000);

    assert!(open.create && open.write && !open.read);
    assert_eq!(rest, 0x0020_0000);
    // A 32-bit `open64` sets O_LARGEFILE, which needs nothing on the host.
    let i686 = PosixAbi::new(false, Some(TargetCpu::X86));

    assert_eq!(i686.decode_open(0x8000 | 2).1, 0);
}
