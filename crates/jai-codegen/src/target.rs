//! LLVM target selection, target-derived layout and native object emission.
use crate::{abi, optimization::Optimization};
use inkwell::{
    context::Context,
    module::Module,
    targets::{
        ByteOrdering, CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetData,
        TargetMachine, TargetTriple,
    },
};
use std::{fmt, path::Path, sync::Once};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetSelection {
    Host,
    Triple(Triple),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Triple(String);
impl Triple {
    pub fn new(value: impl Into<String>) -> Result<Self, Error> {
        let value = value.into();
        let segments: Vec<_> = value.split('-').collect();
        if !(3..=5).contains(&segments.len())
            || segments.iter().any(|segment| {
                segment.is_empty()
                    || !segment
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'.')
            })
        {
            return Err(Error::InvalidTriple(value));
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Cpu {
    #[default]
    Generic,
    Host,
    Named(CpuName),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CpuName(String);
impl CpuName {
    pub fn new(value: impl Into<String>) -> Result<Self, Error> {
        let value = value.into();
        if value.is_empty()
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.')
        {
            return Err(Error::InvalidCpu(value));
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Features(String);
impl Features {
    pub fn new(value: impl Into<String>) -> Result<Self, Error> {
        let value = value.into();
        if !value.is_empty()
            && value.split(',').any(|feature| {
                feature.len() < 2
                    || !matches!(feature.as_bytes()[0], b'+' | b'-')
                    || !feature.as_bytes()[1..]
                        .iter()
                        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
            })
        {
            return Err(Error::InvalidFeatures(value));
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetOptions {
    pub selection: TargetSelection,
    pub cpu: Cpu,
    pub features: Features,
    pub optimization: Optimization,
    pub debug: jai_types::DebugInformation,
}
impl Default for TargetOptions {
    fn default() -> Self {
        Self {
            selection: TargetSelection::Host,
            cpu: Cpu::Generic,
            features: Features::default(),
            optimization: Optimization::default(),
            debug: jai_types::DebugInformation::Off,
        }
    }
}
pub use jai_types::ByteOrder as Endianness;

pub struct NativeTarget {
    pub triple: TargetTriple,
    pub data: TargetData,
    pub machine: TargetMachine,
    pub optimization: Optimization,
    pub debug: jai_types::DebugInformation,
}
impl NativeTarget {
    pub fn new() -> Result<Self, abi::Error> {
        Self::select(&TargetOptions::default())
            .map_err(|error| abi::Error::Target(error.to_string()))
    }
    pub fn select(options: &TargetOptions) -> Result<Self, Error> {
        static INITIALIZE: Once = Once::new();
        INITIALIZE.call_once(|| Target::initialize_all(&InitializationConfig::default()));
        let triple = match &options.selection {
            TargetSelection::Host => TargetMachine::get_default_triple(),
            TargetSelection::Triple(value) => TargetTriple::create(value.as_str()),
        };
        if options.cpu == Cpu::Host && triple != TargetMachine::get_default_triple() {
            return Err(Error::HostCpuForCrossTarget);
        }
        let cpu = match &options.cpu {
            Cpu::Generic => "generic".to_owned(),
            Cpu::Host => TargetMachine::get_host_cpu_name().to_string(),
            Cpu::Named(value) => value.as_str().to_owned(),
        };
        let target =
            Target::from_triple(&triple).map_err(|error| Error::Llvm(error.to_string()))?;
        let machine = target
            .create_target_machine(
                &triple,
                &cpu,
                options.features.as_str(),
                options.optimization.machine_level(),
                RelocMode::PIC,
                CodeModel::Default,
            )
            .ok_or_else(|| Error::Llvm("LLVM could not create target machine".into()))?;
        Ok(Self {
            data: machine.get_target_data(),
            machine,
            triple,
            optimization: options.optimization,
            debug: options.debug,
        })
    }
    pub fn c_platform(&self) -> Result<abi::Platform, abi::Error> {
        abi::Platform::from_triple(&self.triple.as_str().to_string_lossy())
    }
    pub fn layout_policy(&self) -> Result<jai_types::LayoutPolicy, crate::types::Error> {
        crate::types::layout_policy(&Context::create(), &self.data)
    }
    pub fn build_target(&self) -> Result<jai_types::BuildTarget, crate::types::Error> {
        use jai_types::{Architecture as Arch, OperatingSystem as Os};
        let triple = self.triple.as_str().to_string_lossy();
        let arch = triple.split('-').next().unwrap_or_default();
        let architecture = match arch {
            "x86_64" | "amd64" => Arch::X86_64,
            "aarch64" | "arm64" => Arch::Arm64,
            "i386" | "i486" | "i586" | "i686" => Arch::X86,
            "wasm32" => Arch::WebAssembly32,
            "wasm64" => Arch::WebAssembly64,
            value if value.starts_with("arm") || value.starts_with("thumb") => Arch::Arm,
            value => Arch::Other(value.into()),
        };
        let operating_system = if triple.contains("android") {
            Os::Android
        } else if triple.contains("linux") {
            Os::Linux
        } else if triple.contains("windows") || triple.contains("win32") {
            Os::Windows
        } else if triple.contains("ios") {
            Os::IOS
        } else if triple.contains("darwin") || triple.contains("macos") {
            Os::MacOS
        } else if arch.starts_with("wasm") {
            Os::WebAssembly
        } else {
            Os::Other(triple.to_string())
        };
        Ok(jai_types::BuildTarget {
            architecture,
            operating_system,
            layout: self.layout_policy()?,
            byte_order: self.endianness(),
        })
    }
    pub fn endianness(&self) -> Endianness {
        match self.data.get_byte_ordering() {
            ByteOrdering::LittleEndian => Endianness::Little,
            ByteOrdering::BigEndian => Endianness::Big,
        }
    }
    pub fn is_host(&self) -> bool {
        self.triple == TargetMachine::get_default_triple()
    }
    pub fn prepare(&self, module: &Module<'_>) -> Result<(), Error> {
        let triple = module.get_triple();
        if !triple.as_str().to_bytes().is_empty() && triple != self.triple {
            return Err(Error::MismatchedTarget);
        }
        let layout = module.get_data_layout();
        if !layout.as_str().to_bytes().is_empty()
            && layout.as_str() != self.data.get_data_layout().as_str()
        {
            return Err(Error::MismatchedLayout);
        }
        drop(layout);
        module.set_triple(&self.triple);
        module.set_data_layout(&self.data.get_data_layout());
        self.optimization
            .apply(module, &self.machine)
            .map_err(Error::Llvm)
    }
    pub fn write_object(&self, module: &Module<'_>, path: &Path) -> Result<(), Error> {
        self.prepare(module)?;
        self.machine
            .write_to_file(module, FileType::Object, path)
            .map_err(|error| Error::Llvm(error.to_string()))
    }
}
#[derive(Debug)]
pub enum Error {
    InvalidTriple(String),
    InvalidCpu(String),
    InvalidFeatures(String),
    HostCpuForCrossTarget,
    MismatchedTarget,
    MismatchedLayout,
    Llvm(String),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTriple(value) => write!(f, "invalid target triple {value:?}"),
            Self::InvalidCpu(value) => write!(f, "invalid target CPU name {value:?}"),
            Self::InvalidFeatures(value) => write!(
                f,
                "invalid target features {value:?}; expected comma-separated +feature or -feature"
            ),
            Self::HostCpuForCrossTarget => {
                f.write_str("host CPU selection requires the host target triple")
            }
            Self::MismatchedTarget => {
                f.write_str("module target triple does not match the selected object target")
            }
            Self::MismatchedLayout => {
                f.write_str("module data layout does not match the selected object target")
            }
            Self::Llvm(error) => write!(f, "LLVM native target: {error}"),
        }
    }
}
impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimization::BitcodeOptimization;
    use crate::test_native_tools::clang_command;
    use std::{
        fs,
        process::Command,
        sync::atomic::{AtomicU64, Ordering},
    };
    struct Scratch(std::path::PathBuf);
    impl Scratch {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "jai-target-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn fixture(context: &Context) -> Module<'_> {
        let module = context.create_module("target.fixture");
        let function = module.add_function("main", context.i32_type().fn_type(&[], false), None);
        let builder = context.create_builder();
        builder.position_at_end(context.append_basic_block(function, "entry"));
        let local = builder.build_alloca(context.i32_type(), "local").unwrap();
        builder
            .build_store(local, context.i32_type().const_int(42, false))
            .unwrap();
        let result = builder
            .build_load(context.i32_type(), local, "result")
            .unwrap();
        builder.build_return(Some(&result)).unwrap();
        module
    }
    #[test]
    fn typed_boundaries_reject_invalid_configuration() {
        for triple in [
            "",
            "arm64",
            "a-b",
            "x86_64-linux-gnu\0",
            "a-b-c-d-e-f",
            "a--c",
        ] {
            assert!(Triple::new(triple).is_err());
        }
        for cpu in ["", "native\0", "some cpu"] {
            assert!(CpuName::new(cpu).is_err());
        }
        for features in ["neon", "+", "+sse2,", "+sse2\0", "+bad feature"] {
            assert!(Features::new(features).is_err());
        }
        assert!(Features::new("+neon,-sve").is_ok());
    }
    #[test]
    fn object_preparation_rejects_a_different_existing_target_or_layout() {
        let target = NativeTarget::new().unwrap();
        let context = Context::create();
        let module = fixture(&context);
        module.set_triple(&TargetTriple::create("wasm32-unknown-unknown"));
        assert!(matches!(
            target.prepare(&module),
            Err(Error::MismatchedTarget)
        ));
        module.set_triple(&target.triple);
        module.set_data_layout(&TargetData::create("e-p:32:32").get_data_layout());
        assert!(matches!(
            target.prepare(&module),
            Err(Error::MismatchedLayout)
        ));
    }
    #[test]
    fn every_optimization_pipeline_emits_an_actual_object() {
        let scratch = Scratch::new();
        for level in [
            BitcodeOptimization::O0,
            BitcodeOptimization::O1,
            BitcodeOptimization::O2,
            BitcodeOptimization::O3,
            BitcodeOptimization::Os,
            BitcodeOptimization::Oz,
        ] {
            let target = NativeTarget::select(&TargetOptions {
                optimization: Optimization {
                    bitcode: level,
                    ..Default::default()
                },
                ..Default::default()
            })
            .unwrap();
            let context = Context::create();
            let module = fixture(&context);
            let path = scratch.0.join(format!("{level:?}.o"));
            target.write_object(&module, &path).unwrap();
            assert!(fs::metadata(path).unwrap().len() > 0);
            if level != BitcodeOptimization::O0 {
                assert!(!module.print_to_string().to_string().contains("alloca"));
            }
        }
    }
    #[test]
    fn host_object_links_and_executes_with_installed_clang() {
        let scratch = Scratch::new();
        let context = Context::create();
        let module = fixture(&context);
        let target = NativeTarget::new().unwrap();
        assert_eq!(
            target.layout_policy().unwrap().pointer().size,
            u64::from(target.data.get_pointer_byte_size(None))
        );
        let object = scratch.0.join("program.o");
        let executable = scratch.0.join("program");
        target.write_object(&module, &object).unwrap();
        let status = clang_command()
            .arg(&object)
            .arg("-o")
            .arg(&executable)
            .status()
            .unwrap();
        assert!(status.success());
        assert_eq!(Command::new(executable).status().unwrap().code(), Some(42));
    }
    #[test]
    fn floating_remainder_object_links_with_host_math_library() {
        let scratch = Scratch::new();
        let context = Context::create();
        let module = context.create_module("math.fixture");
        let function = module.add_function(
            "fixture_remainder",
            context.f64_type().fn_type(
                &[context.f64_type().into(), context.f64_type().into()],
                false,
            ),
            None,
        );
        let builder = context.create_builder();
        builder.position_at_end(context.append_basic_block(function, "entry"));
        let value = builder
            .build_float_rem(
                function.get_nth_param(0).unwrap().into_float_value(),
                function.get_nth_param(1).unwrap().into_float_value(),
                "remainder",
            )
            .unwrap();
        builder.build_return(Some(&value)).unwrap();
        let target = NativeTarget::new().unwrap();
        let object = scratch.0.join("math.o");
        let source = scratch.0.join("fixture.c");
        let executable = scratch.0.join("math");
        target.write_object(&module, &object).unwrap();
        fs::write(&source, "extern double fixture_remainder(double, double); int main(void) { return fixture_remainder(5.5, 2.0) == 1.5 ? 0 : 1; }").unwrap();
        let mut command = clang_command();
        command.arg(object).arg(source).arg("-o").arg(&executable);
        if target.triple.as_str().to_string_lossy().contains("linux") {
            command.arg("-lm");
        }
        assert!(command.status().unwrap().success());
        assert_eq!(Command::new(executable).status().unwrap().code(), Some(0));
    }
    #[test]
    fn source_target_tags_and_byte_order_follow_the_selected_triple() {
        for (triple, os, cpu, order) in [
            (
                "aarch64-apple-darwin",
                Some("MACOS"),
                Some("ARM64"),
                jai_types::ByteOrder::Little,
            ),
            (
                "x86_64-unknown-linux-gnu",
                Some("LINUX"),
                Some("X64"),
                jai_types::ByteOrder::Little,
            ),
            (
                "aarch64-pc-windows-msvc",
                Some("WINDOWS"),
                Some("ARM64"),
                jai_types::ByteOrder::Little,
            ),
            (
                "powerpc64-unknown-linux-gnu",
                Some("LINUX"),
                None,
                jai_types::ByteOrder::Big,
            ),
        ] {
            let target = NativeTarget::select(&TargetOptions {
                selection: TargetSelection::Triple(Triple::new(triple).unwrap()),
                ..Default::default()
            })
            .unwrap();
            let facts = target.build_target().unwrap();
            assert_eq!(facts.operating_system.source_tag(), os);
            assert_eq!(facts.architecture.source_tag(), cpu);
            assert_eq!(facts.byte_order, order);
        }
    }
    #[test]
    fn cross_target_layout_and_object_are_selected_without_c_abi() {
        let scratch = Scratch::new();
        for (triple, size) in [
            ("x86_64-unknown-linux-gnu", 8),
            ("i686-unknown-linux-gnu", 4),
            ("aarch64-pc-windows-msvc", 8),
        ] {
            let target = NativeTarget::select(&TargetOptions {
                selection: TargetSelection::Triple(Triple::new(triple).unwrap()),
                ..Default::default()
            })
            .unwrap();
            assert_eq!(target.layout_policy().unwrap().pointer().size, size);
            assert_eq!(target.endianness(), Endianness::Little);
            let context = Context::create();
            let module = fixture(&context);
            let object = scratch.0.join(format!("{triple}.o"));
            target.write_object(&module, &object).unwrap();
            assert!(fs::metadata(object).unwrap().len() > 0);
        }
    }
}
