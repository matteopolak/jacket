//! Emitting objects with `jaic-llvm`, linking them, and running them natively or as WebAssembly.
//!
//! A backend test compiles a program once per [`Variant`] (native or wasm64, `-O0` or `-O2`),
//! emits an object, links it with a tiny C driver (native, `clang`) or `wasm-ld` plus a
//! `node` host (wasm), and expects every checked entry point to return 0.
//!
//! ```ignore
//! if !backend::tools_available() { return; }
//! for variant in backend::VARIANTS {
//!     let dir = TempDir::new(&variant.label());
//!     let compiler = variant.compile(source);
//!     let object = dir.join("program.o");
//!     variant.emit(&compiler, &object, None);
//!     backend::check(&variant, &dir, &object, &["my_check"], None, &[], &[]);
//! }
//! ```
//!
//! Needs the `backend` feature.

use crate::{process, toolchain};
use jaic::sema::{Compiler, Options, VirtualFs};
use jaic::target::{TargetCpu, TargetOs};
use jaic_llvm::OptLevel;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

/// One way to build a program: for the host or for wasm64, at `-O0` or `-O2`.
#[derive(Clone, Copy, Debug)]
pub struct Variant {
    pub wasm: bool,
    pub level: OptLevel,
}

/// Native and wasm64, each at `-O0` and `-O2`.
pub const VARIANTS: [Variant; 4] = [
    Variant {
        wasm: false,
        level: OptLevel::O0,
    },
    Variant {
        wasm: false,
        level: OptLevel::O2,
    },
    Variant {
        wasm: true,
        level: OptLevel::O0,
    },
    Variant {
        wasm: true,
        level: OptLevel::O2,
    },
];

impl Variant {
    /// A short name (`native-O0`, `wasm-O2`) for scratch directories and failure messages.
    #[must_use]
    pub fn label(&self) -> String {
        format!(
            "{}-{:?}",
            if self.wasm {
                "wasm"
            } else {
                "native"
            },
            self.level
        )
    }

    /// Compiler options for this variant with the runtime support module off, so the program
    /// is only its own source.
    #[must_use]
    pub fn options(&self) -> Options {
        let mut options = Options::host();
        options.runtime_support = false;
        if self.wasm {
            options.os = TargetOs::Wasm;
            options.cpu = TargetCpu::Wasm;
        }
        options
    }

    /// Compiles `source` as one string program for this variant; panics with the rendered
    /// diagnostic on an error.
    #[must_use]
    pub fn compile(&self, source: &str) -> Compiler {
        let mut c = Compiler::new(self.options(), Rc::new(VirtualFs::default()));
        compile_into(&mut c, source);
        c
    }

    /// The code generation options for this variant, optionally also writing textual IR.
    #[must_use]
    pub fn emit_options(&self, ir: Option<&Path>) -> jaic_llvm::Options {
        jaic_llvm::Options {
            target: self.wasm.then(jaic::target::Target::wasm_module),
            opt_level: self.level,
            emit_ir: ir.map(Path::to_path_buf),
            ..Default::default()
        }
    }

    /// Emits `compiler`'s program as the object file `object`.
    pub fn emit(&self, compiler: &Compiler, object: &Path, ir: Option<&Path>) {
        jaic_llvm::emit_object(&compiler.out.program, &self.emit_options(ir), object)
            .unwrap_or_else(|error| panic!("{}: emit failed: {error}", self.label()));
    }
}

fn compile_into(c: &mut Compiler, source: &str) {
    if let Err(error) = c.compile_sources(&[jaic::sema::ProgramSource::String(source.into())]) {
        panic!("{}", error.render(&c.src.sources));
    }
}

/// Whether this host has what backend tests run with (`clang`, `wasm-ld`, `node`); prints why
/// not, and tests return early. On CI a missing tool fails instead; see [`toolchain`].
#[must_use]
pub fn tools_available() -> bool {
    let clang = toolchain::require("clang").is_some();
    let wasm_ld = toolchain::require("wasm-ld").is_some();
    let node = toolchain::require("node").is_some();
    clang && wasm_ld && node
}

/// Links `objects` into the wasm64 `module`, exporting `exports` and the memory.
pub fn link_wasm(objects: &[&Path], module: &Path, exports: &[&str]) {
    let mut command = Command::new(toolchain::wasm_ld().expect("wasm-ld is installed"));
    command.args([
        "-mwasm64",
        "--no-entry",
        "--export-memory",
        "--allow-undefined",
    ]);
    for export in exports {
        command.arg(format!("--export={export}"));
    }
    process::run(command.args(objects).arg("-o").arg(module));
}

