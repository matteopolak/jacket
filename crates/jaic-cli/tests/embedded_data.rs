//! Byte data in globals (string literals, constant tables, zero-padded arrays and pointers between
//! them) reaches the executable intact at every optimization level.
use std::process::Command;
use test_support::TempDir;

const JAIC: &str = env!("CARGO_BIN_EXE_jaic");

/// A `jaic` command. `JACKET_TEST_CODEGEN_UNITS=N` (set by the CI sweep that divides every native
/// build into parts, docs/native/llvm-backend.md) starts it with `-codegen_units N`; a test that
/// pins its own count passes the flag again later, and the last one wins.
fn jaic() -> Command {
    let mut command = Command::new(JAIC);
    if let Some(units) = std::env::var_os("JACKET_TEST_CODEGEN_UNITS") {
        command.arg("-codegen_units").arg(units);
    }
    command
}

const SOURCE: &str = r#"#import "Basic";

Table :: struct {
    head: [4] u8;
    name: *u8;
    tail: [6] u8;
    wide: [3] u16;
}

bytes := u8.[0, 1, 2, 254, 255, 0, 0, 128];
zeros: [64] u8;
partial := u8.[0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 0];
text := "a\0b\xffc";
table := Table.{ .[1, 2, 3, 4], "name".data, .[0, 0, 7, 0, 0, 0], .[1, 65535, 3] };
words : [5] s32 = .[-1, 0, 2, 0, 100000];

main :: () {
    sum := 0;
    for bytes sum = sum * 31 + it;
    print("bytes %\n", sum);
    zero_sum := 0;
    for zeros zero_sum += it;
    print("zeros % %\n", zeros.count, zero_sum);
    print("partial % %\n", partial[7], partial[11]);
    print("text % % % % % %\n", text.count, text[0], text[1], text[2], text[3], text[4]);
    print("table % % % %\n", table.head[3], table.name[2], table.tail[2], table.wide[1]);
    print("words % % %\n", words[0], words[2], words[4]);
}
"#;

fn build_and_run(level: &str) -> String {
    let dir = TempDir::new(&format!("embedded-data-{level}"));
    std::fs::write(dir.join("main.jai"), SOURCE).unwrap();
    let exe = dir.join(if cfg!(windows) {
        "out.exe"
    } else {
        "out"
    });
    let build = jaic()
        .args(["main.jai", level, "-exe"])
        .arg(&exe)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let run = Command::new(&exe).output().unwrap();
    assert!(run.status.success());
    String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n")
}

#[test]
fn global_byte_data_survives_unoptimized_and_optimized_builds() {
    let mut sum: i64 = 0;
    for b in [0, 1, 2, 254, 255, 0, 0, 128] {
        sum = sum * 31 + b;
    }
    let expected = format!(
        "bytes {sum}\nzeros 64 0\npartial 9 0\ntext 5 97 0 98 255 99\n\
         table 4 109 7 65535\nwords -1 2 100000\n"
    );
    assert_eq!(build_and_run("-O0"), expected);
    assert_eq!(build_and_run("-O2"), expected);
}
