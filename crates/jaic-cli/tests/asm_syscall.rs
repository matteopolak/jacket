//! `#asm` `syscall`: type-checking for Linux x86-64, the sandbox refusing it (memory mapping
//! aside), and real calls on a Linux x86-64 host.
use std::path::Path;
use std::process::Command;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

/// A Linux futex wrapper written with `#asm` `syscall` type-checks for x86-64 Linux, and the
/// sandbox (`-os wasm`) never makes the call.
#[test]
fn asm_syscall_checks_and_sandbox_refuses_it() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-asm-syscall");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("futex.jai");
    std::fs::write(
        &source,
        r#"#import "Basic";
SYS_futex :: 202;
futex :: (futex_lock: *s32, op: s64, val: s64) -> s64 {
    result: s64 = ---;
    #asm SYSCALL_SYSRET {
        mov.q call:     gpr === a, SYS_futex;
        mov.q uaddr:    gpr === di, futex_lock;
        mov.q futex_op: gpr === si, op;
        mov.q value:    gpr === d, val;
        mov.q timeout:  gpr === 10, 0;
        syscall t1:, t2:, call;
        mov result, call;
    }
    return result;
}
main :: () {
    word: s32 = 1;
    // Wait on a word that differs from the expected value: returns -EAGAIN without blocking.
    r := futex(*word, 128, 0);
    print("%\n", r);
}
"#,
    )
    .unwrap();
    let output = Command::new(JAIC)
        .arg("-check")
        .arg(&source)
        .args(["-target", "x86_64-linux-gnu"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    // The sandbox refuses the call with -EPERM instead of running it.
    let output = Command::new(JAIC)
        .arg("-interp")
        .arg(&source)
        .args(["-os", "wasm"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "-1\n");
}

/// On a Linux x86-64 host `syscall` really calls the kernel: `getpid` (39) is the process id,
/// both in the interpreter and in compiled code.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn asm_syscall_getpid_matches_the_process_id() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-asm-getpid");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("getpid.jai");
    std::fs::write(
        &source,
        r#"#import "Basic";
libc :: #system_library "libc";
getpid :: () -> s32 #foreign libc;
main :: () {
    pid: s64 = ---;
    #asm SYSCALL_SYSRET {
        mov.q call: gpr === a, 39;
        syscall t1:, t2:, call;
        mov pid, call;
    }
    assert(pid == getpid(), "getpid via syscall");
    print("%\n", pid == getpid());
}
"#,
    )
    .unwrap();
    let output = Command::new(JAIC)
        .arg("-interp")
        .arg(&source)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "true\n");

    let exe = dir.join("getpid");
    let build = Command::new(JAIC)
        .arg(&source)
        .arg("-exe")
        .arg(&exe)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let output = Command::new(&exe).output().unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "true\n");
}

/// A `write` wrapper that lists its argument registers after rax, out of kernel order.
const LISTED_SYSCALL: &str = r#"#import "Basic";
write_raw :: (fd: s64, data: *u8, count: s64) -> s64 {
    result: s64 = ---;
    #asm SYSCALL_SYSRET {
        clobber_c:  gpr === c;
        clobber_11: gpr === 11;
        mov.q number: gpr === a, 1;
        mov.q third:  gpr === d, count;
        mov.q second: gpr === si, data;
        mov.q first:  gpr === di, fd;
        mov.q unused: gpr === 10, 0;
        syscall clobber_c, clobber_11, number, third, unused, first, second;
        mov result, number;
    }
    return result;
}
main :: () {
    text := "listed\n";
    print("%\n", write_raw(1, text.data, text.count));
}
"#;

/// `syscall` accepts its argument registers listed as operands after rax, in any order; an
/// operand that is not pinned to an argument register is an error.
#[test]
fn asm_syscall_accepts_listed_argument_registers() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-asm-syscall-listed");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("listed.jai");
    std::fs::write(&source, LISTED_SYSCALL).unwrap();
    let output = Command::new(JAIC)
        .arg("-check")
        .arg(&source)
        .args(["-target", "x86_64-linux-gnu"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let bad = dir.join("unpinned.jai");
    std::fs::write(
        &bad,
        LISTED_SYSCALL.replace(
            "syscall clobber_c, clobber_11, number, third,",
            "syscall clobber_c, clobber_11, number, result,",
        ),
    )
    .unwrap();
    let output = Command::new(JAIC)
        .arg("-check")
        .arg(&bad)
        .args(["-target", "x86_64-linux-gnu"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("must be a register pinned to di, si, d, 10, 8 or 9"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// On a Linux x86-64 host the listed argument registers reach the kernel in the right slots.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn asm_syscall_listed_arguments_reach_the_kernel() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-asm-syscall-listed-run");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("listed.jai");
    std::fs::write(&source, LISTED_SYSCALL).unwrap();
    let output = Command::new(JAIC)
        .arg("-interp")
        .arg(&source)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "listed\n7\n");
}

/// On a Linux x86-64 host the sandbox (`-os wasm`) makes anonymous `mmap` and `munmap`, so
/// allocators built on raw system calls work, and still refuses calls with outside effects.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn asm_syscall_sandbox_maps_anonymous_memory() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-asm-syscall-sandbox-mmap");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("mmap.jai");
    std::fs::write(
        &source,
        r#"#import "Basic";
raw :: (number: s64, a1: s64, a2: s64, a3: s64, a4: s64, a5: s64, a6: s64) -> s64 {
    result: s64 = ---;
    #asm SYSCALL_SYSRET {
        mov.q call: gpr === a, number;
        mov.q r1:   gpr === di, a1;
        mov.q r2:   gpr === si, a2;
        mov.q r3:   gpr === d, a3;
        mov.q r4:   gpr === 10, a4;
        mov.q r5:   gpr === 8, a5;
        mov.q r6:   gpr === 9, a6;
        syscall t1:, t2:, call, r1, r2, r3, r4, r5, r6;
        mov result, call;
    }
    return result;
}
main :: () {
    // mmap(null, 4096, READ|WRITE, PRIVATE|ANONYMOUS, -1, 0)
    address := raw(9, 0, 4096, 3, 0x22, -1, 0);
    page := cast(*s64) address;
    page.* = 42;
    value := page.*;
    print("% %\n", value, raw(11, address, 4096, 0, 0, 0, 0));
    // getpid is not a memory call, so the sandbox refuses it.
    print("%\n", raw(39, 0, 0, 0, 0, 0, 0));
}
"#,
    )
    .unwrap();
    let output = Command::new(JAIC)
        .arg("-interp")
        .arg(&source)
        .args(["-os", "wasm"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "42 0\n-1\n");
}
