//! What a program's name says about how to call it.
use std::path::Path;

/// The file stem of `program`, lower case, without `.exe`.
fn stem(program: &str) -> String {
    let name = program
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(program)
        .to_ascii_lowercase();
    name.strip_suffix(".exe").unwrap_or(&name).to_string()
}

/// Whether `program` is a Clang driver: `clang`, `clang++`, `clang-cl`, `clang-23`, or a
/// prefixed one such as `x86_64-w64-mingw32-clang`.
pub fn is_clang(program: &str) -> bool {
    let name = stem(program);
    matches!(name.as_str(), "clang" | "clang++" | "clang-cl")
        || name.ends_with("-clang")
        || name.ends_with("-clang++")
        || name
            .strip_prefix("clang-")
            .is_some_and(|suffix| !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()))
}

/// Whether `program` takes `link.exe`-style arguments (`link`, `lld-link`) rather than a C
/// driver's.
pub fn is_msvc_linker(program: &str) -> bool {
    matches!(stem(program).as_str(), "link" | "lld-link")
}

/// Whether `program` is an archiver that takes `lib.exe` arguments.
pub fn is_lib_tool(program: &Path) -> bool {
    matches!(
        stem(&program.to_string_lossy()).as_str(),
        "lib" | "llvm-lib"
    )
}
