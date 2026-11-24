use super::*;
use crate::testing::FakeHost;

fn target(triple: &str) -> Target {
    Target::parse(triple).unwrap()
}

fn find(host: &FakeHost, triple: &str) -> Result<Found, Missing> {
    find_linker(
        &Lookup::new(host),
        Some(target(triple)),
        &Overrides::default(),
    )
}

fn args(found: &Found) -> Vec<String> {
    found
        .args
        .iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect()
}

const ARMHF: &str = "armv7-unknown-linux-gnueabihf";

#[test]
fn the_host_links_with_its_c_driver() {
    let host = FakeHost::mac();
    let found = find(&host, "aarch64-apple-darwin").unwrap();
    assert_eq!(found.program, PathBuf::from("/usr/bin/cc"));
    assert!(found.args.is_empty());
    // The other architecture of a Mac is the same driver told the target.
    let found = find(&host, "x86_64-apple-darwin").unwrap();
    assert_eq!(args(&found), ["--target=x86_64-apple-darwin"]);
    let host = FakeHost::debian();
    assert_eq!(
        find(&host, "x86_64-unknown-linux-gnu").unwrap().program,
        PathBuf::from("/usr/bin/cc")
    );
    let found = find(&host, "i686-unknown-linux-gnu").unwrap();
    assert_eq!(args(&found), ["-m32"]);
}

#[test]
fn a_missing_c_driver_says_how_to_get_one() {
    let mut host = FakeHost::mac();
    host.without("/usr/bin/cc");
    let error = find(&host, "aarch64-apple-darwin").unwrap_err();
    assert_eq!(error.what, "no linker for aarch64-apple-darwin");
    assert_eq!(
        error.notes,
        ["install the Xcode command line tools with `xcode-select --install`"]
    );
    let mut host = FakeHost::debian();
    host.without("/usr/bin/cc");
    // A C driver that cannot link is the host's own: `cc` is not looked for elsewhere.
    let error = find(&host, "x86_64-unknown-linux-gnu").unwrap_err();
    assert!(
        error.notes[0].contains("apt install gcc-x86-64-linux-gnu"),
        "{error}"
    );
}

#[test]
fn a_gnu_cross_driver_links_a_linux_target_on_linux() {
    let mut host = FakeHost::debian();
    host.file("/usr/bin/arm-linux-gnueabihf-gcc");
    let found = find(&host, ARMHF).unwrap();
    assert_eq!(
        found.program,
        PathBuf::from("/usr/bin/arm-linux-gnueabihf-gcc")
    );
    assert!(found.args.is_empty());
}

#[test]
fn a_debian_machine_is_told_the_cross_package() {
    let host = FakeHost::debian();
    let error = find(&host, ARMHF).unwrap_err();
    assert_eq!(error.what, format!("no linker for {ARMHF}"));
    assert_eq!(
        error.to_string(),
        format!(
            "no linker for {ARMHF}\nnote: install a Linux linker and C library with `apt install gcc-arm-linux-gnueabihf`"
        )
    );
}

#[test]
fn a_mac_cross_links_with_lld_against_a_fetched_sysroot() {
    let mut host = FakeHost::mac();
    host.file("/opt/homebrew/bin/ld.lld");
    host.file("/cache/mac/jacket/toolchains/armv7-unknown-linux-gnueabihf/sysroot/usr/lib/libc.so");
    let found = find(&host, ARMHF).unwrap();
    assert_eq!(found.program, PathBuf::from("/usr/bin/cc"));
    assert_eq!(
        args(&found),
        [
            "--target=armv7-unknown-linux-gnueabihf",
            "--sysroot=/cache/mac/jacket/toolchains/armv7-unknown-linux-gnueabihf/sysroot",
            "--ld-path=/opt/homebrew/bin/ld.lld",
            "-fuse-ld=lld",
        ]
    );
}

#[test]
fn a_mac_without_lld_or_a_sysroot_gets_both_notes() {
    let mut host = FakeHost::mac();
    host.file("/opt/homebrew/bin/brew");
    let error = find(&host, ARMHF).unwrap_err();
    assert_eq!(error.what, format!("no linker for {ARMHF}"));
    assert_eq!(
        error.notes,
        [
            "install LLD with `brew install lld`".to_string(),
            format!("run `jacket toolchain add {ARMHF}` to fetch its C library"),
        ]
    );
    // With LLD but no sysroot only the library is missing.
    host.file("/opt/homebrew/bin/ld.lld");
    let error = find(&host, ARMHF).unwrap_err();
    assert_eq!(error.what, format!("no Linux C library for {ARMHF}"));
    assert_eq!(error.notes.len(), 1);
}

#[test]
fn a_sysroot_override_wins_over_the_cache() {
    let mut host = FakeHost::mac();
    host.file("/opt/homebrew/bin/ld.lld");
    host.file("/cache/mac/jacket/toolchains/armv7-unknown-linux-gnueabihf/sysroot/usr/lib/libc.so");
    host.dir("/mine");
    let overrides = Overrides {
        sysroot: Some(Given {
            value: "/mine".into(),
            from: "-sysroot",
        }),
        ..Overrides::default()
    };
    let found = find_linker(&Lookup::new(&host), Some(target(ARMHF)), &overrides).unwrap();
    assert!(args(&found).contains(&"--sysroot=/mine".to_string()));
    let sysroot = find_sysroot(
        &Lookup::new(&host),
        target(ARMHF),
        &overrides,
        SysrootUse::Run,
    )
    .unwrap()
    .unwrap();
    assert_eq!(sysroot.origin, Origin::Override("-sysroot"));
}