/// A wasm host for modules that import only `memcpy`, `memset` and `memcmp`: it calls each export named
/// on the command line and fails unless it returns 0.
/// (`memcmp` is provided too, for programs that compare memory.)
const WASM_HOST: &str = r"
const fs=require('fs');
const module=new WebAssembly.Module(fs.readFileSync(process.argv[1]));
const env={};let memory;
for(const i of WebAssembly.Module.imports(module)){
  if(i.kind!=='function')throw Error('unexpected import '+i.name);
  env[i.name]=()=>{throw Error('unexpected runtime failure '+i.name);};
}
env.memcpy=(d,s,n)=>{new Uint8Array(memory.buffer).copyWithin(Number(d),Number(s),Number(s+n));return d;};
env.memset=(d,v,n)=>{new Uint8Array(memory.buffer).fill(Number(v),Number(d),Number(d+n));return d;};
env.memcmp=(a,b,n)=>{const bytes=new Uint8Array(memory.buffer);a=Number(a);b=Number(b);n=Number(n);for(let i=0;i<n;i++){const diff=bytes[a+i]-bytes[b+i];if(diff)return diff;}return 0;};
const instance=new WebAssembly.Instance(module,{env});memory=instance.exports.memory;
for(const name of process.argv.slice(2)){
  const result=instance.exports[name]();
  if(result!==0)throw Error(name+' returned '+result);
}
";

/// Runs the wasm `module` under node and requires each of `exports` to return 0, in order.
pub fn run_wasm(module: &Path, exports: &[&str]) {
    let mut node = toolchain::node().expect("node is installed");
    process::run(node.args(["-e", WASM_HOST]).arg(module).args(exports));
}

/// Runs the JavaScript `script` under node with `args` (`process.argv[1..]`).
pub fn node_eval(script: &str, args: &[&Path]) {
    let mut node = toolchain::node().expect("node is installed");
    process::run(node.args(["-e", script]).args(args));
}

/// A C driver that calls each of `exports` (`int32_t f(void)`) and returns the first nonzero
/// result.
#[must_use]
pub fn c_driver(exports: &[&str]) -> String {
    let mut source = String::from("#include <stdint.h>\n");
    for export in exports {
        source += &format!("extern int32_t {export}(void);\n");
    }
    source += "int main(void){int32_t r;\n";
    for export in exports {
        source += &format!("r={export}();if(r)return r;\n");
    }
    source + "return 0;}\n"
}

/// Links the C `driver` source with `inputs` into an executable in `dir` and returns its path.
#[must_use]
pub fn link_native(dir: &Path, driver: &str, inputs: &[&Path]) -> PathBuf {
    let source = dir.join("driver.c");
    std::fs::write(&source, driver).expect("the driver can be written");
    let exe = dir.join(format!("driver{}", std::env::consts::EXE_SUFFIX));
    let clang = toolchain::clang().expect("clang is installed");
    process::run(
        Command::new(clang)
            .args(["-std=c11", "-Wall", "-Wextra", "-Werror"])
            .arg(&source)
            .args(inputs)
            .arg("-o")
            .arg(&exe),
    );
    exe
}

/// Builds `object` (plus the extra inputs) for `variant` in `dir` and requires each of
/// `exports` to return 0. `driver` is the native C driver (default: [`c_driver`]);
/// `native_inputs` are extra C sources or objects linked into the native executable and
/// `wasm_inputs` extra wasm64 objects linked into the module.
pub fn check(
    variant: &Variant,
    dir: &Path,
    object: &Path,
    exports: &[&str],
    driver: Option<&str>,
    native_inputs: &[&Path],
    wasm_inputs: &[&Path],
) {
    if variant.wasm {
        let module = dir.join("program.wasm");
        let mut objects = vec![object];
        objects.extend_from_slice(wasm_inputs);
        link_wasm(&objects, &module, exports);
        run_wasm(&module, exports);
    } else {
        let default = c_driver(exports);
        let mut inputs = native_inputs.to_vec();
        inputs.push(object);
        let exe = link_native(dir, driver.unwrap_or(&default), &inputs);
        process::run(&mut Command::new(exe));
    }
}

/// Compiles C `source` for wasm64 (freestanding) into the object `output`.
pub fn compile_wasm_c(source: &Path, output: &Path) {
    let clang = toolchain::clang().expect("clang is installed");
    process::run(
        Command::new(clang)
            .args([
                "--target=wasm64-unknown-unknown",
                "-ffreestanding",
                "-std=c11",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-c",
            ])
            .arg(source)
            .arg("-o")
            .arg(output),
    );
}

/// Compiles `source` (with the virtual `files`, an optional `temporary_size`) for every
/// [`Variant`] and requires the `entry` procedure, exported with `#program_export`, to return 0.
pub fn execute(
    name: &str,
    entry: &str,
    source: &str,
    files: &[(&str, &str)],
    temporary_size: Option<i64>,
) {
    if !tools_available() {
        return;
    }
    for variant in VARIANTS {
        let dir = crate::TempDir::new(&format!("{name}-{}", variant.label()));
        let mut options = variant.options();
        if let Some(size) = temporary_size {
            options.temporary_storage_size = size;
        }
        options.import_paths = vec!["/lib".into()];
        // A fixture may supply its own independent minimal bootstrap declarations.
        if files
            .iter()
            .any(|(path, _)| *path == "/lib/prelude/module.jai")
        {
            options.preload = Some("/lib/prelude/module.jai".into());
        }
        let mut fs = VirtualFs::default();
        for &(path, text) in files {
            fs.insert(path, text.as_bytes());
        }
        let mut c = Compiler::new(options, Rc::new(fs));
        compile_into(&mut c, source);
        let object = dir.join("program.o");
        variant.emit(&c, &object, None);
        check(&variant, &dir, &object, &[entry], None, &[], &[]);
    }
}
