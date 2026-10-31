//! What a target's C environment is: which runtime it links, how each linker spells it, which
//! Jai library names mean "the C runtime", which symbols the runtime exports under another name,
//! and how its variables are imported.
//!
//! The interpreter, code generation, linking and the tests all read the same [`Toolchain`] for a
//! [`Target`], so a fact is written down once. Finding the programs and SDK directories on disk is
//! a separate concern and lives in `discover`. See `docs/compiler/toolchain.md`.
use crate::target::{Target, TargetAbi, TargetOs};

pub mod discover;
mod linux;
mod macos;
mod wasm;
mod windows_gnu;
mod windows_msvc;

/// The linker family a target uses.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Linker {
    /// macOS and Linux: the system `cc` driver (Clang or GCC). WebAssembly is described with this
    /// family too, but it is linked by `wasm-ld` and never reaches the C driver code.
    Unix,
    /// Windows with the MinGW-w64 runtime (`x86_64-pc-windows-gnu`): a MinGW GCC or Clang
    /// driver. This is what cross builds from macOS and Linux use.
    MinGw,
    /// Windows with the Microsoft toolchain (`x86_64-pc-windows-msvc`, the default on a
    /// Windows host): Clang's driver, `lld-link` or `link.exe`.
    Msvc,
}

impl Linker {
    pub fn is_windows(self) -> bool {
        self != Linker::Unix
    }
}

/// The command-line syntax a link is written in, by the program that runs it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Driver {
    /// A `cc`-style driver: `gcc`, `clang`, a MinGW cross compiler.
    C,
    /// Clang's `cl`-compatible driver; everything after `/link` is `link.exe` syntax.
    ClangCl,
    /// `link.exe` or `lld-link`.
    MsvcLink,
}

/// How Clang's `cc`-style driver is told about a runtime library that `link.exe` is given as
/// `/DEFAULTLIB`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CcSpelling {
    /// `-Wl,/DEFAULTLIB:name`.
    DefaultLib,
    /// `-lname`.
    Lib,
    /// Not named: the driver already passes it.
    Implicit,
}

/// One library of a target's C runtime.
#[derive(Clone, Copy, Debug)]
pub struct RuntimeLib {
    pub name: &'static str,
    /// How a link names it on its own; `None`: only a custom link command lists it (see
    /// [`Toolchain::runtime_inputs`]), since the runtime's own directives bring it in.
    pub default: Option<CcSpelling>,
}

/// One Jai library name that means the C runtime, and where the interpreter finds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeName {
    pub name: &'static str,
    /// The DLL the interpreter loads for this name on a Windows host; `None` where the
    /// interpreter has no such DLL.
    pub dll: Option<&'static str>,
}

impl RuntimeName {
    /// A name with no DLL of its own.
    pub const fn plain(name: &'static str) -> RuntimeName {
        RuntimeName {
            name,
            dll: None,
        }
    }
}

/// The DLLs of a Windows C runtime that the interpreter loads.
#[derive(Clone, Copy, Debug)]
pub struct RuntimeDlls {
    /// Where the generic names (`libc`, `libm`, `msvcrt`, ...) resolve; see [`RuntimeName::dll`].
    pub c: &'static str,
    /// The DLL that exports the `__stdio_common_*` workers behind `printf`.
    pub stdio: &'static str,
}

/// How a runtime library's variables are imported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataImport {
    /// Referenced by name: the runtime is linked statically, or the platform has no import
    /// convention.
    Direct,
    /// A DLL exports a variable only as `__imp_<name>`, so it is `dllimport`; but only a library
    /// named in `only` promises its data comes from the DLL. The generic names also reach the
    /// static import library, which defines some data itself, and a `dllimport` reference to
    /// that cannot link.
    DllImport { only: &'static [&'static str] },
}

/// The C runtime of a target.
#[derive(Clone, Copy, Debug)]
pub struct Runtime {
    /// The libraries, in link order.
    pub libs: &'static [RuntimeLib],
    /// Libraries a `cc`-style driver would add by itself that must not be linked.
    pub exclude: &'static [&'static str],
    /// The `cl` option that selects the DLL runtime.
    pub cl_flag: Option<&'static str>,
    /// The `cc`-style driver's option that selects the DLL runtime.
    pub cc_flag: Option<&'static str>,
    pub dlls: Option<RuntimeDlls>,
}

impl Runtime {
    pub const NONE: Runtime = Runtime {
        libs: &[],
        exclude: &[],
        cl_flag: None,
        cc_flag: None,
        dlls: None,
    };
}

/// How WebAssembly names the host module a foreign procedure is imported from.
#[derive(Clone, Copy, Debug)]
pub struct ForeignModules {
    /// The module for a procedure with no library, and for the library names in `runtime`.
    pub default: &'static str,
    /// Library names (compared by file name) whose functions the host supplies.
    pub runtime: &'static [&'static str],
}

/// One target's C environment.
#[derive(Clone, Copy, Debug)]
pub struct Toolchain {
    pub linker: Linker,
    pub runtime: Runtime,
    /// Jai library names that mean the C runtime on this target: they link nothing by name, and
    /// the interpreter loads the DLL each one maps to.
    pub runtime_names: &'static [RuntimeName],
    /// `runtime_names` is compared without regard to case.
    pub names_fold_case: bool,
    /// Libraries every link adds that nothing else names.
    pub implicit_libraries: &'static [&'static str],
    /// The symbol the runtime really exports for `symbol`, when it is not `symbol` itself.
    pub renames: for<'a> fn(&'a str) -> Option<&'a str>,
    pub runtime_data: DataImport,
    /// Bytes of main-thread stack to reserve, where the platform's default is smaller than the
    /// 8 MiB macOS and Linux give.
    pub main_stack: Option<u64>,
    pub foreign_modules: Option<ForeignModules>,
}

