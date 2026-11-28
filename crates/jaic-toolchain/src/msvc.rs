//! The parts of finding Microsoft's toolchain that only the linker lookup needs: a Clang installed
//! outside `PATH`, and telling Microsoft's `link.exe` from the coreutils `link` that Git for
//! Windows and MSYS2 put on `PATH` (`link FILE1 FILE2`, which stops with `extra operand`).
//!
//! Visual Studio, Windows SDK and `LIB` discovery stays in `jaic::toolchain::discover`, on the
//! `find-msvc-tools` crate, because the interpreter opens import libraries from `LIB` too and
//! this crate depends on `jaic` (see `docs/compiler/architecture.md`).
use std::path::{Path, PathBuf};

use jaic::toolchain::discover;

use crate::host::Host;
use crate::linker::host_target;
use crate::lookup::Lookup;

/// A Clang that is not on `PATH`: the LLVM installer's folder under `Program Files`, then the one
/// Visual Studio's "C++ Clang tools" component adds. Both are programs for the host's CPU, which
/// build for another with `--target`, so the host's folder is searched whatever the target.
pub fn find_clang(lookup: &Lookup) -> Option<PathBuf> {
    let installed: Vec<PathBuf> = lookup
        .host
        .program_files()
        .into_iter()
        .map(|base| base.join("LLVM").join("bin"))
        .collect();

    lookup.in_dirs(&installed, "clang").or_else(|| {
        let host_arch = discover::arch_dir(host_target(lookup.host).cpu);
        lookup.host.visual_studio_tool("clang.exe", host_arch)
    })
}

/// Whether the output of `link /?` is Microsoft's linker's rather than another program named
/// `link` (coreutils prints `link: missing operand`).
fn is_microsoft_linker_banner(output: &str) -> bool {
    output.contains("Microsoft (R)") || output.contains("Incremental Linker")
}

/// Whether `program` is Microsoft's `link.exe`. Its exit status is not looked at: only the banner
/// tells the two `link` programs apart.
pub fn is_microsoft_linker(host: &dyn Host, program: &Path) -> bool {
    host.output(program, &["/?"])
        .is_some_and(|(_, text)| is_microsoft_linker_banner(&text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::Os;
    use crate::testing::FakeHost;

    #[test]
    fn coreutils_link_is_not_the_microsoft_linker() {
        assert!(!is_microsoft_linker_banner(
            "link: missing operand after '/?'\nTry 'link --help' for more information.\n"
        ));
        assert!(is_microsoft_linker_banner(
            "Microsoft (R) Incremental Linker Version 14.44.35215.0\nCopyright (C) Microsoft Corporation.  All rights reserved.\n"
        ));
    }

    #[test]
    fn clang_is_found_under_either_program_files() {
        let mut host = FakeHost::new(Os::Windows, "x86_64");

        host.program_files = vec![r"C:\Program Files".into(), r"C:\Program Files (x86)".into()];
        host.file(r"C:\Program Files (x86)/LLVM/bin/clang.exe");
        assert_eq!(
            find_clang(&Lookup::new(&host)),
            Some(r"C:\Program Files (x86)/LLVM/bin/clang.exe".into())
        );
    }

    #[test]
    fn visual_studios_clang_is_the_hosts_for_a_cross_build() {
        // An x64 machine building for arm64 runs the x64 Clang with `--target`; Visual Studio
        // also ships an arm64 Clang, which would not start here.
        let mut host = FakeHost::new(Os::Windows, "x86_64");

        host.visual_studio("clang.exe", "x64", r"C:\VS\x64\clang.exe");
        host.visual_studio("clang.exe", "arm64", r"C:\VS\arm64\clang.exe");
        let lookup = Lookup::new(&host);

        assert_eq!(find_clang(&lookup), Some(r"C:\VS\x64\clang.exe".into()));
    }

    #[test]
    fn a_link_on_path_counts_only_with_microsofts_banner_whatever_it_exits_with() {
        let mut host = FakeHost::new(Os::Windows, "x86_64");

        host.fails_printing(
            r"C:\git\link.exe",
            &["/?"],
            "link: missing operand after '/?'\n",
        );
        host.fails_printing(
            r"C:\VS\link.exe",
            &["/?"],
            "Microsoft (R) Incremental Linker Version 14.44\n",
        );
        assert!(!is_microsoft_linker(&host, Path::new(r"C:\git\link.exe")));
        assert!(is_microsoft_linker(&host, Path::new(r"C:\VS\link.exe")));
        assert!(!is_microsoft_linker(&host, Path::new(r"C:\none\link.exe")));
    }
}