#[test]
fn qemu_runs_against_the_systems_cross_libraries_first() {
    let mut host = FakeHost::debian();
    host.dir("/usr/arm-linux-gnueabihf/lib");
    let found = find_sysroot(
        &Lookup::new(&host),
        target(ARMHF),
        &Overrides::default(),
        SysrootUse::Run,
    )
    .unwrap()
    .unwrap();
    assert_eq!(found.program, PathBuf::from("/usr/arm-linux-gnueabihf"));
    // A link needs a sysroot clang can use, which Debian's cross packages are not.
    assert!(
        find_sysroot(
            &Lookup::new(&host),
            target(ARMHF),
            &Overrides::default(),
            SysrootUse::Link
        )
        .unwrap()
        .is_none()
    );
}

#[test]
fn an_override_replaces_the_lookup_and_is_labelled() {
    let host = FakeHost::debian();
    let overrides = Overrides {
        linker: Some(Given {
            value: "/opt/gcc/bin/my-gcc".into(),
            from: "JAIC_LINKER",
        }),
        ..Overrides::default()
    };
    let found = find_linker(&Lookup::new(&host), Some(target(ARMHF)), &overrides).unwrap();
    assert_eq!(found.program, PathBuf::from("/opt/gcc/bin/my-gcc"));
    assert_eq!(
        found.origin.label().as_deref(),
        Some("override: JAIC_LINKER")
    );
}

#[test]
fn wasm_and_bare_metal_need_lld() {
    let mut host = FakeHost::mac();
    host.file("/opt/homebrew/bin/brew");
    let error = find(&host, "wasm32-unknown-wasi").unwrap_err();
    assert_eq!(
        error.to_string(),
        "no linker for wasm32-unknown-wasi\nnote: install LLD with `brew install lld`"
    );
    let error = find(&host, "thumbv7em-none-eabihf").unwrap_err();
    assert_eq!(error.what, "no linker for thumbv7em-none-eabihf");
    host.file("/opt/homebrew/bin/wasm-ld");
    host.file("/opt/homebrew/bin/ld.lld");
    host.path.push("/opt/homebrew/bin".into());
    assert_eq!(
        find(&host, "wasm32-unknown-wasi").unwrap().program,
        PathBuf::from("/opt/homebrew/bin/wasm-ld")
    );
    assert_eq!(
        find(&host, "aarch64-none-elf").unwrap().program,
        PathBuf::from("/opt/homebrew/bin/ld.lld")
    );
}

#[test]
fn mingw_is_found_by_the_triple_and_arm64_uses_clang() {
    let mut host = FakeHost::mac();
    host.file("/opt/homebrew/bin/x86_64-w64-mingw32-gcc");
    let found = find(&host, "x86_64-pc-windows-gnu").unwrap();
    assert!(found.args.is_empty());
    host.file("/opt/homebrew/bin/aarch64-w64-mingw32-clang");
    let found = find(&host, "aarch64-pc-windows-gnu").unwrap();
    assert_eq!(args(&found), ["--target=aarch64-pc-windows-gnu"]);
}

#[test]
fn mingw_missing_names_the_package_for_each_system() {
    let mut host = FakeHost::mac();
    host.file("/opt/homebrew/bin/brew");
    let error = find(&host, "x86_64-pc-windows-gnu").unwrap_err();
    assert_eq!(
        error.notes,
        ["install MinGW-w64 with `brew install mingw-w64`"]
    );
    let host = FakeHost::debian();
    let error = find(&host, "x86_64-pc-windows-gnu").unwrap_err();
    assert_eq!(
        error.notes,
        ["install MinGW-w64 with `apt install gcc-mingw-w64`"]
    );
    let host = FakeHost::new(Os::Linux, "x86_64");
    let error = find(&host, "x86_64-pc-windows-gnu").unwrap_err();
    assert!(error.notes[0].contains("llvm-mingw"), "{error}");
}

#[test]
fn macos_cannot_be_linked_for_from_another_system() {
    let host = FakeHost::debian();
    let error = find(&host, "aarch64-apple-darwin").unwrap_err();
    assert_eq!(
        error.notes,
        ["it cannot be redistributed, so build for macOS on a Mac"]
    );
}

#[test]
fn the_archiver_depends_on_the_target() {
    let mut host = FakeHost::mac();
    host.file("/usr/bin/ar");
    host.file("/opt/homebrew/bin/llvm-ar");
    host.path.push("/opt/homebrew/bin".into());
    let lookup = Lookup::new(&host);
    let ar = |triple: &str| {
        find_archiver(&lookup, Some(target(triple)), &Overrides::default())
            .unwrap()
            .program
    };
    assert_eq!(ar("aarch64-apple-darwin"), PathBuf::from("/usr/bin/ar"));
    assert_eq!(
        ar("wasm32-unknown-wasi"),
        PathBuf::from("/opt/homebrew/bin/llvm-ar")
    );
    assert_eq!(
        ar("x86_64-pc-windows-gnu"),
        PathBuf::from("/opt/homebrew/bin/llvm-ar")
    );
}

#[test]
fn a_sysroot_that_is_not_a_folder_is_an_error_naming_it() {
    let host = FakeHost::mac();
    let overrides = Overrides {
        sysroot: Some(Given {
            value: "/nope".into(),
            from: "-sysroot",
        }),
        ..Overrides::default()
    };
    let error = find_sysroot(
        &Lookup::new(&host),
        target(ARMHF),
        &overrides,
        SysrootUse::Link,
    )
    .unwrap_err();
    assert_eq!(error.what, "-sysroot names `/nope`, which is not a folder");
}