/// What a link adds for the runtime, in the syntax of one driver.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeArgs {
    /// Options for the compiler half of the driver (`/MD`); `cl` takes these before `/link`.
    pub compile: Vec<String>,
    /// Linker options and libraries.
    pub link: Vec<String>,
}

impl Toolchain {
    /// The toolchain of `target`.
    pub fn for_target(target: Target) -> &'static Toolchain {
        match (target.os, target.abi) {
            (TargetOs::Windows, TargetAbi::Gnu) => &windows_gnu::WINDOWS_GNU,
            (TargetOs::Windows, _) => &windows_msvc::WINDOWS_MSVC,
            (TargetOs::MacOS, _) => &macos::MACOS,
            (TargetOs::Linux, _) => &linux::LINUX,
            (TargetOs::Wasm, _) => &wasm::WASM,
        }
    }

    /// The toolchain of the C runtime the interpreter calls into on a Windows host: it loads the
    /// runtime DLLs, whichever ABI the compiler itself was built for.
    pub fn interpreter() -> &'static Toolchain {
        &windows_msvc::WINDOWS_MSVC
    }

    /// The table row for a Jai library name that means the C runtime here (or a POSIX library
    /// that is part of it): the link names the runtime itself.
    pub fn runtime_name(&self, library: &str) -> Option<&'static RuntimeName> {
        if self.names_fold_case {
            let lower = library.to_ascii_lowercase();
            self.runtime_names.iter().find(|row| row.name == lower)
        } else {
            self.runtime_names.iter().find(|row| row.name == library)
        }
    }

    /// Whether a Jai library name means the C runtime here.
    pub fn is_runtime(&self, library: &str) -> bool {
        self.runtime_name(library).is_some()
    }

    /// The DLL the interpreter loads for a library name that means the C runtime.
    pub fn interpreter_dll(&self, library: &str) -> Option<&'static str> {
        self.runtime_name(library)?.dll
    }

    /// Whether a foreign variable of the library `name` is imported through its DLL. A system
    /// library is a DLL on Windows; the runtime follows `runtime_data`.
    pub fn imports_data(&self, name: &str, system: bool) -> bool {
        if !system || !self.linker.is_windows() {
            return false;
        }
        if self.is_runtime(name) {
            match self.runtime_data {
                DataImport::Direct => false,
                DataImport::DllImport {
                    only,
                } => only.contains(&name.to_ascii_lowercase().as_str()),
            }
        } else {
            true
        }
    }

    /// The runtime's name for `symbol`.
    pub fn rename<'a>(&self, symbol: &'a str) -> Option<&'a str> {
        (self.renames)(symbol)
    }

    /// The wasm import module for the foreign library `library` (`None`: a native target).
    pub fn import_module<'a>(&self, library: &'a str) -> Option<&'a str> {
        let modules = self.foreign_modules?;
        let name = library.rsplit(['/', '\\']).next().unwrap_or(library);
        Some(if name.is_empty() || modules.runtime.contains(&name) {
            modules.default
        } else {
            name
        })
    }

    /// What a link adds to select the runtime, written for `driver`.
    pub fn runtime_args(&self, driver: Driver) -> RuntimeArgs {
        let runtime = &self.runtime;
        let mut args = RuntimeArgs::default();
        match driver {
            Driver::MsvcLink | Driver::ClangCl => {
                if driver == Driver::ClangCl {
                    args.compile.extend(runtime.cl_flag.map(String::from));
                }
                args.link.extend(
                    runtime
                        .libs
                        .iter()
                        .filter(|lib| lib.default.is_some())
                        .map(|lib| format!("/DEFAULTLIB:{}", lib.name)),
                );
            }
            Driver::C => {
                if self.runtime.libs.is_empty() {
                    return args;
                }
                args.link.extend(runtime.cc_flag.map(String::from));
                args.link.extend(
                    runtime
                        .exclude
                        .iter()
                        .map(|n| format!("-Wl,/NODEFAULTLIB:{n}")),
                );
                for lib in runtime.libs {
                    match lib.default {
                        Some(CcSpelling::DefaultLib) => {
                            args.link.push(format!("-Wl,/DEFAULTLIB:{}", lib.name));
                        }
                        Some(CcSpelling::Lib) => args.link.push(format!("-l{}", lib.name)),
                        Some(CcSpelling::Implicit) | None => {}
                    }
                }
            }
        }
        args
    }

    /// The option that reserves the main thread's stack, written for `driver`.
    pub fn stack_arg(&self, driver: Driver) -> Option<String> {
        let bytes = self.main_stack?;
        Some(match (driver, self.linker) {
            (Driver::MsvcLink | Driver::ClangCl, _) => format!("/STACK:{bytes}"),
            (Driver::C, Linker::Msvc) => format!("-Wl,/STACK:{bytes}"),
            (Driver::C, _) => format!("-Wl,--stack,{bytes}"),
        })
    }

    /// Every runtime library as an import library file name (`name.lib`): what a link command
    /// that replaces the defaults has to name itself.
    pub fn runtime_inputs(&self) -> impl Iterator<Item = String> + '_ {
        self.runtime
            .libs
            .iter()
            .map(|lib| format!("{}.lib", lib.name))
    }
}

#[cfg(test)]
mod tests;
